use crate::aeads::chachapoly1305::ChaCha20Poly1305;
use crate::ciphers::chacha::HChaCha20;
use crate::errors::InvalidMac;

pub struct XChaCha20Poly1305 {
    hchacha: HChaCha20,
}

impl XChaCha20Poly1305 {
    pub fn new(key: &[u8]) -> XChaCha20Poly1305 {
        XChaCha20Poly1305 {
            hchacha: HChaCha20::new(key),
        }
    }

    fn subkey(&self, nonce: &[u8]) -> ([u8; 32], [u8; 12]) {
        let subkey = self.hchacha.keystream(nonce);

        (subkey, [&[0u8; 4], &nonce[16..24]].concat().try_into().unwrap())
    }

    pub fn encrypt(&self, msg: &[u8], nonce: &[u8], ad: &[u8]) -> Vec<u8> {
        let (subkey, encryption_nonce) = self.subkey(nonce);

        let chacha = ChaCha20Poly1305::new(&subkey);

        chacha.encrypt(msg, &encryption_nonce, ad)
    }

    pub fn decrypt(&self, ct: &[u8], nonce: &[u8], ad: &[u8]) -> Result<Vec<u8>, InvalidMac> {
        let (subkey, encryption_nonce) = self.subkey(nonce);

        let chacha = ChaCha20Poly1305::new(&subkey);

        chacha.decrypt(ct, &encryption_nonce, ad)
    }
}
