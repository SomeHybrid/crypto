pub use crate::ciphers::chacha;
pub use crate::errors::InvalidMac;
pub use crate::macs::poly1305::Poly1305;

pub fn encrypt(
    key: Vec<u8>,
    plaintext: &[u8],
    nonce: &[u8],
    ad: &[u8],
    rounds: Option<usize>,
) -> Vec<u8> {
    let ciphertext = chacha::encrypt(key.clone(), plaintext, nonce, rounds);

    let poly1305_key = chacha::keystream(key.clone(), nonce, 0, rounds);
    let mut poly1305 = Poly1305::new(&poly1305_key);

    poly1305.update(ad);
    poly1305.update(&ciphertext);

    let lengths = [(ad.len() as u64).to_le_bytes(), (ciphertext.len() as u64).to_le_bytes()].concat();
    poly1305.update(&lengths);

    let tag = poly1305.tag();

    [ciphertext, tag].concat().to_vec()
}

pub fn decrypt(
    key: Vec<u8>,
    ciphertext: &[u8],
    nonce: &[u8],
    ad: &[u8],
    rounds: Option<usize>,
) -> Result<Vec<u8>, InvalidMac> {
    let plaintext = chacha::decrypt(key.clone(), ciphertext, nonce, rounds);

    let poly1305_key = chacha::keystream(key.clone(), nonce, 0, rounds);
    let mut poly1305 = Poly1305::new(&poly1305_key);

    poly1305.update(ad);
    poly1305.update(&ciphertext);

    let lengths = [(ad.len() as u64).to_le_bytes(), (ciphertext.len() as u64).to_le_bytes()].concat();
    poly1305.update(&lengths);

    if poly1305.verify(&ciphertext[ciphertext.len() - 16..]) {
        Ok(plaintext)
    } else {
        Err(InvalidMac)
    }
}
