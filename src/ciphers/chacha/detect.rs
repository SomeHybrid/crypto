use crate::ciphers::chacha::backends::fallback;
use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] {
        use crate::ciphers::chacha::backends::avx2;
        use crate::ciphers::chacha::backends::sse2;

        pub fn encrypt(key: Vec<u8>, plaintext: &[u8], nonce: &[u8], rounds: Option<usize>) -> Vec<u8> {
            if is_x86_feature_detected!("avx2") {
                avx2::encrypt(key, plaintext, nonce, rounds.unwrap_or(20))
            } else if is_x86_feature_detected!("sse2") {
                sse2::encrypt(key, plaintext, nonce, rounds.unwrap_or(20))
            } else {
                fallback::encrypt(key, plaintext, nonce, rounds.unwrap_or(20))
            }
        }

        pub fn decrypt(key: Vec<u8>, plaintext: &[u8], nonce: &[u8], rounds: Option<usize>) -> Vec<u8> {
            encrypt(key, plaintext, nonce, rounds)
        }

        pub fn keystream(key: Vec<u8>, nonce: &[u8], counter: u32, rounds: Option<usize>) -> [u8; 64] {
             if is_x86_feature_detected!("sse2") {
                 sse2::keystream(key, nonce, counter, rounds.unwrap_or(20))
             } else {
                 fallback::keystream(key, nonce, counter, rounds.unwrap_or(20))
             }
        }

        pub fn hchacha(key: Vec<u8>, nonce: &[u8], rounds: Option<usize>) -> [u8; 32] {
            if is_x86_feature_detected!("sse2") {
                sse2::hchacha(&key, nonce, rounds.unwrap_or(20))
            } else {
                fallback::hchacha(&key, nonce, rounds.unwrap_or(20))
            }
        }
    } else {
        pub fn encrypt(key: Vec<u8>, plaintext: &[u8], nonce: &[u8], rounds: Option<usize>) -> Vec<u8> {
            fallback::encrypt(key, plaintext, nonce, rounds.unwrap_or(20))
        }

        pub fn decrypt(key: Vec<u8>, plaintext: &[u8], nonce: &[u8], rounds: Option<usize>) -> Vec<u8> {
            fallback::encrypt(key, plaintext, nonce, rounds.unwrap_or(20))
        }

        pub fn keystream(key: Vec<u8>, nonce: &[u8], counter: u32, rounds: Option<usize>) -> [u8; 64] {
            fallback::keystream(key, nonce, counter, rounds.unwrap_or(20))
        }

        pub fn hchacha(key: Vec<u8>, nonce: &[u8]) -> [u8; 32] {
            fallback::hchacha(&key, nonce.unwrap_or(20))
        }
    }
}
