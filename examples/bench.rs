//! Runs benchmarks.

use std::time::Instant;
use dandelion::Rng as Dandelion;
use rand_pcg::Lcg128CmDxsm64;
use rand_xoshiro::Xoroshiro128PlusPlus;
use rand::Rng as _;
use rand::RngCore as _;
use rand::SeedableRng as _;

trait Rng {
  fn from_u64(n: u64) -> Self;
  fn u64(&mut self) -> u64;
  fn between_u64(&mut self, lo: u64, hi: u64) -> u64;
  fn f64(&mut self) -> f64;
  fn bytes(&mut self, buf: &mut [u8]);

  #[inline(never)]
  fn u64_noinline(&mut self) -> u64 {
    self.u64()
  }

  #[inline(never)]
  fn between_u64_noinline(&mut self, lo: u64, hi: u64) -> u64 {
    self.between_u64(lo, hi)
  }

  #[inline(never)]
  fn f64_noinline(&mut self) -> f64 {
    self.f64()
  }

  #[inline(never)]
  fn bytes_noinline(&mut self, buf: &mut [u8]) {
    self.bytes(buf)
  }
}

impl Rng for Dandelion {
  fn from_u64(n: u64) -> Self { Self::from_u64(n) }
  fn u64(&mut self) -> u64 { self.u64() }
  fn between_u64(&mut self, lo: u64, hi: u64) -> u64 { self.between_u64(lo, hi) }
  fn f64(&mut self) -> f64 { self.f64() }
  fn bytes(&mut self, buf: &mut [u8]) { self.bytes(buf) }
}

impl Rng for Lcg128CmDxsm64 {
  fn from_u64(n: u64) -> Self { Self::seed_from_u64(n) }
  fn u64(&mut self) -> u64 { self.random() }
  fn between_u64(&mut self, lo: u64, hi: u64) -> u64 { self.random_range(lo ..= hi) }
  fn f64(&mut self) -> f64 { self.random() }
  fn bytes(&mut self, buf: &mut [u8]) { self.fill_bytes(buf) }
}

impl Rng for Xoroshiro128PlusPlus {
  fn from_u64(n: u64) -> Self { Self::seed_from_u64(n) }
  fn u64(&mut self) -> u64 { self.random() }
  fn between_u64(&mut self, lo: u64, hi: u64) -> u64 { self.random_range(lo ..= hi) }
  fn f64(&mut self) -> f64 { self.random() }
  fn bytes(&mut self, buf: &mut [u8]) { self.fill_bytes(buf) }
}

const OUTER: usize = 1024 * 16;
const INNER: usize = 1024;
const COUNT: usize = OUTER * INNER;

#[inline(never)]
fn warmup() {
  let mut s = 1u64;
  for i in 0 .. 1_000_000_000 { s = s.wrapping_mul(i); }
  let _: u64 = std::hint::black_box(s);
}

#[inline(never)]
fn timeit<F: FnMut()>(f: F) -> f64 {
  let mut f = f;
  let start = Instant::now();
  for _ in 0 .. OUTER { f() }
  let stop = Instant::now();
  stop.saturating_duration_since(start).as_nanos() as f64
}

#[inline(never)]
fn fill_0<T: Rng>(rng: &mut T, buf: &mut [u64; INNER]) {
  for elt in buf.iter_mut() {
    *elt = rng.u64();
  }
}

#[inline(never)]
fn fill_1<T: Rng>(rng: &mut T, buf: &mut [u64; INNER]) {
  for elt in buf.iter_mut() {
    *elt = rng.u64_noinline();
  }
}

#[inline(never)]
fn fill_2<T: Rng>(rng: &mut T, buf: &mut [u64; INNER], lo: u64, hi: u64) {
  for elt in buf.iter_mut() {
    *elt = rng.between_u64(lo, hi);
  }
}

#[inline(never)]
fn fill_3<T: Rng>(rng: &mut T, buf: &mut [u64; INNER], lo: u64, hi: u64) {
  for elt in buf.iter_mut() {
    *elt = rng.between_u64_noinline(lo, hi);
  }
}

