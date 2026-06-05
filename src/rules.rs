//! Classification of tagged boxes into collisions plus over-data candidates.
//! Pure: geometry only. Pixel confirmation of candidates happens in Python.

use crate::config::Config;
use crate::geometry::{clip_residual, intersection_fraction, Box4};
use crate::report::{Collision, Kind};

#[derive(Clone, Debug)]
pub struct TaggedBox {
    pub bx: Box4,
    pub clip: Box4,
    pub role: String,
    pub id: String,
}

/// A geometry over-data candidate awaiting pixel confirmation.
#[derive(Clone, Debug)]
pub struct OverDataCandidate {
    pub decoration_id: String,
    pub decoration_role: String,
    pub decoration_box: Box4,
    pub data_id: String,
    pub data_box: Box4,
    pub overlap: f64,
}

fn is_text(role: &str) -> bool {
    matches!(
        role,
        "tick-label" | "axis-label" | "title" | "suptitle" | "legend" | "annotation"
    )
}

fn is_decoration(role: &str) -> bool {
    matches!(role, "legend" | "annotation")
}

fn is_element(role: &str) -> bool {
    matches!(role, "axes-region" | "colorbar" | "suptitle" | "title")
}

/// Returns (final collisions for the three pure classes, over-data candidates).
pub fn classify(boxes: &[TaggedBox], cfg: &Config) -> (Vec<Collision>, Vec<OverDataCandidate>) {
    let mut collisions = Vec::new();
    let mut candidates = Vec::new();

    // Clipping: any box poking outside its clip box.
    for b in boxes {
        let res = clip_residual(b.bx, b.clip);
        if res > cfg.clip_px {
            collisions.push(Collision {
                kind: Kind::Clipping,
                a_role: b.role.clone(),
                a_id: b.id.clone(),
                a_box: b.bx,
                b_role: "clip".into(),
                b_id: "clip".into(),
                b_box: b.clip,
                severity: res,
            });
        }
    }

    // Pairwise checks.
    for i in 0..boxes.len() {
        for j in (i + 1)..boxes.len() {
            let a = &boxes[i];
            let b = &boxes[j];
            let frac = intersection_fraction(a.bx, b.bx);
            if frac < cfg.overlap_fraction {
                continue;
            }

            // text-text
            if is_text(&a.role) && is_text(&b.role) {
                collisions.push(Collision {
                    kind: Kind::TextText,
                    a_role: a.role.clone(),
                    a_id: a.id.clone(),
                    a_box: a.bx,
                    b_role: b.role.clone(),
                    b_id: b.id.clone(),
                    b_box: b.bx,
                    severity: frac,
                });
                continue;
            }

            // element-element
            if is_element(&a.role) && is_element(&b.role) {
                collisions.push(Collision {
                    kind: Kind::ElementElement,
                    a_role: a.role.clone(),
                    a_id: a.id.clone(),
                    a_box: a.bx,
                    b_role: b.role.clone(),
                    b_id: b.id.clone(),
                    b_box: b.bx,
                    severity: frac,
                });
                continue;
            }

            // over-data candidate (decoration over a data artist)
            let (deco, data) = if is_decoration(&a.role) && b.role == "data-artist" {
                (Some(a), Some(b))
            } else if is_decoration(&b.role) && a.role == "data-artist" {
                (Some(b), Some(a))
            } else {
                (None, None)
            };
            if let (Some(deco), Some(data)) = (deco, data) {
                candidates.push(OverDataCandidate {
                    decoration_id: deco.id.clone(),
                    decoration_role: deco.role.clone(),
                    decoration_box: deco.bx,
                    data_id: data.id.clone(),
                    data_box: data.bx,
                    overlap: frac,
                });
            }
        }
    }

    (collisions, candidates)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tb(bx: Box4, clip: Box4, role: &str, id: &str) -> TaggedBox {
        TaggedBox {
            bx,
            clip,
            role: role.into(),
            id: id.into(),
        }
    }

    #[test]
    fn overlapping_tick_labels_flag_text_text() {
        let cfg = Config::default();
        let fig = (0.0, 0.0, 100.0, 100.0);
        let boxes = vec![
            tb((0.0, 0.0, 10.0, 10.0), fig, "tick-label", "t0"),
            tb((5.0, 0.0, 15.0, 10.0), fig, "tick-label", "t1"),
        ];
        let (c, cand) = classify(&boxes, &cfg);
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].kind, Kind::TextText);
        assert!(cand.is_empty());
    }

    #[test]
    fn title_past_figure_edge_flags_clipping() {
        let cfg = Config::default();
        let fig = (0.0, 0.0, 100.0, 100.0);
        let boxes = vec![tb((90.0, 95.0, 130.0, 105.0), fig, "title", "ti")];
        let (c, _) = classify(&boxes, &cfg);
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].kind, Kind::Clipping);
    }

    #[test]
    fn legend_over_data_is_a_candidate_not_a_collision() {
        let cfg = Config::default();
        let fig = (0.0, 0.0, 100.0, 100.0);
        let boxes = vec![
            tb((10.0, 10.0, 30.0, 30.0), fig, "legend", "L"),
            tb((0.0, 0.0, 50.0, 50.0), fig, "data-artist", "D"),
        ];
        let (c, cand) = classify(&boxes, &cfg);
        assert!(c.is_empty());
        assert_eq!(cand.len(), 1);
        assert_eq!(cand[0].decoration_id, "L");
        assert_eq!(cand[0].data_id, "D");
    }

    #[test]
    fn overlapping_axes_regions_flag_element_element() {
        let cfg = Config::default();
        let fig = (0.0, 0.0, 100.0, 100.0);
        let boxes = vec![
            tb((0.0, 0.0, 60.0, 100.0), fig, "axes-region", "axes0"),
            tb((40.0, 0.0, 100.0, 100.0), fig, "axes-region", "axes1"),
        ];
        let (c, _) = classify(&boxes, &cfg);
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].kind, Kind::ElementElement);
    }

    #[test]
    fn sub_threshold_overlap_is_ignored() {
        let cfg = Config::default();
        let fig = (0.0, 0.0, 1000.0, 1000.0);
        // tiny 1x1 overlap between two 100x100 boxes -> fraction 0.0001 < 0.02
        let boxes = vec![
            tb((0.0, 0.0, 100.0, 100.0), fig, "tick-label", "a"),
            tb((99.0, 99.0, 199.0, 199.0), fig, "tick-label", "b"),
        ];
        let (c, _) = classify(&boxes, &cfg);
        assert!(c.is_empty());
    }
}
