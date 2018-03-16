// Copyright 2013-2017 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// https://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Sampling from random distributions.
//!
//! A distribution may have internal state describing the distribution of
//! generated values; for example `Range` needs to know its upper and lower
//! bounds. Distributions use the `Distribution` trait to yield values: call
//! `distr.sample(&mut rng)` to get a random variable.

use Rng;

pub use self::other::Alphanumeric;
pub use self::range::Range;
#[cfg(feature="std")]
pub use self::gamma::{Gamma, ChiSquared, FisherF, StudentT};
#[cfg(feature="std")]
pub use self::normal::{Normal, LogNormal, StandardNormal};
#[cfg(feature="std")]
pub use self::exponential::{Exp, Exp1};

pub mod range;
#[cfg(feature="std")]
pub mod gamma;
#[cfg(feature="std")]
pub mod normal;
#[cfg(feature="std")]
pub mod exponential;

mod float;
mod integer;
mod other;
#[cfg(feature="std")]
mod ziggurat_tables;
#[cfg(feature="std")]
use distributions::float::IntoFloat;

/// Types that can be used to create a random instance of `Support`.
#[deprecated(since="0.5.0", note="use Distribution instead")]
pub trait Sample<Support> {
    /// Generate a random value of `Support`, using `rng` as the
    /// source of randomness.
    fn sample<R: Rng>(&mut self, rng: &mut R) -> Support;
}

/// `Sample`s that do not require keeping track of state.
///
/// Since no state is recorded, each sample is (statistically)
/// independent of all others, assuming the `Rng` used has this
/// property.
#[allow(deprecated)]
#[deprecated(since="0.5.0", note="use Distribution instead")]
pub trait IndependentSample<Support>: Sample<Support> {
    /// Generate a random value.
    fn ind_sample<R: Rng>(&self, &mut R) -> Support;
}

#[allow(deprecated)]
mod impls {
    use Rng;
    use distributions::{Distribution, Sample, IndependentSample,
            WeightedChoice};
    #[cfg(feature="std")]
    use distributions::exponential::Exp;
    #[cfg(feature="std")]
    use distributions::gamma::{Gamma, ChiSquared, FisherF, StudentT};
    #[cfg(feature="std")]
    use distributions::normal::{Normal, LogNormal};
    use distributions::range::{Range, RangeImpl};
    
    impl<'a, T: Clone> Sample<T> for WeightedChoice<'a, T> {
        fn sample<R: Rng>(&mut self, rng: &mut R) -> T {
            Distribution::sample(self, rng)
        }
    }
    impl<'a, T: Clone> IndependentSample<T> for WeightedChoice<'a, T> {
        fn ind_sample<R: Rng>(&self, rng: &mut R) -> T {
            Distribution::sample(self, rng)
        }
    }
    
    impl<T: RangeImpl> Sample<T::X> for Range<T> {
        fn sample<R: Rng>(&mut self, rng: &mut R) -> T::X {
            Distribution::sample(self, rng)
        }
    }
    impl<T: RangeImpl> IndependentSample<T::X> for Range<T> {
        fn ind_sample<R: Rng>(&self, rng: &mut R) -> T::X {
            Distribution::sample(self, rng)
        }
    }
    
    #[cfg(feature="std")]
    macro_rules! impl_f64 {
        ($($name: ident), *) => {
            $(
                impl Sample<f64> for $name {
                    fn sample<R: Rng>(&mut self, rng: &mut R) -> f64 {
                        Distribution::sample(self, rng)
                    }
                }
                impl IndependentSample<f64> for $name {
                    fn ind_sample<R: Rng>(&self, rng: &mut R) -> f64 {
                        Distribution::sample(self, rng)
                    }
                }
            )*
        }
    }
    #[cfg(feature="std")]
    impl_f64!(Exp, Gamma, ChiSquared, FisherF, StudentT, Normal, LogNormal);
}

/// Types (distributions) that can be used to create a random instance of `T`.
pub trait Distribution<T> {
    /// Generate a random value of `T`, using `rng` as the
    /// source of randomness.
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> T;
}

impl<'a, T, D: Distribution<T>> Distribution<T> for &'a D {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> T {
        (*self).sample(rng)
    }
}

