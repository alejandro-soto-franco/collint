//! Pure box math. No matplotlib, no pixels, no pyo3.
//! A box is (x0, y0, x1, y1) in display pixels with x0 <= x1, y0 <= y1.

pub type Box4 = (f64, f64, f64, f64);

/// Area of the intersection of two boxes (0.0 if disjoint).
pub fn intersection_area(a: Box4, b: Box4) -> f64 {
    let w = (a.2.min(b.2) - a.0.max(b.0)).max(0.0);
    let h = (a.3.min(b.3) - a.1.max(b.1)).max(0.0);
    w * h
}

/// Area of a box.
pub fn area(a: Box4) -> f64 {
    (a.2 - a.0).max(0.0) * (a.3 - a.1).max(0.0)
}

/// Intersection area as a fraction of the smaller box (0.0 if either is empty).
pub fn intersection_fraction(a: Box4, b: Box4) -> f64 {
    let smaller = area(a).min(area(b));
    if smaller <= 0.0 {
        return 0.0;
    }
    intersection_area(a, b) / smaller
}

/// How far `inner` pokes outside `clip`, as max overhang in pixels on any side
/// (0.0 if fully contained).
pub fn clip_residual(inner: Box4, clip: Box4) -> f64 {
    let left = (clip.0 - inner.0).max(0.0);
    let bottom = (clip.1 - inner.1).max(0.0);
    let right = (inner.2 - clip.2).max(0.0);
    let top = (inner.3 - clip.3).max(0.0);
    left.max(bottom).max(right).max(top)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disjoint_boxes_have_zero_intersection() {
        assert_eq!(
            intersection_area((0.0, 0.0, 1.0, 1.0), (2.0, 2.0, 3.0, 3.0)),
            0.0
        );
    }

    #[test]
    fn overlapping_boxes_intersection_area() {
        // overlap rectangle is 1x1
        assert_eq!(
            intersection_area((0.0, 0.0, 2.0, 2.0), (1.0, 1.0, 3.0, 3.0)),
            1.0
        );
    }

    #[test]
    fn fraction_is_relative_to_smaller_box() {
        // smaller box area 1, intersection 1 -> fully covered
        let big = (0.0, 0.0, 10.0, 10.0);
        let small = (0.0, 0.0, 1.0, 1.0);
        assert_eq!(intersection_fraction(big, small), 1.0);
    }

    #[test]
    fn edge_touching_is_not_overlap() {
        assert_eq!(
            intersection_area((0.0, 0.0, 1.0, 1.0), (1.0, 0.0, 2.0, 1.0)),
            0.0
        );
    }

    #[test]
    fn contained_box_has_zero_clip_residual() {
        assert_eq!(
            clip_residual((1.0, 1.0, 2.0, 2.0), (0.0, 0.0, 10.0, 10.0)),
            0.0
        );
    }

    #[test]
    fn overhang_measured_in_pixels() {
        // inner pokes 3px past the right edge
        assert_eq!(
            clip_residual((0.0, 0.0, 13.0, 1.0), (0.0, 0.0, 10.0, 10.0)),
            3.0
        );
    }
}
