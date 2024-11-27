#![doc = include_str!("../README.md")]
#![no_std]

#[cfg(feature = "std")]
extern crate std;

use core::num::NonZeroU128;

/// A high performance non-cryptographic random number generator.

#[derive(Clone)]
pub struct Rng { state: NonZeroU128 }

#[inline(always)]
const fn get_chunk<T, const N: usize>(slice: &[T], offset: usize) -> &[T; N] {
  assert!(offset <= slice.len() && N <= slice.len() - offset);
  unsafe { &*(slice.as_ptr().add(offset) as *const [T; N]) }
}

#[inline(always)]
fn get_chunk_mut<T, const N: usize>(slice: &mut [T], offset: usize) -> &mut [T; N] {
  assert!(offset <= slice.len() && N <= slice.len() - offset);
  unsafe { &mut *(slice.as_mut_ptr().add(offset) as *mut [T; N]) }
}

#[inline(always)]
const fn front_chunk<T, const N: usize>(slice: &[T]) -> &[T; N] {
  get_chunk(slice, 0)
}

#[inline(always)]
fn front_chunk_mut<T, const N: usize>(slice: &mut [T]) -> &mut [T; N] {
  get_chunk_mut(slice, 0)
}

#[inline(always)]
const fn hash(x: NonZeroU128) -> NonZeroU128 {
  // The hash uses the multiplier
  //
  //   M = round_nearest_odd(EULER_MASCHERONI * 2¹²⁸)
  //
  // The Euler-Mascheroni constant was selected because it is a well-known
  // number in the range (0.5, 1.0).

  const M: u128 = 0x93c4_67e3_7db0_c7a4_d1be_3f81_0152_cb57;

  let x = x.get();
  let x = x.wrapping_mul(M);
  let x = x.swap_bytes();
  let x = x.wrapping_mul(M);
  let x = x.swap_bytes();
  let x = x.wrapping_mul(M);
  unsafe { NonZeroU128::new_unchecked(x) }
}

impl Rng {
  /// Creates a random number generator with an initial state derived by
  /// hashing the given byte array.

  pub const fn new(seed: [u8; 15]) -> Self {
    let x = u64::from_le_bytes(*get_chunk(&seed, 0));
    let y = u64::from_le_bytes(*get_chunk(&seed, 7));
    let s = x as u128 | ((y >> 8) as u128) << 64;
    let s = s | 1 << 120;
    let s = NonZeroU128::new(s).unwrap();
    Self { state: hash(s) }
  }

  /// Creates a random number generator with an initial state derived by
  /// hashing the given `u64` seed.

  pub const fn from_u64(seed: u64) -> Self {
    let s = seed as u128;
    let s = s | 1 << 64;
    let s = NonZeroU128::new(s).unwrap();
    Self { state: hash(s) }
  }

  /// Retrieves the current state of the random number generator.

  #[inline(always)]
  pub const fn state(&self) -> NonZeroU128 {
    self.state
  }

  /// Creates a random number generator with a particular initial state.
  ///
  /// <div class="warning">
  ///
  /// If you want to deterministically initialize a generator from a small
  /// integer or other weak seed, you should *NOT* use this function and should
  /// instead use [Rng::new] or [Rng::from_u64] which hash their arguments.
  ///
  /// </div>

  #[inline(always)]
  pub const fn from_state(state: NonZeroU128) -> Self {
    Self { state }
  }

  /// Creates a random number generator with a random seed retrieved from the
  /// operating system.

  #[cfg(feature = "getrandom")]
  #[inline(never)]
  #[cold]
  pub fn from_operating_system() -> Self {
    let mut buf = [0; 16];
    getrandom::fill(&mut buf).expect("getrandom::fill failed!");
    let s = u128::from_le_bytes(buf);
    let s = s | 1;
    let s = NonZeroU128::new(s).unwrap();
    Self { state: s }
  }

  /// Splits off a new random number generator that may be used along with the
  /// original.

  #[inline(always)]
  pub fn split(&mut self) -> Self {
    let x = self.u64();
    let y = self.u64();
    let s = x as u128 ^ (y as u128) << 64;
    let s = s | 1;
    let s = NonZeroU128::new(s).unwrap();
    Self { state: s }
  }

  /// Samples a `bool` from the Bernoulli distribution where `true` appears
  /// with probability approximately equal to `p`.
  ///
  /// Probabilities `p` <= 0 or NaN are treated as 0, and `p` >= 1 are
  /// treated as 1.

