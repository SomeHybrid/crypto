#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use crate::utils::from_le_bytes;

const SIGMA: [u32; 4] = [0x61707865, 0x3320646e, 0x79622d32, 0x6b206574];

// taken from rustcrypto/stream-ciphers
#[inline(always)]
unsafe fn rows_to_cols(vs: &mut [__m256i; 4]) {
    vs[2] = _mm256_shuffle_epi32(vs[2], 0b_00_11_10_01);
    vs[3] = _mm256_shuffle_epi32(vs[3], 0b_01_00_11_10);
    vs[0] = _mm256_shuffle_epi32(vs[0], 0b_10_01_00_11);
}

#[inline(always)]
unsafe fn cols_to_rows(vs: &mut [__m256i; 4]) {
    vs[2] = _mm256_shuffle_epi32(vs[2], 0b_10_01_00_11);
    vs[3] = _mm256_shuffle_epi32(vs[3], 0b_01_00_11_10);
    vs[0] = _mm256_shuffle_epi32(vs[0], 0b_00_11_10_01);
}

#[inline(always)]
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

#[inline(always)]
unsafe fn double_quarter_round(mut data: [__m256i; 4]) -> [__m256i; 4] {
    permute(&mut data);
    rows_to_cols(&mut data);
    permute(&mut data);
    cols_to_rows(&mut data);

    data
}

#[target_feature(enable = "avx2")]
pub unsafe fn rounds(mut items: [__m256i; 4]) -> [__m256i; 4] {
    items[3] = _mm256_add_epi32(items[3], _mm256_set_epi32(0, 0, 0, 1, 0, 0, 0, 0));
    let initial_state = items.clone();

    for _ in 0..10 {
        items = double_quarter_round(items);
    }

    for i in 0..4 {
        items[i] = _mm256_add_epi32(items[i], initial_state[i]);
    }

    let mut output = [_mm256_setzero_si256(); 4];

    output[0] = _mm256_permute2x128_si256(items[0], items[1], 0x20);
    output[1] = _mm256_permute2x128_si256(items[2], items[3], 0x20);
    output[2] = _mm256_permute2x128_si256(items[0], items[1], 0x31);
    output[3] = _mm256_permute2x128_si256(items[2], items[3], 0x31);

    output
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn encrypt_remainder(
    block_ptr: *const __m256i,
    keystream: __m256i,
    ciphertext: &mut Vec<u8>,
) {
    let plaintext_block = _mm256_loadu_si256(block_ptr);

    let ciphertext_block = _mm256_xor_si256(plaintext_block, keystream);

    let mut output_block = [0u8; 32];
    _mm256_storeu_si256(output_block.as_mut_ptr() as *mut __m256i, ciphertext_block);

    let start = ciphertext.len() - (ciphertext.len() % 32);
    let end = ciphertext.len();

    ciphertext[start..].copy_from_slice(&output_block[..(end - start)]);
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn encrypt_block(
    block: &[u8],
    keystream: [__m256i; 4],
    ciphertext: &mut Vec<u8>,
    mut ct_ptr: *mut __m256i,
) {
    let mut ptr = block.as_ptr() as *const __m256i;

    for i in 0..4 {
        if i * 32 > block.len() {
            return;
        }
        if (i + 1) * 32 > block.len() {
            encrypt_remainder(ptr, keystream[i], ciphertext);
            continue;
        }

        let plaintext_block = _mm256_loadu_si256(ptr);

        let ciphertext_block = _mm256_xor_si256(plaintext_block, keystream[i]);

        _mm256_storeu_si256(ct_ptr, ciphertext_block);

        ptr = ptr.add(1);
        ct_ptr = ct_ptr.add(1);
    }
}

pub struct ChaCha20 {
    state: [__m256i; 3],
}

// lower level functions
impl ChaCha20 {
    #[inline(always)]
    pub fn new(key: &[u8]) -> ChaCha20 {
        unsafe {
            let s0 = _mm256_broadcastsi128_si256(_mm_loadu_si128(SIGMA.as_ptr() as *const __m128i));

            let mut s1 = _mm256_loadu_si256(key.as_ptr() as *const __m256i);
            let s2 = _mm256_permute2x128_si256(s1, s1, 0x11);
            s1 = _mm256_permute2x128_si256(s1, s1, 0x00);

            ChaCha20 {
                state: [s0, s1, s2],
            }
        }
    }

    /// Generates a keystream block. Should not be used.
    #[inline]
    #[target_feature(enable = "avx2")]
    unsafe fn _keystream(&self, nonce: __m256i) -> [__m256i; 4] {
        let state = [self.state[0], self.state[1], self.state[2], nonce];
        rounds(state)
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

        let mut ciphertext: Vec<u8> = vec![0u8; plaintext.len()];
        let mut ct_ptr = ciphertext.as_mut_ptr() as *mut __m256i;

        for block in plaintext.chunks(128) {
            let keystream = self._keystream(nonce);

            nonce = _mm256_add_epi32(nonce, _mm256_set_epi32(0, 0, 0, 2, 0, 0, 0, 2));

            encrypt_block(block, keystream, &mut ciphertext, ct_ptr);
            ct_ptr = ct_ptr.add(4);
        }

        ciphertext
    }
}

impl ChaCha20 {
    pub fn encrypt(&self, plaintext: &[u8], nonce: &[u8]) -> Vec<u8> {
        unsafe { self._encrypt(plaintext, nonce) }
    }

    pub fn keystream(&self, nonce: &[u8], counter: u32) -> [u8; 64] {
        let mut nonce_block = [
            counter,
            from_le_bytes(&nonce[0..4]),
            from_le_bytes(&nonce[4..8]),
            from_le_bytes(&nonce[8..12]),
        ];

        unsafe {
            let nonce = _mm256_broadcastsi128_si256(_mm_loadu_si128(
                nonce_block.as_mut_ptr() as *mut __m128i
            ));

            let keystream = self._keystream(nonce);

            let mut output = [0u8; 64];

            _mm256_storeu_si256(output.as_mut_ptr() as *mut __m256i, keystream[0]);
            _mm256_storeu_si256((output.as_mut_ptr() as *mut __m256i).add(1), keystream[1]);

            output
        }
    }
}
