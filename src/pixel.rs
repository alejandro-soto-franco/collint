//! Ink-coverage analysis over a borrowed RGBA view. Pure; no matplotlib, no Figure.

use ndarray::ArrayView3;
use rayon::prelude::*;

/// A region of the buffer in pixel coords: rows [r0, r1), cols [c0, c1).
pub type Region = (usize, usize, usize, usize);

/// True if pixel differs from `bg` (RGB) by more than `tol` on any channel.
#[inline]
fn is_foreground(r: u8, g: u8, b: u8, bg: [u8; 3], tol: u8) -> bool {
    let d = |x: u8, y: u8| -> u8 { if x > y { x - y } else { y - x } };
    d(r, bg[0]) > tol || d(g, bg[1]) > tol || d(b, bg[2]) > tol
}

/// Fraction of pixels in `region` that are foreground relative to `bg`.
/// Reduction is parallelized over rows with rayon. Returns 0.0 for an empty region.
pub fn ink_coverage(view: &ArrayView3<u8>, region: Region, bg: [u8; 3], tol: u8) -> f64 {
    let (r0, c0, r1, c1) = region;
    let (h, w, _) = view.dim();
    let r0 = r0.min(h);
    let r1 = r1.min(h);
    let c0 = c0.min(w);
    let c1 = c1.min(w);
    if r1 <= r0 || c1 <= c0 {
        return 0.0;
    }
    let total = ((r1 - r0) * (c1 - c0)) as f64;
    let fg: usize = (r0..r1)
        .into_par_iter()
        .map(|row| {
            let mut count = 0usize;
            for col in c0..c1 {
                let r = view[[row, col, 0]];
                let g = view[[row, col, 1]];
                let b = view[[row, col, 2]];
                if is_foreground(r, g, b, bg, tol) {
                    count += 1;
                }
            }
            count
        })
        .sum();
    fg as f64 / total
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::Array3;

    fn white(h: usize, w: usize) -> Array3<u8> {
        Array3::from_elem((h, w, 4), 255u8)
    }

    #[test]
    fn all_background_is_zero_coverage() {
        let buf = white(10, 10);
        let cov = ink_coverage(&buf.view(), (0, 0, 10, 10), [255, 255, 255], 8);
        assert_eq!(cov, 0.0);
    }

    #[test]
    fn all_black_on_white_bg_is_full_coverage() {
        let mut buf = white(10, 10);
        for r in 0..10 {
            for c in 0..10 {
                buf[[r, c, 0]] = 0;
                buf[[r, c, 1]] = 0;
                buf[[r, c, 2]] = 0;
            }
        }
        let cov = ink_coverage(&buf.view(), (0, 0, 10, 10), [255, 255, 255], 8);
        assert_eq!(cov, 1.0);
    }

    #[test]
    fn half_inked_region_is_half_coverage() {
        let mut buf = white(10, 10);
        for r in 0..5 {
            for c in 0..10 {
                buf[[r, c, 0]] = 0;
                buf[[r, c, 1]] = 0;
                buf[[r, c, 2]] = 0;
            }
        }
        let cov = ink_coverage(&buf.view(), (0, 0, 10, 10), [255, 255, 255], 8);
        assert!((cov - 0.5).abs() < 1e-9);
    }

    #[test]
    fn within_tolerance_counts_as_background() {
        let mut buf = white(4, 4);
        for r in 0..4 {
            for c in 0..4 {
                buf[[r, c, 0]] = 250; // 5 off from 255, tol 8 -> background
                buf[[r, c, 1]] = 250;
                buf[[r, c, 2]] = 250;
            }
        }
        let cov = ink_coverage(&buf.view(), (0, 0, 4, 4), [255, 255, 255], 8);
        assert_eq!(cov, 0.0);
    }

    #[test]
    fn empty_region_is_zero() {
        let buf = white(10, 10);
        assert_eq!(ink_coverage(&buf.view(), (5, 5, 5, 9), [255, 255, 255], 8), 0.0);
    }
}
