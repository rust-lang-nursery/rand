#![feature(test)]
#![cfg_attr(all(feature="i128_support", feature="nightly"), allow(stable_features))] // stable since 2018-03-27
#![cfg_attr(all(feature="i128_support", feature="nightly"), feature(i128_type, i128))]

extern crate test;
extern crate rand;

const RAND_BENCH_N: u64 = 1000;

use std::mem::size_of;
use test::Bencher;

use rand::{Rng, FromEntropy};
use rand::rngs::SmallRng;
use rand::distributions::*;

macro_rules! distr_int {
    ($fnn:ident, $ty:ty, $distr:expr) => {
        #[bench]
        fn $fnn(b: &mut Bencher) {
            let mut rng = SmallRng::from_entropy();
            let distr = $distr;

            b.iter(|| {
                let mut accum = 0 as $ty;
                for _ in 0..::RAND_BENCH_N {
                    let x: $ty = distr.sample(&mut rng);
                    accum = accum.wrapping_add(x);
                }
                accum
            });
            b.bytes = size_of::<$ty>() as u64 * ::RAND_BENCH_N;
        }
    }
}

macro_rules! distr_float {
    ($fnn:ident, $ty:ty, $distr:expr) => {
        #[bench]
        fn $fnn(b: &mut Bencher) {
            let mut rng = SmallRng::from_entropy();
            let distr = $distr;

            b.iter(|| {
                let mut accum = 0.0;
                for _ in 0..::RAND_BENCH_N {
                    let x: $ty = distr.sample(&mut rng);
                    accum += x;
                }
                accum
            });
            b.bytes = size_of::<$ty>() as u64 * ::RAND_BENCH_N;
        }
    }
}

macro_rules! distr {
    ($fnn:ident, $ty:ty, $distr:expr) => {
        #[bench]
        fn $fnn(b: &mut Bencher) {
            let mut rng = SmallRng::from_entropy();
            let distr = $distr;

            b.iter(|| {
                let mut accum = 0u32;
                for _ in 0..::RAND_BENCH_N {
                    let x: $ty = distr.sample(&mut rng);
                    accum = accum.wrapping_add(x as u32);
                }
                accum
            });
            b.bytes = size_of::<$ty>() as u64 * ::RAND_BENCH_N;
        }
    }
}

// uniform
distr_int!(distr_uniform_i8, i8, Uniform::new(20i8, 100));
distr_int!(distr_uniform_i16, i16, Uniform::new(-500i16, 2000));
distr_int!(distr_uniform_i32, i32, Uniform::new(-200_000_000i32, 800_000_000));
distr_int!(distr_uniform_i64, i64, Uniform::new(3i64, 123_456_789_123));
#[cfg(feature = "i128_support")]
distr_int!(distr_uniform_i128, i128, Uniform::new(-123_456_789_123i128, 123_456_789_123_456_789));

distr_float!(distr_uniform_f32, f32, Uniform::new(2.26f32, 2.319));
distr_float!(distr_uniform_f64, f64, Uniform::new(2.26f64, 2.319));
distr_float!(distr_highprecision1_f32, f32, HighPrecision::new(2.26f32, 2.319));
distr_float!(distr_highprecision2_f32, f32, HighPrecision::new(-1.0f32 / 3.0, 2.319));
distr_float!(distr_highprecision3_f32, f32, HighPrecision::new(0.001f32, 123_456_789_012_345.987));
distr_float!(distr_highprecision1_f64, f64, HighPrecision::new(2.26f64, 2.319));
distr_float!(distr_highprecision2_f64, f64, HighPrecision::new(-1.0f64 / 3.0, 2.319));
distr_float!(distr_highprecision3_f64, f64, HighPrecision::new(0.001f64, 123_456_789_012_345.987));

// standard
distr_int!(distr_standard_i8, i8, Standard);
distr_int!(distr_standard_i16, i16, Standard);
distr_int!(distr_standard_i32, i32, Standard);
distr_int!(distr_standard_i64, i64, Standard);
#[cfg(feature = "i128_support")]
distr_int!(distr_standard_i128, i128, Standard);