/// A generic random value distribution. Generates values for various types
/// with numerically uniform distribution.
/// 
/// For floating-point numbers, this generates values from the open range
/// `(0, 1)` (i.e. excluding 0.0 and 1.0).
///
/// ## Built-in Implementations
///
/// This crate implements the distribution `Uniform` for various primitive
/// types.  Assuming the provided `Rng` is well-behaved, these implementations
/// generate values with the following ranges and distributions:
///
/// * Integers (`i32`, `u32`, `isize`, `usize`, etc.): Uniformly distributed
///   over all values of the type.
/// * `char`: Uniformly distributed over all Unicode scalar values, i.e. all
///   code points in the range `0...0x10_FFFF`, except for the range
///   `0xD800...0xDFFF` (the surrogate code points). This includes
///   unassigned/reserved code points.
/// * `bool`: Generates `false` or `true`, each with probability 0.5.
/// * Floating point types (`f32` and `f64`): Uniformly distributed in the
///   open range `(0, 1)`.
///
/// The following aggregate types also implement the distribution `Uniform` as
/// long as their component types implement it:
///
/// * Tuples and arrays: Each element of the tuple or array is generated
///   independently, using the `Uniform` distribution recursively.
/// * `Option<T>`: Returns `None` with probability 0.5; otherwise generates a
///   random `T` and returns `Some(T)`.
///
/// # Example
/// ```rust
/// use rand::{NewRng, SmallRng, Rng};
/// use rand::distributions::Uniform;
///
/// let val: f32 = SmallRng::new().sample(Uniform);
/// println!("f32 from [0,1): {}", val);
/// ```
///
/// With dynamic dispatch (type erasure of `Rng`):
/// 
/// ```rust
/// use rand::{thread_rng, Rng, RngCore, Void};
/// use rand::distributions::Uniform;
///
/// let mut rng = thread_rng();
/// let mut erased_rng: &mut RngCore<Error=Void> = &mut rng;
/// let val: f32 = erased_rng.sample(Uniform);
/// println!("f32 from [0,1): {}", val);
/// ```
///
/// [`Exp1`]: struct.Exp1.html
/// [`StandardNormal`]: struct.StandardNormal.html
#[derive(Debug)]
pub struct Uniform;

#[allow(deprecated)]
impl<T> ::Rand for T where Uniform: Distribution<T> {
    fn rand<R: Rng>(rng: &mut R) -> Self {
        Uniform.sample(rng)
    }
}


/// A value with a particular weight for use with `WeightedChoice`.
#[derive(Copy, Clone, Debug)]
pub struct Weighted<T> {
    /// The numerical weight of this item
    pub weight: u32,
    /// The actual item which is being weighted
    pub item: T,
}

/// A distribution that selects from a finite collection of weighted items.
///
/// Each item has an associated weight that influences how likely it
/// is to be chosen: higher weight is more likely.
///
/// The `Clone` restriction is a limitation of the `Distribution` trait.
/// Note that `&T` is (cheaply) `Clone` for all `T`, as is `u32`, so one can
/// store references or indices into another vector.
///
/// # Example
///
/// ```rust
/// use rand::distributions::{Weighted, WeightedChoice, Distribution};
///
/// let mut items = vec!(Weighted { weight: 2, item: 'a' },
///                      Weighted { weight: 4, item: 'b' },
///                      Weighted { weight: 1, item: 'c' });
/// let wc = WeightedChoice::new(&mut items);
/// let mut rng = rand::thread_rng();
/// for _ in 0..16 {
///      // on average prints 'a' 4 times, 'b' 8 and 'c' twice.
///      println!("{}", wc.sample(&mut rng));
/// }
/// ```
#[derive(Debug)]
pub struct WeightedChoice<'a, T:'a> {
    items: &'a mut [Weighted<T>],
    weight_range: Range<range::RangeInt<u32>>,
}

