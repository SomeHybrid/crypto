#![feature(portable_simd)]
pub(crate) mod backend;
pub mod chacha20;
pub mod poly1305;
pub(crate) mod utils;

pub use crate::chacha20::*;
