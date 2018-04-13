#![feature(test)]
#![cfg_attr(all(feature="i128_support", feature="nightly"), allow(stable_features))] // stable since 2018-03-27
#![cfg_attr(all(feature="i128_support", feature="nightly"), feature(i128_type, i128))]

extern crate test;
extern crate rand;

const RAND_BENCH_N: u64 = 1000;
const BYTES_LEN: usize = 1024;

use std::mem::size_of;
use test::{black_box, Bencher};

use rand::{RngCore, Rng, SeedableRng, NewRng};
use rand::{StdRng, SmallRng, OsRng, EntropyRng, ReseedingRng};
use rand::prng::{XorShiftRng, Hc128Rng, IsaacRng, Isaac64Rng, ChaChaRng};
use rand::prng::hc128::Hc128Core;
use rand::jitter::JitterRng;
use rand::thread_rng;

macro_rules! gen_bytes {
    ($fnn:ident, $gen:expr) => {
        #[bench]
        fn $fnn(b: &mut Bencher) {
            let mut rng = $gen;
            let mut buf = [0u8; BYTES_LEN];
            b.iter(|| {
                for _ in 0..RAND_BENCH_N {
                    rng.fill_bytes(&mut buf);
                    black_box(buf);
                }
            });
            b.bytes = BYTES_LEN as u64 * RAND_BENCH_N;
        }
    }
}

gen_bytes!(gen_bytes_xorshift, XorShiftRng::new());
gen_bytes!(gen_bytes_hc128, Hc128Rng::new());
gen_bytes!(gen_bytes_isaac, IsaacRng::new());
gen_bytes!(gen_bytes_isaac64, Isaac64Rng::new());
gen_bytes!(gen_bytes_std, StdRng::new());
gen_bytes!(gen_bytes_small, SmallRng::new());
gen_bytes!(gen_bytes_os, OsRng::new().unwrap());

macro_rules! gen_uint {
    ($fnn:ident, $ty:ty, $gen:expr) => {
        #[bench]
        #[allow(unused_mut)]
        fn $fnn(b: &mut Bencher) {
            let mut rng = $gen;
            b.iter(|| {
                let mut accum: $ty = 0;
                for _ in 0..RAND_BENCH_N {
                    accum = accum.wrapping_add(rng.gen::<$ty>());
                }
                black_box(accum);
            });
            b.bytes = size_of::<$ty>() as u64 * RAND_BENCH_N;
        }
    }
}

gen_uint!(gen_u32_xorshift, u32, XorShiftRng::new());
gen_uint!(gen_u32_hc128, u32, Hc128Rng::new());
gen_uint!(gen_u32_isaac, u32, IsaacRng::new());
gen_uint!(gen_u32_isaac64, u32, Isaac64Rng::new());
gen_uint!(gen_u32_std, u32, StdRng::new());
gen_uint!(gen_u32_small, u32, SmallRng::new());
gen_uint!(gen_u32_os, u32, OsRng::new().unwrap());

gen_uint!(gen_u64_xorshift, u64, XorShiftRng::new());
gen_uint!(gen_u64_hc128, u64, Hc128Rng::new());
gen_uint!(gen_u64_isaac, u64, IsaacRng::new());
gen_uint!(gen_u64_isaac64, u64, Isaac64Rng::new());
gen_uint!(gen_u64_std, u64, StdRng::new());
gen_uint!(gen_u64_small, u64, SmallRng::new());
gen_uint!(gen_u64_os, u64, OsRng::new().unwrap());

#[cfg(feature = "i128_support")] gen_uint!(gen_u128_xorshift, u128, XorShiftRng::new());
#[cfg(feature = "i128_support")] gen_uint!(gen_u128_hc128, u128, Hc128Rng::new());
#[cfg(feature = "i128_support")] gen_uint!(gen_u128_os, u128, OsRng::new().unwrap());
#[cfg(feature = "i128_support")]
gen_uint!(gen_u128_hc128_trait_obj, u128, &mut Hc128Rng::new() as &mut RngCore);

// Do not test JitterRng like the others by running it RAND_BENCH_N times per,
// measurement, because it is way too slow. Only run it once.
#[bench]
fn gen_u64_jitter(b: &mut Bencher) {
    let mut rng = JitterRng::new().unwrap();
    b.iter(|| {
        black_box(rng.gen::<u64>());
    });
    b.bytes = size_of::<u64>() as u64;
}

