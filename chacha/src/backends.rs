mod avx2;
mod sse2;

use std::arch::{is_aarch64_feature_detected, is_x86_feature_detected};
use std::mem;

unsafe fn _rounds(data: [[u32; 4]; 4], rounds: usize, hchacha: bool) -> [u8; 128] {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    let stuff = [
        _mm_loadu_si128(data[0].as_ptr() as *const __m128i),
        _mm_loadu_si128(data[1].as_ptr() as *const __m128i),
        _mm_loadu_si128(data[2].as_ptr() as *const __m128i),
        _mm_loadu_si128(data[3].as_ptr() as *const __m128i),
    ];
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    let stuff = data.clone();

    let mut output = [0u8; 128];

    if is_x86_feature_detected!("avx2") {
        let (mut _output, _original) = unsafe { avx2::rounds(stuff, rounds) };

        for (vec, other) in _output.iter_mut().zip(_original.iter()) {
            *vec = _mm256_add_epi32(*vec, *other);
        }

        for i in 0..4 {
            _mm256_storeu_si256(output.as_ptr().add(i) as *mut _, _output[i])
        }
    } else if is_x86_feature_detected!("sse2") {
        let stuff1 = unsafe { sse2::rounds(stuff.clone(), rounds, hchacha) };

        _mm_storeu_si128(output.as_ptr() as *mut _, stuff1[0]);
    }

    output
}

pub fn rounds(data: [[u32; 4]; 4], rounds: usize, hchacha: bool) -> [u8; 128] {
    let mut stuff = unsafe { _rounds(data, rounds, hchacha) };

    unsafe { mem::transmute(stuff) }
}