  #[inline(always)]
  pub fn bernoulli(&mut self, p: f64) -> bool {
    // For every `p` that is representable as a `f64`, is in the range [0, 1],
    // and is an exact multiple of 2⁻¹²⁸, this procedure samples exactly from
    // the corresponding Bernoulli distribution, given the (false!) assumption
    // that `dandelion::u64` samples exactly uniformly.
    //
    // In particular `bernoulli(0)` is always `false` and `bernoulli(1)` is
    // always `true`.

    let x = self.u64();
    let e = 1022 - x.trailing_zeros() as u64;
    let t = f64::from_bits((e << 52) + (x >> 12));
    t < p
  }

  /// Samples a `bool` from the uniform distribution.

  #[inline(always)]
  pub fn bool(&mut self) -> bool {
    self.i64() < 0
  }

  /// Samples a `i32` from the uniform distribution.

  #[inline(always)]
  pub fn i32(&mut self) -> i32 {
    self.u64() as i32
  }

  /// Samples a `i64` from the uniform distribution.

  #[inline(always)]
  pub fn i64(&mut self) -> i64 {
    self.u64() as i64
  }

  /// Samples a `u32` from the uniform distribution.

  #[inline(always)]
  pub fn u32(&mut self) -> u32 {
    self.u64() as u32
  }

  /// Samples a `u64` from the uniform distribution.

  #[inline(always)]
  pub fn u64(&mut self) -> u64 {
    let s = self.state.get();
    let x = s as u64;
    let y = (s >> 64) as u64;
    let u = y ^ y >> 19;
    let v = x ^ y.rotate_right(7);
    let w = x as u128 * x as u128;
    let z = y.wrapping_add(w as u64 ^ (w >> 64) as u64);
    let s = u as u128 ^ (v as u128) << 64;
    self.state = unsafe { NonZeroU128::new_unchecked(s) };
    z
  }

  /// Samples a `u32` from the uniform distribution over the range `0 ... n`.
  ///
  /// The upper bound is inclusive.

  #[inline(always)]
  pub fn bounded_u32(&mut self, n: u32) -> u32 {
    // Cf. `bounded_u64`.

    let x = self.u64() as u128;
    let y = self.u64() as u128;
    let n = n as u128;
    let u = x * n + x >> 64;
    let v = y * n + y;
    let z = u + v >> 64;
    z as u32
  }

  /// Samples a `u64` from the uniform distribution over the range `0 ... n`.
  ///
  /// The upper bound is inclusive.

  #[inline(always)]
  pub fn bounded_u64(&mut self, n: u64) -> u64 {
    // This procedure computes
    //
    //   floor((k * n + k) / 2¹²⁸)
    //
    // where k is sampled approximately uniformly from 0 ... 2¹²⁸ - 1.  The
    // result is a very low bias sample from the desired distribution.

    //     y x                  x        y 0      v v 0
    // *     n            *     n    *     n    +   u _
    // +   y x  ------->  +     x    +   y 0
    // -------            -------    -------    -------
    //   z _ _                u _      v v 0      z _ _

    let x = self.u64() as u128;
    let y = self.u64() as u128;
    let n = n as u128;
    let u = x * n + x >> 64;
    let v = y * n + y;
    let z = u + v >> 64;
    z as u64
  }

  /// Samples a `i32` from the uniform distribution over the range `lo ... hi`.
  ///
  /// The lower and upper bounds are inclusive, and the range can wrap around
  /// from `i32::MAX` to `i32::MIN`.

  #[inline(always)]
  pub fn between_i32(&mut self, lo: i32, hi: i32) -> i32 {
    self.between_u32(lo as u32, hi as u32) as i32
  }

  /// Samples a `i64` from the uniform distribution over the range `lo ... hi`.
  ///
  /// The lower and upper bounds are inclusive, and the range can wrap around
  /// from `i64::MAX` to `i64::MIN`.

  #[inline(always)]
  pub fn between_i64(&mut self, lo: i64, hi: i64) -> i64 {
    self.between_u64(lo as u64, hi as u64) as i64
  }

  /// Samples a `u32` from the uniform distribution over the range `lo ... hi`.
  ///
  /// The lower and upper bounds are inclusive, and the range can wrap around
  /// from `u32::MAX` to `u32::MIN`.

