use rand::Rng as _;
use rand::RngExt as _;
use rand::SeedableRng as _;
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand_pcg::Lcg128CmDxsm64;
use rand_xoshiro::Xoroshiro128PlusPlus;

pub(crate) trait RngForBench: Clone {
  fn from_u64(n: u64) -> Self;
  fn u64(&mut self) -> u64;
  fn range_u64(&mut self, lo: u64, hi: u64) -> u64;
  fn f64(&mut self) -> f64;
  fn bernoulli(&mut self, p: f64) -> bool;
  fn fill(&mut self, buf: &mut [u8]);
  fn shuffle<T>(&mut self, slice: &mut [T]);

  #[inline(never)]
  fn u64_noinline(&mut self) -> u64 { self.u64() }

  #[inline(never)]
  fn range_u64_noinline(&mut self, lo: u64, hi: u64) -> u64 { self.range_u64(lo, hi) }

  #[inline(never)]
  fn f64_noinline(&mut self) -> f64 { self.f64() }

  #[inline(never)]
  fn fill_noinline(&mut self, buf: &mut [u8]) { self.fill(buf) }
}

impl RngForBench for dandelion::Rng {
  fn from_u64(n: u64) -> Self { Self::from_u64(n) }
  fn u64(&mut self) -> u64 { self.uniform() }
  fn range_u64(&mut self, lo: u64, hi: u64) -> u64 { self.between(lo, hi) }
  fn f64(&mut self) -> f64 { self.float() }
  fn bernoulli(&mut self, p: f64) -> bool { self.bernoulli(p) }
  fn fill(&mut self, buf: &mut [u8]) { self.fill(buf) }
  fn shuffle<T>(&mut self, slice: &mut [T]) { self.shuffle(slice) }
}

impl RngForBench for Lcg128CmDxsm64 {
  fn from_u64(n: u64) -> Self { Self::seed_from_u64(n) }
  fn u64(&mut self) -> u64 { self.random() }
  fn range_u64(&mut self, lo: u64, hi: u64) -> u64 { self.random_range(lo ..= hi) }
  fn f64(&mut self) -> f64 { self.random() }
  fn bernoulli(&mut self, p: f64) -> bool { self.random_bool(p) }
  fn fill(&mut self, buf: &mut [u8]) { self.fill_bytes(buf) }
  fn shuffle<T>(&mut self, slice: &mut [T]) { <[T] as SliceRandom>::shuffle(slice, self) }
}

impl RngForBench for Xoroshiro128PlusPlus {
  fn from_u64(n: u64) -> Self { Self::seed_from_u64(n) }
  fn u64(&mut self) -> u64 { self.random() }
  fn range_u64(&mut self, lo: u64, hi: u64) -> u64 { self.random_range(lo ..= hi) }
  fn f64(&mut self) -> f64 { self.random() }
  fn bernoulli(&mut self, p: f64) -> bool { self.random_bool(p) }
  fn fill(&mut self, buf: &mut [u8]) { self.fill_bytes(buf) }
  fn shuffle<T>(&mut self, slice: &mut [T]) { <[T] as SliceRandom>::shuffle(slice, self) }
}

impl RngForBench for SmallRng {
  fn from_u64(n: u64) -> Self { Self::seed_from_u64(n) }
  fn u64(&mut self) -> u64 { self.random() }
  fn range_u64(&mut self, lo: u64, hi: u64) -> u64 { self.random_range(lo ..= hi) }
  fn f64(&mut self) -> f64 { self.random() }
  fn bernoulli(&mut self, p: f64) -> bool { self.random_bool(p) }
  fn fill(&mut self, buf: &mut [u8]) { self.fill_bytes(buf) }
  fn shuffle<T>(&mut self, slice: &mut [T]) { <[T] as SliceRandom>::shuffle(slice, self) }
}
