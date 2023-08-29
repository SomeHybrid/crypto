pub mod chacha;
pub mod x25519;
pub mod poly1305;
pub mod util;

use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::wrap_pymodule;

#[pymodule]
fn encryption(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pymodule!(chacha::chacha))?;

    // inject into sys.modules
    let sys = PyModule::import(py, "sys")?;
    let sys_modules: &PyDict = sys.getattr("modules")?.downcast()?;
    sys_modules.set_item("encryption.chacha", m.getattr("chacha")?)?;
   Ok(())
}