  #[inline(always)]
  pub fn between_u32(&mut self, lo: u32, hi: u32) -> u32 {
    lo.wrapping_add(self.bounded_u32(hi.wrapping_sub(lo)))
  }

  /// Samples a `u64` from the uniform distribution over the range `lo ... hi`.
  ///
  /// The lower and upper bounds are inclusive, and the range can wrap around
  /// from `u64::MAX` to `u64::MIN`.

  #[inline(always)]
  pub fn between_u64(&mut self, lo: u64, hi: u64) -> u64 {
    lo.wrapping_add(self.bounded_u64(hi.wrapping_sub(lo)))
  }

  /// Samples a `f32` from a distribution that approximates the uniform
  /// distribution over the real interval [0, 1].
  ///
  /// The distribution is the same as the one produced by the following
  /// procedure:
  ///
  /// - Sample a real number from the uniform distribution on [0, 1].
  /// - Round to the nearest multiple of 2⁻⁶³.
  /// - Round to a `f32` using the default rounding mode.
  ///
  /// A zero output will always be +0, never -0.

  #[inline(always)]
  pub fn f32(&mut self) -> f32 {
    let x = self.i64();
    let x = f32::from_bits(0x2000_0000) * x as f32;
    f32::from_bits(0x7fff_ffff & x.to_bits())
  }

  /// Samples a `f64` from a distribution that approximates the uniform
  /// distribution over the real interval [0, 1].
  ///
  /// The distribution is the same as the one produced by the following
  /// procedure:
  ///
  /// - Sample a real number from the uniform distribution on [0, 1].
  /// - Round to the nearest multiple of 2⁻⁶³.
  /// - Round to a `f64` using the default rounding mode.
  ///
  /// A zero output will always be +0, never -0.

  #[inline(always)]
  pub fn f64(&mut self) -> f64 {
    // The conversion into a `f64` is two instructions on aarch64:
    //
    //   scvtf d0, x8, #63
    //   fabs d0, d0

    let x = self.i64();
    let x = f64::from_bits(0x3c00_0000_0000_0000) * x as f64;
    f64::from_bits(0x7fff_ffff_ffff_ffff & x.to_bits())
  }

  #[inline(always)]
  fn bytes_inlined(&mut self, dst: &mut [u8]) {
    let mut dst = dst;

    if dst.len() == 0 {
      return;
    }

    while dst.len() >= 17 {
      let x = self.u64();
      let y = self.u64();
      *front_chunk_mut(dst) = x.to_le_bytes();
      dst = &mut dst[8 ..];
      *front_chunk_mut(dst) = y.to_le_bytes();
      dst = &mut dst[8 ..];
    }

    if dst.len() >= 9 {
      let x = self.u64();
      *front_chunk_mut(dst) = x.to_le_bytes();
      dst = &mut dst[8 ..];
    }

    let x = self.u64();

    match dst.len() {
      1 => *front_chunk_mut(dst) = *front_chunk::<u8, 1>(&x.to_le_bytes()),
      2 => *front_chunk_mut(dst) = *front_chunk::<u8, 2>(&x.to_le_bytes()),
      3 => *front_chunk_mut(dst) = *front_chunk::<u8, 3>(&x.to_le_bytes()),
      4 => *front_chunk_mut(dst) = *front_chunk::<u8, 4>(&x.to_le_bytes()),
      5 => *front_chunk_mut(dst) = *front_chunk::<u8, 5>(&x.to_le_bytes()),
      6 => *front_chunk_mut(dst) = *front_chunk::<u8, 6>(&x.to_le_bytes()),
      7 => *front_chunk_mut(dst) = *front_chunk::<u8, 7>(&x.to_le_bytes()),
      8 => *front_chunk_mut(dst) = *front_chunk::<u8, 8>(&x.to_le_bytes()),
      _ => unreachable!(),
    }
  }

  /// Fills the provided buffer with independent uniformly distributed bytes.

  pub fn bytes(&mut self, dst: &mut [u8]) {
    self.bytes_inlined(dst);
  }

  /// Samples an array of independent uniformly distributed bytes.

  pub fn byte_array<const N: usize>(&mut self) -> [u8; N] {
    let mut buf = [0; N];
    self.bytes_inlined(&mut buf);
    buf
  }
}

#[cfg(feature = "rand_core")]
impl rand_core::RngCore for Rng {
  #[inline(always)]
  fn next_u32(&mut self) -> u32 {
    self.u32()
  }

