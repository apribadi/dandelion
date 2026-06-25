//! unified benchmarks

mod rngs;
mod doubledandelion;

use dandelion::Rng;
use divan::Bencher;
use doubledandelion::DoubleDandelion;
use rand::rngs::SmallRng;
use rand_pcg::Lcg128CmDxsm64;
use rand_xoshiro::Xoroshiro128PlusPlus;
use rngs::RngForBench;
use std::hint::black_box;

const N: usize = 1_000;
const LO: u64 = 1;
const HI: u64 = 0x1101_0000_0000_0001;
// const HI: u64 = 6;

fn main() {
  divan::main();
}

#[cfg(feature = "thread_local")]
#[divan::bench]
fn bench_thread_local_u64() -> u64 {
  let mut a = 0u64;
  for _ in 0 .. N { a ^= dandelion::thread_local::uniform::<u64>(); }
  a
}

#[cfg(feature = "thread_local")]
#[divan::bench]
fn bench_thread_local_u128() -> u128 {
  let mut a = 0u128;
  for _ in 0 .. N { a ^= dandelion::thread_local::uniform::<u128>(); }
  a
}

#[cfg(feature = "thread_local")]
#[divan::bench]
fn bench_thread_local_u64_noinline() -> u64 {
  #[inline(never)]
  fn next_u64() -> u64 { dandelion::thread_local::uniform::<u64>() }
  let mut a = 0u64;
  for _ in 0 .. N { a ^= next_u64(); };
  a
}

#[cfg(feature = "thread_local")]
#[divan::bench]
fn bench_thread_local_rand_u64() -> u64 {
  let mut a = 0u64;
  for _ in 0 .. N { a ^= rand::random::<u64>(); }
  a
}

#[cfg(feature = "thread_local")]
#[divan::bench]
fn bench_thread_local_rand_u128() -> u128 {
  let mut a = 0u128;
  for _ in 0 .. N { a ^= rand::random::<u128>(); }
  a
}

#[cfg(feature = "thread_local")]
#[divan::bench]
fn bench_thread_local_rand_u64_noinline() -> u64 {
  #[inline(never)]
  fn next_u64() -> u64 { rand::random::<u64>() }
  let mut a = 0u64;
  for _ in 0 .. N { a ^= next_u64(); }
  a
}

#[divan::bench(types = [DoubleDandelion, Rng, Lcg128CmDxsm64, Xoroshiro128PlusPlus, SmallRng])]
fn bench_u64<T: RngForBench>(bencher: Bencher<'_, '_>) {
  #[inline(never)]
  fn go<U: RngForBench>(rng: &mut U, buf: &mut [u64; N]) {
    for elt in buf.iter_mut() { *elt = rng.u64(); }
  }
  let mut buf = [0u64; N];
  let mut rng = T::from_u64(black_box(0));
  bencher.bench_local(|| for _ in 0 .. N { go(&mut rng, &mut buf) });
}

#[divan::bench(types = [DoubleDandelion, Rng, Lcg128CmDxsm64, Xoroshiro128PlusPlus, SmallRng])]
fn bench_u64_noinline<T: RngForBench>(bencher: Bencher<'_, '_>) {
  #[inline(never)]
  fn next_u64<T: RngForBench>(rng: &mut T) -> u64 {
    rng.u64()
  }
  #[inline(never)]
  fn go<U: RngForBench>(rng: &mut U, buf: &mut [u64; N]) {
    for elt in buf.iter_mut() { *elt = next_u64(rng); }
  }
  let mut buf = [0u64; N];
  let mut rng = T::from_u64(black_box(0));
  bencher.bench_local(|| for _ in 0 .. N { go(&mut rng, &mut buf) });
}

#[divan::bench(types = [Rng, Lcg128CmDxsm64, Xoroshiro128PlusPlus, SmallRng])]
fn bench_range_u64<T: RngForBench>(bencher: Bencher<'_, '_>) {
  #[inline(never)]
  fn go<U: RngForBench>(rng: &mut U, buf: &mut [u64; N], lo: u64, hi: u64) {
    for elt in buf.iter_mut() { *elt = rng.range_u64(lo, hi); }
  }
  let mut buf = [0u64; N];
  let mut rng = T::from_u64(black_box(0));
  let lo = black_box(LO);
  let hi = black_box(HI);
  bencher.bench_local(|| for _ in 0 .. N { go(&mut rng, &mut buf, lo, hi) });
}

#[divan::bench(types = [Rng, Lcg128CmDxsm64, Xoroshiro128PlusPlus, SmallRng])]
fn bench_range_u64_noinline<T: RngForBench>(bencher: Bencher<'_, '_>) {
  #[inline(never)]
  fn range_u64<T: RngForBench>(rng: &mut T, a: u64, b: u64) -> u64 {
    rng.range_u64(a, b)
  }
  #[inline(never)]
  fn go<U: RngForBench>(rng: &mut U, buf: &mut [u64; N], lo: u64, hi: u64) {
    for elt in buf.iter_mut() { *elt = range_u64(rng, lo, hi); }
  }
  let mut buf = [0u64; N];
  let mut rng = T::from_u64(black_box(0));
  let lo = black_box(LO);
  let hi = black_box(HI);
  bencher.bench_local(|| for _ in 0 .. N { go(&mut rng, &mut buf, lo, hi) } );
}

#[divan::bench(types = [Rng, Lcg128CmDxsm64, Xoroshiro128PlusPlus, SmallRng])]
fn bench_shuffle<T: RngForBench>(bencher: Bencher<'_, '_>) {
  #[inline(never)]
  fn go<U: RngForBench>(rng: &mut U, buf: &mut [u32; N]) {
    rng.shuffle(buf)
  }
  let mut buf = [0u32; N];
  let mut rng = T::from_u64(black_box(0));
  bencher.bench_local(|| for _ in 0 .. N { go(&mut rng, &mut buf) });
}

