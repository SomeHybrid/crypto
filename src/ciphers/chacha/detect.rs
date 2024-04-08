use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(all(any(target_arch="x86", target_arch="x86_64"), target_feature="avx2"))] {
        pub use crate::ciphers::chacha::backends::avx2::ChaCha20;
        pub use crate::ciphers::chacha::backends::sse2::HChaCha20;
    }
    else if #[cfg(all(any(target_arch="x86", target_arch="x86_64"), target_feature="sse2"))] {
        pub use crate::ciphers::chacha::backends::sse2::ChaCha20;
        pub use crate::ciphers::chacha::backends::sse2::HChaCha20;
    }
    else {
        pub use crate::ciphers::chacha::backends::fallback::*;
    }
}
