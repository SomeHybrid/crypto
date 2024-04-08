use benchmark_simple::*;
use raycrypt::aegis256::encrypt;
use raycrypt::chachapoly1305::ChaCha20Poly1305;
use raycrypt::xchachapoly1305::XChaCha20Poly1305;
use raycrypt::ciphers::chacha::ChaCha20;

fn chapoly(key: &[u8], msg: &[u8], nonce: &[u8], ad: &[u8]) -> Vec<u8> {
    ChaCha20Poly1305::new(key).encrypt(msg, nonce, ad)
}

fn xchapoly(key: &[u8], msg: &[u8], nonce: &[u8], ad: &[u8]) -> Vec<u8> {
    XChaCha20Poly1305::new(key).encrypt(msg, nonce, ad)
}

fn chacha(key: &[u8], msg: &[u8], nonce: &[u8]) -> Vec<u8> {
    ChaCha20::new(key).encrypt(msg, nonce)
}

#[inline(always)]
fn test_aegis(key: &[u8], nonce: &[u8], msg: &[u8]) {
    encrypt::<16>(key, msg, nonce, &[0u8]);
}

#[inline(always)]
fn test_chapoly(key: &[u8], nonce: &[u8], msg: &[u8]) {
    chapoly(key, msg, nonce, &[0u8]);
}

#[inline(always)]
fn test_chacha(key: &[u8], nonce: &[u8], msg: &[u8]) {
    chacha(key, msg, nonce);
}

#[inline(always)]
fn test_xchapoly(key: &[u8], nonce: &[u8], msg: &[u8]) {
    xchapoly(key, msg, nonce, &[0u8]);
}

fn main() {
    let bench = Bench::new();
    let m = vec![0u8; 16384];
    let k = vec![0u8; 32];
    let nonce = k.clone();

    let options = &Options {
        iterations: 1000,
        warmup_iterations: 100,
        min_samples: 5,
        max_samples: 10,
        max_rsd: 1.0,
        ..Default::default()
    };

    let res = bench.run(&options, || test_aegis(&k, &nonce, &m));
    println!("aegis256: {}", res.throughput(m.len() as u128));

    let res = bench.run(&options, || test_chapoly(&k, &nonce, &m));
    println!("chacha20poly1305: {}", res.throughput(m.len() as u128));

    let res = bench.run(&options, || test_xchapoly(&k, &nonce, &m));
    println!("xchacha20poly1305: {}", res.throughput(m.len() as u128));

    let res = bench.run(&options, || test_chacha(&k, &nonce, &m));
    println!("chacha20: {}", res.throughput(m.len() as u128));

    #[cfg(target_arch = "x86_64")]
    unsafe {
        use core::arch::x86_64::__rdtscp;

        let mut tmp = [0u8; 32];

        let a = __rdtscp(tmp.as_mut_ptr() as *mut u32);
        test_aegis(&k, &nonce, &m);
        let b = __rdtscp(tmp.as_mut_ptr() as *mut u32);

        println!("aegis256: CPU cycles {}", b - a);

        let a = __rdtscp(tmp.as_mut_ptr() as *mut u32);
        test_chapoly(&k, &nonce, &m);
        let b = __rdtscp(tmp.as_mut_ptr() as *mut u32);

        println!("chacha20poly1305: CPU cycles {}", b - a);

        let a = __rdtscp(tmp.as_mut_ptr() as *mut u32);
        test_xchapoly(&k, &nonce, &m);
        let b = __rdtscp(tmp.as_mut_ptr() as *mut u32);

        println!("xchacha20poly1305: CPU cycles {}", b - a);

        let a = __rdtscp(tmp.as_mut_ptr() as *mut u32);
        test_chacha(&k, &nonce, &m);
        let b = __rdtscp(tmp.as_mut_ptr() as *mut u32);
        println!("chacha20: CPU cycles {}", b - a);
    }
}
