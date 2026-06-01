#![doc = include_str!("../README.md")]
#![no_std]

#[cfg(feature = "std")]
extern crate std;

use core::fmt::Debug;
use core::fmt::Formatter;
use core::fmt;
use core::hint::cold_path;
use core::mem::MaybeUninit;
use core::num::NonZeroU128;
use core::num::NonZeroU16;
use core::num::NonZeroU32;
use core::num::NonZeroU64;
use core::num::NonZeroU8;

/// A high performance non-cryptographic random number generator.
#[derive(Clone)]
pub struct Rng { state: NonZeroU128 }

#[inline(always)]
const fn concat(x: u64, y: u64) -> u128 {
  x as u128 ^ (y as u128) << 64
}

#[inline(always)]
const fn lo(x: u128) -> u64 {
  x as u64
}

#[inline(always)]
const fn hi(x: u128) -> u64 {
  (x >> 64) as u64
}

#[inline(always)]
const fn asr(x: u64, a: usize) -> u64 {
  ((x as i64) >> a) as u64
}

#[inline(always)]
const fn lsl(x: u64, a: usize) -> u64 {
  x << a
}

#[inline(always)]
const fn mulhi(x: u64, y: u64) -> u64 {
  ((x as u128 * y as u128) >> 64) as u64
}

impl Rng {
  /// Creates a random number generator with an initial state derived by
  /// hashing the given seed.
  pub const fn new(seed: NonZeroU128) -> Self {
    // The hash uses the multiplier
    //
    //   M = round_nearest_odd(EULER_MASCHERONI * 2¹²⁸)
    //
    // The Euler-Mascheroni constant was selected because it is a well-known
    // number in the range (0.5, 1.0).
    const M: u128 = 0x93c4_67e3_7db0_c7a4_d1be_3f81_0152_cb57;
    let x = seed.get();
    let x = x.wrapping_mul(M);
    let x = x.swap_bytes();
    let x = x.wrapping_mul(M);
    let x = x.swap_bytes();
    let x = x.wrapping_mul(M);
    let s = unsafe { NonZeroU128::new_unchecked(x) };
    Self { state: s }
  }

  /// Creates a random number generator with an initial state derived by
  /// hashing the given `u64` seed.
  pub const fn from_u64(seed: u64) -> Self {
    let s = concat(seed, 1);
    let s = unsafe { NonZeroU128::new_unchecked(s) };
    Self::new(s)
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
    let s = 1 | u128::from_le_bytes(buf);
    let s = unsafe { NonZeroU128::new_unchecked(s) };
    Self { state: s }
  }

  /// Samples a `bool` from the Bernoulli distribution where `true` appears
  /// with probability approximately equal to `p`.
  ///
  /// A probability `p` <= 0 or NaN is treated as 0, and a probability `p` >= 1
  /// is treated as 1.
  #[inline(always)]
  pub const fn bernoulli(&mut self, p: f64) -> bool {
    // The specification of the float-to-integer cast in Rust is such that
    // - rounding is toward zero,
    // - NaN produces zero, and
    // - out of range values saturate.
    //
    // Also, note that `n == u64::MAX` if and only if `p >= 1`, so we use that
    // as a sentinal value.
    //
    // On aarch64 we get a nice sequence:
    //
    //   fcvtzu x8, d0, #64
    //   ...
    //   cmn x8, #1
    //   ccmp x9, x8, #0, ne
    //   cset w0, lo
    //
    // On x86-64, the bounds checks against 0.0 and 2.0 ** 64 are performed
    // directly.
    let n = (p * f64::from_bits(0x43f0_0000_0000_0000)) as u64;
    let x = self.u64();
    x < n || n == u64::MAX
  }

  /// Samples a `bool` from the uniform distribution.
  #[inline(always)]
  pub const fn bool(&mut self) -> bool {
    self.i64() < 0
  }

  /// Samples a `i8` from the uniform distribution.
  #[inline(always)]
  pub const fn i8(&mut self) -> i8 {
    self.u64() as i8
  }

  /// Samples a `i16` from the uniform distribution.
  #[inline(always)]
  pub const fn i16(&mut self) -> i16 {
    self.u64() as i16
  }

  /// Samples a `i32` from the uniform distribution.
  #[inline(always)]
  pub const fn i32(&mut self) -> i32 {
    self.u64() as i32
  }

