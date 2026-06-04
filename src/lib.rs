use pyo3::prelude::*;

mod config;
mod geometry;
mod report;

#[pyfunction]
fn _ping() -> &'static str {
    "collint"
}

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(_ping, m)?)?;
    Ok(())
}
