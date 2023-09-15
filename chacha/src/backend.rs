use core::simd::u32x4;
use std::mem;

use std::arch::is_aarch64_feature_detected;

#[target_feature(enable = "neon")]
#[cfg(target_arch = "aarch64")]
unsafe fn _rotl_neon<const C: i32, const D: i32>(x: &mut u32x4) -> u32x4 {
    use core::arch::aarch64::*;
    let data: uint32x4_t = (*x).into();
    mem::transmute(veorq_u32(
        vshlq_n_u32(data.clone(), C),
        vshrq_n_u32(data, D),
    ))
}

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    any(target_feature = "avx2", target_feature = "sse2")
))]
unsafe fn _rotl_x86<const C: i32, const D: i32>(x: &mut u32x4) -> u32x4 {
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::*;
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    let data: __m128i = (*x).into();

    mem::transmute(_mm_xor_si128(
        _mm_slli_epi32(data.clone(), C),
        _mm_srli_epi32(data, D),
    ))
}

fn rotl<const C: i32, const D: i32>(x: &mut u32x4) {
    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    if is_x86_feature_detected!("sse2") || is_x86_feature_detected!("avx2") {
        *x = unsafe { _rotl_x86::<C, D>(x) };

        return;
    }

    #[cfg(target_arch = "aarch64")]
    if is_aarch64_feature_detected!("neon") {
        *x = unsafe { _rotl_neon::<C, D>(x) };
        return;
    }

    let data = x.as_mut_array();
    for i in data.iter_mut() {
        *i = i.rotate_left(C as u32);
    }

    *x = u32x4::from_array(*data);
}

fn quarter_round(data: &mut [u32x4; 4]) {
    data[0] += data[1];
    data[3] ^= data[0];
    rotl::<16, 16>(&mut data[3]);

    data[2] += data[3];
    data[1] ^= data[2];
    rotl::<12, 20>(&mut data[1]);

    data[0] += data[1];
    data[3] ^= data[0];
    rotl::<8, 24>(&mut data[3]);

    data[2] += data[3];
    data[1] ^= data[2];
    rotl::<7, 25>(&mut data[1]);
}

pub fn rounds(mut data: [u32x4; 4], rounds: usize) -> [u32x4; 4] {
    for _ in 0..rounds / 2 {
        quarter_round(&mut data);

        data[1] = data[1].rotate_lanes_left::<1>();
        data[2] = data[2].rotate_lanes_left::<2>();
        data[3] = data[3].rotate_lanes_left::<3>();

        quarter_round(&mut data);

        data[1] = data[1].rotate_lanes_right::<1>();
        data[2] = data[2].rotate_lanes_right::<2>();
        data[3] = data[3].rotate_lanes_right::<3>();
    }

    data
}
