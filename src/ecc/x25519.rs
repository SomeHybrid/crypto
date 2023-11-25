use crate::ecc::field::FieldElement;
use zeroize::Zeroize;

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
    let mut x3 = x1.clone();
    let mut z3 = FieldElement::one();

    let mut swap = 0;
    for pos in (0..255).rev() {
        let bit = (t[pos / 8] >> (pos & 7)) & 1;
        swap ^= bit as i32;
        x2.swap(&mut x3, swap);
        z2.swap(&mut z3, swap);
        swap = bit as i32;

        let a = &x2 + &z2;
        let b = &x2 - &z2;
        let aa = a.square();
        let bb = b.square();
        x2 = &aa * &bb;
        let e = &aa - &bb;
        let mut da = &x3 - &z3;
        da = da * a;
        let mut cb = &x3 + &z3;
        cb = cb * b;
        x3 = &da + &cb;
        x3 = x3.square();
        z3 = &da - &cb;
        z3 = z3.square();
        z3 = &z3 * &x1;
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
