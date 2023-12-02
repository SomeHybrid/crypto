#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use crate::utils::from_le_bytes;

const SIGMA: [u32; 4] = [0x61707865, 0x3320646e, 0x79622d32, 0x6b206574];
// taken from rustcrypto/stream-ciphers
#[inline]
#[target_feature(enable = "avx2")]
unsafe fn rows_to_cols(vs: &mut [__m256i; 4]) {
    vs[2] = _mm256_shuffle_epi32(vs[2], 0b_00_11_10_01);
    vs[3] = _mm256_shuffle_epi32(vs[3], 0b_01_00_11_10);
    vs[0] = _mm256_shuffle_epi32(vs[0], 0b_10_01_00_11);
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn cols_to_rows(vs: &mut [__m256i; 4]) {
    vs[2] = _mm256_shuffle_epi32(vs[2], 0b_10_01_00_11);
    vs[3] = _mm256_shuffle_epi32(vs[3], 0b_01_00_11_10);
    vs[0] = _mm256_shuffle_epi32(vs[0], 0b_00_11_10_01);
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn permute(data: &mut [__m256i; 4]) {
    data[0] = _mm256_add_epi32(data[0], data[1]);
    data[3] = _mm256_xor_si256(data[3], data[0]);
    data[3] = _mm256_xor_si256(
        _mm256_slli_epi32(data[3], 16),
        _mm256_srli_epi32(data[3], 16),
    );

    data[2] = _mm256_add_epi32(data[2], data[3]);
    data[1] = _mm256_xor_si256(data[1], data[2]);
    data[1] = _mm256_xor_si256(
        _mm256_slli_epi32(data[1], 12),
        _mm256_srli_epi32(data[1], 20),
    );

    data[0] = _mm256_add_epi32(data[0], data[1]);
    data[3] = _mm256_xor_si256(data[3], data[0]);
    data[3] = _mm256_xor_si256(
        _mm256_slli_epi32(data[3], 8),
        _mm256_srli_epi32(data[3], 24),
    );

    data[2] = _mm256_add_epi32(data[2], data[3]);
    data[1] = _mm256_xor_si256(data[1], data[2]);
    data[1] = _mm256_xor_si256(
        _mm256_slli_epi32(data[1], 7),
        _mm256_srli_epi32(data[1], 25),
    );
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn double_quarter_round(mut data: [__m256i; 4]) -> [__m256i; 4] {
    permute(&mut data);
    rows_to_cols(&mut data);
    permute(&mut data);
    cols_to_rows(&mut data);

    data
}

#[inline]
#[target_feature(enable = "avx2")]
pub unsafe fn rounds(mut items: [__m256i; 4], rounds: usize) -> [__m256i; 4] {
    items[3] = _mm256_add_epi32(items[3], _mm256_set_epi32(0, 0, 0, 1, 0, 0, 0, 0));
    let initial_state = items.clone();

    for _ in 0..rounds {
        items = double_quarter_round(items);
    }

    for i in 0..4 {
        items[i] = _mm256_add_epi32(items[i], initial_state[i]);
    }

    let mut output = [_mm256_setzero_si256(); 4];

    output[0] = _mm256_inserti128_si256(items[0], _mm256_extracti128_si256(items[1], 0), 1);
    output[1] = _mm256_inserti128_si256(items[2], _mm256_extracti128_si256(items[3], 0), 1);
    output[2] = _mm256_inserti128_si256(items[1], _mm256_extracti128_si256(items[0], 1), 0);
    output[3] = _mm256_inserti128_si256(items[3], _mm256_extracti128_si256(items[2], 1), 0);

    output
}

unsafe fn encrypt_block(block: &[u8], keystream: [__m256i; 4], ciphertext: &mut Vec<u8>) {
    for i in 0..4 {
        if i * 32 > block.len() {
            break;
        }

        let plaintext_block = _mm256_loadu_si256(block[i * 32..].as_ptr() as *const __m256i);

        let ciphertext_block = _mm256_xor_si256(plaintext_block, keystream[i]);

        let mut output_block = [0u8; 32];
        _mm256_storeu_si256(output_block.as_mut_ptr() as *mut __m256i, ciphertext_block);

        ciphertext.append(&mut output_block.to_vec());
    }
}

pub struct Backend {
    state: [__m256i; 3],
    rounds: usize,
}

// lower level functions
impl Backend {
    #[inline]
    #[target_feature(enable = "avx2")]
    unsafe fn _new(key: Vec<u8>, rounds: usize) -> Backend {
        let s0 = _mm256_broadcastsi128_si256(_mm_loadu_si128(SIGMA.as_ptr() as *const __m128i));

        let mut s1 = _mm256_loadu_si256(key.as_ptr() as *const __m256i);
        let s2 = _mm256_permute2x128_si256(s1, s1, 0x11);
        s1 = _mm256_permute2x128_si256(s1, s1, 0x00);

        Backend {
            state: [s0, s1, s2],
            rounds: rounds / 2,
        }
    }

    pub fn new(key: Vec<u8>, rounds: usize) -> Backend {
        unsafe { Backend::_new(key, rounds) }
    }

    /// Generates a keystream block. Should not be used.
    #[inline]
    #[target_feature(enable = "avx2")]
    unsafe fn _keystream(&self, nonce: __m256i) -> [__m256i; 4] {
        let state = [self.state[0], self.state[1], self.state[2], nonce];
        rounds(state, self.rounds)
    }

    #[inline]
    #[target_feature(enable = "avx2")]
    pub unsafe fn _encrypt(&self, plaintext: &[u8], nonce: &[u8]) -> Vec<u8> {
        let nonce_block = [
            1,
            from_le_bytes(&nonce[0..4]),
            from_le_bytes(&nonce[4..8]),
            from_le_bytes(&nonce[8..12]),
        ];

        let nonce_vector = _mm_loadu_si128(nonce_block.as_ptr() as *const __m128i);

        let mut nonce = _mm256_broadcastsi128_si256(nonce_vector);

        let mut ciphertext: Vec<u8> = Vec::new();

        for block in plaintext.chunks(128) {
            let keystream = self._keystream(nonce);

            nonce = _mm256_add_epi32(nonce, _mm256_set_epi32(0, 0, 0, 2, 0, 0, 0, 2));

            encrypt_block(block, keystream, &mut ciphertext);
        }

        ciphertext[..plaintext.len()].to_vec()
    }
}

impl Backend {
    pub fn encrypt(&self, plaintext: &[u8], nonce: &[u8]) -> Vec<u8> {
        unsafe { self._encrypt(plaintext, nonce) }
    }
}

pub(crate) fn encrypt(key: Vec<u8>, plaintext: &[u8], nonce: &[u8], rounds: usize) -> Vec<u8> {
    Backend::new(key, rounds).encrypt(plaintext, nonce)
}

