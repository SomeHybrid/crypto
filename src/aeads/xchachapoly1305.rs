use crate::aeads::chachapoly1305;
use crate::ciphers::chacha::hchacha;
use crate::errors::InvalidMac;

pub fn encrypt(
    key: &[u8],
    plaintext: &[u8],
    nonce: &[u8],
    ad: &[u8],
    rounds: Option<usize>,
) -> Vec<u8> {
    let subkey = hchacha(key, &nonce[0..16], rounds);

    let mut chacha_nonce = [0u8; 12];
    chacha_nonce[4..].copy_from_slice(&nonce[16..24]);

    chachapoly1305::encrypt(&subkey, plaintext, &chacha_nonce, ad, rounds)
}

pub fn decrypt(
    key: &[u8],
    plaintext: &[u8],
    nonce: &[u8],
    ad: &[u8],
    rounds: Option<usize>,
) -> Result<Vec<u8>, InvalidMac> {
    let subkey = hchacha(key, &nonce[0..16], rounds);

    let mut chacha_nonce = [0u8; 12];
    chacha_nonce[4..].copy_from_slice(&nonce[16..24]);

    chachapoly1305::decrypt(&subkey, plaintext, &chacha_nonce, ad, rounds)
}
