//! benchmarks

mod rngs;

use dandelion::Rng as Dandelion;
use rand::rngs::SmallRng;
use rand_pcg::Lcg128CmDxsm64 as Pcg;
use rand_xoshiro::Xoroshiro128PlusPlus as Xoroshiro;
use rngs::Rng;
use std::hint::black_box;
use std::time::Instant;

const M: usize = 5_000;
const N: usize = 2_000;
const A: u64 = 1;
const B: u64 = 0x1101_0000_0000_0001;

#[inline]
fn timeit<T, F: FnOnce() -> T>(f: F) -> f64 {
  let a = Instant::now();
  let x = black_box(f());
  let b = Instant::now();
  let _ = x;
  b.saturating_duration_since(a).as_nanos() as f64
}

#[inline(never)]
fn inner_uniform_u64<T: Rng>(g: &mut T, r: &mut [u64; N], m: usize) {
  for _ in 0 .. m {
    for e in r.iter_mut() {
      *e = g.uniform_u64();
    }
  }
}

#[inline(never)]
fn bench_uniform_u64<T: Rng>() -> f64 {
  let mut g = T::from_u64(black_box(0));
  let mut r = [0u64; N];
  let d = timeit(|| inner_uniform_u64(&mut g, &mut r, black_box(M)));
  d / (M * N) as f64
}

#[inline(never)]
fn inner_between_u64<T: Rng>(g: &mut T, r: &mut [u64; N], m: usize, a: u64, b: u64) {
  for _ in 0 .. m {
    for e in r.iter_mut() {
      *e = g.between_u64(a, b);
    }
  }
}

#[inline(never)]
fn bench_between_u64<T: Rng>() -> f64 {
  let mut g = T::from_u64(black_box(0));
  let mut r = [0u64; N];
  let d = timeit(|| inner_between_u64(&mut g, &mut r, black_box(M), black_box(A), black_box(B)));
  d / (M * N) as f64
}

#[inline(never)]
fn warmup() {
  let mut r = [0u128; N];
  let mut x = 1u128;
  for _ in 0 .. 1_000_000 {
    let r = black_box(&mut r);
    for e in r.iter_mut() {
      *e = x;
      x = x.wrapping_mul(0xf00f);
    }
  }
}

fn main() {
  warmup();
  let a = bench_uniform_u64::<Dandelion>();
  let b = bench_uniform_u64::<Pcg>();
  let c = bench_uniform_u64::<Xoroshiro>();
  let d = bench_uniform_u64::<SmallRng>();
  let e = bench_between_u64::<Dandelion>();
  let f = bench_between_u64::<Pcg>();
  let g = bench_between_u64::<Xoroshiro>();

  print!("{:.3}\n", a);
  print!("{:.3}\n", b);
  print!("{:.3}\n", c);
  print!("{:.3}\n", d);
  print!("{:.3}\n", e);
  print!("{:.3}\n", f);
  print!("{:.3}\n", g);
}