  /// Samples a `i64` from the uniform distribution.
  #[inline(always)]
  pub const fn i64(&mut self) -> i64 {
    self.u64() as i64
  }

  /// Samples a `i128` from the uniform distribution.
  #[inline(always)]
  pub const fn i128(&mut self) -> i128 {
    self.u128() as i128
  }

  /// Samples a `u8` from the uniform distribution.
  #[inline(always)]
  pub const fn u8(&mut self) -> u8 {
    self.u64() as u8
  }

  /// Samples a `u16` from the uniform distribution.
  #[inline(always)]
  pub const fn u16(&mut self) -> u16 {
    self.u64() as u16
  }

  /// Samples a `u32` from the uniform distribution.
  #[inline(always)]
  pub const fn u32(&mut self) -> u32 {
    self.u64() as u32
  }

  /// Samples a `u64` from the uniform distribution.
  #[inline(always)]
  pub const fn u64(&mut self) -> u64 {
    // This is the core generator.
    let s = self.state.get();
    let x = lo(s);
    let y = hi(s);
    let s = concat(y ^ asr(x, 4), x ^ lsl(y, 7));
    let s = unsafe { NonZeroU128::new_unchecked(s) };
    self.state = s;
    y.wrapping_add(x.wrapping_mul(x)) ^ mulhi(x, x)
  }

  /// Samples a `u128` from the uniform distribution.
  #[inline(always)]
  pub const fn u128(&mut self) -> u128 {
    let x = self.u64();
    let y = self.u64();
    concat(x, y)
  }

  /// Samples a `NonZeroU8` from the uniform distribution.
  #[inline(always)]
  pub const fn non_zero_u8(&mut self) -> NonZeroU8 {
    loop {
      if let Some(x) = NonZeroU8::new(self.u8()) {
        break x
      }
    }
  }

  /// Samples a `NonZeroU16` from the uniform distribution.
  #[inline(always)]
  pub const fn non_zero_u16(&mut self) -> NonZeroU16 {
    loop {
      if let Some(x) = NonZeroU16::new(self.u16()) {
        break x
      }
    }
  }

  /// Samples a `NonZeroU32` from the uniform distribution.
  #[inline(always)]
  pub const fn non_zero_u32(&mut self) -> NonZeroU32 {
    loop {
      if let Some(x) = NonZeroU32::new(self.u32()) {
        break x
      }
    }
  }

  /// Samples a `NonZeroU64` from the uniform distribution.
  #[inline(always)]
  pub const fn non_zero_u64(&mut self) -> NonZeroU64 {
    loop {
      if let Some(x) = NonZeroU64::new(self.u64()) {
        break x
      }
    }
  }

  /// Samples a `NonZeroU128` from the uniform distribution.
  #[inline(always)]
  pub const fn non_zero_u128(&mut self) -> NonZeroU128 {
    loop {
      if let Some(x) = NonZeroU128::new(self.u128()) {
        break x
      }
    }
  }

  /// Samples a `u8` from the uniform distribution over the range `0 ... n`.
  ///
  /// The upper bound is inclusive.
  #[inline(always)]
  pub const fn bounded_u8(&mut self, n: u8) -> u8 {
    let x = self.u64();
    let n = n as u64;
    let m = n + 1;
    let a = mulhi(x, m);
    a as u8
  }

  /// Samples a `u16` from the uniform distribution over the range `0 ... n`.
  ///
  /// The upper bound is inclusive.
  #[inline(always)]
  pub const fn bounded_u16(&mut self, n: u16) -> u16 {
    let x = self.u64();
    let n = n as u64;
    let m = n + 1;
    let a = mulhi(x, m);
    a as u16
  }

  /// Samples a `u32` from the uniform distribution over the range `0 ... n`.
  ///
  /// The upper bound is inclusive.
  #[inline(always)]
  pub const fn bounded_u32(&mut self, n: u32) -> u32 {
    // Cf. `bounded_u64`.
    let x = self.u64();
    let n = n as u64;
    let m = n + 1;
    let a = mulhi(x, m);
    let b = x.wrapping_mul(m);
    if b.overflowing_add(n).1 {
      let mut r = b;
      loop {
        let y = self.u64();
        let c = mulhi(y, m);
        let d = y.wrapping_mul(m);
        let v = r.overflowing_add(c);
        if v.0 != u64::MAX { break (a + v.1 as u64) as u32 }
        r = d;
        cold_path();
      }
    } else {
      a as u32
    }
  }

