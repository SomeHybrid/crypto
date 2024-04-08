#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use crate::utils::from_le_bytes;

const SIGMA: [u32; 4] = [0x61707865, 0x3320646e, 0x79622d32, 0x6b206574];

#[inline]
#[target_feature(enable = "sse2")]
unsafe fn rows_to_cols(vs: &mut [__m128i; 4]) {
    vs[2] = _mm_shuffle_epi32(vs[2], 0b_00_11_10_01);
    vs[3] = _mm_shuffle_epi32(vs[3], 0b_01_00_11_10);
    vs[0] = _mm_shuffle_epi32(vs[0], 0b_10_01_00_11);
}

#[inline]
#[target_feature(enable = "sse2")]
unsafe fn cols_to_rows(vs: &mut [__m128i; 4]) {
    vs[2] = _mm_shuffle_epi32(vs[2], 0b_10_01_00_11);
    vs[3] = _mm_shuffle_epi32(vs[3], 0b_01_00_11_10);
    vs[0] = _mm_shuffle_epi32(vs[0], 0b_00_11_10_01);
}

#[inline]
#[target_feature(enable = "sse2")]
unsafe fn rotl<const C: i32, const D: i32>(x: __m128i) -> __m128i {
    _mm_or_si128(_mm_slli_epi32(x, C), _mm_srli_epi32(x, D))
}

#[inline]
#[target_feature(enable = "sse2")]
unsafe fn permute(data: &mut [__m128i; 4]) {
    data[0] = _mm_add_epi32(data[0], data[1]);
    data[3] = _mm_xor_si128(data[3], data[0]);
    data[3] = rotl::<16, 16>(data[3]);

    data[2] = _mm_add_epi32(data[2], data[3]);
    data[1] = _mm_xor_si128(data[1], data[2]);
    data[1] = rotl::<12, 20>(data[1]);

    data[0] = _mm_add_epi32(data[0], data[1]);
    data[3] = _mm_xor_si128(data[3], data[0]);
    data[3] = rotl::<8, 24>(data[3]);

    data[2] = _mm_add_epi32(data[2], data[3]);
    data[1] = _mm_xor_si128(data[1], data[2]);
    data[1] = rotl::<7, 25>(data[1]);
}

#[inline]
#[target_feature(enable = "sse2")]
unsafe fn double_quarter_round(mut data: [__m128i; 4]) -> [__m128i; 4] {
    permute(&mut data);
    rows_to_cols(&mut data);
    permute(&mut data);
    cols_to_rows(&mut data);

    data
}

#[inline]
#[target_feature(enable = "sse2")]
pub unsafe fn rounds(data: [__m128i; 4], hchacha: bool) -> [__m128i; 4] {
    let mut stuff = data.clone();

    let original = stuff.clone();

    for _ in 0..10 {
        stuff = double_quarter_round(stuff);
    }

    if !hchacha {
        for i in 0..4 {
            stuff[i] = _mm_add_epi32(stuff[i], original[i]);
        }
    }

    let mut a = [0u32; 16];
    for i in 0..4 {
        _mm_storeu_si128(a[i * 4..].as_mut_ptr() as *mut _, stuff[i]);
    }

    stuff
}

pub struct ChaCha20 {
    state: [__m128i; 3],
}

unsafe fn _encrypt_block(block_ptr: *const __m128i, keystream: __m128i, ct_pointer: *mut __m128i) {
    let plaintext_block = _mm_loadu_si128(block_ptr);

    let ciphertext_block = _mm_xor_si128(plaintext_block, keystream);

    _mm_storeu_si128(ct_pointer, ciphertext_block);
}

