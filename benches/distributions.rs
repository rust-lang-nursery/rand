#![feature(test)]
#![cfg_attr(feature = "i128_support", feature(i128_type, i128))]

extern crate test;
extern crate rand;

const RAND_BENCH_N: u64 = 1000;

use std::mem::size_of;
use test::{black_box, Bencher};

use rand::{Rng, FromEntropy, XorShiftRng};
use rand::distributions::*;

macro_rules! distr_int {
    ($fnn:ident, $ty:ty, $distr:expr) => {
        #[bench]
        fn $fnn(b: &mut Bencher) {
            let mut rng = XorShiftRng::from_entropy();
            let distr = $distr;

            b.iter(|| {
                let mut accum = 0 as $ty;
                for _ in 0..::RAND_BENCH_N {
                    let x: $ty = distr.sample(&mut rng);
                    accum = accum.wrapping_add(x);
                }
                black_box(accum);
            });
            b.bytes = size_of::<$ty>() as u64 * ::RAND_BENCH_N;
        }
    }
}

macro_rules! distr_float {
    ($fnn:ident, $ty:ty, $distr:expr) => {
        #[bench]
        fn $fnn(b: &mut Bencher) {
            let mut rng = XorShiftRng::from_entropy();
            let distr = $distr;

            b.iter(|| {
                let mut accum = 0.0;
                for _ in 0..::RAND_BENCH_N {
                    let x: $ty = distr.sample(&mut rng);
                    accum += x;
                }
                black_box(accum);
            });
            b.bytes = size_of::<$ty>() as u64 * ::RAND_BENCH_N;
        }
    }
}

macro_rules! distr {
    ($fnn:ident, $ty:ty, $distr:expr) => {
        #[bench]
        fn $fnn(b: &mut Bencher) {
            let mut rng = XorShiftRng::from_entropy();
            let distr = $distr;

            b.iter(|| {
                for _ in 0..::RAND_BENCH_N {
                    let x: $ty = distr.sample(&mut rng);
                    black_box(x);
                }
            });
            b.bytes = size_of::<$ty>() as u64 * ::RAND_BENCH_N;
        }
    }
}

// range
distr_int!(distr_range_i8, i8, Range::new(20i8, 100));
distr_int!(distr_range_i16, i16, Range::new(-500i16, 2000));
distr_int!(distr_range_i32, i32, Range::new(-200_000_000i32, 800_000_000));
distr_int!(distr_range_i64, i64, Range::new(3i64, 123_456_789_123));
#[cfg(feature = "i128_support")]
distr_int!(distr_range_i128, i128, Range::new(-123_456_789_123i128, 123_456_789_123_456_789));

distr_float!(distr_range_f32, f32, Range::new(2.26f32, 2.319));
distr_float!(distr_range_f64, f64, Range::new(2.26f64, 2.319));

// uniform
distr_int!(distr_uniform_i8, i8, Standard);
distr_int!(distr_uniform_i16, i16, Standard);
distr_int!(distr_uniform_i32, i32, Standard);
distr_int!(distr_uniform_i64, i64, Standard);
#[cfg(feature = "i128_support")]
distr_int!(distr_uniform_i128, i128, Standard);

distr!(distr_uniform_bool, bool, Standard);
distr!(distr_uniform_alphanumeric, char, Alphanumeric);
distr!(distr_uniform_codepoint, char, Standard);

distr_float!(distr_uniform_f32, f32, Standard);
distr_float!(distr_uniform_f64, f64, Standard);
distr_float!(distr_high_precision_f32, f32, HighPrecision01);
distr_float!(distr_high_precision_f64, f64, HighPrecision01);

// distributions
distr_float!(distr_exp, f64, Exp::new(1.23 * 4.56));
distr_float!(distr_normal, f64, Normal::new(-1.23, 4.56));
distr_float!(distr_log_normal, f64, LogNormal::new(-1.23, 4.56));
distr_float!(distr_gamma_large_shape, f64, Gamma::new(10., 1.0));
distr_float!(distr_gamma_small_shape, f64, Gamma::new(0.1, 1.0));
distr_int!(distr_binomial, u64, Binomial::new(20, 0.7));
distr_int!(distr_poisson, u64, Poisson::new(4.0));


// construct and sample from a range
macro_rules! gen_range_int {
    ($fnn:ident, $ty:ident, $low:expr, $high:expr) => {
        #[bench]
        fn $fnn(b: &mut Bencher) {
            let mut rng = XorShiftRng::from_entropy();

            b.iter(|| {
                let mut high = $high;
                let mut accum: $ty = 0;
                for _ in 0..::RAND_BENCH_N {
                    accum = accum.wrapping_add(rng.gen_range($low, high));
                    // force recalculation of range each time
                    high = high.wrapping_add(1) & std::$ty::MAX;
                }
                black_box(accum);
            });
            b.bytes = size_of::<$ty>() as u64 * ::RAND_BENCH_N;
        }
    }
}

gen_range_int!(gen_range_i8, i8, -20i8, 100);
gen_range_int!(gen_range_i16, i16, -500i16, 2000);
gen_range_int!(gen_range_i32, i32, -200_000_000i32, 800_000_000);
gen_range_int!(gen_range_i64, i64, 3i64, 123_456_789_123);
#[cfg(feature = "i128_support")]
gen_range_int!(gen_range_i128, i128, -12345678901234i128, 123_456_789_123_456_789);

#[bench]
fn dist_iter(b: &mut Bencher) {
    let mut rng = XorShiftRng::from_entropy();
    let distr = Normal::new(-2.71828, 3.14159);
    let mut iter = distr.sample_iter(&mut rng);

    b.iter(|| {
        let mut accum = 0.0;
        for _ in 0..::RAND_BENCH_N {
            accum += iter.next().unwrap();
        }
        black_box(accum);
    });
    b.bytes = size_of::<f64>() as u64 * ::RAND_BENCH_N;
}
