pub mod aeads;
pub mod ciphers;
pub mod ecc;
pub mod errors;
pub mod macs;
pub(crate) mod utils;

pub use ecc::x25519::{PrivateKey, PublicKey};
pub use getrandom::getrandom;

pub fn encrypt(key: Vec<u8>, msg: &[u8]) -> Vec<u8> {
    let mut nonce = [0u8; 32];
    let mut ad = [0u8; 32];

    let _ = getrandom(&mut nonce);
    let _ = getrandom(&mut ad);

    let mut output = aeads::aegis256::encrypt::<16>(&key, msg, &nonce, &ad);
    output.append(&mut nonce.to_vec());
    output.append(&mut ad.to_vec());

    output
}

pub fn decrypt(key: Vec<u8>, msg: &[u8]) -> Result<Vec<u8>, errors::InvalidMac> {
    let nonce = &msg[msg.len() - 64..msg.len() - 32];
    let ad = &msg[msg.len() - 32..];
    let msg = &msg[..msg.len() - 64];

    aeads::aegis256::decrypt::<16>(&key, msg, nonce, ad)
}
