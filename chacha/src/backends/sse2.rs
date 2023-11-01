#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[target_feature(enable = "sse2")]
unsafe fn rows_to_cols(vs: &mut [__m128i; 4]) {
    vs[0] = _mm_shuffle_epi32(vs[0], 0b_00_11_10_01);
    vs[1] = _mm_shuffle_epi32(vs[1], 0b_01_00_11_10);
    vs[2] = _mm_shuffle_epi32(vs[2], 0b_10_01_00_11);
}

#[inline]
#[target_feature(enable = "sse2")]
unsafe fn cols_to_rows(vs: &mut [__m128i; 4]) {
    vs[0] = _mm_shuffle_epi32(vs[0], 0b_10_01_00_11);
    vs[1] = _mm_shuffle_epi32(vs[1], 0b_01_00_11_10);
    vs[2] = _mm_shuffle_epi32(vs[2], 0b_00_11_10_01);
}

#[inline]
#[target_feature(enable = "sse2")]
unsafe fn rotl<const c: i32, const d: i32>(x: __m128i) -> __m128i {
    _mm_xor_si128(_mm_slli_epi32(x, c), _mm_srli_epi32(x, d))
}

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
    data[3] = rotl::<8, 16>(data[3]);

    data[2] = _mm_add_epi32(data[2], data[3]);
    data[1] = _mm_xor_si128(data[1], data[2]);
    data[1] = rotl::<12, 20>(data[1]);
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
pub unsafe fn rounds(data: [__m128i; 4], rounds: usize, hchacha: bool) -> [__m128i; 4] {
    let mut stuff = data.clone();

    let original = stuff.clone();

    for _ in 0..(rounds / 2) {
        stuff = double_quarter_round(stuff);
    }

    if !hchacha {
        for i in 0..4 {
            stuff[i] = _mm_add_epi32(stuff[i], original[i]);
        }
    }

    stuff
}
