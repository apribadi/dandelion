//! benchmarks

mod rngs;

use dandelion::Rng as Dandelion;
use rand::rngs::SmallRng;
use rand_pcg::Lcg128CmDxsm64 as PcgDxsm;
use rand_xoshiro::Xoroshiro128PlusPlus as Xoroshiro128pp;
use rngs::Rng;
use std::hint::black_box;
use std::time::Instant;

const M: usize = 5_000;
const N: usize = 2_000;
const A: u64 = 1;
const B: u64 = 0x1101_0000_0000_0001;

fn timeit(f: &mut dyn FnMut()) -> f64 {
  let a = Instant::now();
  f();
  let b = Instant::now();
  b.saturating_duration_since(a).as_nanos() as f64
}

#[inline(never)]
fn bench_uniform<T: Rng>(g: &mut T, r: &mut [u64; N], m: usize) {
  for _ in 0 .. m {
    for e in r.iter_mut() {
      *e = g.uniform();
    }
  }
}

#[inline(never)]
fn bench_between<T: Rng>(g: &mut T, r: &mut [u64; N], m: usize, a: u64, b: u64) {
  for _ in 0 .. m {
    for e in r.iter_mut() {
      *e = g.between(a, b);
    }
  }
}

#[inline(never)]
fn bench<T: Rng>() -> [f64; 2] {
  let mut g = T::from_u64(black_box(0));
  let mut buf = [0u64; N];
  [
    timeit(&mut || bench_uniform(black_box(&mut g), black_box(&mut buf), black_box(M))),
    timeit(&mut || bench_between(black_box(&mut g), black_box(&mut buf), black_box(M), black_box(A), black_box(B))),
  ]
}

fn bench_all() -> [(&'static str, [f64; 2]); 3] {
  [
    ("dandelion     ", bench::<Dandelion>()),
    ("xoroshiro128++", bench::<Xoroshiro128pp>()),
    ("pcg-dxsm      ", bench::<PcgDxsm>()),
  ]
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
  let t = bench_all();
  for &(ref name, ref a) in t.iter() {
    print!("{}", name);
    for &b in a.iter() {
      let x = b / (M * N) as f64;
      print!(" {:.3}", x);
    }
    print!("\n");
  }
}

