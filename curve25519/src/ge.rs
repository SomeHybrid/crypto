use crate::field::FieldElement;
use crate::precomp::Precomp;
use crate::precomp::{BASE, BI};

use std::ops::{Add, Sub};

const D2: FieldElement = FieldElement::from([
    -21827239, -5839606, -30745221, 13898782, 229458, 15978800, -12551817, -6495438, 29715968,
    9444199,
]);

fn slide(a: &[u8]) -> [i8; 256] {
    let mut output = [0i8; 256];

    for i in 0..256 {
        output[i] = (1 & (a[i >> 3] >> (i & 7))) as i8;
    }

    for i in 0..256 {
        if !(output[i] != 0) {
            continue;
        }

        for b in 1..=6 {
            if !(i + b < 256 && output[i + b] != 0) {
                break;
            }

            if output[i] + (output[i + b] << b) <= 15 {
                output[i] += output[i + b] << b;
                output[i + b] = 0;
            } else if output[i] - (output[i + b] << b) >= -15 {
                output[i] -= output[i + b] << b;

                for k in (i + b)..256 {
                    if output[k] == 0 {
                        output[k] = 1;
                        break;
                    }

                    output[k] = 0;
                }
            } else {
                break;
            }
        }
    }

    output
}

struct GeP2 {
    x: FieldElement,
    y: FieldElement,
    z: FieldElement,
}

#[derive(Clone, Copy)]
struct GeP3 {
    x: FieldElement,
    y: FieldElement,
    z: FieldElement,
    t: FieldElement,
}

struct GeP1P1 {
    x: FieldElement,
    y: FieldElement,
    z: FieldElement,
    t: FieldElement,
}

#[derive(Clone, Copy)]
struct GeCached {
    y_plus_x: FieldElement,
    y_minus_x: FieldElement,
    z: FieldElement,
    t2d: FieldElement,
}

impl GeCached {
    pub fn new() -> GeCached {
        GeCached {
            y_plus_x: FieldElement::zero(),
            y_minus_x: FieldElement::zero(),
            z: FieldElement::zero(),
            t2d: FieldElement::zero(),
        }
    }
}

impl GeP3 {
    pub fn new() -> GeP3 {
        GeP3 {
            x: FieldElement::zero(),
            y: FieldElement::zero(),
            z: FieldElement::zero(),
            t: FieldElement::zero(),
        }
    }

    pub fn to_p2(&self) -> GeP2 {
        GeP2 {
            x: self.x.clone(),
            y: self.y.clone(),
            z: self.z.clone(),
        }
    }

    pub fn to_cached(&self) -> GeCached {
        let mut output = GeCached::new();

        output.y_plus_x = &self.y + &self.x;
        output.y_minus_x = &self.y - &self.x;
        output.z = self.z.clone();
        output.t2d = &self.t * &D2;

        output
    }

    pub fn double(&self) -> GeP1P1 {
        self.to_p2().double()
    }
}

impl GeP2 {
    pub fn new() -> GeP2 {
        GeP2 {
            x: FieldElement::zero(),
            y: FieldElement::zero(),
            z: FieldElement::zero(),
        }
    }

    pub fn to_p3(&self) -> GeP3 {
        GeP3 {
            x: self.x.clone(),
            y: self.y.clone(),
            z: self.z.clone(),
            t: self.x.clone() * self.y.clone(),
        }
    }

    pub fn double(&self) -> GeP1P1 {
        let mut output = GeP1P1::new();

        output.x = self.x.square();
        output.z = self.y.square();
        output.t = self.z.square().double();
        output.y = &self.x + &self.y;

        let t0 = output.y.square();

        output.y = &output.z + &output.x;
        output.z = &output.z - &output.x;
        output.x = t0 - &output.y;
        output.t = &output.t - &output.z;

        output
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        let recip = self.z.invert();
        let x = &self.x * &recip;
        let y = &self.y * &recip;

        let mut s = y.to_bytes();
        s[31] ^= (x.negative() as u8) << 7;

        s
    }
}

impl GeP1P1 {
    pub fn new() -> GeP1P1 {
        GeP1P1 {
            x: FieldElement::zero(),
            y: FieldElement::zero(),
            z: FieldElement::zero(),
            t: FieldElement::zero(),
        }
    }

    pub fn to_p2(&self) -> GeP2 {
        GeP2 {
            x: &self.x * &self.t,
            y: &self.y * &self.z,
            z: &self.z * &self.t,
        }
    }

    pub fn to_p3(&self) -> GeP3 {
        GeP3 {
            x: &self.x * &self.t,
            y: &self.y * &self.z,
            z: &self.z * &self.t,
            t: &self.x * &self.y,
        }
    }
}

impl Add<GeCached> for GeP3 {
    type Output = GeP1P1;

    fn add(self, other: GeCached) -> GeP1P1 {
        let mut output = GeP1P1::new();

        output.x = &self.y + &self.x;
        output.y = &self.y - &self.x;
        output.z = &output.x * &other.y_plus_x;
        output.y = &output.y * &other.y_minus_x;
        output.t = &other.t2d * &self.t;
        output.x = &self.z * &other.z;

        let t0 = &output.x + &output.x;

        output.x = &output.z - &output.y;
        output.y = &output.z + &output.y;
        output.z = &t0 + &output.t;
        output.t = &t0 - &output.t;

        output
    }
}

impl Add<&GeCached> for &GeP3 {
    type Output = GeP1P1;

