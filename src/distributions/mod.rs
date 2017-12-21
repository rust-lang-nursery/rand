// Copyright 2013 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Sampling from random distributions.
//!
//! A distribution may have internal state describing the distribution of
//! generated values; for example `Range` needs to know its upper and lower
//! bounds. Distributions use the `Distribution` trait to yield values: call
//! `distr.sample(&mut rng)` to get a random variable.

use Rng;

pub use self::default::Default;
pub use self::uniform::{Uniform, Uniform01, Open01, Closed01, Codepoint, Alphanumeric};
pub use self::range::Range;

#[cfg(feature="std")]
pub use self::gamma::{Gamma, ChiSquared, FisherF, StudentT};
#[cfg(feature="std")]
pub use self::normal::{Normal, LogNormal};
#[cfg(feature="std")]
pub use self::exponential::Exp;

use Sample;

mod default;
mod uniform;
#[cfg(feature="std")]
mod ziggurat_tables;

pub mod range;

#[cfg(feature="std")]
pub mod gamma;
#[cfg(feature="std")]
pub mod normal;
#[cfg(feature="std")]
pub mod exponential;


/// Return a bool with a 1 in n chance of being true
/// 
/// This uses [`range`] internally, so for repeated uses it would be faster to
/// create a `Range` distribution and test its samples:
/// `range.sample(rng) == 0`.
///
/// # Example
///
/// ```rust
/// use rand::distributions::weighted_bool;
///
/// let mut rng = rand::thread_rng();
/// println!("{}", weighted_bool(3, &mut rng));
/// ```
pub fn weighted_bool<R: Rng+?Sized>(n: u32, rng: &mut R) -> bool {
    n <= 1 || rng.gen_range(0, n) == 0
}

/// Types (distributions) that can be used to create a random instance of `T`.
pub trait Distribution<T> {
    /// Generate a random value of `T`, using `rng` as the
    /// source of randomness.
    fn sample<R: Rng+?Sized>(&self, rng: &mut R) -> T;
}

impl<'a, T, D: Distribution<T>> Distribution<T> for &'a D {
    fn sample<R: Rng+?Sized>(&self, rng: &mut R) -> T {
        (*self).sample(rng)
    }
}

/// Trait for sampling values from the `Default` distribution.
/// 
/// This is mostly a shim around `Default` for backwards compatibility; the
/// implementation is simply `Default.sample(rng)`.
/// 
/// # Example
/// ```rust
/// use rand::{Rand, thread_rng};
/// 
/// let mut rng = thread_rng();
/// println!("Random byte: {}", u8::rand(&mut rng));
/// ```
pub trait Rand : Sized {
    /// Generate a random value of the given type, using the `Default`
    /// distribution.
    fn rand<R: Rng+?Sized>(rng: &mut R) -> Self;
}

impl<T> Rand for T where Default: Distribution<T> {
    fn rand<R: Rng+?Sized>(rng: &mut R) -> Self {
        Default.sample(rng)
    }
}

/// Sample a random number using the Ziggurat method (specifically the
/// ZIGNOR variant from Doornik 2005). Most of the arguments are
/// directly from the paper:
///
/// * `rng`: source of randomness
/// * `symmetric`: whether this is a symmetric distribution, or one-sided with P(x < 0) = 0.
/// * `X`: the $x_i$ abscissae.
/// * `F`: precomputed values of the PDF at the $x_i$, (i.e. $f(x_i)$)
/// * `F_DIFF`: precomputed values of $f(x_i) - f(x_{i+1})$
/// * `pdf`: the probability density function
/// * `zero_case`: manual sampling from the tail when we chose the
///    bottom box (i.e. i == 0)

// the perf improvement (25-50%) is definitely worth the extra code
// size from force-inlining.
#[cfg(feature="std")]
#[inline(always)]
fn ziggurat<R: Rng+?Sized, P, Z>(
            rng: &mut R,
            symmetric: bool,
            x_tab: ziggurat_tables::ZigTable,
            f_tab: ziggurat_tables::ZigTable,
            mut pdf: P,
            mut zero_case: Z)
            -> f64 where P: FnMut(f64) -> f64, Z: FnMut(&mut R, f64) -> f64 {
    use utils::FloatConversions;
    
    loop {
        // As an optimisation convert the random u64 to a f64 using only
        // 53 bits, as many as will fit in the float's fraction.
        // Of the remaining 11 least significant bits we use 8 to construct `i`.
        // This saves us generating a whole extra random number, while the added
        // precision of using 64 bits for f64 does not buy us much.
        // Because for some RNG's the least significant bits can be of lower
        // statistical quality, we use bits 3..10 for i.
        let bits: u64 = rng.sample(Uniform);

        // u is either U(-1, 1) or U(0, 1) depending on if this is a
        // symmetric distribution or not.
        // FIXME: the distribution is not open, but closed-open.
        //        Can that cause problems or a bias?
        let u = if symmetric {
                    bits.closed_open11_fixed()
                } else {
                    bits.closed_open01_fixed()
                };
        let i = ((bits >> 3) & 0xff) as usize;

        let x = u * x_tab[i];

        let test_x = if symmetric { x.abs() } else {x};

        // algebraically equivalent to |u| < x_tab[i+1]/x_tab[i] (or u < x_tab[i+1]/x_tab[i])
        if test_x < x_tab[i + 1] {
            return x;
        }
        if i == 0 {
            return zero_case(rng, u);
        }
        // algebraically equivalent to f1 + DRanU()*(f0 - f1) < 1
        if f_tab[i + 1] + (f_tab[i] - f_tab[i + 1]) * rng.sample::<f64, _>(Uniform01) < pdf(x) {
            return x;
        }
    }
}

#[cfg(test)]
mod test {
    use {Rng, thread_rng};
    use distributions::{weighted_bool};

    #[test]
    fn test_fn_weighted_bool() {
        let mut r = thread_rng();
        assert_eq!(weighted_bool(0, &mut r), true);
        let s: &mut Rng = &mut r;
        assert_eq!(weighted_bool(1, s), true);
    }
}
