// adapted from the rust-crypto ed25519 implementation

use crate::ecc::ge::{ge_scalarmult_base, sc_muladd, sc_reduce, GeP2, GeP3};
use crate::ecc::InvalidKey;
use crate::utils::const_time_eq;
use getrandom::getrandom;
use sha2::{Digest, Sha512};

fn check_s_lt_l(s: &[u8]) -> bool {
    static L: [u8; 32] = [
        0xed, 0xd3, 0xf5, 0x5c, 0x1a, 0x63, 0x12, 0x58, 0xd6, 0x9c, 0xf7, 0xa2, 0xde, 0xf9, 0xde,
        0x14, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x10,
    ];
    let mut c: u8 = 0;
    let mut n: u8 = 1;

    for i in (0..32).rev() {
        c |= ((((s[i] as i32) - (L[i] as i32)) >> 8) as u8) & n;
        n &= ((((s[i] ^ L[i]) as i32) - 1) >> 8) as u8;
    }

    c != 0
}

/// A class to sign keys
pub struct SigningKey {
    az: [u8; 64],
    public: [u8; 32],
}

impl SigningKey {
    /// A constructor to generate a secret key from a 64-byte secret key
    pub fn from(secret: &[u8]) -> Result<SigningKey, InvalidKey> {
        if secret.len() != 64 {
            return Err(InvalidKey);
        }

        let az: [u8; 64] = {
            let mut hasher = Sha512::new();
            hasher.update(&secret[0..32]);
            let mut hash_output: [u8; 64] = hasher.finalize().into();
            hash_output[0] &= 248;
            hash_output[31] &= 63;
            hash_output[31] |= 64;
            hash_output
        };

        Ok(SigningKey {
            az,
            public: secret[32..64].try_into().unwrap(),
        })
    }

    /// A constructor to generate a secret key from a 32-byte seed
    pub fn from_seed(seed: &[u8]) -> Result<SigningKey, InvalidKey> {
        if seed.len() != 32 {
            return Err(InvalidKey);
        }

        let mut secret: [u8; 64] = {
            let mut hasher = Sha512::new();
            hasher.update(seed);
            let mut hash_output: [u8; 64] = hasher.finalize().into();
            hash_output[0] &= 248;
            hash_output[31] &= 63;
            hash_output[31] |= 64;
            hash_output
        };

        let a = ge_scalarmult_base(&secret[0..32]);
        let public_key = a.to_bytes();

        secret[0..32].copy_from_slice(seed);
        secret[32..64].copy_from_slice(&public_key);

        SigningKey::from(&secret)
    }

    /// A constructor that randomly generates a secret key
    pub fn generate() -> Result<SigningKey, InvalidKey> {
        let mut seed = [0u8; 32];
        let result = getrandom(&mut seed);

        if result.is_err() {
            return Err(InvalidKey);
        }

        let mut secret: [u8; 64] = {
            let mut hasher = Sha512::new();
            hasher.update(&seed);
            let mut hash_output: [u8; 64] = hasher.finalize().into();
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

        let mut secret: [u8; 64] = {
            let mut hasher = Sha512::new();
            hasher.update(&seed);
            let mut hash_output: [u8; 64] = hasher.finalize().into();
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

        SigningKey::from(&secret)
    }

    /// Returns the public key for verifying
    pub fn public_key(&self) -> VerifyingKey {
        VerifyingKey::from(&self.public).unwrap()
    }

    /// Returns the public key for verifying
    pub fn verifier(&self) -> VerifyingKey {
        VerifyingKey::from(&self.public).unwrap()
    }

    /// Signs messages
    pub fn sign(&self, msg: &[u8]) -> [u8; 64] {
        let nonce = {
            let mut hasher = Sha512::new();
            hasher.update(&self.az[32..64]);
            hasher.update(msg);
            let mut hash_output: [u8; 64] = hasher.finalize().into();
            sc_reduce(&mut hash_output[0..64]);
            hash_output
        };

        let mut signature: [u8; 64] = [0; 64];
        let r: GeP3 = ge_scalarmult_base(&nonce[0..32]);
        signature[0..32].copy_from_slice(&r.to_bytes());
        signature[32..64].copy_from_slice(&self.public);

        {
            let mut hasher = Sha512::new();
            hasher.update(signature.as_ref());
            hasher.update(msg);
            let mut hram: [u8; 64] = hasher.finalize().into();
            sc_reduce(&mut hram);
            sc_muladd(
                &mut signature[32..64],
                &hram[0..32],
                &self.az[0..32],
                &nonce[0..32],
            );
        }

        signature
    }
}

/// A class to verify messages signed with the corresponding secret key
pub struct VerifyingKey {
    public: [u8; 32],
}

impl VerifyingKey {
    /// Constructs a verifier from a public key
    pub fn from(public_key: &[u8]) -> Result<VerifyingKey, InvalidKey> {
        if public_key.len() != 32 {
            return Err(InvalidKey);
        }
        Ok(VerifyingKey {
            public: public_key.try_into().unwrap(),
        })
    }

    pub fn verify(&self, message: &[u8], signature: &[u8]) -> bool {
        if signature.len() != 64 {
            return false;
        }

        if !check_s_lt_l(&signature[32..64]) {
            return false;
        }

        let a = match GeP3::from_bytes_negate_vartime(&self.public) {
            Some(g) => g,
            None => {
                return false;
            }
        };
        let mut d = 0;
        for pk_byte in self.public.iter() {
            d |= *pk_byte;
        }
        if d == 0 {
            return false;
        }

        let mut hasher = Sha512::new();
        hasher.update(&signature[0..32]);
        hasher.update(&self.public);
        hasher.update(message);
        let mut hash: [u8; 64] = hasher.finalize().into();
        sc_reduce(&mut hash);

        let r = GeP2::double_scalarmult_vartime(hash.as_ref(), a, &signature[32..64]);
        let rcheck = r.to_bytes();

        const_time_eq(rcheck.as_ref(), &signature[0..32])
    }
}