distr!(distr_standard_bool, bool, Standard);
distr!(distr_standard_alphanumeric, char, Alphanumeric);
distr!(distr_standard_codepoint, char, Standard);

distr_float!(distr_standard_f32, f32, Standard);
distr_float!(distr_standard_f64, f64, Standard);
distr_float!(distr_open01_f32, f32, Open01);
distr_float!(distr_open01_f64, f64, Open01);
distr_float!(distr_openclosed01_f32, f32, OpenClosed01);
distr_float!(distr_openclosed01_f64, f64, OpenClosed01);
distr_float!(distr_high_precision_f32, f32, HighPrecision01);
distr_float!(distr_high_precision_f64, f64, HighPrecision01);

// distributions
distr_float!(distr_exp, f64, Exp::new(1.23 * 4.56));
distr_float!(distr_normal, f64, Normal::new(-1.23, 4.56));
distr_float!(distr_log_normal, f64, LogNormal::new(-1.23, 4.56));
distr_float!(distr_gamma_large_shape, f64, Gamma::new(10., 1.0));
distr_float!(distr_gamma_small_shape, f64, Gamma::new(0.1, 1.0));
distr_float!(distr_cauchy, f64, Cauchy::new(4.2, 6.9));
distr_int!(distr_binomial, u64, Binomial::new(20, 0.7));
distr_int!(distr_poisson, u64, Poisson::new(4.0));
distr!(distr_bernoulli, bool, Bernoulli::new(0.18));

// Weighted
distr_int!(distr_weighted_i8, usize, WeightedIndex::new(&[1i8, 2, 3, 4, 12, 0, 2, 1]).unwrap());
distr_int!(distr_weighted_u32, usize, WeightedIndex::new(&[1u32, 2, 3, 4, 12, 0, 2, 1]).unwrap());
distr_int!(distr_weighted_f64, usize, WeightedIndex::new(&[1.0f64, 0.001, 1.0/3.0, 4.01, 0.0, 3.3, 22.0, 0.001]).unwrap());
distr_int!(distr_weighted_large_set, usize, WeightedIndex::new((0..10000).rev().chain(1..10001)).unwrap());

// construct and sample from a range
macro_rules! gen_range_int {
    ($fnn:ident, $ty:ident, $low:expr, $high:expr) => {
        #[bench]
        fn $fnn(b: &mut Bencher) {
            let mut rng = SmallRng::from_entropy();

            b.iter(|| {
                let mut high = $high;
                let mut accum: $ty = 0;
                for _ in 0..::RAND_BENCH_N {
                    accum = accum.wrapping_add(rng.gen_range($low, high));
                    // force recalculation of range each time
                    high = high.wrapping_add(1) & std::$ty::MAX;
                }
                accum
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

// construct and sample from a floating-point range
macro_rules! gen_range_float {
    ($fnn:ident, $ty:ident, $low:expr, $high:expr) => {
        #[bench]
        fn $fnn(b: &mut Bencher) {
            let mut rng = SmallRng::from_entropy();

            b.iter(|| {
                let mut high = $high;
                let mut low = $low;
                let mut accum: $ty = 0.0;
                for _ in 0..::RAND_BENCH_N {
                    accum += rng.gen_range(low, high);
                    // force recalculation of range each time
                    low += 0.9;
                    high += 1.1;
                }
                accum
            });
            b.bytes = size_of::<$ty>() as u64 * ::RAND_BENCH_N;
        }
    }
}

gen_range_float!(gen_range_f32, f32, -20000.0f32, 100000.0);
gen_range_float!(gen_range_f64, f64, 123.456f64, 7890.12);

#[bench]
fn dist_iter(b: &mut Bencher) {
    let mut rng = SmallRng::from_entropy();
    let distr = Normal::new(-2.71828, 3.14159);
    let mut iter = distr.sample_iter(&mut rng);

    b.iter(|| {
        let mut accum = 0.0;
        for _ in 0..::RAND_BENCH_N {
            accum += iter.next().unwrap();
        }
        accum
    });
    b.bytes = size_of::<f64>() as u64 * ::RAND_BENCH_N;
}