macro_rules! init_gen {
    ($fnn:ident, $gen:ident) => {
        #[bench]
        fn $fnn(b: &mut Bencher) {
            let mut rng = XorShiftRng::new();
            b.iter(|| {
                let r2 = $gen::from_rng(&mut rng).unwrap();
                black_box(r2);
            });
        }
    }
}

init_gen!(init_xorshift, XorShiftRng);
init_gen!(init_hc128, Hc128Rng);
init_gen!(init_isaac, IsaacRng);
init_gen!(init_isaac64, Isaac64Rng);
init_gen!(init_chacha, ChaChaRng);

#[bench]
fn init_jitter(b: &mut Bencher) {
    b.iter(|| {
        black_box(JitterRng::new().unwrap());
    });
}

macro_rules! chacha_rounds {
    ($fn1:ident, $fn2:ident, $fn3:ident, $rounds:expr) => {
        #[bench]
        fn $fn1(b: &mut Bencher) {
            let mut rng = ChaChaRng::new();
            rng.set_rounds($rounds);
            let mut buf = [0u8; BYTES_LEN];
            b.iter(|| {
                for _ in 0..RAND_BENCH_N {
                    rng.fill_bytes(&mut buf);
                    black_box(buf);
                }
            });
            b.bytes = BYTES_LEN as u64 * RAND_BENCH_N;
        }

        #[bench]
        fn $fn2(b: &mut Bencher) {
            let mut rng = ChaChaRng::new();
            rng.set_rounds($rounds);
            b.iter(|| {
                let mut accum: u32 = 0;
                for _ in 0..RAND_BENCH_N {
                    accum = accum.wrapping_add(rng.gen::<u32>());
                }
                black_box(accum);
            });
            b.bytes = size_of::<u32>() as u64 * RAND_BENCH_N;
        }

        #[bench]
        fn $fn3(b: &mut Bencher) {
            let mut rng = ChaChaRng::new();
            rng.set_rounds($rounds);
            b.iter(|| {
                let mut accum: u64 = 0;
                for _ in 0..RAND_BENCH_N {
                    accum = accum.wrapping_add(rng.gen::<u64>());
                }
                black_box(accum);
            });
            b.bytes = size_of::<u64>() as u64 * RAND_BENCH_N;
        }
    }
}

chacha_rounds!(gen_bytes_chacha8, gen_u32_chacha8, gen_u64_chacha8, 8);
chacha_rounds!(gen_bytes_chacha12, gen_u32_chacha12, gen_u64_chacha12, 12);
chacha_rounds!(gen_bytes_chacha20, gen_u32_chacha20, gen_u64_chacha20, 20);


const RESEEDING_THRESHOLD: u64 = 1024*1024*1024; // something high enough to get
                                                 // deterministic measurements

#[bench]
fn reseeding_hc128_bytes(b: &mut Bencher) {
    let mut rng = ReseedingRng::new(Hc128Core::new(),
                                    RESEEDING_THRESHOLD,
                                    EntropyRng::new());
    let mut buf = [0u8; BYTES_LEN];
    b.iter(|| {
        for _ in 0..RAND_BENCH_N {
            rng.fill_bytes(&mut buf);
            black_box(buf);
        }
    });
    b.bytes = BYTES_LEN as u64 * RAND_BENCH_N;
}

macro_rules! reseeding_uint {
    ($fnn:ident, $ty:ty) => {
        #[bench]
        fn $fnn(b: &mut Bencher) {
            let mut rng = ReseedingRng::new(Hc128Core::new(),
                                            RESEEDING_THRESHOLD,
                                            EntropyRng::new());
            b.iter(|| {
                let mut accum: $ty = 0;
                for _ in 0..RAND_BENCH_N {
                    accum = accum.wrapping_add(rng.gen::<$ty>());
                }
                black_box(accum);
            });
            b.bytes = size_of::<$ty>() as u64 * RAND_BENCH_N;
        }
    }
}

reseeding_uint!(reseeding_hc128_u32, u32);
reseeding_uint!(reseeding_hc128_u64, u64);


macro_rules! threadrng_uint {
    ($fnn:ident, $ty:ty) => {
        #[bench]
        fn $fnn(b: &mut Bencher) {
            let mut rng = thread_rng();
            b.iter(|| {
                let mut accum: $ty = 0;
                for _ in 0..RAND_BENCH_N {
                    accum = accum.wrapping_add(rng.gen::<$ty>());
                }
                black_box(accum);
            });
            b.bytes = size_of::<$ty>() as u64 * RAND_BENCH_N;
        }
    }
}

threadrng_uint!(thread_rng_u32, u32);
threadrng_uint!(thread_rng_u64, u64);
