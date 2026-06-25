use std::hint::black_box;
use rand::Rng as _;
use rand::RngExt as _;
use rand::SeedableRng as _;
use rand::seq::SliceRandom as _;

pub(crate) trait Rng {
  fn new() -> Self;
  fn uniform(&mut self) -> u64;
  fn between(&mut self, lo: u64, hi: u64) -> u64;
  fn float(&mut self) -> f64;
  fn bool(&mut self, p: f64) -> bool;
  fn fill_b(&mut self, buf: &mut [u8]);
  fn fill_h(&mut self, buf: &mut [u16]);
  fn shuffle<T>(&mut self, slice: &mut [T]);
}

impl Rng for dandelion::Rng {
  fn new() -> Self { Self::from_u64(black_box(0)) }
  fn uniform(&mut self) -> u64 { self.uniform() }
  fn between(&mut self, lo: u64, hi: u64) -> u64 { self.between(lo, hi) }
  fn float(&mut self) -> f64 { self.float() }
  fn bool(&mut self, p: f64) -> bool { self.bool(p) }
  fn fill_b(&mut self, buf: &mut [u8]) { self.fill(buf); }
  fn fill_h(&mut self, buf: &mut [u16]) { self.fill(buf); }
  fn shuffle<T>(&mut self, slice: &mut [T]) { self.shuffle(slice); }
}

impl Rng for rand::rngs::SmallRng {
  fn new() -> Self { Self::seed_from_u64(black_box(0)) }
  fn uniform(&mut self) -> u64 { self.next_u64() }
  fn between(&mut self, lo: u64, hi: u64) -> u64 { self.random_range(lo ..= hi) }
  fn float(&mut self) -> f64 { self.random() }
  fn bool(&mut self, p: f64) -> bool { self.random_bool(p) }
  fn fill_b(&mut self, buf: &mut [u8]) { self.fill_bytes(buf); }
  fn fill_h(&mut self, buf: &mut [u16]) { self.fill(buf); }
  fn shuffle<T>(&mut self, slice: &mut [T]) { slice.shuffle(self); }
}

impl Rng for rand_pcg::Lcg128CmDxsm64 {
  fn new() -> Self { Self::seed_from_u64(black_box(0)) }
  fn uniform(&mut self) -> u64 { self.next_u64() }
  fn between(&mut self, lo: u64, hi: u64) -> u64 { self.random_range(lo ..= hi) }
  fn float(&mut self) -> f64 { self.random() }
  fn bool(&mut self, p: f64) -> bool { self.random_bool(p) }
  fn fill_b(&mut self, buf: &mut [u8]) { self.fill_bytes(buf); }
  fn fill_h(&mut self, buf: &mut [u16]) { self.fill(buf); }
  fn shuffle<T>(&mut self, slice: &mut [T]) { slice.shuffle(self); }
}

impl Rng for rand_xoshiro::Xoroshiro128PlusPlus {
  fn new() -> Self { Self::seed_from_u64(black_box(0)) }
  fn uniform(&mut self) -> u64 { self.next_u64() }
  fn between(&mut self, lo: u64, hi: u64) -> u64 { self.random_range(lo ..= hi) }
  fn float(&mut self) -> f64 { self.random() }
  fn bool(&mut self, p: f64) -> bool { self.random_bool(p) }
  fn fill_b(&mut self, buf: &mut [u8]) { self.fill_bytes(buf); }
  fn fill_h(&mut self, buf: &mut [u16]) { self.fill(buf); }
  fn shuffle<T>(&mut self, slice: &mut [T]) { slice.shuffle(self); }
}

#[cfg(feature = "thread_local")]
pub(crate) struct DandelionThreadLocal;

#[cfg(feature = "thread_local")]
impl Rng for DandelionThreadLocal {
  fn new() -> Self { DandelionThreadLocal }
  fn uniform(&mut self) -> u64 { dandelion::thread_local::uniform() }
  fn between(&mut self, lo: u64, hi: u64) -> u64 { dandelion::thread_local::between(lo, hi) }
  fn float(&mut self) -> f64 { dandelion::thread_local::float() }
  fn bool(&mut self, p: f64) -> bool { dandelion::thread_local::bool(p) }
  fn fill_b(&mut self, buf: &mut [u8]) { dandelion::thread_local::fill(buf); }
  fn fill_h(&mut self, buf: &mut [u16]) { dandelion::thread_local::fill(buf); }
  fn shuffle<T>(&mut self, slice: &mut [T]) { dandelion::thread_local::shuffle(slice); }
}

#[cfg(feature = "thread_local")]
pub(crate) struct RandThreadLocal;

#[cfg(feature = "thread_local")]
impl Rng for RandThreadLocal {
  fn new() -> Self { RandThreadLocal }
  fn uniform(&mut self) -> u64 { rand::random() }
  fn between(&mut self, lo: u64, hi: u64) -> u64 { rand::random_range(lo ..= hi) }
  fn float(&mut self) -> f64 { rand::random() }
  fn bool(&mut self, p: f64) -> bool { rand::random_bool(p) }
  fn fill_b(&mut self, buf: &mut [u8]) { rand::fill(buf); }
  fn fill_h(&mut self, buf: &mut [u16]) { rand::fill(buf); }
  fn shuffle<T>(&mut self, slice: &mut [T]) { slice.shuffle(&mut rand::rng()); }
}
