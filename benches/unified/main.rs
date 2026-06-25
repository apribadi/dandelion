//! benchmarks

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
fn bench_fill_bl<T: Rng>(g: &mut T, r: &mut [u8], m: usize) {
  for _ in 0 .. m {
    g.fill_b(r);
  }
}

#[inline(never)]
fn bench_fill_hl<T: Rng>(g: &mut T, r: &mut [u16], m: usize) {
  for _ in 0 .. m {
    g.fill_h(r);
  }
}

#[inline(never)]
fn bench_shuffle<T: Rng>(g: &mut T, r: &mut [u8], m: usize) {
  for _ in 0 .. m {
    g.shuffle(r);
  }
}

#[inline(never)]
fn bench<T: Rng>() -> [f64; L] {
  let mut g = T::new();
  [
    { let mut buf = [0u64; N];
      timeit(&mut || bench_uniform(black_box(&mut g), black_box(&mut buf), black_box(M)))
    },
    { let mut buf = [0u64; N];
      timeit(&mut || bench_uniform_ni(black_box(&mut g), black_box(&mut buf), black_box(M)))
    },
    { let mut buf = [0u64; N];
      timeit(&mut || bench_between(black_box(&mut g), black_box(&mut buf), black_box(M), black_box(A), black_box(B)))
    },
    { let mut buf = [0u64; N];
      timeit(&mut || bench_between_ni(black_box(&mut g), black_box(&mut buf), black_box(M), black_box(A), black_box(B)))
    },
    { let mut buf = [0f64; N];
      timeit(&mut || bench_float(black_box(&mut g), black_box(&mut buf), black_box(M)))
    },
    { let mut buf = [false; N];
      timeit(&mut || bench_bool(black_box(&mut g), black_box(&mut buf), black_box(M), black_box(P)))
    },
    { let mut buf = [0u8; 8 * N];
      timeit(&mut || bench_fill_bl(black_box(&mut g), black_box(buf.as_mut_slice()), black_box(M)))
    },
    { let mut buf = [0u16; 4 * N];
      timeit(&mut || bench_fill_hl(black_box(&mut g), black_box(buf.as_mut_slice()), black_box(M)))
    },
    { let mut buf = [0u8; N];
      timeit(&mut || bench_shuffle(black_box(&mut g), black_box(buf.as_mut_slice()), black_box(M)))
    },
  ]
}

fn bench_all() -> [(&'static str, [f64; L]); 4] {
  [
    ("rand-small-rng        ", bench::<SmallRng>()),
    ("dandelion             ", bench::<Dandelion>()),
    ("xoroshiro128++        ", bench::<Xoroshiro128pp>()),
    ("pcg-dxsm              ", bench::<PcgDxsm>()),
  ]
}

#[cfg(feature = "thread_local")]
fn bench_thread_local() -> [(&'static str, [f64; L]); 2] {
  [
    ("rand-thread-local     ", bench::<rngs::RandThreadLocal>()),
    ("dandelion-thread-local", bench::<rngs::DandelionThreadLocal>()),
  ]
}

fn display<const K: usize>(t: &[(&'static str, [f64; L]); K]) {
  for &(ref name, ref a) in t.iter() {
    print!("{} ", name);
    for (i, &b) in a.iter().enumerate() {
      if i != 0 {
        print!("   ");
      }
      let x = b / (M * N) as f64;
      print!("{:.3}", x);
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
  warmup();
  print!("                       uni     uni-ni  bet     bet-ni  flt     bool    fill-bl fill-hl shuffle\n");
  display(&bench_all());
  #[cfg(feature = "thread_local")] display(&bench_thread_local());
}

