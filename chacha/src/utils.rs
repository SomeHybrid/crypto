use core::simd::{u32x4, u8x64};

pub(crate) fn from_le_bytes(x: &[u8]) -> u32 {
    u32::from_le_bytes([x[0], x[1], x[2], x[3]])
}

pub(crate) fn deserialize(state: [u32x4; 4]) -> [u8; 64] {
    let mut result = [0u8; 64];

    if cfg!(target_endian = "little") {
        result = unsafe { std::mem::transmute(state) };
    } else {
        let mut vec = Vec::new();

        for chunk in state.iter() {
            for item in chunk.as_array().iter() {
                vec.extend_from_slice(&item.to_le_bytes());
            }
        }

        result[..64].clone_from_slice(&vec[..64]);
    }

    result
}

pub(crate) fn convert(state: [u32x4; 4]) -> u8x64 {
    let mut result = [0u8; 64];

    if cfg!(target_endian = "little") {
        result = unsafe { std::mem::transmute(state) };
    } else {
        let mut vec = Vec::new();
        for chunk in state.iter() {
            for item in chunk.as_array().iter() {
                vec.extend_from_slice(&item.to_le_bytes());
            }
        }

        result[..64].clone_from_slice(&vec[..64]);
    }

    u8x64::from_array(result)
}

pub(crate) fn serialize(state: [u8; 64]) -> u8x64 {
    unsafe { std::mem::transmute(state) }
}
