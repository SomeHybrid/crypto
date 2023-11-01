#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn rows_to_cols(vs: &mut [__m512i; 4]) {
    vs[0] = _mm512_shuffle_epi32(vs[0], 57);
    vs[1] = _mm512_shuffle_epi32(vs[1], 78);
    vs[2] = _mm512_shuffle_epi32(vs[2], 147);
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn cols_to_rows(vs: &mut [__m512i; 4]) {
    vs[0] = _mm512_shuffle_epi32(vs[0], 147);
    vs[1] = _mm512_shuffle_epi32(vs[1], 78);
    vs[2] = _mm512_shuffle_epi32(vs[2], 57);
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn permute(data: &mut [__m512i; 4]) {
    data[0] = _mm512_add_epi32(data[0], data[1]);
    data[3] = _mm512_xor_si512(data[3], data[0]);
    data[3] = _mm512_rol_epi32(data[3], 16);

    data[2] = _mm512_add_epi32(data[2], data[3]);
    data[1] = _mm512_xor_si512(data[1], data[2]);
    data[1] = _mm512_rol_epi32(data[1], 12);

    data[0] = _mm512_add_epi32(data[0], data[1]);
    data[3] = _mm512_xor_si512(data[3], data[0]);
    data[3] = _mm512_rol_epi32(data[3], 8);

    data[2] = _mm512_add_epi32(data[2], data[3]);
    data[1] = _mm512_xor_si512(data[1], data[2]);
    data[1] = _mm512_rol_epi32(data[1], 7);
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn double_quarter_round(mut data: [__m512i; 4]) -> [__m512i; 4] {
    permute(&mut data);
    rows_to_cols(&mut data);
    permute(&mut data);
    cols_to_rows(&mut data);

    data
}

#[inline]
#[target_feature(enable = "avx2")]
pub unsafe fn rounds(data: [__m128i; 4], rounds: usize) -> ([__m512i; 4], [__m512i; 4]) {
    let mut stuff = [_mm512_setzero_si512(); 4];

    for i in 0..4 {
        stuff[i] = _mm512_broadcast_i32x4(data[i]);
    }

    stuff[3] = _mm512_add_epi32(
        stuff[3],
        _mm512_set_epi32(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
    );

    let original = stuff.clone();

    for _ in 0..(rounds / 2) {
        stuff = double_quarter_round(stuff);
    }

    (stuff, original)
}