  /// Samples a `u64` from the uniform distribution over the range `0 ... n`.
  ///
  /// The upper bound is inclusive.
  #[inline(always)]
  pub const fn bounded_u64(&mut self, n: u64) -> u64 {
    let x = self.u64();
    let m = n.wrapping_add(1);
    let a = mulhi(x, m);
    let b = x.wrapping_mul(m);
    let u = if m == 0 { x } else { a };
    if b.overflowing_add(n).1 {
      debug_assert!(m != 0);
      let mut r = b;
      loop {
        let y = self.u64();
        let c = mulhi(y, m);
        let d = y.wrapping_mul(m);
        let v = r.overflowing_add(c);
        if v.0 != u64::MAX { break a + v.1 as u64 }
        // NB: We get here with negligible probability, but we include the loop
        // anyway to prevent the compiler from doing the pessimization of
        // if-converting the control flow away.
        r = d;
        cold_path();
      }
    } else {
      u
    }
  }

  /// Samples a `usize` from the uniform distribution over the range `0 ... n`.
  ///
  /// The upper bound is inclusive.
  #[inline(always)]
  pub const fn bounded_usize(&mut self, n: usize) -> usize {
    match const { usize::BITS } {
      32 => self.bounded_u32(n as u32) as usize,
      64 => self.bounded_u64(n as u64) as usize,
      _ => unimplemented!(),
    }
  }

  /// Samples a `i8` from the uniform distribution over the range `a ... b`.
  ///
  /// The lower and upper bounds are inclusive, and the range can wrap around
  /// from `i8::MAX` to `i8::MIN`.
  #[inline(always)]
  pub const fn range_i8(&mut self, a: i8, b: i8) -> i8 {
    self.range_u8(a as u8, b as u8) as i8
  }

  /// Samples a `i16` from the uniform distribution over the range `a ... b`.
  ///
  /// The lower and upper bounds are inclusive, and the range can wrap around
  /// from `i16::MAX` to `i16::MIN`.
  #[inline(always)]
  pub const fn range_i16(&mut self, a: i16, b: i16) -> i16 {
    self.range_u16(a as u16, b as u16) as i16
  }

  /// Samples a `i32` from the uniform distribution over the range `a ... b`.
  ///
  /// The lower and upper bounds are inclusive, and the range can wrap around
  /// from `i32::MAX` to `i32::MIN`.
  #[inline(always)]
  pub const fn range_i32(&mut self, a: i32, b: i32) -> i32 {
    self.range_u32(a as u32, b as u32) as i32
  }

  /// Samples a `i64` from the uniform distribution over the range `a ... b`.
  ///
  /// The lower and upper bounds are inclusive, and the range can wrap around
  /// from `i64::MAX` to `i64::MIN`.
  #[inline(always)]
  pub const fn range_i64(&mut self, a: i64, b: i64) -> i64 {
    self.range_u64(a as u64, b as u64) as i64
  }

  /// Samples a `isize` from the uniform distribution over the range `a ... b`.
  ///
  /// The lower and upper bounds are inclusive, and the range can wrap around
  /// from `isize::MAX` to `isize::MIN`.
  #[inline(always)]
  pub const fn range_isize(&mut self, a: isize, b: isize) -> isize {
    self.range_usize(a as usize, b as usize) as isize
  }

  /// Samples a `u8` from the uniform distribution over the range `a ... b`.
  ///
  /// The lower and upper bounds are inclusive, and the range can wrap around
  /// from `u8::MAX` to `u8::MIN`.
  #[inline(always)]
  pub const fn range_u8(&mut self, a: u8, b: u8) -> u8 {
    a.wrapping_add(self.bounded_u8(b.wrapping_sub(a)))
  }

  /// Samples a `u16` from the uniform distribution over the range `a ... b`.
  ///
  /// The lower and upper bounds are inclusive, and the range can wrap around
  /// from `u16::MAX` to `u16::MIN`.
  #[inline(always)]
  pub const fn range_u16(&mut self, a: u16, b: u16) -> u16 {
    a.wrapping_add(self.bounded_u16(b.wrapping_sub(a)))
  }

