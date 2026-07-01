//! unified benchmarks

mod rngs;

use dandelion::Rng as Dandelion;
use rand::rngs::SmallRng;
use rand_pcg::Lcg128CmDxsm64 as PcgDxsm;
use rand_xoshiro::Xoroshiro128PlusPlus as Xoroshiro128pp;
use rngs::Rng;
use std::hint::black_box;
use std::time::Instant;

const L: usize = 9;
const M: usize = 5_000;
const N: usize = 2_000;
const A: u64 = 1;
const B: u64 = 0x1101_0000_0000_0001;
const P: f64 = 0.75;

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
fn uniform_ni<T: Rng>(g: &mut T) -> u64 {
  g.uniform()
}

#[inline(never)]
fn bench_uniform_ni<T: Rng>(g: &mut T, r: &mut [u64; N], m: usize) {
  for _ in 0 .. m {
    for e in r.iter_mut() {
      *e = uniform_ni(g);
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
fn between_ni<T: Rng>(g: &mut T, a: u64, b: u64) -> u64 {
  g.between(a, b)
}

#[inline(never)]
fn bench_between_ni<T: Rng>(g: &mut T, r: &mut [u64; N], m: usize, a: u64, b: u64) {
  for _ in 0 .. m {
    for e in r.iter_mut() {
      *e = between_ni(g, a, b);
    }
  }
}

#[inline(never)]
fn bench_float<T: Rng>(g: &mut T, r: &mut [f64; N], m: usize) {
  for _ in 0 .. m {
    for e in r.iter_mut() {
      *e = g.float();
    }
  }
}


#[inline(never)]
fn bench_bool<T: Rng>(g: &mut T, r: &mut [bool; N], m: usize, p: f64) {
  for _ in 0 .. m {
    for e in r.iter_mut() {
      *e = g.bool(p);
    }
  }
}

#[inline(never)]
fn bench_fill<T: Rng>(g: &mut T, r: &mut [u8], m: usize) {
  for _ in 0 .. m {
    g.fill(r);
  }
}

#[inline(never)]
fn bench_fill_small<T: Rng>(g: &mut T, r: &mut [Box<[u8]>; N], m: usize) {
  for _ in 0 .. m {
    for e in r.iter_mut() {
      g.fill(e);
    }
  }
}

#[inline(never)]
fn bench_shuffle<T: Rng>(g: &mut T, r: &mut [u8], m: usize) {
  for _ in 0 .. m {
    g.shuffle(r);
  }
}

struct Bufs(
  Box<[u64; N]>,
  Box<[f64; N]>,
  Box<[bool; N]>,
  Box<[u8]>,
  Box<[u8]>,
  Box<[Box<[u8]>; N]>,
);

#[inline(never)]
fn bench<T: Rng>(bufs: &mut Bufs) -> [f64; L] {
  let mut g = T::new();
  [
    timeit(&mut || bench_uniform(black_box(&mut g), &mut bufs.0, black_box(M))),
    timeit(&mut || bench_uniform_ni(black_box(&mut g), &mut bufs.0, black_box(M))),
    timeit(&mut || bench_between(black_box(&mut g), &mut bufs.0, black_box(M), black_box(A), black_box(B))),
    timeit(&mut || bench_between_ni(black_box(&mut g), &mut bufs.0, black_box(M), black_box(A), black_box(B))),
    timeit(&mut || bench_float(black_box(&mut g), &mut bufs.1, black_box(M))),
    timeit(&mut || bench_bool(black_box(&mut g), &mut bufs.2, black_box(M), black_box(P))),
    timeit(&mut || bench_fill(black_box(&mut g), &mut bufs.3, black_box(M))),
    timeit(&mut || bench_fill_small(black_box(&mut g), &mut bufs.5, black_box(M))),
    timeit(&mut || bench_shuffle(black_box(&mut g), &mut bufs.4, black_box(M))),
  ]
}

fn bench_all(bufs: &mut Bufs) -> [(&'static str, [f64; L]); 4] {
  [
    ("dandelion", bench::<Dandelion>(bufs)),
    ("xoroshiro128++", bench::<Xoroshiro128pp>(bufs)),
    ("pcg-dxsm", bench::<PcgDxsm>(bufs)),
    ("rand-small-rng", bench::<SmallRng>(bufs)),
  ]
}

#[cfg(feature = "thread_local")]
fn bench_thread_local(bufs: &mut Bufs) -> [(&'static str, [f64; L]); 2] {
  [
    ("dandelion-thread-local", bench::<rngs::DandelionThreadLocal>(bufs)),
    ("rand-thread-local", bench::<rngs::RandThreadLocal>(bufs)),
  ]
}

fn display<const K: usize>(t: &[(&'static str, [f64; L]); K]) {
  for &(ref name, ref a) in t.iter() {
    print!("{}", name);
    for b in a.iter() {
      let x = b / (M * N) as f64;
      print!(", {:.3}", x);
    }
    print!("\n");
  }
}

#[inline(never)]
fn warmup() {
  let mut r = [(0u64, 0f64, false); N];
  let mut x = 1u64;
  for _ in 0 .. 100_000 {
    let r = black_box(&mut r);
    for e in r.iter_mut() {
      e.0 = x;
      x = x.wrapping_mul(0xf00f);
    }
  }
}

fn main() {
  let mut bufs =
    Bufs(
      vec![0u64; N].try_into().unwrap(),
      vec![0f64; N].try_into().unwrap(),
      vec![false; N].try_into().unwrap(),
      vec![0u8; 8 * N].into_boxed_slice(),
      vec![0u8; N].into_boxed_slice(),
      {
        let mut g: u64 = 0x93c4_67e3_7db0_c7a5;
        let mut a = Vec::new();
        for _ in 0 .. N {
          g ^= g << 7;
          g ^= g >> 9;
          a.push(vec![0u8; (g % 9) as usize].into_boxed_slice());
        }
        a.try_into().unwrap()
      },
    );
  warmup();
  print!(", uniform, uniform-ni, between, between-ni, float, bool, fill, fill-sm, shuffle\n");
  display(&bench_all(&mut bufs));
  #[cfg(feature = "thread_local")] display(&bench_thread_local(&mut bufs));
}