  #[inline(always)]
  fn next_u64(&mut self) -> u64 {
    self.u64()
  }

  fn fill_bytes(&mut self, dst: &mut [u8]) {
    self.bytes(dst)
  }
}

#[cfg(feature = "rand_core")]
impl rand_core::SeedableRng for Rng {
  type Seed = [u8; 16];

  fn from_seed(seed: Self::Seed) -> Self {
    let s = u128::from_le_bytes(seed);
    let s = s | 1;
    let s = NonZeroU128::new(s).unwrap();
    Self::from_state(s)
  }

  fn seed_from_u64(seed: u64) -> Self {
    Self::from_u64(seed)
  }

  fn from_rng(rng: &mut impl rand_core::RngCore) -> Self {
    let x = rng.next_u64();
    let y = rng.next_u64();
    let s = x as u128 ^ (y as u128) << 64;
    let s = s | 1;
    let s = NonZeroU128::new(s).unwrap();
    Self::from_state(s)
  }
}

#[cfg(feature = "thread_local")]
pub mod thread_local {
  //! Access a thread-local random number generator.
  //!
  //! If you want to generate many random numbers, you should create a local
  //! generator with [dandelion::thread_local::split](split).

  use core::cell::Cell;
  use core::num::NonZeroU128;

  use crate::Rng;

  std::thread_local! {
    static RNG: Cell<Option<NonZeroU128>> = const {
      Cell::new(None)
    };
  }

  // The function `with` is *NOT* logically re-entrant, so we must not expose
  // it publicly.

  #[inline(always)]
  fn with<F, T>(f: F) -> T
  where
    F: FnOnce(&mut Rng) -> T
  {
    RNG.with(|cell| {
      let mut rng =
        match cell.get() {
          None =>
            Rng::from_operating_system(),
          Some(s) =>
            Rng::from_state(s),
        };
      let x = f(&mut rng);
      cell.set(Some(rng.state()));
      x
    })
  }

  /// See [Rng::split].

  pub fn split() -> Rng {
    with(|rng| rng.split())
  }

  /// See [Rng::bernoulli].

  pub fn bernoulli(p: f64) -> bool {
    with(|rng| rng.bernoulli(p))
  }

  /// See [Rng::bool].

  pub fn bool() -> bool {
    with(|rng| rng.bool())
  }

  /// See [Rng::i32].

  pub fn i32() -> i32 {
    with(|rng| rng.i32())
  }

  /// See [Rng::i64].

  pub fn i64() -> i64 {
    with(|rng| rng.i64())
  }

  /// See [Rng::u32].

  pub fn u32() -> u32 {
    with(|rng| rng.u32())
  }

  /// See [Rng::u64].

  pub fn u64() -> u64 {
    with(|rng| rng.u64())
  }

  /// See [Rng::bounded_u32].

  pub fn bounded_u32(n: u32) -> u32 {
    with(|rng| rng.bounded_u32(n))
  }

  /// See [Rng::bounded_u64].

  pub fn bounded_u64(n: u64) -> u64 {
    with(|rng| rng.bounded_u64(n))
  }

  /// See [Rng::between_i32].

  pub fn between_i32(lo: i32, hi: i32) -> i32 {
    with(|rng| rng.between_i32(lo, hi))
  }

  /// See [Rng::between_i64].

  pub fn between_i64(lo: i64, hi: i64) -> i64 {
    with(|rng| rng.between_i64(lo, hi))
  }

  /// See [Rng::between_u32].

  pub fn between_u32(lo: u32, hi: u32) -> u32 {
    with(|rng| rng.between_u32(lo, hi))
  }

  /// See [Rng::between_u64].

  pub fn between_u64(lo: u64, hi: u64) -> u64 {
    with(|rng| rng.between_u64(lo, hi))
  }

  /// See [Rng::f32].

  pub fn f32() -> f32 {
    with(|rng| rng.f32())
  }

  /// See [Rng::f64].

  pub fn f64() -> f64 {
    with(|rng| rng.f64())
  }

  /// See [Rng::bytes].

  pub fn bytes(dst: &mut [u8]) {
    with(|rng| rng.bytes(dst))
  }

  /// See [Rng::byte_array].

  pub fn byte_array<const N: usize>() -> [u8; N] {
    with(|rng| rng.byte_array())
  }
}
