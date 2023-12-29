use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(all(any(target_arch="x86", target_arch="x86_64"), target_feature="avx2"))] {
        pub use crate::ciphers::chacha::backends::avx2::ChaCha;
        pub use crate::ciphers::chacha::backends::sse2::hchacha;
    }
    else if #[cfg(all(any(target_arch="x86", target_arch="x86_64"), target_feature="sse2"))] {
        pub use crate::ciphers::chacha::backends::sse2::ChaCha;
        pub use crate::ciphers::chacha::backends::sse2::hchacha;
    }
    else {
        pub use crate::ciphers::chacha::backends::fallback::*;
    }
}

pub fn encrypt(key: &[u8], plaintext: &[u8], nonce: &[u8], rounds: Option<usize>) -> Vec<u8> {
    ChaCha::new(key, rounds).encrypt(plaintext, nonce)
}

pub fn decrypt(key: &[u8], plaintext: &[u8], nonce: &[u8], rounds: Option<usize>) -> Vec<u8> {
    ChaCha::new(key, rounds).encrypt(plaintext, nonce)
}

pub fn keystream(key: &[u8], nonce: &[u8], counter: u32, rounds: Option<usize>) -> [u8; 64] {
    ChaCha::new(key, rounds).keystream(nonce, counter)
}
