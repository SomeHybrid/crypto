pub mod aeads;
pub mod ciphers;
pub mod ecc;
pub mod errors;
pub mod macs;
pub(crate) mod utils;

pub fn encrypt(key: Vec<u8>, msg: &[u8], nonce: &[u8], ad: &[u8]) -> Vec<u8> {
    aeads::aegis256::encrypt::<16>(&key, msg, nonce, ad)
}

pub fn decrypt(
    key: Vec<u8>,
    msg: &[u8],
    nonce: &[u8],
    ad: &[u8],
) -> Result<Vec<u8>, errors::InvalidMac> {
    aeads::aegis256::decrypt::<16>(&key, msg, nonce, ad)
}
