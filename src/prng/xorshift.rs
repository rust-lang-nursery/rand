// Copyright 2017 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Xorshift generators

use core::num::Wrapping as w;
use core::fmt;
use {Rng, SeedFromRng, SeedableRng, Error};
use rand_core::le;

/// An Xorshift[1] random number
/// generator.
///
/// The Xorshift algorithm is not suitable for cryptographic purposes
/// but is very fast. If you do not know for sure that it fits your
/// requirements, use a more secure one such as `IsaacRng` or `OsRng`.
///
/// [1]: Marsaglia, George (July 2003). ["Xorshift
/// RNGs"](http://www.jstatsoft.org/v08/i14/paper). *Journal of
/// Statistical Software*. Vol. 8 (Issue 14).
#[derive(Clone)]
pub struct XorShiftRng {
    x: w<u32>,
    y: w<u32>,
    z: w<u32>,
    w: w<u32>,
}

// Custom Debug implementation that does not expose the internal state
impl fmt::Debug for XorShiftRng {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "XorShiftRng {{}}")
    }
}

impl XorShiftRng {
    /// Creates a new XorShiftRng instance which is not seeded.
    ///
    /// The initial values of this RNG are constants, so all generators created
    /// by this function will yield the same stream of random numbers. It is
    /// highly recommended that this is created through `SeedableRng` instead of
    /// this function
    pub fn new_unseeded() -> XorShiftRng {
        XorShiftRng {
            x: w(0x193a6754),
            y: w(0xa8a7d469),
            z: w(0x97830e05),
            w: w(0x113ba7bb),
        }
    }
}

impl SeedFromRng for XorShiftRng {
    fn from_rng<R: Rng>(mut rng: R) -> Result<Self, Error> {
        let mut tuple: (u32, u32, u32, u32);
        loop {
            tuple = (rng.next_u32(), rng.next_u32(), rng.next_u32(), rng.next_u32());
            if tuple != (0, 0, 0, 0) {
                break;
            }
        }
        let (x, y, z, w_) = tuple;
        Ok(XorShiftRng { x: w(x), y: w(y), z: w(z), w: w(w_) })
    }
}

impl Rng for XorShiftRng {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        let x = self.x;
        let t = x ^ (x << 11);
        self.x = self.y;
        self.y = self.z;
        self.z = self.w;
        let w_ = self.w;
        self.w = w_ ^ (w_ >> 19) ^ (t ^ (t >> 8));
        self.w.0
    }
    
    fn next_u64(&mut self) -> u64 {
        ::rand_core::impls::next_u64_via_u32(self)
    }
    #[cfg(feature = "i128_support")]
    fn next_u128(&mut self) -> u128 {
        ::rand_core::impls::next_u128_via_u64(self)
    }
    
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        ::rand_core::impls::fill_bytes_via_u32(self, dest);
    }

    fn try_fill(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        Ok(self.fill_bytes(dest))
    }
}

impl SeedableRng for XorShiftRng {
    type Seed = [u8; 16];
    /// Create a new XorShiftRng. This will panic if `seed` is entirely 0.
    fn from_seed(seed: Self::Seed) -> Self {
        assert!(!seed.iter().all(|&x| x == 0),
                "XorShiftRng::from_seed called with an all zero seed.");

        XorShiftRng {
            x: w(le::read_u32(array_ref!(seed, 0, 4))),
            y: w(le::read_u32(array_ref!(seed, 4, 4))),
            z: w(le::read_u32(array_ref!(seed, 8, 4))),
            w: w(le::read_u32(array_ref!(seed, 12, 4))),
        }
    }
}
