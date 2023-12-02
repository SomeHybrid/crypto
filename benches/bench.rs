use benchmark_simple::*;
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce
};
use raycrypt::aeads::aegis256::encrypt;
use raycrypt::aeads::chachapoly1305::encrypt as chapoly;
use serde_json::ser::CharEscape;

fn test_aegis(key: &[u8], nonce: &[u8], msg: &[u8]) {
    encrypt::<16>(key, msg, nonce, &[0u8]);
}

fn test_chapoly(key: &[u8], nonce: &[u8], msg: &[u8]) {
    chapoly(key.to_vec(), msg, nonce, &[0u8], None);
}

fn test_rustcrypto(key: &[u8], nonce: &[u8], msg: &[u8]) {
    let key = chacha20poly1305::Key::from_slice(&[0u8; 32]);
    let nonce = chacha20poly1305::Nonce::from_slice(&[0u8; 12]);
    let state = ChaCha20Poly1305::new(key);
    state.encrypt(nonce, msg).unwrap();
}

fn main() {
    let bench = Bench::new();
    let mut m = vec![0u8; 16384];
    let mut k = vec![0u8; 32];
    let mut nonce = k.clone();

    let options = &Options {
        iterations: 100,
        warmup_iterations: 50,
        min_samples: 5,
        max_samples: 10,
        max_rsd: 1.0,
        ..Default::default()
    };

    let res = bench.run(&options, || test_aegis(&k, &nonce, &m));
    println!("{}", res.throughput(m.len() as u128));

    let res = bench.run(&options, || test_chapoly(&k, &nonce, &m));
    println!("{}", res.throughput(m.len() as u128));

    let res = bench.run(&options, || test_rustcrypto(&k, &nonce, &m));
    println!("{}", res.throughput(m.len() as u128)); 
}
