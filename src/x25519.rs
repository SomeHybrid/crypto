use std::ops::{Add, Mul, Sub};
use zeroize::Zeroize;

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

#[derive(Clone, Copy)]
struct FieldElement([i32; 10]);

impl PartialEq for FieldElement {
    fn eq(&self, other: &FieldElement) -> bool {
        self.0 == other.0
    }
}

impl Eq for FieldElement {}

impl Zeroize for FieldElement {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

impl FieldElement {
    pub fn zero() -> FieldElement {
        FieldElement([0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
    }

    pub fn one() -> FieldElement {
        FieldElement([1, 0, 0, 0, 0, 0, 0, 0, 0, 0])
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

    pub fn invert(&self) -> FieldElement {
        let mut t0 = self.square();
        let mut t1 = t0.square();
        t1 = t1.square();
        t1 = *self * t1;
        t0 = t0 * t1;
        let mut t2 = t0.square();
        t1 = t1 * t2;
        t2 = t1.square();

        for _ in 1..5 {
            t2 = t2.square();
        }

        t1 = t2 * t1;
        t2 = t1.square();

        for _ in 1..10 {
            t2 = t2.square();
        }

        t2 = t2 * t1;
        let mut t3 = t2.square();

        for _ in 1..20 {
            t3 = t3.square();
        }

        t2 = t3 * t2;

        for _ in 1..11 {
            t2 = t2.square();
        }

        t1 = t2 * t1;
        t2 = t1.square();

        for _ in 1..50 {
            t2 = t2.square();
        }

        t2 = t2 * t1;
        t3 = t2.square();

        for _ in 1..100 {
            t3 = t3.square();
        }

        t2 = t3 * t2;

        for _ in 1..51 {
            t2 = t2.square();
        }

        t1 = t2 * t1;

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
}

impl Add for FieldElement {
    type Output = FieldElement;

    fn add(self, rhs: FieldElement) -> FieldElement {
        let f = self.0;
        let other = rhs.0;

        let mut result = [0i32; 10];
        for i in 0..10 {
            result[i] = f[i] + other[i];
        }

        FieldElement(result)
    }
}

impl Sub for FieldElement {
    type Output = FieldElement;

    fn sub(self, rhs: FieldElement) -> FieldElement {
        let f = self.0;
        let other = rhs.0;

        let mut result = [0i32; 10];
        for i in 0..10 {
            result[i] = f[i] - other[i];
        }

        FieldElement(result)
    }
}

impl Mul for FieldElement {
    type Output = FieldElement;

    fn mul(self, rhs: FieldElement) -> FieldElement {
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

pub fn scalarmult(n: &[u8], p: &[u8]) -> [u8; 32] {
    let mut t = [0u8; 32];

    for i in 0..32 {
        t[i] = n[i];
    }

    t[0] &= 248;
    t[31] &= 127;
    t[31] |= 64;

    let x1 = FieldElement::from_bytes(p);
    let mut x2 = FieldElement::one();
    let mut z2 = FieldElement::zero();
    let mut x3 = x1;
    let mut z3 = FieldElement::one();

    let mut swap = 0;
    for pos in (0..255).rev() {
        let bit = (t[pos / 8] >> (pos & 7)) & 1;
        swap ^= bit as i32;
        x2.swap(&mut x3, swap);
        z2.swap(&mut z3, swap);
        swap = bit as i32;

        let a = x2 + z2;
        let b = x2 - z2;
        let aa = a.square();
        let bb = b.square();
        x2 = aa * bb;
        let e = aa - bb;
        let mut da = x3 - z3;
        da = da * a;
        let mut cb = x3 + z3;
        cb = cb * b;
        x3 = da + cb;
        x3 = x3.square();
        z3 = da - cb;
        z3 = z3.square();
        z3 = z3 * x1;
        z2 = e.mul32(121666);
        z2 = z2 + bb;
        z2 = z2 * e;
    }

    x2.swap(&mut x3, swap);
    z2.swap(&mut z3, swap);

    let output = (z2.invert() * x2).to_bytes();

    t.zeroize();

    output
}

pub fn scalarmult_base(x: &[u8]) -> [u8; 32] {
    let mut base: [u8; 32] = [0; 32];
    base[0] = 9;
    scalarmult(x, base.as_ref())
}

