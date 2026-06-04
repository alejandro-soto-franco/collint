//! Detection thresholds. Pure data, no matplotlib.

#[derive(Clone, Copy, Debug)]
pub struct Config {
    /// Min intersection area (fraction of smaller box) to count as overlap.
    pub overlap_fraction: f64,
    /// Min clip overhang in pixels to count as clipping.
    pub clip_px: f64,
    /// Min foreground ink coverage (fraction of region) to confirm over-data.
    pub ink_fraction: f64,
    /// Per-channel tolerance (0..=255) for calling a pixel "background".
    pub bg_tolerance: u8,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            overlap_fraction: 0.02,
            clip_px: 1.0,
            ink_fraction: 0.05,
            bg_tolerance: 8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_spec() {
        let c = Config::default();
        assert_eq!(c.overlap_fraction, 0.02);
        assert_eq!(c.clip_px, 1.0);
        assert_eq!(c.ink_fraction, 0.05);
        assert_eq!(c.bg_tolerance, 8);
    }
}