#[divan::bench(types = [Rng, Lcg128CmDxsm64, Xoroshiro128PlusPlus, SmallRng])]
fn bench_f64<T: RngForBench>(bencher: Bencher<'_, '_>) {
  #[inline(never)]
  fn go<U: RngForBench>(rng: &mut U, buf: &mut [f64; N]) {
    for elt in buf.iter_mut() { *elt = rng.f64(); }
  }
  let mut buf = [0_f64; N];
  let mut rng = T::from_u64(black_box(0));
  bencher.bench_local(|| for _ in 0 .. N { go(&mut rng, &mut buf) });
}

#[divan::bench(types = [Rng, Lcg128CmDxsm64, Xoroshiro128PlusPlus, SmallRng])]
fn bench_f64_noinline<T: RngForBench>(bencher: Bencher<'_, '_>) {
  #[inline(never)]
  fn next_f64<T: RngForBench>(rng: &mut T) -> f64 {
    rng.f64()
  }
  #[inline(never)]
  fn go<U: RngForBench>(rng: &mut U, buf: &mut [f64; N]) {
    for elt in buf.iter_mut() { *elt = next_f64(rng); }
  }
  let mut buf = [0_f64; N];
  let mut rng = T::from_u64(black_box(0));
  bencher.bench_local(|| for _ in 0 .. N { go(&mut rng, &mut buf) });
}

#[divan::bench(types = [DoubleDandelion, Rng, Lcg128CmDxsm64, Xoroshiro128PlusPlus, SmallRng])]
fn bench_fill_b_large<T: RngForBench>(bencher: Bencher<'_, '_>) {
  let mut buf = [0u8; 8 * 1_000];
  let mut rng = T::from_u64(black_box(0));
  bencher.bench_local(|| for _ in 0 .. N { T::fill_b(&mut rng, &mut buf) });
}

#[divan::bench(types = [Rng, Lcg128CmDxsm64, Xoroshiro128PlusPlus, SmallRng])]
fn bench_fill_h_large<T: RngForBench>(bencher: Bencher<'_, '_>) {
  let mut buf = [0u16; 4 * 1_000];
  let mut rng = T::from_u64(black_box(0));
  bencher.bench_local(|| for _ in 0 .. N { T::fill_h(&mut rng, &mut buf) });
}

fn make_boxed_byte_slices() -> [Box<[u8]>; N] {
  std::array::from_fn(|_| {
    let mut g: u64 = 0x93c4_67e3_7db0_c7a5;
    g = g ^ g << 7;
    g = g ^ g >> 9;
    vec![0u8; (g % 9) as usize].into_boxed_slice()
  })
}

#[divan::bench(types = [Rng, Lcg128CmDxsm64, Xoroshiro128PlusPlus, SmallRng])]
fn bench_fill_b_small<T: RngForBench>(bencher: Bencher<'_, '_>) {
  #[inline(never)]
  fn go<U: RngForBench>(rng: &mut U, buf: &mut [Box<[u8]>; N]) {
    for elt in buf.iter_mut() { rng.fill_b(elt); }
  }
  let mut buf: [Box<[u8]>; N] = make_boxed_byte_slices();
  let mut rng = T::from_u64(black_box(0));
  bencher.bench_local(|| for _ in 0 .. N { go(&mut rng, &mut buf) });
}

#[divan::bench(types = [Rng, Lcg128CmDxsm64, Xoroshiro128PlusPlus, SmallRng])]
fn bench_fill_b_small_noinline<T: RngForBench>(bencher: Bencher<'_, '_>) {
  #[inline(never)]
  fn fill_b<T: RngForBench>(rng: &mut T, buf: &mut [u8]) {
    rng.fill_b(buf)
  }
  #[inline(never)]
  fn go<U: RngForBench>(rng: &mut U, buf: &mut [Box<[u8]>; N]) {
    for elt in buf.iter_mut() { fill_b(rng, elt); }
  }
  let mut buf: [Box<[u8]>; N] = make_boxed_byte_slices();
  let mut rng = T::from_u64(black_box(0));
  bencher.bench_local(|| for _ in 0 .. N { go(&mut rng, &mut buf) });
}

#[divan::bench(types = [Rng, Lcg128CmDxsm64, Xoroshiro128PlusPlus, SmallRng])]
fn bench_bernoulli<T: RngForBench>(bencher: Bencher<'_, '_>) {
  #[inline(never)]
  fn go<U: RngForBench>(rng: &mut U, buf: &mut [bool; N], p: f64) {
    for elt in buf.iter_mut() { *elt = rng.bernoulli(p); }
  }
  let mut buf = [false; N];
  let mut rng = T::from_u64(black_box(0));
  let p = black_box(0.75);
  bencher.bench_local(|| for _ in 0 .. N { go(&mut rng, &mut buf, p) });
}

#[divan::bench(types = [Rng, Lcg128CmDxsm64, Xoroshiro128PlusPlus, SmallRng])]
fn bench_bernoulli_noinline<T: RngForBench>(bencher: Bencher<'_, '_>) {
  #[inline(never)]
  fn next_bernoulli<T: RngForBench>(rng: &mut T, p: f64) -> bool {
    rng.bernoulli(p)
  }
  #[inline(never)]
  fn go<U: RngForBench>(rng: &mut U, buf: &mut [bool; N], p: f64) {
    for elt in buf.iter_mut() { *elt = next_bernoulli(rng, p); }
  }
  let mut buf = [false; N];
  let mut rng = T::from_u64(black_box(0));
  let p = black_box(0.75);
  bencher.bench_local(|| for _ in 0 .. N { go(&mut rng, &mut buf, p) });
}
