#![feature(stdsimd)]
pub(crate) mod backends;
pub mod cipher;
pub mod poly1305;
pub(crate) mod utils;

pub use crate::cipher::*;

use pyo3::prelude::*;

#[pymodule]
fn chacha20(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<ChaCha>()?;
    m.add_class::<XChaChaPoly1305>()?;
    m.add_class::<ChaChaPoly1305>()?;
    Ok(())
}
