use rand::{thread_rng, RngCore};

pub(crate) fn randbytes<const BITS: usize>() -> Vec<u8> {
    let mut bytes = vec![0u8; BITS / 8];
    let mut rng = thread_rng();
    rng.fill_bytes(&mut bytes);
    bytes
}