impl<'a, T: Clone> WeightedChoice<'a, T> {
    /// Create a new `WeightedChoice`.
    ///
    /// Panics if:
    ///
    /// - `items` is empty
    /// - the total weight is 0
    /// - the total weight is larger than a `u32` can contain.
    pub fn new(items: &'a mut [Weighted<T>]) -> WeightedChoice<'a, T> {
        // strictly speaking, this is subsumed by the total weight == 0 case
        assert!(!items.is_empty(), "WeightedChoice::new called with no items");

        let mut running_total: u32 = 0;

        // we convert the list from individual weights to cumulative
        // weights so we can binary search. This *could* drop elements
        // with weight == 0 as an optimisation.
        for item in items.iter_mut() {
            running_total = match running_total.checked_add(item.weight) {
                Some(n) => n,
                None => panic!("WeightedChoice::new called with a total weight \
                               larger than a u32 can contain")
            };

            item.weight = running_total;
        }
        assert!(running_total != 0, "WeightedChoice::new called with a total weight of 0");

        WeightedChoice {
            items: items,
            // we're likely to be generating numbers in this range
            // relatively often, so might as well cache it
            weight_range: Range::new(0, running_total)
        }
    }
}

impl<'a, T: Clone> Distribution<T> for WeightedChoice<'a, T> {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> T {
        // we want to find the first element that has cumulative
        // weight > sample_weight, which we do by binary since the
        // cumulative weights of self.items are sorted.

        // choose a weight in [0, total_weight)
        let sample_weight = self.weight_range.sample(rng);

        // short circuit when it's the first item
        if sample_weight < self.items[0].weight {
            return self.items[0].item.clone();
        }

        let mut idx = 0;
        let mut modifier = self.items.len();

        // now we know that every possibility has an element to the
        // left, so we can just search for the last element that has
        // cumulative weight <= sample_weight, then the next one will
        // be "it". (Note that this greatest element will never be the
        // last element of the vector, since sample_weight is chosen
        // in [0, total_weight) and the cumulative weight of the last
        // one is exactly the total weight.)
        while modifier > 1 {
            let i = idx + modifier / 2;
            if self.items[i].weight <= sample_weight {
                // we're small, so look to the right, but allow this
                // exact element still.
                idx = i;
                // we need the `/ 2` to round up otherwise we'll drop
                // the trailing elements when `modifier` is odd.
                modifier += 1;
            } else {
                // otherwise we're too big, so go left. (i.e. do
                // nothing)
            }
            modifier /= 2;
        }
        return self.items[idx + 1].item.clone();
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
fn ziggurat<R: Rng + ?Sized, P, Z>(
            rng: &mut R,
            symmetric: bool,
            x_tab: ziggurat_tables::ZigTable,
            f_tab: ziggurat_tables::ZigTable,
            mut pdf: P,
            mut zero_case: Z)
            -> f64 where P: FnMut(f64) -> f64, Z: FnMut(&mut R, f64) -> f64 {
    loop {
        // As an optimisation we re-implement the conversion to a f64.
        // From the remaining 12 most significant bits we use 8 to construct `i`.
        // This saves us generating a whole extra random number, while the added
        // precision of using 64 bits for f64 does not buy us much.
        let bits = rng.next_u64();
        let i = bits as usize & 0xff;

        let u = if symmetric {
            // Convert to a value in the range [2,4) and substract to get [-1,1)
            // We can't convert to an open range directly, that would require
            // substracting `3.0 - EPSILON`, which is not representable.
            // It is possible with an extra step, but an open range does not
            // seem neccesary for the ziggurat algorithm anyway.
            (bits >> 12).into_float_with_exponent(1) - 3.0
        } else {
            // Convert to a value in the range [1,2) and substract to get (0,1)
            (bits >> 12).into_float_with_exponent(0)
            - (1.0 - ::core::f64::EPSILON / 2.0)
        };
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
        if f_tab[i + 1] + (f_tab[i] - f_tab[i + 1]) * rng.gen::<f64>() < pdf(x) {
            return x;
        }
    }
}

#[cfg(test)]
mod tests {
    use Rng;
    use mock::StepRng;
    use super::{WeightedChoice, Weighted, Distribution};

