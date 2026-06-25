use rand::Rng as _;
use rand::RngExt as _;
use rand::SeedableRng as _;
use rand::seq::SliceRandom as _;

pub(crate) trait Rng {
  fn from_u64(n: u64) -> Self;
  fn uniform(&mut self) -> u64;
  fn between(&mut self, lo: u64, hi: u64) -> u64;
  fn f64(&mut self) -> f64;
  fn bernoulli(&mut self, p: f64) -> bool;
  fn fill_b(&mut self, buf: &mut [u8]);
  fn fill_h(&mut self, buf: &mut [u16]);
  fn shuffle<T>(&mut self, slice: &mut [T]);
}

impl Rng for dandelion::Rng {
  fn from_u64(n: u64) -> Self { Self::from_u64(n) }
  fn uniform(&mut self) -> u64 { self.uniform() }
  fn between(&mut self, lo: u64, hi: u64) -> u64 { self.between(lo, hi) }
  fn f64(&mut self) -> f64 { self.float() }
  fn bernoulli(&mut self, p: f64) -> bool { self.bernoulli(p) }
  fn fill_b(&mut self, buf: &mut [u8]) { self.fill(buf); }
  fn fill_h(&mut self, buf: &mut [u16]) { self.fill(buf); }
  fn shuffle<T>(&mut self, slice: &mut [T]) { self.shuffle(slice); }
}

impl Rng for rand::rngs::SmallRng {
  fn from_u64(n: u64) -> Self { Self::seed_from_u64(n) }
  fn uniform(&mut self) -> u64 { self.next_u64() }
  fn between(&mut self, lo: u64, hi: u64) -> u64 { self.random_range(lo ..= hi) }
  fn f64(&mut self) -> f64 { self.random() }
  fn bernoulli(&mut self, p: f64) -> bool { self.random_bool(p) }
  fn fill_b(&mut self, buf: &mut [u8]) { self.fill_bytes(buf); }
  fn fill_h(&mut self, buf: &mut [u16]) { self.fill(buf); }
  fn shuffle<T>(&mut self, slice: &mut [T]) { slice.shuffle(self); }
}

impl Rng for rand_pcg::Lcg128CmDxsm64 {
  fn from_u64(n: u64) -> Self { Self::seed_from_u64(n) }
  fn uniform(&mut self) -> u64 { self.next_u64() }
  fn between(&mut self, lo: u64, hi: u64) -> u64 { self.random_range(lo ..= hi) }
  fn f64(&mut self) -> f64 { self.random() }
  fn bernoulli(&mut self, p: f64) -> bool { self.random_bool(p) }
  fn fill_b(&mut self, buf: &mut [u8]) { self.fill_bytes(buf); }
  fn fill_h(&mut self, buf: &mut [u16]) { self.fill(buf); }
  fn shuffle<T>(&mut self, slice: &mut [T]) { slice.shuffle(self); }
}

impl Rng for rand_xoshiro::Xoroshiro128PlusPlus {
  fn from_u64(n: u64) -> Self { Self::seed_from_u64(n) }
  fn uniform(&mut self) -> u64 { self.next_u64() }
  fn between(&mut self, lo: u64, hi: u64) -> u64 { self.random_range(lo ..= hi) }
  fn f64(&mut self) -> f64 { self.random() }
  fn bernoulli(&mut self, p: f64) -> bool { self.random_bool(p) }
  fn fill_b(&mut self, buf: &mut [u8]) { self.fill_bytes(buf); }
  fn fill_h(&mut self, buf: &mut [u16]) { self.fill(buf); }
  fn shuffle<T>(&mut self, slice: &mut [T]) { slice.shuffle(self); }
}
