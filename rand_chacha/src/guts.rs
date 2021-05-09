// Copyright 2019 The CryptoCorrosion Contributors
// Copyright 2020 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! The ChaCha random number generator.

use ppv_lite86::{dispatch, dispatch_light128};

pub use ppv_lite86::Machine;
use ppv_lite86::{vec128_storage, ArithOps, BitOps32, LaneWords4, MultiLane, StoreBytes, Vec4};

#[cfg(feature = "serde1")]
use serde::de::{self, Deserializer, MapAccess, SeqAccess, Visitor};
#[cfg(feature = "serde1")] use serde::ser::SerializeStruct;
#[cfg(feature = "serde1")] use serde::{Deserialize, Serialize, Serializer};
#[cfg(feature = "serde1")] use std::fmt;

pub(crate) const BLOCK: usize = 64;
pub(crate) const BLOCK64: u64 = BLOCK as u64;
const LOG2_BUFBLOCKS: u64 = 2;
const BUFBLOCKS: u64 = 1 << LOG2_BUFBLOCKS;
pub(crate) const BUFSZ64: u64 = BLOCK64 * BUFBLOCKS;
pub(crate) const BUFSZ: usize = BUFSZ64 as usize;

#[derive(Clone, PartialEq, Eq)]
pub struct ChaCha {
    pub(crate) b: vec128_storage,
    pub(crate) c: vec128_storage,
    pub(crate) d: vec128_storage,
}

#[derive(Clone)]
pub struct State<V> {
    pub(crate) a: V,
    pub(crate) b: V,
    pub(crate) c: V,
    pub(crate) d: V,
}

#[inline(always)]
pub(crate) fn round<V: ArithOps + BitOps32>(mut x: State<V>) -> State<V> {
    x.a += x.b;
    x.d = (x.d ^ x.a).rotate_each_word_right16();
    x.c += x.d;
    x.b = (x.b ^ x.c).rotate_each_word_right20();
    x.a += x.b;
    x.d = (x.d ^ x.a).rotate_each_word_right24();
    x.c += x.d;
    x.b = (x.b ^ x.c).rotate_each_word_right25();
    x
}

#[inline(always)]
pub(crate) fn diagonalize<V: LaneWords4>(mut x: State<V>) -> State<V> {
    x.b = x.b.shuffle_lane_words3012();
    x.c = x.c.shuffle_lane_words2301();
    x.d = x.d.shuffle_lane_words1230();
    x
}
#[inline(always)]
pub(crate) fn undiagonalize<V: LaneWords4>(mut x: State<V>) -> State<V> {
    x.b = x.b.shuffle_lane_words1230();
    x.c = x.c.shuffle_lane_words2301();
    x.d = x.d.shuffle_lane_words3012();
    x
}

impl ChaCha {
    #[inline(always)]
    pub fn new(key: &[u8; 32], nonce: &[u8]) -> Self {
        init_chacha(key, nonce)
    }

    #[inline(always)]
    fn pos64<M: Machine>(&self, m: M) -> u64 {
        let d: M::u32x4 = m.unpack(self.d);
        ((d.extract(1) as u64) << 32) | d.extract(0) as u64
    }

    /// Produce 4 blocks of output, advancing the state
    #[inline(always)]
    pub fn refill4(&mut self, drounds: u32, out: &mut [u8; BUFSZ]) {
        refill_wide(self, drounds, out)
    }

    #[inline(always)]
    pub fn set_stream_param(&mut self, param: u32, value: u64) {
        set_stream_param(self, param, value)
    }

    #[inline(always)]
    pub fn get_stream_param(&self, param: u32) -> u64 {
        get_stream_param(self, param)
    }

    /// Return whether rhs is equal in all parameters except current 64-bit position.
    #[inline]
    pub fn stream64_eq(&self, rhs: &Self) -> bool {
        let self_d: [u32; 4] = self.d.into();
        let rhs_d: [u32; 4] = rhs.d.into();
        self.b == rhs.b && self.c == rhs.c && self_d[3] == rhs_d[3] && self_d[2] == rhs_d[2]
    }
}

