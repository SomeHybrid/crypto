#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

// taken from rustcrypto/stream-ciphers
#[inline]
#[target_feature(enable = "avx2")]
unsafe fn rows_to_cols(vs: &mut [__m256i; 4]) {
    vs[0] = _mm256_shuffle_epi32(vs[0], 0b_00_11_10_01);
    vs[1] = _mm256_shuffle_epi32(vs[1], 0b_01_00_11_10);
    vs[2] = _mm256_shuffle_epi32(vs[2], 0b_10_01_00_11);
}

// taken from rustcrypto/stream-ciphers
#[inline]
#[target_feature(enable = "avx2")]
unsafe fn cols_to_rows(vs: &mut [__m256i; 4]) {
    vs[0] = _mm256_shuffle_epi32(vs[0], 0b_10_01_00_11);
    vs[1] = _mm256_shuffle_epi32(vs[1], 0b_01_00_11_10);
    vs[2] = _mm256_shuffle_epi32(vs[2], 0b_00_11_10_01);
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
pub unsafe fn rounds(data: [__m128i; 4], rounds: usize) -> ([__m256i; 4], [__m256i; 4]) {
    let mut stuff = [_mm256_setzero_si256(); 4];

    for i in 0..4 {
        stuff[i] = _mm256_broadcastsi128_si256(data[i]);
    }

    stuff[3] = _mm256_add_epi32(stuff[3], _mm256_set_epi32(0, 0, 0, 1, 0, 0, 0, 0));

    let original = stuff.clone();

    for _ in 0..(rounds / 2) {
        stuff = double_quarter_round(stuff);
    }

    (stuff, original)
}
