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
  g.uniform()
}

#[inline(never)]
pub fn u32(g: &mut Rng) -> u32 {
  g.uniform()
}

#[inline(never)]
pub fn u64(g: &mut Rng) -> u64 {
  g.uniform()
}

#[inline(never)]
pub fn u128(g: &mut Rng) -> u128 {
  g.uniform()
}

#[inline(never)]
pub fn bounded_u8(g: &mut Rng, n: u8) -> u8 {
  g.bounded(n)
}

#[inline(never)]
pub fn bounded_u16(g: &mut Rng, n: u16) -> u16 {
  g.bounded(n)
}

#[inline(never)]
pub fn bounded_u32(g: &mut Rng, n: u32) -> u32 {
  g.bounded(n)
}

#[inline(never)]
pub fn bounded_u64(g: &mut Rng, n: u64) -> u64 {
  g.bounded(n)
}

#[inline(never)]
pub fn bounded_u64_0(g: &mut Rng) -> u64 {
  g.bounded(0)
}

#[inline(never)]
pub fn bounded_u64_255(g: &mut Rng) -> u64 {
  g.bounded(255)
}

#[inline(never)]
pub fn bounded_usize(g: &mut Rng, n: usize) -> usize {
  g.bounded(n)
}

#[inline(never)]
pub fn between_u32(g: &mut Rng, lo: u32, hi: u32) -> u32 {
  g.between(lo, hi)
}

#[inline(never)]
pub fn between_u64(g: &mut Rng, lo: u64, hi: u64) -> u64 {
  g.between(lo, hi)
}

#[inline(never)]
pub fn between_isize(g: &mut Rng, lo: isize, hi: isize) -> isize {
  g.between(lo, hi)
}

#[inline(never)]
pub fn between_usize(g: &mut Rng, lo: usize, hi: usize) -> usize {
  g.between(lo, hi)
}

#[inline(never)]
pub fn non_zero_u32(g: &mut Rng) -> NonZeroU32 {
  g.uniform()
}

#[inline(never)]
pub fn non_zero_u64(g: &mut Rng) -> NonZeroU64 {
  g.uniform()
}

#[inline(never)]
pub fn non_zero_u128(g: &mut Rng) -> NonZeroU128 {
  g.uniform()
}

#[inline(never)]
pub fn f32(g: &mut Rng) -> f32 {
  g.float()
}

#[inline(never)]
pub fn f32x4(g: &mut Rng) -> [f32; 4] {
  std::array::from_fn(|_| g.float())
}

#[inline(never)]
pub fn f64(g: &mut Rng) -> f64 {
  g.float()
}

#[inline(never)]
pub fn float_biunit_f32(g: &mut Rng) -> f32 {
  g.float_biunit()
}

#[inline(never)]
pub fn float_biunit_f64(g: &mut Rng) -> f64 {
  g.float_biunit()
}

#[inline(never)]
pub fn uniform_bools(g: &mut Rng) -> [bool; 6] {
  g.uniform()
}

#[inline(never)]
pub fn uniform_4u16(g: &mut Rng) -> [u16; 4] {
  g.uniform()
}

#[inline(never)]
pub fn uniform_6i16(g: &mut Rng) -> [i16; 6] {
  g.uniform()
}

#[inline(never)]
pub fn uniform_6u16(g: &mut Rng) -> [u16; 6] {
  g.uniform()
}

#[inline(never)]
pub fn uniform_6i32(g: &mut Rng) -> [i32; 6] {
  g.uniform()
}

#[inline(never)]
pub fn uniform_6u32(g: &mut Rng) -> [u32; 6] {
  g.uniform()
}

#[inline(never)]
pub fn fill_u8(g: &mut Rng, buf: &mut [u8]) {
  g.fill(buf)
}

#[inline(never)]
pub fn fill_i16(g: &mut Rng, buf: &mut [i16]) {
  g.fill(buf)
}

#[inline(never)]
pub fn fill_u16(g: &mut Rng, buf: &mut [u16]) {
  g.fill(buf)
}

#[inline(never)]
pub fn fill_u32(g: &mut Rng, buf: &mut [u32]) {
  g.fill(buf)
}

#[inline(never)]
pub fn fill_u64(g: &mut Rng, buf: &mut [u64]) {
  g.fill(buf)
}

#[inline(never)]
pub fn fill_u128(g: &mut Rng, buf: &mut [u128]) {
  g.fill(buf)
}

#[inline(never)]
pub fn shuffle(g: &mut Rng, a: &mut [u32]) {
  g.shuffle(a)
}

#[cfg(feature = "thread_local")]
#[inline(never)]
pub fn thread_local_u64() -> u64 {
  dandelion::thread_local::uniform()
}

#[cfg(feature = "thread_local")]
#[inline(never)]
pub fn thread_local_u64x3() -> u64 {
  let x = dandelion::thread_local::uniform::<u64>();
  let y = dandelion::thread_local::uniform::<u64>();
  let z = dandelion::thread_local::uniform::<u64>();
  x ^ y ^ z
}

#[cfg(feature = "thread_local")]
#[inline(never)]
pub fn thread_local_u64_loop() -> u64 {
  let mut x = 0;
  for _ in 0 .. 100 {
    x ^= dandelion::thread_local::uniform::<u64>();
  }
  x
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
pub fn xoroshiro_bounded_u64_loop(g: &mut rand_xoshiro::Xoroshiro128PlusPlus, n: u64) -> u64 {
  let mut x = 0u64;
  for _ in 0 .. 100 {
    x ^= g.random_range(0 ..= n);
  }
  x
}

#[inline(never)]
pub fn xoroshiro_bounded_u64_255(g: &mut rand_xoshiro::Xoroshiro128PlusPlus) -> u64 {
  g.random_range(0 ..= 255)
}

#[inline(never)]
pub fn xoroshiro_bernoulli(g: &mut rand_xoshiro::Xoroshiro128PlusPlus, p: f64) -> bool {
  g.random_bool(p)
}