unsafe fn encrypt_block(
    block: &[u8],
    keystream: [__m128i; 4],
    mut ct_pointer: *mut __m128i,
    ciphertext: &mut Vec<u8>,
) {
    let mut ptr = block.as_ptr() as *const __m128i;
    for i in 0..4 {
        if i * 16 > block.len() {
            return;
        }

        if (i + 1) * 16 > block.len() {
            let mut output_block = [0u8; 16];
            _encrypt_block(ptr, keystream[i], output_block.as_mut_ptr() as *mut __m128i);

            let start = ciphertext.len() - (ciphertext.len() % 16);
            let end = ciphertext.len();

            ciphertext[start..].copy_from_slice(&output_block[..(end - start)]);

            return;
        }

        _encrypt_block(ptr, keystream[i], ct_pointer);
        ptr = ptr.add(1);
        ct_pointer = ct_pointer.add(1);
    }
}

impl ChaCha20 {
    pub fn new(key: &[u8]) -> Self {
        unsafe {
            ChaCha20 {
                state: [
                    _mm_loadu_si128(SIGMA.as_ptr() as *const __m128i),
                    _mm_loadu_si128(key.as_ptr() as *const __m128i),
                    _mm_loadu_si128(key[16..].as_ptr() as *const __m128i),
                ],
            }
        }
    }

    unsafe fn _keystream(&self, nonce: &__m128i) -> [__m128i; 4] {
        rounds([self.state[0], self.state[1], self.state[2], *nonce], false)
    }

    unsafe fn _encrypt(&self, plaintext: &[u8], nonce: &[u8]) -> Vec<u8> {
        let nonce_block = [
            1,
            from_le_bytes(&nonce[0..4]),
            from_le_bytes(&nonce[4..8]),
            from_le_bytes(&nonce[8..12]),
        ];

        let mut nonce = _mm_loadu_si128(nonce_block.as_ptr() as *const __m128i);

        let mut ciphertext: Vec<u8> = vec![0u8; plaintext.len()];
        let mut ct_pointer = ciphertext.as_mut_ptr() as *mut __m128i;

        for block in plaintext.chunks(64) {
            let keystream = self._keystream(&nonce);

            nonce = _mm_add_epi32(nonce, _mm_set_epi32(0, 0, 0, 1));

            encrypt_block(block, keystream, ct_pointer, &mut ciphertext);

            ct_pointer = ct_pointer.add(4);
        }

        ciphertext[..plaintext.len()].to_vec()
    }
}

impl ChaCha20 {
    pub fn keystream(&self, nonce: &[u8], counter: u32) -> [u8; 64] {
        unsafe {
            let nonce_block = [
                counter,
                from_le_bytes(&nonce[0..4]),
                from_le_bytes(&nonce[4..8]),
                from_le_bytes(&nonce[8..12]),
            ];

            let nonce = _mm_loadu_si128(nonce_block.as_ptr() as *const __m128i);

            let ks = self._keystream(&nonce);

            let mut output = [0u8; 64];

            for (index, i) in ks.iter().enumerate() {
                _mm_storeu_si128((output.as_mut_ptr() as *mut __m128i).add(index), *i);
            }

            output
        }
    }

    pub fn encrypt(&self, plaintext: &[u8], nonce: &[u8]) -> Vec<u8> {
        unsafe { self._encrypt(plaintext, nonce) }
    }
}

pub struct HChaCha20 {
    state: [__m128i; 3],
}

impl HChaCha20 {
    #[inline(always)]
    pub fn new(key: &[u8]) -> HChaCha20 {
        unsafe {
            HChaCha20 {
                state: [
                    _mm_loadu_si128(SIGMA.as_ptr() as *const __m128i),
                    _mm_loadu_si128(key.as_ptr() as *const __m128i),
                    _mm_loadu_si128(key[16..].as_ptr() as *const __m128i),
                ],
            }
        }
    }

    pub fn keystream(&self, nonce: &[u8]) -> [u8; 32] {
        unsafe {
            let out_state = rounds(
                [
                    self.state[0],
                    self.state[1],
                    self.state[2],
                    _mm_loadu_si128(nonce.as_ptr() as *const __m128i),
                ],
                true,
            );

            let mut output = [0u8; 32];

            _mm_storeu_si128(output.as_mut_ptr() as *mut __m128i, out_state[0]);
            _mm_storeu_si128((output.as_mut_ptr() as *mut __m128i).add(1), out_state[3]);

            output
        }
    }
}
