pub use crate::ciphers::chacha::ChaCha20;
pub use crate::errors::InvalidMac;
pub use crate::macs::poly1305::Poly1305;
use crate::utils::const_time_eq;

pub struct ChaCha20Poly1305 {
    chacha: ChaCha20,
}

impl ChaCha20Poly1305 {
    pub fn new(key: &[u8]) -> ChaCha20Poly1305 {
        ChaCha20Poly1305 {
            chacha: ChaCha20::new(key),
        }
    }

    fn mac(&self, nonce: &[u8], ad: &[u8], ct: &[u8]) -> [u8; 16] {
        let poly1305_key: [u8; 32] = self.chacha.keystream(nonce, 0)[..32].try_into().unwrap();
        let mut poly1305 = Poly1305::new(poly1305_key);

        poly1305.update(ad);
        poly1305.update(ct);

        poly1305.update_unpadded(&(ad.len() as u64).to_le_bytes());
        poly1305.update_unpadded(&(ct.len() as u64).to_le_bytes());

        poly1305.tag()
    }

    pub fn encrypt(&self, msg: &[u8], nonce: &[u8], ad: &[u8]) -> Vec<u8> {
        let ct = self.chacha.encrypt(msg, nonce);
        let tag = self.mac(&nonce, &ad, &ct);

        [ct, tag.to_vec()].concat()
    }

    pub fn decrypt(&self, ct: &[u8], nonce: &[u8], ad: &[u8]) -> Result<Vec<u8>, InvalidMac> {
        let (ciphertext, tag) = (&ct[0..ct.len() - 16], &ct[ct.len() - 16..]);
        let msg = self.chacha.encrypt(ciphertext, nonce);
        let mac = self.mac(&nonce, &ad, &ct);

        if !const_time_eq(tag, &mac) {
            return Err(InvalidMac);
        }

        Ok(msg)
    }
}
