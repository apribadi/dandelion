//! benchmarks

mod rngs;

use dandelion::Rng as Dandelion;
use rand::rngs::SmallRng;
use rand_pcg::Lcg128CmDxsm64 as PcgDxsm;
use rand_xoshiro::Xoroshiro128PlusPlus as Xoroshiro128pp;
use rngs::Rng;
use std::hint::black_box;
use std::time::Instant;

const L: usize = 10;
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
fn bench_fill_bs<T: Rng>(g: &mut T, r: &mut [Box<[u8]>; N], m: usize) {
  for _ in 0 .. m {
    for e in r.iter_mut() {
      g.fill_b(e);
    }
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

struct Bufs {
  buf0: Box<[u64; N]>,
  buf1: Box<[f64; N]>,
  buf2: Box<[bool; N]>,
  buf3: Box<[u8; 8 * N]>,
  buf4: Box<[u16; 4 * N]>,
  buf5: Box<[u8; N]>,
  buf6: Box<[Box<[u8]>; N]>,
}

#[inline(never)]
fn bench<T: Rng>(bufs: &mut Bufs) -> [f64; L] {
  let mut g = T::new();
  let g = black_box(&mut g);
  [
    timeit(&mut || bench_uniform(g, &mut bufs.buf0, black_box(M))),
    timeit(&mut || bench_uniform_ni(g, &mut bufs.buf0, black_box(M))),
    timeit(&mut || bench_between(g, &mut bufs.buf0, black_box(M), black_box(A), black_box(B))),
    timeit(&mut || bench_between_ni(g, &mut bufs.buf0, black_box(M), black_box(A), black_box(B))),
    timeit(&mut || bench_float(g, &mut bufs.buf1, black_box(M))),
    timeit(&mut || bench_bool(g, &mut bufs.buf2, black_box(M), black_box(P))),
    timeit(&mut || bench_fill_bl(g, bufs.buf3.as_mut_slice(), black_box(M))),
    timeit(&mut || bench_fill_bs(g, &mut bufs.buf6, black_box(M))),
    timeit(&mut || bench_fill_hl(g, bufs.buf4.as_mut_slice(), black_box(M))),
    timeit(&mut || bench_shuffle(g, bufs.buf5.as_mut_slice(), black_box(M))),
  ]
}

fn bench_all(bufs: &mut Bufs) -> [(&'static str, [f64; L]); 4] {
  [
    ("rand-small-rng", bench::<SmallRng>(bufs)),
    ("dandelion", bench::<Dandelion>(bufs)),
    ("xoroshiro128++", bench::<Xoroshiro128pp>(bufs)),
    ("pcg-dxsm", bench::<PcgDxsm>(bufs)),
  ]
}

#[cfg(feature = "thread_local")]
fn bench_thread_local(bufs: &mut Bufs) -> [(&'static str, [f64; L]); 2] {
  [
    ("rand-thread-local", bench::<rngs::RandThreadLocal>(bufs)),
    ("dandelion-thread-local", bench::<rngs::DandelionThreadLocal>(bufs)),
  ]
}

fn display<const K: usize>(t: &[(&'static str, [f64; L]); K]) {
  for &(ref name, ref a) in t.iter() {
    print!("{:<22} ", name);
    for (i, &b) in a.iter().enumerate() {
      if i != 0 {
        print!(" ");
      }
      let x = b / (M * N) as f64;
      print!("{:>8.3}", x);
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
    Bufs {
      buf0: vec![0u64; N].try_into().unwrap(),
      buf1: vec![0f64; N].try_into().unwrap(),
      buf2: vec![false; N].try_into().unwrap(),
      buf3: vec![0u8; 8 * N].try_into().unwrap(),
      buf4: vec![016; 4 * N].try_into().unwrap(),
      buf5: vec![0u8; N].try_into().unwrap(),
      buf6: {
        let mut g: u64 = 0x93c4_67e3_7db0_c7a5;
        Box::new(
          std::array::from_fn(|_| {
            g = g ^ g << 7;
            g = g ^ g >> 9;
            vec![0u8; (g % 9) as usize].into_boxed_slice()
          })
        )
      },
    };
  warmup();
  print!("{:22} ", "");
  for (i, s) in
    [ "uni",
      "uni-ni",
      "bet",
      "bet-ni",
      "float",
      "bool",
      "fill-bl",
      "fill-bs",
      "fill-hl",
      "shuffle"
    ].iter().enumerate()
  {
    if i != 0 {
      print!(" ");
    }
    print!("{:>8}", s);
  }
  print!("\n");
  display(&bench_all(&mut bufs));
  #[cfg(feature = "thread_local")] display(&bench_thread_local(&mut bufs));
}

