pub mod aeads;
pub mod ciphers;
pub mod ecc;
pub mod errors;
pub mod macs;
pub(crate) mod utils;

pub fn encrypt(key: Vec<u8>, msg: &[u8], nonce: &[u8]) -> Vec<u8> {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if is_x86_feature_detected!("aes") {
        return aeads::aegis256::encrypt::<16>(&key, msg, nonce, &[0u8]);
    }

    aeads::xchachapoly1305::encrypt(key, msg, nonce, &[0u8], None)
}

pub fn decrypt(key: Vec<u8>, msg: &[u8], nonce: &[u8]) -> Result<Vec<u8>, errors::InvalidMac> {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if is_x86_feature_detected!("aes") {
        return aeads::aegis256::decrypt::<16>(&key, msg, nonce, &[0u8]);
    }

    aeads::xchachapoly1305::decrypt(key, msg, nonce, &[0u8], None)
}
