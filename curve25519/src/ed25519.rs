use crate::ge::*;
use sha2::{Digest, Sha512};

pub fn signature(message: &[u8], secret_key: &[u8], public_key: &[u8]) -> [u8; 64] {
    let az: [u8; 64] = {
        let mut hash_output: [u8; 64] = [0; 64];
        let mut hasher = Sha512::new();
        hasher.update(secret_key);
        hash_output = hasher.finalize().into();
        hash_output[0] &= 248;
        hash_output[31] &= 63;
        hash_output[31] |= 64;
        hash_output
    };

    let nonce = {
        let mut hash_output: [u8; 64] = [0; 64];
        let mut hasher = Sha512::new();
        hasher.update(&az[32..64]);
        hasher.update(message);
        hash_output = hasher.finalize().into();
        sc_reduce(&mut hash_output[0..64]);
        hash_output
    };

    let mut signature: [u8; 64] = [0; 64];
    let r: GeP3 = ge_scalarmult_base(&nonce[0..32]);
    for (result_byte, source_byte) in (&mut signature[0..32]).iter_mut().zip(r.to_bytes().iter()) {
        *result_byte = *source_byte;
    }
    for (result_byte, source_byte) in (&mut signature[32..64]).iter_mut().zip(public_key.iter()) {
        *result_byte = *source_byte;
    }

    {
        let mut hasher = Sha512::new();
        hasher.update(signature.as_ref());
        hasher.update(message);
        let mut hram: [u8; 64] = [0; 64];
        hram = hasher.finalize().into();
        sc_reduce(&mut hram);
        sc_muladd(
            &mut signature[32..64],
            &hram[0..32],
            &az[0..32],
            &nonce[0..32],
        );
    }

    signature
}

fn const_time_equals(x: &[u8], y: &[u8]) -> bool {
    let mut r = x[0] ^ y[0];

    for i in 1..32 {
        r |= x[i] ^ y[i];
    }

    r == 0
}

pub fn keypair(seed: &[u8]) -> ([u8; 64], [u8; 32]) {
    let mut secret: [u8; 64] = {
        let mut hash_output: [u8; 64] = [0; 64];
        let mut hasher = Sha512::new();
        hasher.update(seed);
        hash_output = hasher.finalize().into();
        hash_output[0] &= 248;
        hash_output[31] &= 63;
        hash_output[31] |= 64;
        hash_output
    };

    let a = ge_scalarmult_base(&secret[0..32]);
    let public_key = a.to_bytes();
    for (dest, src) in (&mut secret[32..64]).iter_mut().zip(public_key.iter()) {
        *dest = *src;
    }
    for (dest, src) in (&mut secret[0..32]).iter_mut().zip(seed.iter()) {
        *dest = *src;
    }
    (secret, public_key)
}