    fn add(self, other: &GeCached) -> GeP1P1 {
        let mut output = GeP1P1::new();

        output.x = &self.y + &self.x;
        output.y = &self.y - &self.x;
        output.z = &output.x * &other.y_plus_x;
        output.y = &output.y * &other.y_minus_x;
        output.t = &other.t2d * &self.t;
        output.x = &self.z * &other.z;

        let t0 = &output.x + &output.x;

        output.x = &output.z - &output.y;
        output.y = &output.z + &output.y;
        output.z = &t0 + &output.t;
        output.t = &t0 - &output.t;

        output
    }
}

impl Add<&&GeCached> for &GeP3 {
    type Output = GeP1P1;

    fn add(self, other: &&GeCached) -> GeP1P1 {
        self + *other
    }
}

impl Add<Precomp> for GeP3 {
    type Output = GeP1P1;

    fn add(self, other: Precomp) -> GeP1P1 {
        let mut output = GeP1P1::new();

        output.x = &self.y + &self.x;
        output.y = &self.y - &self.x;
        output.z = &output.x * &other.yplusx;
        output.y = &output.y * &other.yminusx;
        output.t = &other.xy2d * &self.t;

        let t0 = &output.z + &output.z;

        output.x = &output.z - &output.y;
        output.y = &output.z + &output.y;
        output.z = &t0 + &output.t;
        output.t = &t0 - &output.t;

        output
    }
}

impl Add<&Precomp> for &GeP3 {
    type Output = GeP1P1;

    fn add(self, other: &Precomp) -> GeP1P1 {
        let mut output = GeP1P1::new();

        output.x = &self.y + &self.x;
        output.y = &self.y - &self.x;
        output.z = &output.x * &other.yplusx;
        output.y = &output.y * &other.yminusx;
        output.t = &other.xy2d * &self.t;

        let t0 = &output.z + &output.z;

        output.x = &output.z - &output.y;
        output.y = &output.z + &output.y;
        output.z = &t0 + &output.t;
        output.t = &t0 - &output.t;

        output
    }
}

impl Sub<GeCached> for GeP3 {
    type Output = GeP1P1;

    fn sub(self, other: GeCached) -> GeP1P1 {
        let mut output = GeP1P1::new();

        output.x = &self.y + &self.x;
        output.y = &self.y - &self.x;
        output.z = &output.x * &other.y_minus_x;
        output.y = &output.y * &other.y_plus_x;
        output.t = &other.t2d * &self.t;
        output.x = &self.z * &other.z;

        let t0 = &output.x + &output.x;

        output.x = &output.z - &output.y;
        output.y = &output.z + &output.y;
        output.z = &t0 - &output.t;
        output.t = &t0 + &output.t;

        output
    }
}

impl Sub<&Precomp> for &GeP3 {
    type Output = GeP1P1;

    fn sub(self, other: &Precomp) -> GeP1P1 {
        let mut output = GeP1P1::new();

        output.x = &self.y + &self.x;
        output.y = &self.y - &self.x;
        output.z = &output.x * &other.yminusx;
        output.y = &output.y * &other.yplusx;
        output.t = &other.xy2d * &self.t;
        output.x = &output.z - &output.y;
        output.y = &output.z + &output.y;

        let t0 = self.t.double();
        output.z = &t0 - &output.t;
        output.t = &t0 + &output.t;

        output
    }
}

pub fn ge_double_scalarmult_vartime(a: &[u8], A: &GeP3, b: &[u8]) -> GeP2 {
    let aslide = slide(a);
    let bslide = slide(b);

    let temp = GeCached::new();

    let mut ai = [temp; 8];
    ai[0] = A.to_cached();

    let a2 = A.double().to_p3();

    ai[1] = (&a2 + &ai[0]).to_p3().to_cached();
    ai[2] = (&a2 + &ai[1]).to_p3().to_cached();
    ai[3] = (&a2 + &ai[2]).to_p3().to_cached();
    ai[4] = (&a2 + &ai[3]).to_p3().to_cached();
    ai[5] = (&a2 + &ai[4]).to_p3().to_cached();
    ai[6] = (&a2 + &ai[5]).to_p3().to_cached();
    ai[7] = (&a2 + &ai[6]).to_p3().to_cached();

    let mut i: usize = 255;

    loop {
        if aslide[i] == 0 || bslide[i] == 0 {
            break;
        }

        i -= 1;
    }

    let mut r: GeP2 = GeP2::new();

    loop {
        let mut t = r.double();

        if aslide[i] > 0 {
            t = t.to_p3() + ai[(aslide[i] / 2) as usize];
        } else if aslide[i] < 0 {
            t = t.to_p3() - ai[((-aslide[i]) / 2) as usize];
        }

        if bslide[i] > 0 {
            t = &t.to_p3() + &BI[(bslide[i] / 2) as usize];
        } else if bslide[i] < 0 {
            t = &t.to_p3() - &BI[((-bslide[i]) / 2) as usize];
        }

        r = t.to_p2();

        if i == 0 {
            break;
        }

        i -= 1;
    }

    r
}

const D: FieldElement = FieldElement::from([
    -10913610, 13857413, -15372611, 6949391, 114729, -8787816, -6275908, -3247719, -18696448,
    -12055116,
]);

fn ge_frombytes_negate_vartime(data: &[u8]) -> GeP3 {
    let mut output = GeP3::new();

    output.y = FieldElement::from_bytes(data);
    output.z = FieldElement::one();

    let u = output.y.square();
    let v = &u * &D;

    output
}
