use crate::curve25519::fe::FieldElement;
use crate::curve25519::precomp::Precomp;
use crate::curve25519::precomp::{BASE, BI};

struct GeP2 {
    x: FieldElement,
    y: FieldElement,
    z: FieldElement,
}

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

struct GeCached 