  /// Samples a `u32` from the uniform distribution over the range `a ... b`.
  ///
  /// The lower and upper bounds are inclusive, and the range can wrap around
  /// from `u32::MAX` to `u32::MIN`.
  #[inline(always)]
  pub const fn range_u32(&mut self, a: u32, b: u32) -> u32 {
    a.wrapping_add(self.bounded_u32(b.wrapping_sub(a)))
  }

  /// Samples a `u64` from the uniform distribution over the range `a ... b`.
  ///
  /// The lower and upper bounds are inclusive, and the range can wrap around
  /// from `u64::MAX` to `u64::MIN`.
  #[inline(always)]
  pub const fn range_u64(&mut self, a: u64, b: u64) -> u64 {
    a.wrapping_add(self.bounded_u64(b.wrapping_sub(a)))
  }

  /// Samples a `usize` from the uniform distribution over the range `a ... b`.
  ///
  /// The lower and upper bounds are inclusive, and the range can wrap around
  /// from `usize::MAX` to `usize::MIN`.
  #[inline(always)]
  pub const fn range_usize(&mut self, a: usize, b: usize) -> usize {
    a.wrapping_add(self.bounded_usize(b.wrapping_sub(a)))
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
  /// Every output, including zero, has a positive sign bit.
  #[inline(always)]
  pub const fn f32(&mut self) -> f32 {
    let x = self.i64();
    let x = f32::from_bits(0x2000_0000) * (x as f32);
    x.abs()
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
  /// Every output, including zero, has a positive sign bit.
  #[inline(always)]
  pub const fn f64(&mut self) -> f64 {
    // The conversion into a `f64` is two instructions on aarch64:
    //
    //   scvtf d0, x8, #63
    //   fabs d0, d0
    //
    let x = self.i64();
    let x = f64::from_bits(0x3c00_0000_0000_0000) * (x as f64);
    x.abs()
  }

  /// Samples a `f32` from a distribution that approximates the uniform
  /// distribution over the real interval [-1, 1].
  ///
  /// The distribution is the same as the one produced by the following
  /// procedure:
  ///
  /// - Sample a real number from the uniform distribution on [-1, 1].
  /// - Round to the nearest multiple of 2⁻⁶².
  /// - Round to a `f32` using the default rounding mode.
  #[inline(always)]
  pub const fn biunit_f32(&mut self) -> f32 {
    let x = self.i64();
    let x = (x & 1) + (x >> 1);
    f32::from_bits(0x2080_0000) * (x as f32)
  }

  /// Samples a `f64` from a distribution that approximates the uniform
  /// distribution over the real interval [-1, 1].
  ///
  /// The distribution is the same as the one produced by the following
  /// procedure:
  ///
  /// - Sample a real number from the uniform distribution on [-1, 1].
  /// - Round to the nearest multiple of 2⁻⁶².
  /// - Round to a `f64` using the default rounding mode.
  #[inline(always)]
  pub const fn biunit_f64(&mut self) -> f64 {
    // The conversion into a `f64` is three instructions on aarch64:
    //
    //   and x1, x0, #0x1
    //   add x2, x1, x0, asr #1
    //   scvtf d0, x2, #62
    //
    let x = self.i64();
    let x = (x & 1) + (x >> 1);
    f64::from_bits(0x3c10_0000_0000_0000) * (x as f64)
  }

  #[inline(always)]
  const unsafe fn fill_unchecked_inlined(&mut self, dst: *mut u8, len: usize) {
    let mut p = dst;
    let mut n = len;
    if n == 0 { return }
    while n >= 17 {
      let x = self.u64().to_le_bytes();
      let y = self.u64().to_le_bytes();
      unsafe { p.copy_from_nonoverlapping(&raw const x as _, 8) };
      p = unsafe { p.add(8) };
      n = n - 8;
      unsafe { p.copy_from_nonoverlapping(&raw const y as _, 8) };
      p = unsafe { p.add(8) };
      n = n - 8;
    }
    if n >= 9 {
      let x = self.u64().to_le_bytes();
      unsafe { p.copy_from_nonoverlapping(&raw const x as _, 8) };
      p = unsafe { p.add(8) };
      n = n - 8;
    }
    let x = self.u64().to_le_bytes();
    match n {
      1 => unsafe { p.copy_from_nonoverlapping(&raw const x as _, 1) },
      2 => unsafe { p.copy_from_nonoverlapping(&raw const x as _, 2) },
      3 => unsafe { p.copy_from_nonoverlapping(&raw const x as _, 3) },
      4 => unsafe { p.copy_from_nonoverlapping(&raw const x as _, 4) },
      5 => unsafe { p.copy_from_nonoverlapping(&raw const x as _, 5) },
      6 => unsafe { p.copy_from_nonoverlapping(&raw const x as _, 6) },
      7 => unsafe { p.copy_from_nonoverlapping(&raw const x as _, 7) },
      8 => unsafe { p.copy_from_nonoverlapping(&raw const x as _, 8) },
      _ => unreachable!(),
    }
  }

  /// Fills the provided buffer with independent uniformly distributed bytes.
  pub const fn fill(&mut self, dst: &mut [u8]) {
    let n = dst.len();
    unsafe { self.fill_unchecked(&raw mut *dst as _, n) };
  }

  /// Fills the provided buffer with independent uniformly distributed bytes.
  ///
  /// # Safety
  ///
  /// It must be valid to write `len` arbitrary bytes at `dst`.
  pub const unsafe fn fill_unchecked(&mut self, dst: *mut u8, len: usize) {
    unsafe { self.fill_unchecked_inlined(dst, len) };
  }

  /// Fills the provided buffer with independent uniformly distributed bytes,
  /// returning a reference to the initialized buffer. The returned buffer is a
  /// reference the same memory and has the same length as the input buffer.
  pub const fn fill_uninit<'a>(&mut self, dst: &'a mut [MaybeUninit<u8>]) -> &'a mut [u8] {
    let n = dst.len();
    unsafe { self.fill_unchecked(&raw mut *dst as _, n) };
    unsafe { dst.assume_init_mut() }
  }