#[allow(clippy::many_single_char_names)]
#[inline(always)]
fn refill_wide_impl<Mach: Machine>(
    m: Mach, state: &mut ChaCha, drounds: u32, out: &mut [u8; BUFSZ],
) {
    let k = m.vec([0x6170_7865, 0x3320_646e, 0x7962_2d32, 0x6b20_6574]);
    let mut pos = state.pos64(m);
    let d0: Mach::u32x4 = m.unpack(state.d);
    pos = pos.wrapping_add(1);
    let d1 = d0.insert((pos >> 32) as u32, 1).insert(pos as u32, 0);
    pos = pos.wrapping_add(1);
    let d2 = d0.insert((pos >> 32) as u32, 1).insert(pos as u32, 0);
    pos = pos.wrapping_add(1);
    let d3 = d0.insert((pos >> 32) as u32, 1).insert(pos as u32, 0);

    let b = m.unpack(state.b);
    let c = m.unpack(state.c);
    let mut x = State {
        a: Mach::u32x4x4::from_lanes([k, k, k, k]),
        b: Mach::u32x4x4::from_lanes([b, b, b, b]),
        c: Mach::u32x4x4::from_lanes([c, c, c, c]),
        d: m.unpack(Mach::u32x4x4::from_lanes([d0, d1, d2, d3]).into()),
    };
    for _ in 0..drounds {
        x = round(x);
        x = undiagonalize(round(diagonalize(x)));
    }
    let mut pos = state.pos64(m);
    let d0: Mach::u32x4 = m.unpack(state.d);
    pos = pos.wrapping_add(1);
    let d1 = d0.insert((pos >> 32) as u32, 1).insert(pos as u32, 0);
    pos = pos.wrapping_add(1);
    let d2 = d0.insert((pos >> 32) as u32, 1).insert(pos as u32, 0);
    pos = pos.wrapping_add(1);
    let d3 = d0.insert((pos >> 32) as u32, 1).insert(pos as u32, 0);
    pos = pos.wrapping_add(1);
    let d4 = d0.insert((pos >> 32) as u32, 1).insert(pos as u32, 0);

    let (a, b, c, d) = (
        x.a.to_lanes(),
        x.b.to_lanes(),
        x.c.to_lanes(),
        x.d.to_lanes(),
    );
    let sb = m.unpack(state.b);
    let sc = m.unpack(state.c);
    let sd = [m.unpack(state.d), d1, d2, d3];
    state.d = d4.into();
    let mut words = out.chunks_exact_mut(16);
    for ((((&a, &b), &c), &d), &sd) in a.iter().zip(&b).zip(&c).zip(&d).zip(&sd) {
        (a + k).write_le(words.next().unwrap());
        (b + sb).write_le(words.next().unwrap());
        (c + sc).write_le(words.next().unwrap());
        (d + sd).write_le(words.next().unwrap());
    }
}

dispatch!(m, Mach, {
    fn refill_wide(state: &mut ChaCha, drounds: u32, out: &mut [u8; BUFSZ]) {
        refill_wide_impl(m, state, drounds, out);
    }
});

// Single-block, rounds-only; shared by try_apply_keystream for tails shorter than BUFSZ
// and XChaCha's setup step.
dispatch!(m, Mach, {
    fn refill_narrow_rounds(state: &mut ChaCha, drounds: u32) -> State<vec128_storage> {
        let k: Mach::u32x4 = m.vec([0x6170_7865, 0x3320_646e, 0x7962_2d32, 0x6b20_6574]);
        let mut x = State {
            a: k,
            b: m.unpack(state.b),
            c: m.unpack(state.c),
            d: m.unpack(state.d),
        };
        for _ in 0..drounds {
            x = round(x);
            x = undiagonalize(round(diagonalize(x)));
        }
        State {
            a: x.a.into(),
            b: x.b.into(),
            c: x.c.into(),
            d: x.d.into(),
        }
    }
});

dispatch_light128!(m, Mach, {
    fn set_stream_param(state: &mut ChaCha, param: u32, value: u64) {
        let d: Mach::u32x4 = m.unpack(state.d);
        state.d = d
            .insert((value >> 32) as u32, (param << 1) | 1)
            .insert(value as u32, param << 1)
            .into();
    }
});

dispatch_light128!(m, Mach, {
    fn get_stream_param(state: &ChaCha, param: u32) -> u64 {
        let d: Mach::u32x4 = m.unpack(state.d);
        ((d.extract((param << 1) | 1) as u64) << 32) | d.extract(param << 1) as u64
    }
});

fn read_u32le(xs: &[u8]) -> u32 {
    assert_eq!(xs.len(), 4);
    u32::from(xs[0]) | (u32::from(xs[1]) << 8) | (u32::from(xs[2]) << 16) | (u32::from(xs[3]) << 24)
}

