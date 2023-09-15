#![feature(portable_simd)]
pub(crate) mod backend;
pub mod chacha20;
pub mod poly1305;
pub(crate) mod utils;

pub use crate::chacha20::*;

use pyo3::prelude::*;

#[pymodule]
fn chacha(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<ChaCha>()?;
    m.add_class::<XChaChaPoly1305>()?;
    m.add_class::<ChaChaPoly1305>()?;

    m.add_wrapped(wrap_pyfunction!(encrypt))?;
    m.add_wrapped(wrap_pyfunction!(decrypt))?;
    Ok(())
}