#[inline(never)]
fn fill_4<T: Rng>(rng: &mut T, buf: &mut [f64; INNER]) {
  for elt in buf.iter_mut() {
    *elt = rng.f64();
  }
}

#[inline(never)]
fn fill_5<T: Rng>(rng: &mut T, buf: &mut [f64; INNER]) {
  for elt in buf.iter_mut() {
    *elt = rng.f64_noinline();
  }
}

#[inline(never)]
fn fill_6<T: Rng>(rng: &mut T, buf: &mut [u8; INNER * 8]) {
  rng.bytes(buf)
}

#[inline(never)]
fn fill_7<T: Rng>(rng: &mut T, buf: &mut [Box<[u8]>; INNER]) {
  for elt in buf.iter_mut() {
    rng.bytes(elt);
  }
}

#[inline(never)]
fn fill_8<T: Rng>(rng: &mut T, buf: &mut [Box<[u8]>; INNER]) {
  for elt in buf.iter_mut() {
    rng.bytes_noinline(elt);
  }
}

#[inline(never)]
fn go<T: Rng>(
    name: &str,
    buf_0: &mut [u64; INNER],
    buf_1: &mut [f64; INNER],
    buf_2: &mut [u8; INNER * 8],
    buf_3: &mut [Box<[u8]>; INNER],
  )
{
  let lo = 0;
  let hi = 0x1100_0000_0000_0000;

  let mut rng = T::from_u64(0);

  let e0 = timeit(|| fill_0(&mut rng, buf_0));
  let e1 = timeit(|| fill_1(&mut rng, buf_0));
  let e2 = timeit(|| fill_2(&mut rng, buf_0, lo, hi));
  let e3 = timeit(|| fill_3(&mut rng, buf_0, lo, hi));
  let e4 = timeit(|| fill_4(&mut rng, buf_1));
  let e5 = timeit(|| fill_5(&mut rng, buf_1));
  let e6 = timeit(|| fill_6(&mut rng, buf_2));
  let e7 = timeit(|| fill_7(&mut rng, buf_3));
  let e8 = timeit(|| fill_8(&mut rng, buf_3));

  println!("{}", name);
  println!("{:6.3} ns/word - u64", e0 / COUNT as f64);
  println!("{:6.3} ns/word - u64 noinline", e1 / COUNT as f64);
  println!("{:6.3} ns/word - between_u64", e2 / COUNT as f64);
  println!("{:6.3} ns/word - between_u64 noinline", e3 / COUNT as f64);
  println!("{:6.3} ns/word - f64", e4 / COUNT as f64);
  println!("{:6.3} ns/word - f64 noinline", e5 / COUNT as f64);
  println!("{:6.3} ns/word - bytes large fill", e6 / COUNT as f64);
  println!("{:6.3} ns/word - bytes small fill", e7 / COUNT as f64);
  println!("{:6.3} ns/word - bytes small fill noinline", e8 / COUNT as f64);
  println!("");
}

fn main() {
  let buf_0 = &mut [0_u64; INNER];

  let buf_1 = &mut [0_f64; INNER];

  let buf_2 = &mut [0_u8; INNER * 8];

  let buf_3: &mut [Box<[u8]>; INNER] = &mut {
    let mut rng: u64 = 0x93c4_67e3_7db0_c7a5;
    core::array::from_fn(|_| {
      rng = rng ^ rng << 7;
      rng = rng ^ rng >> 9;
      vec![0_u8; (rng % 9) as usize].into_boxed_slice()
    })
  };

  warmup();

  go::<Dandelion>("dandelion", buf_0, buf_1, buf_2, buf_3);

  go::<Lcg128CmDxsm64>("pcgdxsm128", buf_0, buf_1, buf_2, buf_3);

  go::<Xoroshiro128PlusPlus>("xoroshiro128++", buf_0, buf_1, buf_2, buf_3);
}