  /// Samples an array of independent uniformly distributed bytes.
  pub const fn byte_array<const N: usize>(&mut self) -> [u8; N] {
    let mut buf = [0; N];
    unsafe { self.fill_unchecked_inlined(&raw mut buf as _, N) };
    buf
  }

  /// Shuffles a mutable slice in place with a random permutation.
  pub const fn shuffle<T>(&mut self, slice: &mut [T]) {
    let n = slice.len();
    if n >= 2 {
      let p = &raw mut *slice as *mut T;
      let mut i = 1;
      loop {
        let j = self.bounded_usize(i);
        unsafe { p.add(i).swap(p.add(j)) };
        i = i + 1;
        if i == n { break }
      }
    }
  }
}

impl Debug for Rng {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    f.debug_struct("Rng").finish_non_exhaustive()
  }
}

#[cfg(feature = "rand_core")]
impl rand_core::TryRng for Rng {
  type Error = rand_core::Infallible;

  #[inline(always)]
  fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
    Ok(self.u32())
  }

  #[inline(always)]
  fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
    Ok(self.u64())
  }

  fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Self::Error> {
    self.fill(dst);
    Ok(())
  }
}

#[cfg(feature = "rand_core")]
impl rand_core::SeedableRng for Rng {
  type Seed = [u8; 16];

  #[inline(always)]
  fn from_seed(seed: Self::Seed) -> Self {
    let s = 1 | u128::from_le_bytes(seed);
    let s = unsafe { NonZeroU128::new_unchecked(s) };
    Self::from_state(s)
  }

  fn seed_from_u64(seed: u64) -> Self {
    Self::from_u64(seed)
  }

  fn from_rng<T: rand_core::Rng + ?Sized>(g: &mut T) -> Self {
    let Ok(x) = Self::try_from_rng(g);
    x
  }

  fn try_from_rng<T: rand_core::TryRng + ?Sized>(g: &mut T) -> Result<Self, T::Error> {
    let x = g.try_next_u64()?;
    let y = g.try_next_u64()?;
    let s = 1 | concat(x, y);
    let s = unsafe { NonZeroU128::new_unchecked(s) };
    Ok(Self::from_state(s))
  }

  // We keep the default implementations of `fork` and `try_fork`.
}

#[cfg(feature = "thread_local")]
pub mod thread_local {
  //! Access a thread-local random number generator.

  use std::cell::Cell;
  use std::mem::MaybeUninit;
  use std::num::NonZeroU128;
  use std::num::NonZeroU32;
  use std::num::NonZeroU64;
  use std::thread_local;
  use super::Rng;

  thread_local! {
    static RNG: Cell<Option<NonZeroU128>> = const {
      Cell::new(None)
    };
  }

