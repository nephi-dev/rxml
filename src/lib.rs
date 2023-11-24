use pyo3::prelude::*;
mod common;
mod entities;
mod read;
mod write;

#[pymodule]
fn rxml(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<entities::Node>()?;
    m.add_function(wrap_pyfunction!(read::read_file, m)?)?;
    m.add_function(wrap_pyfunction!(read::read_string, m)?)?;
    m.add_function(wrap_pyfunction!(write::write_file, m)?)?;
    m.add_function(wrap_pyfunction!(write::write_string, m)?)?;
    Ok(())
}
