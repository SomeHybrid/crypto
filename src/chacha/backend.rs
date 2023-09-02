use core::simd::u32x4;

const ROUNDS: usize = 20;

fn rotl(x: &mut u32x4, c: u32) {
    let mut data = x.as_mut_array();
    for i in data.iter_mut() {
        *i = i.rotate_left(c);
    }

    *x = u32x4::from_array(*data);
}

fn quarter_round(data: &mut [u32x4; 4]) {
    data[0] += data[1];
    data[3] ^= data[0];
    rotl(&mut data[3], 16);

    data[2] += data[3];
    data[1] ^= data[2];
    rotl(&mut data[1], 12);

    data[0] += data[1];
    data[3] ^= data[0];
    rotl(&mut data[3], 8);

    data[2] += data[3];
    data[1] ^= data[2];
    rotl(&mut data[1], 7);
}

pub fn rounds(mut data: [u32x4; 4]) -> [u32x4; 4] {
    for _ in 0..ROUNDS / 2 {
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
