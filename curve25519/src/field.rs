use core::slice;
use std::ops::{Add, Index, IndexMut, Mul, Sub};
use zeroize::{Zeroize, ZeroizeOnDrop};

macro_rules! forward_ref_binop {
    (impl $imp:ident, $method:ident for $t:ty, $u:ty) => {
        impl<'a> $imp<$u> for &'a $t {
            type Output = <$t as $imp<$u>>::Output;

            #[inline]
            fn $method(self, other: $u) -> <$t as $imp<$u>>::Output {
                $imp::$method(*self, other)
            }
        }

        impl $imp<&$u> for $t {
            type Output = <$t as $imp<$u>>::Output;

            #[inline]
            fn $method(self, other: &$u) -> <$t as $imp<$u>>::Output {
                $imp::$method(self, *other)
            }
        }

        impl $imp<&$u> for &$t {
            type Output = <$t as $imp<$u>>::Output;

            #[inline]
            fn $method(self, other: &$u) -> <$t as $imp<$u>>::Output {
                $imp::$method(*self, *other)
            }
        }
    };
}

#[derive(PartialEq, Eq, Zeroize, ZeroizeOnDrop, Clone)]
pub struct Fe([u64; 5]);

impl Fe {
    pub fn new() -> Fe {
        Fe([0, 0, 0, 0, 0])
    }

    pub fn iter_mut(&self) -> slice::IterMut<u64> {
        self.0.iter_mut()
    }

    pub fn square_times(&self, rhs: &Fe, mut count: u64) -> Fe {
        let mut r = self.clone();

        loop {
            let d0 = r[0] * 2;
            let d1 = r[1] * 2;
            let d2 = r[2] * 2 * 19;
            let d419 = r[4] * 19;
            let d4 = d419 * 2;

            let mut t = [
                r[0] as u128 * r[0] as u128 + d4 as u128 * r[1] as u128 + d2 as u128 * r[3] as u128,
                d0 as u128 * r[1] as u128
                    + d4 as u128 * r[2] as u128
                    + r[3] as u128 * (r[3] * 19) as u128,
                d0 as u128 * r[2] as u128 + r[1] as u128 * r[1] as u128 + d4 as u128 * r[3] as u128,
                d0 as u128 * r[3] as u128 + d1 as u128 * r[2] as u128 + r[4] as u128 * d419 as u128,
                d0 as u128 * r[4] as u128 + d1 as u128 * r[3] as u128 + r[2] as u128 * r[2] as u128,
            ];

            let mut c = 0u64;
            for i in 0..5 {
                t[i] += c as u128;
                r[i] = (t[i] as u64) & 0x7ffffffffffff;
                c = (t[0] >> 51) as u64;
            }

            r[0] += c * 19;
            c = r[0] >> 51;
            r[0] &= 0x7ffffffffffff;

            r[1] += c;
            c = r[1] >> 51;
            r[1] &= 0x7ffffffffffff;

            r[2] += c;

            count -= 1;
            if count == 0 {
                break;
            }
        }

        r
    }
}

// based from https://doc.rust-lang.org/src/core/internal_macros.rs.html

impl IntoIterator for Fe {
    type Item = u64;
    type IntoIter = std::array::IntoIter<u64, 5>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Index<usize> for Fe {
    type Output = u64;

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for Fe {
    #[inline(always)]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl Add for Fe {
    type Output = Fe;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        let mut output = Fe::new();

        for i in 0..5 {
            *&mut output[i] = self[i] + rhs[i];
        }

        output
    }
}

forward_ref_binop! { impl Add, add for Fe, Fe }

const TWO54M152: u64 = (1u64 << 54u64) - 152u64;
const TWO54M8: u64 = (1u64 << 54u64) - 8u64;

impl Sub for Fe {
    type Output = Fe;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        let mut output = Fe::new();

        output[0] = self[0] + TWO54M152 - output[0];
        output[1] = self[1] + TWO54M8 - output[1];
        output[2] = self[2] + TWO54M8 - output[2];
        output[3] = self[3] + TWO54M8 - output[3];
        output[4] = self[4] + TWO54M8 - output[4];

        output
    }
}

forward_ref_binop! { impl Sub, sub for Fe, Fe }

const LIM: u64 = 0x7ffffffffffff;

impl Mul<u64> for Fe {
    type Output = Fe;

    fn mul(self, rhs: u64) -> Self::Output {
        let mut output = Fe::new();

        let mut a = (self[0] as u128) * rhs as u128;
        output[0] = (a as u64) & LIM;

        for (i, j) in output.iter_mut().zip(self.into_iter()) {
            a = (j as u128) * rhs as u128 + ((a >> 51) as u64) as u128;
            *i = (a as u64) & LIM;
        }

        output[0] += ((a >> 51) * 19) as u64;

        output
    }
}

forward_ref_binop! { impl Mul, mul for Fe, u64 }

impl Mul for Fe {
    type Output = Fe;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut r = self.clone();
        let mut s = rhs.clone();

        let mut t = [
            r[0] as u128 * s[0] as u128,
            r[0] as u128 * s[1] as u128 + r[1] as u128 * s[0] as u128,
            r[0] as u128 * s[2] as u128 + r[2] as u128 * s[0] as u128 + r[1] as u128 * s[1] as u128,
            r[0] as u128 * s[3] as u128
                + r[3] as u128 * s[0] as u128
                + r[1] as u128 * s[2] as u128
                + r[2] as u128 * s[1] as u128,
            r[0] as u128 * s[4] as u128
                + r[4] as u128 * s[0] as u128
                + r[3] as u128 * s[1] as u128
                + r[1] as u128 * s[3] as u128
                + r[2] as u128 * s[2] as u128,
        ];

        for i in 1..5 {
            *&mut r[i] *= 19;
        }

        t[0] += r[4] as u128 * s[1] as u128
            + r[1] as u128 * s[4] as u128
            + r[2] as u128 * s[3] as u128
            + r[3] as u128 * s[2] as u128;
        t[1] +=
            r[4] as u128 * s[2] as u128 + r[2] as u128 * s[4] as u128 + r[3] as u128 * s[3] as u128;
        t[2] += r[4] as u128 * s[3] as u128 + r[3] as u128 * s[4] as u128;
        t[3] += r[4] as u128 * s[4] as u128;

        let mut c = 0u64;
        for i in 0..5 {
            t[i] += c as u128;
            r[i] = (t[i] as u64) & 0x7ffffffffffff;
            c = (t[0] >> 51) as u64;
        }

        r[0] += c * 19;
        c = r[0] >> 51;
        r[0] &= 0x7ffffffffffff;

        r[1] += c;
        c = r[1] >> 51;
        r[1] &= 0x7ffffffffffff;

        r[2] += c;

        r
    }
}