    #[test]
    fn test_weighted_choice() {
        // this makes assumptions about the internal implementation of
        // WeightedChoice. It may fail when the implementation in
        // `distributions::range::RangeInt changes.

        macro_rules! t {
            ($items:expr, $expected:expr) => {{
                let mut items = $items;
                let mut total_weight = 0;
                for item in &items { total_weight += item.weight; }

                let wc = WeightedChoice::new(&mut items);
                let expected = $expected;

                // Use extremely large steps between the random numbers, because
                // we test with small ranges and RangeInt is designed to prefer
                // the most significant bits.
                let mut rng = StepRng::new(0, !0 / (total_weight as u64));

                for &val in expected.iter() {
                    assert_eq!(wc.sample(&mut rng), val)
                }
            }}
        }

        t!([Weighted { weight: 1, item: 10}], [10]);

        // skip some
        t!([Weighted { weight: 0, item: 20},
            Weighted { weight: 2, item: 21},
            Weighted { weight: 0, item: 22},
            Weighted { weight: 1, item: 23}],
           [21, 21, 23]);

        // different weights
        t!([Weighted { weight: 4, item: 30},
            Weighted { weight: 3, item: 31}],
           [30, 31, 30, 31, 30, 31, 30]);

        // check that we're binary searching
        // correctly with some vectors of odd
        // length.
        t!([Weighted { weight: 1, item: 40},
            Weighted { weight: 1, item: 41},
            Weighted { weight: 1, item: 42},
            Weighted { weight: 1, item: 43},
            Weighted { weight: 1, item: 44}],
           [40, 41, 42, 43, 44]);
        t!([Weighted { weight: 1, item: 50},
            Weighted { weight: 1, item: 51},
            Weighted { weight: 1, item: 52},
            Weighted { weight: 1, item: 53},
            Weighted { weight: 1, item: 54},
            Weighted { weight: 1, item: 55},
            Weighted { weight: 1, item: 56}],
           [50, 54, 51, 55, 52, 56, 53]);
    }

    #[test]
    fn test_weighted_clone_initialization() {
        let initial : Weighted<u32> = Weighted {weight: 1, item: 1};
        let clone = initial.clone();
        assert_eq!(initial.weight, clone.weight);
        assert_eq!(initial.item, clone.item);
    }

    #[test] #[should_panic]
    fn test_weighted_clone_change_weight() {
        let initial : Weighted<u32> = Weighted {weight: 1, item: 1};
        let mut clone = initial.clone();
        clone.weight = 5;
        assert_eq!(initial.weight, clone.weight);
    }

    #[test] #[should_panic]
    fn test_weighted_clone_change_item() {
        let initial : Weighted<u32> = Weighted {weight: 1, item: 1};
        let mut clone = initial.clone();
        clone.item = 5;
        assert_eq!(initial.item, clone.item);

    }

    #[test] #[should_panic]
    fn test_weighted_choice_no_items() {
        WeightedChoice::<isize>::new(&mut []);
    }
    #[test] #[should_panic]
    fn test_weighted_choice_zero_weight() {
        WeightedChoice::new(&mut [Weighted { weight: 0, item: 0},
                                  Weighted { weight: 0, item: 1}]);
    }
    #[test] #[should_panic]
    fn test_weighted_choice_weight_overflows() {
        let x = ::core::u32::MAX / 2; // x + x + 2 is the overflow
        WeightedChoice::new(&mut [Weighted { weight: x, item: 0 },
                                  Weighted { weight: 1, item: 1 },
                                  Weighted { weight: x, item: 2 },
                                  Weighted { weight: 1, item: 3 }]);
    }
    
    #[test] #[allow(deprecated)]
    fn test_backwards_compat_sample() {
        use distributions::{Sample, IndependentSample};
        
        struct Constant<T> { val: T }
        impl<T: Copy> Sample<T> for Constant<T> {
            fn sample<R: Rng>(&mut self, _: &mut R) -> T { self.val }
        }
        impl<T: Copy> IndependentSample<T> for Constant<T> {
            fn ind_sample<R: Rng>(&self, _: &mut R) -> T { self.val }
        }
        
        let mut sampler = Constant{ val: 293 };
        assert_eq!(sampler.sample(&mut ::test::rng(233)), 293);
        assert_eq!(sampler.ind_sample(&mut ::test::rng(234)), 293);
    }
    
    #[cfg(feature="std")]
    #[test] #[allow(deprecated)]
    fn test_backwards_compat_exp() {
        use distributions::{IndependentSample, Exp};
        let sampler = Exp::new(1.0);
        sampler.ind_sample(&mut ::test::rng(235));
    }
}
