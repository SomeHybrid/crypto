use crate::utils::const_time_eq;
use core::ops::{Add, Index, IndexMut, Mul, Sub};
use zeroize::{Zeroize, ZeroizeOnDrop};

fn load3(s: &[u8]) -> i64 {
    let mut result = s[0] as u64;
    result |= (s[1] as u64) << 8;
    result |= (s[2] as u64) << 16;

    result as i64
}

fn load4(s: &[u8]) -> i64 {
    let mut result = s[0] as u64;
    result |= (s[1] as u64) << 8;
    result |= (s[2] as u64) << 16;
    result |= (s[3] as u64) << 24;

    result as i64
}

#[derive(Clone, Copy, Zeroize)]
pub struct FieldElement(pub(crate) [i32; 10]);

impl PartialEq for FieldElement {
    fn eq(&self, other: &FieldElement) -> bool {
        self.0 == other.0
    }
}

impl Eq for FieldElement {}

impl FieldElement {
    pub fn zero() -> FieldElement {
        FieldElement([0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
    }

    pub fn one() -> FieldElement {
        FieldElement([1, 0, 0, 0, 0, 0, 0, 0, 0, 0])
    }

    pub const fn from(data: [i32; 10]) -> FieldElement {
        FieldElement(data)
    }

    pub fn from_bytes(s: &[u8]) -> FieldElement {
        let mut h0 = load4(&s[0..4]);
        let mut h1 = load3(&s[4..7]) << 6;
        let mut h2 = load3(&s[7..10]) << 5;
        let mut h3 = load3(&s[10..13]) << 3;
        let mut h4 = load3(&s[13..16]) << 2;
        let mut h5 = load4(&s[16..20]);
        let mut h6 = load3(&s[20..23]) << 7;
        let mut h7 = load3(&s[23..26]) << 5;
        let mut h8 = load3(&s[26..29]) << 4;
        let mut h9 = (load3(&s[29..32]) & 8388607) << 2;

        let carry9 = (h9 + (1 << 24)) >> 25;
        h0 += carry9 * 19;
        h9 -= carry9 << 25;
        let carry1 = (h1 + (1 << 24)) >> 25;
        h2 += carry1;
        h1 -= carry1 << 25;
        let carry3 = (h3 + (1 << 24)) >> 25;
        h4 += carry3;
        h3 -= carry3 << 25;
        let carry5 = (h5 + (1 << 24)) >> 25;
        h6 += carry5;
        h5 -= carry5 << 25;
        let carry7 = (h7 + (1 << 24)) >> 25;
        h8 += carry7;
        h7 -= carry7 << 25;

        let carry0 = (h0 + (1 << 25)) >> 26;
        h1 += carry0;
        h0 -= carry0 << 26;
        let carry2 = (h2 + (1 << 25)) >> 26;
        h3 += carry2;
        h2 -= carry2 << 26;
        let carry4 = (h4 + (1 << 25)) >> 26;
        h5 += carry4;
        h4 -= carry4 << 26;
        let carry6 = (h6 + (1 << 25)) >> 26;
        h7 += carry6;
        h6 -= carry6 << 26;
        let carry8 = (h8 + (1 << 25)) >> 26;
        h9 += carry8;
        h8 -= carry8 << 26;

        FieldElement([
            h0 as i32, h1 as i32, h2 as i32, h3 as i32, h4 as i32, h5 as i32, h6 as i32, h7 as i32,
            h8 as i32, h9 as i32,
        ])
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        let mut h = self.0.clone();

        let mut q = (19 * h[9] + (1 << 24)) >> 25;

        for i in 0..10 {
            let shift = if i & 1 == 1 { 25 } else { 26 };
            q = (q + h[i]) >> shift;
        }

        h[0] += 19 * q;

        for i in 0..10 {
            let shift = if i & 1 == 1 { 25 } else { 26 };
            let carry = h[i] >> shift;
            if i != 9 {
                h[i + 1] += carry;
            }

            h[i] -= carry << shift;
        }

        [
            (h[0] >> 0) as u8,
            (h[0] >> 8) as u8,
            (h[0] >> 16) as u8,
            ((h[0] >> 24) | (h[1] << 2)) as u8,
            (h[1] >> 6) as u8,
            (h[1] >> 14) as u8,
            ((h[1] >> 22) | (h[2] << 3)) as u8,
            (h[2] >> 5) as u8,
            (h[2] >> 13) as u8,
            ((h[2] >> 21) | (h[3] << 5)) as u8,
            (h[3] >> 3) as u8,
            (h[3] >> 11) as u8,
            ((h[3] >> 19) | (h[4] << 6)) as u8,
            (h[4] >> 2) as u8,
            (h[4] >> 10) as u8,
            (h[4] >> 18) as u8,
            (h[5] >> 0) as u8,
            (h[5] >> 8) as u8,
            (h[5] >> 16) as u8,
            ((h[5] >> 24) | (h[6] << 1)) as u8,
            (h[6] >> 7) as u8,
            (h[6] >> 15) as u8,
            ((h[6] >> 23) | (h[7] << 3)) as u8,
            (h[7] >> 5) as u8,
            (h[7] >> 13) as u8,
            ((h[7] >> 21) | (h[8] << 4)) as u8,
            (h[8] >> 4) as u8,
            (h[8] >> 12) as u8,
            ((h[8] >> 20) | (h[9] << 6)) as u8,
            (h[9] >> 2) as u8,
            (h[9] >> 10) as u8,
            (h[9] >> 18) as u8,
        ]
    }

    pub fn swap(&mut self, other: &mut FieldElement, swap: i32) {
        let mut f = self.0.clone();
        let mut g = other.0.clone();
        let mut x = [0i32; 10];

        for i in 0..10 {
            x[i] = f[i] ^ g[i];
        }

        let b = -swap;
        for i in 0..10 {
            x[i] &= b;
        }

        for i in 0..10 {
            f[i] ^= x[i];
            g[i] ^= x[i];
        }

        *self = FieldElement(f);
        *other = FieldElement(g);
    }

    pub fn mov(&self, other: FieldElement) -> FieldElement {
        let f = self.0.clone();
        let mut g = other.0.clone();
        let mut x = [0i32; 10];

        for i in 0..10 {
            x[i] = f[i] ^ g[i];
        }

        for i in 0..10 {
            g[i] ^= x[i];
        }

        FieldElement(g)
    }

    pub fn maybe_set(&mut self, other: &FieldElement, swap: i32) {
        let mut x = [0i32; 10];

        let b = -swap;

        for i in 0..10 {
            x[i] = (&self)[i] ^ (&other)[i];
            x[i] &= b;
        }

        let mut output = FieldElement::zero();

        for i in 0..10 {
            output[i] = (&self)[i] ^ (&other)[i];
        }

        *self = output
    }

    pub fn square_and_double(&self) -> FieldElement {
        let f0 = self[0];
        let f1 = self[1];
        let f2 = self[2];
        let f3 = self[3];
        let f4 = self[4];
        let f5 = self[5];
        let f6 = self[6];
        let f7 = self[7];
        let f8 = self[8];
        let f9 = self[9];
        let f0_2 = 2 * f0;
        let f1_2 = 2 * f1;
        let f2_2 = 2 * f2;
        let f3_2 = 2 * f3;
        let f4_2 = 2 * f4;
        let f5_2 = 2 * f5;
        let f6_2 = 2 * f6;
        let f7_2 = 2 * f7;
        let f5_38 = 38 * f5; /* 1.959375*2^30 */
        let f6_19 = 19 * f6; /* 1.959375*2^30 */
        let f7_38 = 38 * f7; /* 1.959375*2^30 */
        let f8_19 = 19 * f8; /* 1.959375*2^30 */
        let f9_38 = 38 * f9; /* 1.959375*2^30 */
        let f0f0 = (f0 as i64) * (f0 as i64);
        let f0f1_2 = (f0_2 as i64) * (f1 as i64);
        let f0f2_2 = (f0_2 as i64) * (f2 as i64);
        let f0f3_2 = (f0_2 as i64) * (f3 as i64);
        let f0f4_2 = (f0_2 as i64) * (f4 as i64);
        let f0f5_2 = (f0_2 as i64) * (f5 as i64);
        let f0f6_2 = (f0_2 as i64) * (f6 as i64);
        let f0f7_2 = (f0_2 as i64) * (f7 as i64);
        let f0f8_2 = (f0_2 as i64) * (f8 as i64);
        let f0f9_2 = (f0_2 as i64) * (f9 as i64);
        let f1f1_2 = (f1_2 as i64) * (f1 as i64);
        let f1f2_2 = (f1_2 as i64) * (f2 as i64);
        let f1f3_4 = (f1_2 as i64) * (f3_2 as i64);
        let f1f4_2 = (f1_2 as i64) * (f4 as i64);
        let f1f5_4 = (f1_2 as i64) * (f5_2 as i64);
        let f1f6_2 = (f1_2 as i64) * (f6 as i64);
        let f1f7_4 = (f1_2 as i64) * (f7_2 as i64);
        let f1f8_2 = (f1_2 as i64) * (f8 as i64);
        let f1f9_76 = (f1_2 as i64) * (f9_38 as i64);
        let f2f2 = (f2 as i64) * (f2 as i64);
        let f2f3_2 = (f2_2 as i64) * (f3 as i64);
        let f2f4_2 = (f2_2 as i64) * (f4 as i64);
        let f2f5_2 = (f2_2 as i64) * (f5 as i64);
        let f2f6_2 = (f2_2 as i64) * (f6 as i64);
        let f2f7_2 = (f2_2 as i64) * (f7 as i64);
        let f2f8_38 = (f2_2 as i64) * (f8_19 as i64);
        let f2f9_38 = (f2 as i64) * (f9_38 as i64);
        let f3f3_2 = (f3_2 as i64) * (f3 as i64);
        let f3f4_2 = (f3_2 as i64) * (f4 as i64);
        let f3f5_4 = (f3_2 as i64) * (f5_2 as i64);
        let f3f6_2 = (f3_2 as i64) * (f6 as i64);
        let f3f7_76 = (f3_2 as i64) * (f7_38 as i64);
        let f3f8_38 = (f3_2 as i64) * (f8_19 as i64);
        let f3f9_76 = (f3_2 as i64) * (f9_38 as i64);
        let f4f4 = (f4 as i64) * (f4 as i64);
        let f4f5_2 = (f4_2 as i64) * (f5 as i64);
        let f4f6_38 = (f4_2 as i64) * (f6_19 as i64);
        let f4f7_38 = (f4 as i64) * (f7_38 as i64);
        let f4f8_38 = (f4_2 as i64) * (f8_19 as i64);
        let f4f9_38 = (f4 as i64) * (f9_38 as i64);
        let f5f5_38 = (f5 as i64) * (f5_38 as i64);
        let f5f6_38 = (f5_2 as i64) * (f6_19 as i64);
        let f5f7_76 = (f5_2 as i64) * (f7_38 as i64);
        let f5f8_38 = (f5_2 as i64) * (f8_19 as i64);
        let f5f9_76 = (f5_2 as i64) * (f9_38 as i64);
        let f6f6_19 = (f6 as i64) * (f6_19 as i64);
        let f6f7_38 = (f6 as i64) * (f7_38 as i64);
        let f6f8_38 = (f6_2 as i64) * (f8_19 as i64);
        let f6f9_38 = (f6 as i64) * (f9_38 as i64);
        let f7f7_38 = (f7 as i64) * (f7_38 as i64);
        let f7f8_38 = (f7_2 as i64) * (f8_19 as i64);
        let f7f9_76 = (f7_2 as i64) * (f9_38 as i64);
        let f8f8_19 = (f8 as i64) * (f8_19 as i64);
        let f8f9_38 = (f8 as i64) * (f9_38 as i64);
        let f9f9_38 = (f9 as i64) * (f9_38 as i64);
        let mut h0 = f0f0 + f1f9_76 + f2f8_38 + f3f7_76 + f4f6_38 + f5f5_38;
        let mut h1 = f0f1_2 + f2f9_38 + f3f8_38 + f4f7_38 + f5f6_38;
        let mut h2 = f0f2_2 + f1f1_2 + f3f9_76 + f4f8_38 + f5f7_76 + f6f6_19;
        let mut h3 = f0f3_2 + f1f2_2 + f4f9_38 + f5f8_38 + f6f7_38;
        let mut h4 = f0f4_2 + f1f3_4 + f2f2 + f5f9_76 + f6f8_38 + f7f7_38;
        let mut h5 = f0f5_2 + f1f4_2 + f2f3_2 + f6f9_38 + f7f8_38;
        let mut h6 = f0f6_2 + f1f5_4 + f2f4_2 + f3f3_2 + f7f9_76 + f8f8_19;
        let mut h7 = f0f7_2 + f1f6_2 + f2f5_2 + f3f4_2 + f8f9_38;
        let mut h8 = f0f8_2 + f1f7_4 + f2f6_2 + f3f5_4 + f4f4 + f9f9_38;
        let mut h9 = f0f9_2 + f1f8_2 + f2f7_2 + f3f6_2 + f4f5_2;
        let mut carry0: i64;
        let carry1: i64;
        let carry2: i64;
        let carry3: i64;
        let mut carry4: i64;
        let carry5: i64;
        let carry6: i64;
        let carry7: i64;
        let carry8: i64;
        let carry9: i64;

        h0 += h0;
        h1 += h1;
        h2 += h2;
        h3 += h3;
        h4 += h4;
        h5 += h5;
        h6 += h6;
        h7 += h7;
        h8 += h8;
        h9 += h9;

        carry0 = (h0 + (1 << 25)) >> 26;
        h1 += carry0;
        h0 -= carry0 << 26;
        carry4 = (h4 + (1 << 25)) >> 26;
        h5 += carry4;
        h4 -= carry4 << 26;

        carry1 = (h1 + (1 << 24)) >> 25;
        h2 += carry1;
        h1 -= carry1 << 25;
        carry5 = (h5 + (1 << 24)) >> 25;
        h6 += carry5;
        h5 -= carry5 << 25;

        carry2 = (h2 + (1 << 25)) >> 26;
        h3 += carry2;
        h2 -= carry2 << 26;
        carry6 = (h6 + (1 << 25)) >> 26;
        h7 += carry6;
        h6 -= carry6 << 26;

        carry3 = (h3 + (1 << 24)) >> 25;
        h4 += carry3;
        h3 -= carry3 << 25;
        carry7 = (h7 + (1 << 24)) >> 25;
        h8 += carry7;
        h7 -= carry7 << 25;

        carry4 = (h4 + (1 << 25)) >> 26;
        h5 += carry4;
        h4 -= carry4 << 26;
        carry8 = (h8 + (1 << 25)) >> 26;
        h9 += carry8;
        h8 -= carry8 << 26;

        carry9 = (h9 + (1 << 24)) >> 25;
        h0 += carry9 * 19;
        h9 -= carry9 << 25;

        carry0 = (h0 + (1 << 25)) >> 26;
        h1 += carry0;
        h0 -= carry0 << 26;

        FieldElement([
            h0 as i32, h1 as i32, h2 as i32, h3 as i32, h4 as i32, h5 as i32, h6 as i32, h7 as i32,
            h8 as i32, h9 as i32,
        ])
    }

    pub fn square(&self) -> FieldElement {
        let FieldElement(f) = self;

        let f0 = f[0];
        let f1 = f[1];
        let f2 = f[2];
        let f3 = f[3];
        let f4 = f[4];
        let f5 = f[5];
        let f6 = f[6];
        let f7 = f[7];
        let f8 = f[8];
        let f9 = f[9];
        let f0_2 = 2 * f0;
        let f1_2 = 2 * f1;
        let f2_2 = 2 * f2;
        let f3_2 = 2 * f3;
        let f4_2 = 2 * f4;
        let f5_2 = 2 * f5;
        let f6_2 = 2 * f6;
        let f7_2 = 2 * f7;
        let f5_38 = 38 * f5;
        let f6_19 = 19 * f6;
        let f7_38 = 38 * f7;
        let f8_19 = 19 * f8;
        let f9_38 = 38 * f9;
        let f0f0 = (f0 as i64) * (f0 as i64);
        let f0f1_2 = (f0_2 as i64) * (f1 as i64);
        let f0f2_2 = (f0_2 as i64) * (f2 as i64);
        let f0f3_2 = (f0_2 as i64) * (f3 as i64);
        let f0f4_2 = (f0_2 as i64) * (f4 as i64);
        let f0f5_2 = (f0_2 as i64) * (f5 as i64);
        let f0f6_2 = (f0_2 as i64) * (f6 as i64);
        let f0f7_2 = (f0_2 as i64) * (f7 as i64);
        let f0f8_2 = (f0_2 as i64) * (f8 as i64);
        let f0f9_2 = (f0_2 as i64) * (f9 as i64);
        let f1f1_2 = (f1_2 as i64) * (f1 as i64);
        let f1f2_2 = (f1_2 as i64) * (f2 as i64);
        let f1f3_4 = (f1_2 as i64) * (f3_2 as i64);
        let f1f4_2 = (f1_2 as i64) * (f4 as i64);
        let f1f5_4 = (f1_2 as i64) * (f5_2 as i64);
        let f1f6_2 = (f1_2 as i64) * (f6 as i64);
        let f1f7_4 = (f1_2 as i64) * (f7_2 as i64);
        let f1f8_2 = (f1_2 as i64) * (f8 as i64);
        let f1f9_76 = (f1_2 as i64) * (f9_38 as i64);
        let f2f2 = (f2 as i64) * (f2 as i64);
        let f2f3_2 = (f2_2 as i64) * (f3 as i64);
        let f2f4_2 = (f2_2 as i64) * (f4 as i64);
        let f2f5_2 = (f2_2 as i64) * (f5 as i64);
        let f2f6_2 = (f2_2 as i64) * (f6 as i64);
        let f2f7_2 = (f2_2 as i64) * (f7 as i64);
        let f2f8_38 = (f2_2 as i64) * (f8_19 as i64);
        let f2f9_38 = (f2 as i64) * (f9_38 as i64);
        let f3f3_2 = (f3_2 as i64) * (f3 as i64);
        let f3f4_2 = (f3_2 as i64) * (f4 as i64);
        let f3f5_4 = (f3_2 as i64) * (f5_2 as i64);
        let f3f6_2 = (f3_2 as i64) * (f6 as i64);
        let f3f7_76 = (f3_2 as i64) * (f7_38 as i64);
        let f3f8_38 = (f3_2 as i64) * (f8_19 as i64);
        let f3f9_76 = (f3_2 as i64) * (f9_38 as i64);
        let f4f4 = (f4 as i64) * (f4 as i64);
        let f4f5_2 = (f4_2 as i64) * (f5 as i64);
        let f4f6_38 = (f4_2 as i64) * (f6_19 as i64);
        let f4f7_38 = (f4 as i64) * (f7_38 as i64);
        let f4f8_38 = (f4_2 as i64) * (f8_19 as i64);
        let f4f9_38 = (f4 as i64) * (f9_38 as i64);
        let f5f5_38 = (f5 as i64) * (f5_38 as i64);
        let f5f6_38 = (f5_2 as i64) * (f6_19 as i64);
        let f5f7_76 = (f5_2 as i64) * (f7_38 as i64);
        let f5f8_38 = (f5_2 as i64) * (f8_19 as i64);
        let f5f9_76 = (f5_2 as i64) * (f9_38 as i64);
        let f6f6_19 = (f6 as i64) * (f6_19 as i64);
        let f6f7_38 = (f6 as i64) * (f7_38 as i64);
        let f6f8_38 = (f6_2 as i64) * (f8_19 as i64);
        let f6f9_38 = (f6 as i64) * (f9_38 as i64);
        let f7f7_38 = (f7 as i64) * (f7_38 as i64);
        let f7f8_38 = (f7_2 as i64) * (f8_19 as i64);
        let f7f9_76 = (f7_2 as i64) * (f9_38 as i64);
        let f8f8_19 = (f8 as i64) * (f8_19 as i64);
        let f8f9_38 = (f8 as i64) * (f9_38 as i64);
        let f9f9_38 = (f9 as i64) * (f9_38 as i64);

        let mut h = [
            f0f0 + f1f9_76 + f2f8_38 + f3f7_76 + f4f6_38 + f5f5_38,
            f0f1_2 + f2f9_38 + f3f8_38 + f4f7_38 + f5f6_38,
            f0f2_2 + f1f1_2 + f3f9_76 + f4f8_38 + f5f7_76 + f6f6_19,
            f0f3_2 + f1f2_2 + f4f9_38 + f5f8_38 + f6f7_38,
            f0f4_2 + f1f3_4 + f2f2 + f5f9_76 + f6f8_38 + f7f7_38,
            f0f5_2 + f1f4_2 + f2f3_2 + f6f9_38 + f7f8_38,
            f0f6_2 + f1f5_4 + f2f4_2 + f3f3_2 + f7f9_76 + f8f8_19,
            f0f7_2 + f1f6_2 + f2f5_2 + f3f4_2 + f8f9_38,
            f0f8_2 + f1f7_4 + f2f6_2 + f3f5_4 + f4f4 + f9f9_38,
            f0f9_2 + f1f8_2 + f2f7_2 + f3f6_2 + f4f5_2,
        ];

        for i in 0..10 {
            let shift = if i & 1 == 1 { 24 } else { 25 };
            let carry = (h[i] + (1 << shift.clone())) >> shift.clone() + 1;

            let add = if i == 9 { carry * 19 } else { carry };

            h[(i + 1) % 10] += add;
            h[i] -= carry << (shift + 1);
        }

        let carry = (h[0] + (1 << 25)) >> 26;
        h[1] += carry;
        h[0] -= carry << 26;

        let mut output = [0i32; 10];

        for i in 0..10 {
            output[i] = h[i] as i32;
        }

        FieldElement(output)
    }

    pub fn invert(self) -> FieldElement {
        let mut t0 = self.square();
        let mut t1 = t0.square();
        t1 = t1.square();
        t1 = &self * &t1;
        t0 = &t0 * &t1;
        let mut t2 = t0.square();
        t1 = &t1 * &t2;
        t2 = t1.square();

        for _ in 1..5 {
            t2 = t2.square();
        }

        t1 = &t2 * &t1;
        t2 = t1.square();

        for _ in 1..10 {
            t2 = t2.square();
        }

        t2 = &t2 * &t1;
        let mut t3 = t2.square();

        for _ in 1..20 {
            t3 = t3.square();
        }

        t2 = &t3 * &t2;

        for _ in 1..11 {
            t2 = t2.square();
        }

        t1 = &t2 * &t1;
        t2 = t1.square();

        for _ in 1..50 {
            t2 = t2.square();
        }

        t2 = &t2 * &t1;
        t3 = t2.square();

        for _ in 1..100 {
            t3 = t3.square();
        }

        t2 = &t3 * &t2;

        for _ in 1..51 {
            t2 = t2.square();
        }

        t1 = &t2 * &t1;

        for _ in 1..6 {
            t1 = t1.square();
        }

        t1 * t0
    }

    pub fn mul32(&self, n: i64) -> FieldElement {
        let f = self.0.clone();

        let mut h = [0i64; 10];

        for i in 0..10 {
            h[i] = (f[i] as i64) * n;
        }

        for i in 0..10 {
            let shift = if i & 1 == 1 { 25 } else { 26 };
            let carry = (h[i] + (shift.clone() - 1)) >> shift.clone();

            let add = if i == 9 { carry * 19 } else { carry };

            h[(i + 1) % 10] += add;
            h[i] -= carry * (1 << shift);
        }

        let mut output = [0i32; 10];
        for i in 0..10 {
            output[i] = h[i] as i32;
        }

        FieldElement(output)
    }

    pub fn pow25523(&self) -> FieldElement {
        let z2 = &self.square();
        let z8 = (0..2).fold(z2.clone(), |x, _| x.square());
        let z9 = self.clone() * z8;
        let z11 = z2.clone() * z9.clone();
        let z22 = z11.square();
        let z_5_0 = z9.clone() * z22;
        let z_10_5 = (0..5).fold(z_5_0.clone(), |x, _| x.square());
        let z_10_0 = z_10_5 * z_5_0;
        let z_20_10 = (0..10).fold(z_10_0.clone(), |x, _| x.square());
        let z_20_0 = &z_20_10 * &z_10_0;
        let z_40_20 = (0..20).fold(z_20_0.clone(), |x, _| x.square());
        let z_40_0 = z_40_20 * z_20_0;
        let z_50_10 = (0..10).fold(z_40_0, |x, _| x.square());
        let z_50_0 = z_50_10 * z_10_0;
        let z_100_50 = (0..50).fold(z_50_0.clone(), |x, _| x.square());
        let z_100_0 = &z_100_50 * &z_50_0;
        let z_200_100 = (0..100).fold(z_100_0.clone(), |x, _| x.square());
        let z_200_0 = z_200_100 * z_100_0;
        let z_250_50 = (0..50).fold(z_200_0, |x, _| x.square());
        let z_250_0 = z_250_50 * z_50_0;
        let z_252_2 = (0..2).fold(z_250_0, |x, _| x.square());
        let z_252_3 = z_252_2.clone() * self.clone();

        z_252_3
    }

    pub fn is_nonzero(&self) -> bool {
        let bs = self.to_bytes();
        let zero = [0; 32];
        !const_time_eq(bs.as_ref(), zero.as_ref())
    }

    pub fn is_negative(&self) -> bool {
        (self.to_bytes()[0] & 1) != 0
    }

    pub fn neg(&self) -> FieldElement {
        FieldElement([
            -self[0], -self[1], -self[2], -self[3], -self[4], -self[5], -self[6], -self[7],
            -self[8], -self[9],
        ])
    }
}

impl Add for &FieldElement {
    type Output = FieldElement;

    fn add(self, rhs: &FieldElement) -> FieldElement {
        let f = self.0;
        let other = rhs.0;

        let mut result = [0i32; 10];
        for i in 0..10 {
            result[i] = f[i] + other[i];
        }

        FieldElement(result)
    }
}

impl Add for FieldElement {
    type Output = FieldElement;

    fn add(self, rhs: FieldElement) -> FieldElement {
        &self + &rhs
    }
}

impl Sub for &FieldElement {
    type Output = FieldElement;

    fn sub(self, rhs: &FieldElement) -> FieldElement {
        let f = self.0;
        let other = rhs.0;

        let mut result = [0i32; 10];
        for i in 0..10 {
            result[i] = f[i] - other[i];
        }

        FieldElement(result)
    }
}

impl Sub for FieldElement {
    type Output = FieldElement;

    fn sub(self, rhs: FieldElement) -> FieldElement {
        &self - &rhs
    }
}

impl Sub<&FieldElement> for FieldElement {
    type Output = FieldElement;

    fn sub(self, rhs: &FieldElement) -> FieldElement {
        &self - rhs
    }
}

impl Mul for &FieldElement {
    type Output = FieldElement;

    fn mul(self, rhs: &FieldElement) -> FieldElement {
        let f = self.0;
        let g = rhs.0;
        let f0 = f[0];
        let f1 = f[1];
        let f2 = f[2];
        let f3 = f[3];
        let f4 = f[4];
        let f5 = f[5];
        let f6 = f[6];
        let f7 = f[7];
        let f8 = f[8];
        let f9 = f[9];
        let g0 = g[0];
        let g1 = g[1];
        let g2 = g[2];
        let g3 = g[3];
        let g4 = g[4];
        let g5 = g[5];
        let g6 = g[6];
        let g7 = g[7];
        let g8 = g[8];
        let g9 = g[9];
        let g1_19 = 19 * g1; /* 1.4*2^29 */
        let g2_19 = 19 * g2; /* 1.4*2^30; still ok */
        let g3_19 = 19 * g3;
        let g4_19 = 19 * g4;
        let g5_19 = 19 * g5;
        let g6_19 = 19 * g6;
        let g7_19 = 19 * g7;
        let g8_19 = 19 * g8;
        let g9_19 = 19 * g9;
        let f1_2 = 2 * f1;
        let f3_2 = 2 * f3;
        let f5_2 = 2 * f5;
        let f7_2 = 2 * f7;
        let f9_2 = 2 * f9;
        let f0g0 = (f0 as i64) * (g0 as i64);
        let f0g1 = (f0 as i64) * (g1 as i64);
        let f0g2 = (f0 as i64) * (g2 as i64);
        let f0g3 = (f0 as i64) * (g3 as i64);
        let f0g4 = (f0 as i64) * (g4 as i64);
        let f0g5 = (f0 as i64) * (g5 as i64);
        let f0g6 = (f0 as i64) * (g6 as i64);
        let f0g7 = (f0 as i64) * (g7 as i64);
        let f0g8 = (f0 as i64) * (g8 as i64);
        let f0g9 = (f0 as i64) * (g9 as i64);
        let f1g0 = (f1 as i64) * (g0 as i64);
        let f1g1_2 = (f1_2 as i64) * (g1 as i64);
        let f1g2 = (f1 as i64) * (g2 as i64);
        let f1g3_2 = (f1_2 as i64) * (g3 as i64);
        let f1g4 = (f1 as i64) * (g4 as i64);
        let f1g5_2 = (f1_2 as i64) * (g5 as i64);
        let f1g6 = (f1 as i64) * (g6 as i64);
        let f1g7_2 = (f1_2 as i64) * (g7 as i64);
        let f1g8 = (f1 as i64) * (g8 as i64);
        let f1g9_38 = (f1_2 as i64) * (g9_19 as i64);
        let f2g0 = (f2 as i64) * (g0 as i64);
        let f2g1 = (f2 as i64) * (g1 as i64);
        let f2g2 = (f2 as i64) * (g2 as i64);
        let f2g3 = (f2 as i64) * (g3 as i64);
        let f2g4 = (f2 as i64) * (g4 as i64);
        let f2g5 = (f2 as i64) * (g5 as i64);
        let f2g6 = (f2 as i64) * (g6 as i64);
        let f2g7 = (f2 as i64) * (g7 as i64);
        let f2g8_19 = (f2 as i64) * (g8_19 as i64);
        let f2g9_19 = (f2 as i64) * (g9_19 as i64);
        let f3g0 = (f3 as i64) * (g0 as i64);
        let f3g1_2 = (f3_2 as i64) * (g1 as i64);
        let f3g2 = (f3 as i64) * (g2 as i64);
        let f3g3_2 = (f3_2 as i64) * (g3 as i64);
        let f3g4 = (f3 as i64) * (g4 as i64);
        let f3g5_2 = (f3_2 as i64) * (g5 as i64);
        let f3g6 = (f3 as i64) * (g6 as i64);
        let f3g7_38 = (f3_2 as i64) * (g7_19 as i64);
        let f3g8_19 = (f3 as i64) * (g8_19 as i64);
        let f3g9_38 = (f3_2 as i64) * (g9_19 as i64);
        let f4g0 = (f4 as i64) * (g0 as i64);
        let f4g1 = (f4 as i64) * (g1 as i64);
        let f4g2 = (f4 as i64) * (g2 as i64);
        let f4g3 = (f4 as i64) * (g3 as i64);
        let f4g4 = (f4 as i64) * (g4 as i64);
        let f4g5 = (f4 as i64) * (g5 as i64);
        let f4g6_19 = (f4 as i64) * (g6_19 as i64);
        let f4g7_19 = (f4 as i64) * (g7_19 as i64);
        let f4g8_19 = (f4 as i64) * (g8_19 as i64);
        let f4g9_19 = (f4 as i64) * (g9_19 as i64);
        let f5g0 = (f5 as i64) * (g0 as i64);
        let f5g1_2 = (f5_2 as i64) * (g1 as i64);
        let f5g2 = (f5 as i64) * (g2 as i64);
        let f5g3_2 = (f5_2 as i64) * (g3 as i64);
        let f5g4 = (f5 as i64) * (g4 as i64);
        let f5g5_38 = (f5_2 as i64) * (g5_19 as i64);
        let f5g6_19 = (f5 as i64) * (g6_19 as i64);
        let f5g7_38 = (f5_2 as i64) * (g7_19 as i64);
        let f5g8_19 = (f5 as i64) * (g8_19 as i64);
        let f5g9_38 = (f5_2 as i64) * (g9_19 as i64);
        let f6g0 = (f6 as i64) * (g0 as i64);
        let f6g1 = (f6 as i64) * (g1 as i64);
        let f6g2 = (f6 as i64) * (g2 as i64);
        let f6g3 = (f6 as i64) * (g3 as i64);
        let f6g4_19 = (f6 as i64) * (g4_19 as i64);
        let f6g5_19 = (f6 as i64) * (g5_19 as i64);
        let f6g6_19 = (f6 as i64) * (g6_19 as i64);
        let f6g7_19 = (f6 as i64) * (g7_19 as i64);
        let f6g8_19 = (f6 as i64) * (g8_19 as i64);
        let f6g9_19 = (f6 as i64) * (g9_19 as i64);
        let f7g0 = (f7 as i64) * (g0 as i64);
        let f7g1_2 = (f7_2 as i64) * (g1 as i64);
        let f7g2 = (f7 as i64) * (g2 as i64);
        let f7g3_38 = (f7_2 as i64) * (g3_19 as i64);
        let f7g4_19 = (f7 as i64) * (g4_19 as i64);
        let f7g5_38 = (f7_2 as i64) * (g5_19 as i64);
        let f7g6_19 = (f7 as i64) * (g6_19 as i64);
        let f7g7_38 = (f7_2 as i64) * (g7_19 as i64);
        let f7g8_19 = (f7 as i64) * (g8_19 as i64);
        let f7g9_38 = (f7_2 as i64) * (g9_19 as i64);
        let f8g0 = (f8 as i64) * (g0 as i64);
        let f8g1 = (f8 as i64) * (g1 as i64);
        let f8g2_19 = (f8 as i64) * (g2_19 as i64);
        let f8g3_19 = (f8 as i64) * (g3_19 as i64);
        let f8g4_19 = (f8 as i64) * (g4_19 as i64);
        let f8g5_19 = (f8 as i64) * (g5_19 as i64);
        let f8g6_19 = (f8 as i64) * (g6_19 as i64);
        let f8g7_19 = (f8 as i64) * (g7_19 as i64);
        let f8g8_19 = (f8 as i64) * (g8_19 as i64);
        let f8g9_19 = (f8 as i64) * (g9_19 as i64);
        let f9g0 = (f9 as i64) * (g0 as i64);
        let f9g1_38 = (f9_2 as i64) * (g1_19 as i64);
        let f9g2_19 = (f9 as i64) * (g2_19 as i64);
        let f9g3_38 = (f9_2 as i64) * (g3_19 as i64);
        let f9g4_19 = (f9 as i64) * (g4_19 as i64);
        let f9g5_38 = (f9_2 as i64) * (g5_19 as i64);
        let f9g6_19 = (f9 as i64) * (g6_19 as i64);
        let f9g7_38 = (f9_2 as i64) * (g7_19 as i64);
        let f9g8_19 = (f9 as i64) * (g8_19 as i64);
        let f9g9_38 = (f9_2 as i64) * (g9_19 as i64);

        let mut h = [
            f0g0 + f1g9_38
                + f2g8_19
                + f3g7_38
                + f4g6_19
                + f5g5_38
                + f6g4_19
                + f7g3_38
                + f8g2_19
                + f9g1_38,
            f0g1 + f1g0
                + f2g9_19
                + f3g8_19
                + f4g7_19
                + f5g6_19
                + f6g5_19
                + f7g4_19
                + f8g3_19
                + f9g2_19,
            f0g2 + f1g1_2
                + f2g0
                + f3g9_38
                + f4g8_19
                + f5g7_38
                + f6g6_19
                + f7g5_38
                + f8g4_19
                + f9g3_38,
            f0g3 + f1g2 + f2g1 + f3g0 + f4g9_19 + f5g8_19 + f6g7_19 + f7g6_19 + f8g5_19 + f9g4_19,
            f0g4 + f1g3_2 + f2g2 + f3g1_2 + f4g0 + f5g9_38 + f6g8_19 + f7g7_38 + f8g6_19 + f9g5_38,
            f0g5 + f1g4 + f2g3 + f3g2 + f4g1 + f5g0 + f6g9_19 + f7g8_19 + f8g7_19 + f9g6_19,
            f0g6 + f1g5_2 + f2g4 + f3g3_2 + f4g2 + f5g1_2 + f6g0 + f7g9_38 + f8g8_19 + f9g7_38,
            f0g7 + f1g6 + f2g5 + f3g4 + f4g3 + f5g2 + f6g1 + f7g0 + f8g9_19 + f9g8_19,
            f0g8 + f1g7_2 + f2g6 + f3g5_2 + f4g4 + f5g3_2 + f6g2 + f7g1_2 + f8g0 + f9g9_38,
            f0g9 + f1g8 + f2g7 + f3g6 + f4g5 + f5g4 + f6g3 + f7g2 + f8g1 + f9g0,
        ];

        for i in 0..10 {
            let shift = if i & 1 == 1 { 24 } else { 25 };
            let carry = (h[i] + (1 << shift.clone())) >> shift.clone() + 1;

            let add = if i == 9 { carry * 19 } else { carry };

            h[(i + 1) % 10] += add;
            h[i] -= carry << (shift + 1);
        }

        let carry = (h[0] + (1 << 25)) >> 26;
        h[1] += carry;
        h[0] -= carry << 26;

        let mut output = [0i32; 10];

        for i in 0..10 {
            output[i] = h[i] as i32;
        }

        FieldElement(output)
    }
}

impl Mul for FieldElement {
    type Output = FieldElement;

    fn mul(self, rhs: FieldElement) -> FieldElement {
        &self * &rhs
    }
}

impl Index<usize> for FieldElement {
    type Output = i32;

    fn index(&self, index: usize) -> &i32 {
        &self.0[index]
    }
}

impl IndexMut<usize> for FieldElement {
    fn index_mut(&mut self, index: usize) -> &mut i32 {
        &mut self.0[index]
    }
}
