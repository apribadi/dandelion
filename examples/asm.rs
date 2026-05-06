#![allow(missing_docs)]

use dandelion::Rng;
use rand::RngExt as _;
use std::num::NonZeroU128;
use std::num::NonZeroU32;
use std::num::NonZeroU64;

#[inline(never)]
pub fn new(s: NonZeroU128) -> Rng {
  Rng::new(s)
}

#[inline(never)]
pub fn bernoulli(g: &mut Rng, p: f64) -> bool {
  g.bernoulli(p)
}

#[inline(never)]
pub fn bool(g: &mut Rng) -> bool {
  g.bool()
}

#[inline(never)]
pub fn u32(g: &mut Rng) -> u32 {
  g.u32()
}

#[inline(never)]
pub fn u64(g: &mut Rng) -> u64 {
  g.u64()
}

#[inline(never)]
pub fn u128(g: &mut Rng) -> u128 {
  g.u128()
}

#[inline(never)]
pub fn bounded_u32(g: &mut Rng, n: u32) -> u32 {
  g.bounded_u32(n)
}

#[inline(never)]
pub fn bounded_u64(g: &mut Rng, n: u64) -> u64 {
  g.bounded_u64(n)
}

#[inline(never)]
pub fn bounded_u64_zero(g: &mut Rng) -> u64 {
  g.bounded_u64(0)
}

#[inline(never)]
pub fn bounded_u64_255(g: &mut Rng) -> u64 {
  g.bounded_u64(255)
}

#[inline(never)]
pub fn bounded_usize(g: &mut Rng, n: usize) -> usize {
  g.bounded_usize(n)
}

#[inline(never)]
pub fn range_isize(g: &mut Rng, lo: isize, hi: isize) -> isize {
  g.range_isize(lo, hi)
}

#[inline(never)]
pub fn range_u32(g: &mut Rng, lo: u32, hi: u32) -> u32 {
  g.range_u32(lo, hi)
}

#[inline(never)]
pub fn range_u64(g: &mut Rng, lo: u64, hi: u64) -> u64 {
  g.range_u64(lo, hi)
}

#[inline(never)]
pub fn range_usize(g: &mut Rng, lo: usize, hi: usize) -> usize {
  g.range_usize(lo, hi)
}

#[inline(never)]
pub fn non_zero_u32(g: &mut Rng) -> NonZeroU32 {
  g.non_zero_u32()
}

#[inline(never)]
pub fn non_zero_u64(g: &mut Rng) -> NonZeroU64 {
  g.non_zero_u64()
}

#[inline(never)]
pub fn non_zero_u128(g: &mut Rng) -> NonZeroU128 {
  g.non_zero_u128()
}

#[inline(never)]
pub fn f32(g: &mut Rng) -> f32 {
  g.f32()
}

#[inline(never)]
pub fn f64(g: &mut Rng) -> f64 {
  g.f64()
}

#[inline(never)]
pub fn biunit_f32(g: &mut Rng) -> f32 {
  g.biunit_f32()
}

#[inline(never)]
pub fn biunit_f64(g: &mut Rng) -> f64 {
  g.biunit_f64()
}

#[inline(never)]
pub fn byte_array_32(g: &mut Rng) -> [u8; 32] {
  g.byte_array()
}

#[inline(never)]
pub fn shuffle(g: &mut Rng, a: &mut [u32]) {
  g.shuffle(a)
}

#[cfg(feature = "rand_core")]
#[inline(never)]
pub fn fork(g: &mut Rng) -> Rng {
  <Rng as rand::SeedableRng>::fork(g)
}

#[cfg(feature = "rand_core")]
#[inline(never)]
pub fn try_fork(g: &mut Rng) -> Result<Rng, rand_core::Infallible> {
  <Rng as rand::SeedableRng>::try_fork(g)
}

#[inline(never)]
pub fn pcg_u64(g: &mut rand_pcg::Lcg128CmDxsm64) -> u64 {
  g.random()
}

#[inline(never)]
pub fn pcg_non_zero_u64(g: &mut rand_pcg::Lcg128CmDxsm64) -> NonZeroU64 {
  g.random()
}

#[inline(never)]
pub fn xoroshiro_u64(g: &mut rand_xoshiro::Xoroshiro128PlusPlus) -> u64 {
  g.random()
}

#[inline(never)]
pub fn xoroshiro_non_zero_u32(g: &mut rand_xoshiro::Xoroshiro128PlusPlus) -> NonZeroU32 {
  g.random()
}

#[inline(never)]
pub fn xoroshiro_non_zero_u64(g: &mut rand_xoshiro::Xoroshiro128PlusPlus) -> NonZeroU64 {
  g.random()
}

#[inline(never)]
pub fn xoroshiro_bounded_u64(g: &mut rand_xoshiro::Xoroshiro128PlusPlus, n: u64) -> u64 {
  g.random_range(0 ..= n)
}

#[inline(never)]
pub fn xoroshiro_bernoulli(g: &mut rand_xoshiro::Xoroshiro128PlusPlus, p: f64) -> bool {
  g.random_bool(p)
}