  // This function, while safe, is NOT logically re-entrant, so we should not
  // expose it publicly.
  #[inline(always)]
  fn with<T>(f: impl FnOnce(&mut Rng) -> T) -> T {
    RNG.with(|state| {
      let mut g =
        match state.get() {
          None => Rng::from_operating_system(),
          Some(s) => Rng::from_state(s),
        };
      let x = f(&mut g);
      state.set(Some(g.state()));
      x
    })
  }

  /// See [Rng::bernoulli].
  pub fn bernoulli(p: f64) -> bool {
    with(|g| g.bernoulli(p))
  }

  /// See [Rng::bool].
  pub fn bool() -> bool {
    with(|g| g.bool())
  }

  /// See [Rng::i32].
  pub fn i32() -> i32 {
    with(|g| g.i32())
  }

  /// See [Rng::i64].
  pub fn i64() -> i64 {
    with(|g| g.i64())
  }

  /// See [Rng::i128].
  pub fn i128() -> i128 {
    with(|g| g.i128())
  }

  /// See [Rng::u32].
  pub fn u32() -> u32 {
    with(|g| g.u32())
  }

  /// See [Rng::u64].
  pub fn u64() -> u64 {
    with(|g| g.u64())
  }

  /// See [Rng::u128].
  pub fn u128() -> u128 {
    with(|g| g.u128())
  }

  /// See [Rng::non_zero_u32].
  pub fn non_zero_u32() -> NonZeroU32 {
    with(|g| g.non_zero_u32())
  }

  /// See [Rng::non_zero_u64].
  pub fn non_zero_u64() -> NonZeroU64 {
    with(|g| g.non_zero_u64())
  }

  /// See [Rng::non_zero_u128].
  pub fn non_zero_u128() -> NonZeroU128 {
    with(|g| g.non_zero_u128())
  }
  /// See [Rng::bounded_u32].
  pub fn bounded_u32(n: u32) -> u32 {
    with(|g| g.bounded_u32(n))
  }

  /// See [Rng::bounded_u64].
  pub fn bounded_u64(n: u64) -> u64 {
    with(|g| g.bounded_u64(n))
  }

  /// See [Rng::bounded_usize].
  pub fn bounded_usize(n: usize) -> usize {
    with(|g| g.bounded_usize(n))
  }

  /// See [Rng::range_i32].
  pub fn range_i32(a: i32, b: i32) -> i32 {
    with(|g| g.range_i32(a, b))
  }

  /// See [Rng::range_i64].
  pub fn range_i64(a: i64, b: i64) -> i64 {
    with(|g| g.range_i64(a, b))
  }

  /// See [Rng::range_isize].
  pub fn range_isize(a: isize, b: isize) -> isize {
    with(|g| g.range_isize(a, b))
  }

  /// See [Rng::range_u32].
  pub fn range_u32(a: u32, b: u32) -> u32 {
    with(|g| g.range_u32(a, b))
  }

  /// See [Rng::range_u64].
  pub fn range_u64(a: u64, b: u64) -> u64 {
    with(|g| g.range_u64(a, b))
  }

  /// See [Rng::range_usize].
  pub fn range_usize(a: usize, b: usize) -> usize {
    with(|g| g.range_usize(a, b))
  }

  /// See [Rng::f32].
  pub fn f32() -> f32 {
    with(|g| g.f32())
  }

  /// See [Rng::f64].
  pub fn f64() -> f64 {
    with(|g| g.f64())
  }

  /// See [Rng::biunit_f32].
  pub fn biunit_f32() -> f32 {
    with(|g| g.biunit_f32())
  }

  /// See [Rng::biunit_f64].
  pub fn biunit_f64() -> f64 {
    with(|g| g.biunit_f64())
  }

  /// See [Rng::fill].
  pub fn fill(dst: &mut [u8]) {
    with(|g| g.fill(dst))
  }

  /// See [Rng::fill_unchecked].
  pub unsafe fn fill_unchecked(dst: *mut u8, len: usize) {
    with(|g| unsafe { g.fill_unchecked(dst, len) })
  }

  /// See [Rng::fill_uninit].
  pub fn fill_uninit(dst: &mut [MaybeUninit<u8>]) -> &mut [u8] {
    with(|g| g.fill_uninit(dst))
  }

  /// See [Rng::byte_array].
  pub fn byte_array<const N: usize>() -> [u8; N] {
    with(|g| g.byte_array())
  }
}