dispatch_light128!(m, Mach, {
    fn init_chacha(key: &[u8; 32], nonce: &[u8]) -> ChaCha {
        let ctr_nonce = [
            0,
            if nonce.len() == 12 {
                read_u32le(&nonce[0..4])
            } else {
                0
            },
            read_u32le(&nonce[nonce.len() - 8..nonce.len() - 4]),
            read_u32le(&nonce[nonce.len() - 4..]),
        ];
        let key0: Mach::u32x4 = m.read_le(&key[..16]);
        let key1: Mach::u32x4 = m.read_le(&key[16..]);
        ChaCha {
            b: key0.into(),
            c: key1.into(),
            d: ctr_nonce.into(),
        }
    }
});

dispatch_light128!(m, Mach, {
    fn init_chacha_x(key: &[u8; 32], nonce: &[u8; 24], rounds: u32) -> ChaCha {
        let key0: Mach::u32x4 = m.read_le(&key[..16]);
        let key1: Mach::u32x4 = m.read_le(&key[16..]);
        let nonce0: Mach::u32x4 = m.read_le(&nonce[..16]);
        let mut state = ChaCha {
            b: key0.into(),
            c: key1.into(),
            d: nonce0.into(),
        };
        let x = refill_narrow_rounds(&mut state, rounds);
        let ctr_nonce1 = [0, 0, read_u32le(&nonce[16..20]), read_u32le(&nonce[20..24])];
        state.b = x.a;
        state.c = x.d;
        state.d = ctr_nonce1.into();
        state
    }
});

#[cfg(feature = "serde1")]
impl Serialize for ChaCha {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        // 3 is the number of fields in the struct.
        let mut state = serializer.serialize_struct("ChaCha", 3)?;
        let b: &[u32; 4] = (&self.b).into();
        let c: &[u32; 4] = (&self.c).into();
        let d: &[u32; 4] = (&self.d).into();
        let b: &[u128; 1] = unsafe { std::mem::transmute(b) };
        let c: &[u128; 1] = unsafe { std::mem::transmute(c) };
        let d: &[u128; 1] = unsafe { std::mem::transmute(d) };
        state.serialize_field("b", &b)?;
        state.serialize_field("c", &c)?;
        state.serialize_field("d", &d)?;
        state.end()
    }
}

#[cfg(feature = "serde1")]
impl<'de> Deserialize<'de> for ChaCha {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            B,
            C,
            D,
        }

        struct ChaChaVisitor;

        impl<'de> Visitor<'de> for ChaChaVisitor {
            type Value = ChaCha;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct ChaCha")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<ChaCha, V::Error>
            where V: SeqAccess<'de> {
                let b: [u128; 1] = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let c: [u128; 1] = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let d: [u128; 1] = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;

                let b = unsafe { std::mem::transmute(b) };
                let c = unsafe { std::mem::transmute(c) };
                let d = unsafe { std::mem::transmute(d) };

                Ok(ChaCha { b, c, d })
            }

            fn visit_map<V>(self, mut map: V) -> Result<ChaCha, V::Error>
            where V: MapAccess<'de> {
                let mut b: Option<vec128_storage> = None;
                let mut c: Option<vec128_storage> = None;
                let mut d: Option<vec128_storage> = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::B => {
                            if b.is_some() {
                                return Err(de::Error::duplicate_field("b"));
                            }
                            let raw_b: [u128; 1] = map.next_value()?;
                            let raw_b: [u32; 4] = unsafe { std::mem::transmute(raw_b) };
                            b = Some(raw_b.into());
                        }
                        Field::C => {
                            if c.is_some() {
                                return Err(de::Error::duplicate_field("c"));
                            }
                            let raw_c: [u128; 1] = map.next_value()?;
                            let raw_c: [u32; 4] = unsafe { std::mem::transmute(raw_c) };
                            c = Some(raw_c.into());
                        }
                        Field::D => {
                            if d.is_some() {
                                return Err(de::Error::duplicate_field("d"));
                            }
                            let raw_d: [u128; 1] = map.next_value()?;
                            let raw_d: [u32; 4] = unsafe { std::mem::transmute(raw_d) };
                            d = Some(raw_d.into());
                        }
                    }
                }
                let b = b.ok_or_else(|| de::Error::missing_field("b"))?;
                let c = c.ok_or_else(|| de::Error::missing_field("c"))?;
                let d = d.ok_or_else(|| de::Error::missing_field("d"))?;
                Ok(ChaCha { b, c, d })
            }
        }

        const FIELDS: &'static [&'static str] = &["b", "c", "d"];
        deserializer.deserialize_struct("ChaCha", FIELDS, ChaChaVisitor)
    }
}
