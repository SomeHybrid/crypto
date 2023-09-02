// takes a while

use dudect_bencher::{ctbench_main, BenchRng, Class, CtRunner};
use encryption::x25519;

use rand::Rng;
use rand::RngCore;

fn rand_arr<const LEN: usize>(rng: &mut BenchRng) -> [u8; LEN] {
    let mut arr = [0u8; LEN];
    rng.fill_bytes(&mut arr);
    arr
}

pub fn generate_input_classes<const LEN: usize>(
    rng: &mut BenchRng,
) -> (Vec<([u8; LEN], [u8; LEN])>, Vec<Class>) {
    let mut inputs = Vec::new();
    let mut classes = Vec::new();

    for _ in 0..10_000_000 {
        let v1 = rand_arr::<{ LEN }>(rng);
        if rng.gen::<bool>() {
            inputs.push((v1.clone(), v1));
            classes.push(Class::Left);
        } else {
            inputs.push((v1, rand_arr::<{ LEN }>(rng)));
            classes.push(Class::Right);
        }
    }

    (inputs, classes)
}

fn test_x25519(runner: &mut CtRunner, rng: &mut BenchRng) {
    let (inputs, classes) = generate_input_classes::<32>(rng);

    for (class, input) in classes.into_iter().zip(inputs.into_iter()) {
        runner.run_one(class, || {
            x25519::scalarmult(input.0.as_slice(), input.1.as_slice())
        });
    }
}

ctbench_main!(test_x25519);
