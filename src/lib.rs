use numpy::PyReadonlyArray3;
use pyo3::prelude::*;

mod config;
mod geometry;
mod pixel;
mod report;
mod rules;

use config::Config;
use rules::TaggedBox as RTaggedBox;

/// Build a Config from optional overrides (None -> default).
fn make_config(
    overlap_fraction: Option<f64>,
    clip_px: Option<f64>,
    ink_fraction: Option<f64>,
    bg_tolerance: Option<u8>,
) -> Config {
    let d = Config::default();
    Config {
        overlap_fraction: overlap_fraction.unwrap_or(d.overlap_fraction),
        clip_px: clip_px.unwrap_or(d.clip_px),
        ink_fraction: ink_fraction.unwrap_or(d.ink_fraction),
        bg_tolerance: bg_tolerance.unwrap_or(d.bg_tolerance),
    }
}

/// A collision exposed to Python.
#[pyclass]
#[derive(Clone)]
struct Collision {
    #[pyo3(get)]
    kind: String,
    #[pyo3(get)]
    a_role: String,
    #[pyo3(get)]
    a_id: String,
    #[pyo3(get)]
    a_box: (f64, f64, f64, f64),
    #[pyo3(get)]
    b_role: String,
    #[pyo3(get)]
    b_id: String,
    #[pyo3(get)]
    b_box: (f64, f64, f64, f64),
    #[pyo3(get)]
    severity: f64,
}

impl From<report::Collision> for Collision {
    fn from(c: report::Collision) -> Self {
        Collision {
            kind: c.kind.as_str().to_string(),
            a_role: c.a_role,
            a_id: c.a_id,
            a_box: c.a_box,
            b_role: c.b_role,
            b_id: c.b_id,
            b_box: c.b_box,
            severity: c.severity,
        }
    }
}

#[pymethods]
impl Collision {
    #[new]
    fn new(
        kind: String,
        a_role: String,
        a_id: String,
        a_box: (f64, f64, f64, f64),
        b_role: String,
        b_id: String,
        b_box: (f64, f64, f64, f64),
        severity: f64,
    ) -> Self {
        Collision { kind, a_role, a_id, a_box, b_role, b_id, b_box, severity }
    }

    fn __repr__(&self) -> String {
        format!("Collision(kind={}, severity={:.3})", self.kind, self.severity)
    }
}

/// An over-data candidate handed back to Python for pixel confirmation.
#[pyclass]
#[derive(Clone)]
struct OverDataCandidate {
    #[pyo3(get)]
    decoration_id: String,
    #[pyo3(get)]
    decoration_role: String,
    #[pyo3(get)]
    decoration_box: (f64, f64, f64, f64),
    #[pyo3(get)]
    data_id: String,
    #[pyo3(get)]
    data_box: (f64, f64, f64, f64),
    #[pyo3(get)]
    overlap: f64,
}

/// The report exposed to Python.
#[pyclass]
struct CollisionReport {
    #[pyo3(get)]
    collisions: Vec<Collision>,
}

#[pymethods]
impl CollisionReport {
    #[new]
    fn new(collisions: Vec<Collision>) -> Self {
        CollisionReport { collisions }
    }

    #[getter]
    fn has_collisions(&self) -> bool {
        !self.collisions.is_empty()
    }

    #[getter]
    fn severity(&self) -> f64 {
        self.collisions.iter().map(|c| c.severity).fold(0.0, f64::max)
    }

    fn total(&self) -> f64 {
        self.collisions.iter().map(|c| c.severity).sum()
    }

    fn exit_code(&self) -> i32 {
        if self.collisions.is_empty() { 0 } else { 1 }
    }

    fn by_type(&self, kind: &str) -> Vec<Collision> {
        self.collisions.iter().filter(|c| c.kind == kind).cloned().collect()
    }

    fn to_json(&self) -> String {
        let recs: Vec<report::Collision> = self
            .collisions
            .iter()
            .map(|c| report::Collision {
                kind: match c.kind.as_str() {
                    "text_text" => report::Kind::TextText,
                    "clipping" => report::Kind::Clipping,
                    "over_data" => report::Kind::OverData,
                    _ => report::Kind::ElementElement,
                },
                a_role: c.a_role.clone(),
                a_id: c.a_id.clone(),
                a_box: c.a_box,
                b_role: c.b_role.clone(),
                b_id: c.b_id.clone(),
                b_box: c.b_box,
                severity: c.severity,
            })
            .collect();
        report::Report { collisions: recs }.to_json()
    }
}

/// Classify tagged boxes. Each input tuple is
/// (bx, clip, role, id) where bx/clip are (x0,y0,x1,y1).
#[pyfunction]
#[pyo3(signature = (boxes, overlap_fraction=None, clip_px=None))]
fn classify(
    boxes: Vec<((f64, f64, f64, f64), (f64, f64, f64, f64), String, String)>,
    overlap_fraction: Option<f64>,
    clip_px: Option<f64>,
) -> (Vec<Collision>, Vec<OverDataCandidate>) {
    let cfg = make_config(overlap_fraction, clip_px, None, None);
    let rboxes: Vec<RTaggedBox> = boxes
        .into_iter()
        .map(|(bx, clip, role, id)| RTaggedBox { bx, clip, role, id })
        .collect();
    let (collisions, candidates) = rules::classify(&rboxes, &cfg);
    let pc: Vec<Collision> = collisions.into_iter().map(Collision::from).collect();
    let cand: Vec<OverDataCandidate> = candidates
        .into_iter()
        .map(|c| OverDataCandidate {
            decoration_id: c.decoration_id,
            decoration_role: c.decoration_role,
            decoration_box: c.decoration_box,
            data_id: c.data_id,
            data_box: c.data_box,
            overlap: c.overlap,
        })
        .collect();
    (pc, cand)
}

/// Foreground ink coverage of a region of an RGBA buffer.
/// region is (row0, col0, row1, col1) in buffer pixels.
#[pyfunction]
#[pyo3(signature = (buffer, region, bg, tol=8))]
fn ink_coverage(
    py: Python<'_>,
    buffer: PyReadonlyArray3<u8>,
    region: (usize, usize, usize, usize),
    bg: (u8, u8, u8),
    tol: u8,
) -> f64 {
    let view = buffer.as_array();
    py.allow_threads(|| pixel::ink_coverage(&view, region, [bg.0, bg.1, bg.2], tol))
}

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Collision>()?;
    m.add_class::<OverDataCandidate>()?;
    m.add_class::<CollisionReport>()?;
    m.add_function(wrap_pyfunction!(classify, m)?)?;
    m.add_function(wrap_pyfunction!(ink_coverage, m)?)?;
    Ok(())
}
