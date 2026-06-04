//! Collision records and the report. Pure Rust + serde; pyo3 wrappers added in lib.rs.

use serde::Serialize;

pub type Box4 = (f64, f64, f64, f64);

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    TextText,
    Clipping,
    OverData,
    ElementElement,
}

impl Kind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Kind::TextText => "text_text",
            Kind::Clipping => "clipping",
            Kind::OverData => "over_data",
            Kind::ElementElement => "element_element",
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Collision {
    pub kind: Kind,
    pub a_role: String,
    pub a_id: String,
    pub a_box: Box4,
    pub b_role: String,
    pub b_id: String,
    pub b_box: Box4,
    /// Overlap fraction for overlaps, or clip overhang in px for clipping.
    pub severity: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct Report {
    pub collisions: Vec<Collision>,
}

#[allow(dead_code)] // mirrored by the pyo3 CollisionReport; exercised by unit tests
impl Report {
    pub fn has_collisions(&self) -> bool {
        !self.collisions.is_empty()
    }

    /// Max severity across all collisions (0.0 if none).
    pub fn severity(&self) -> f64 {
        self.collisions.iter().map(|c| c.severity).fold(0.0, f64::max)
    }

    /// Total overlap-style severity, used by autofix to compare two figures.
    pub fn total(&self) -> f64 {
        self.collisions.iter().map(|c| c.severity).sum()
    }

    pub fn exit_code(&self) -> i32 {
        if self.has_collisions() {
            1
        } else {
            0
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.collisions).unwrap_or_else(|_| "[]".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(kind: Kind, severity: f64) -> Collision {
        Collision {
            kind,
            a_role: "legend".into(),
            a_id: "0".into(),
            a_box: (0.0, 0.0, 1.0, 1.0),
            b_role: "data-artist".into(),
            b_id: "1".into(),
            b_box: (0.0, 0.0, 1.0, 1.0),
            severity,
        }
    }

    #[test]
    fn empty_report_is_clean() {
        let r = Report { collisions: vec![] };
        assert!(!r.has_collisions());
        assert_eq!(r.exit_code(), 0);
        assert_eq!(r.severity(), 0.0);
    }

    #[test]
    fn nonempty_report_flags_and_reports_max_severity() {
        let r = Report {
            collisions: vec![sample(Kind::OverData, 0.3), sample(Kind::TextText, 0.7)],
        };
        assert!(r.has_collisions());
        assert_eq!(r.exit_code(), 1);
        assert_eq!(r.severity(), 0.7);
        assert!((r.total() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn json_contains_kind_string() {
        let r = Report { collisions: vec![sample(Kind::OverData, 0.3)] };
        assert!(r.to_json().contains("over_data"));
    }
}
