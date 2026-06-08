#![doc = include_str!("../README.md")]
#![no_std]

// TODO: statement about total variation distances

#[cfg(feature = "std")]
extern crate std;

mod int;

use core::fmt::Debug;
use core::fmt::Formatter;
use core::fmt;
use core::hint::cold_path;
use core::hint::select_unpredictable;
use core::mem::transmute;
use core::num::NonZeroI128;
use core::num::NonZeroI16;
use core::num::NonZeroI32;
use core::num::NonZeroI64;
use core::num::NonZeroI8;
use core::num::NonZeroIsize;
use core::num::NonZeroU128;
use core::num::NonZeroU16;
use core::num::NonZeroU32;
use core::num::NonZeroU64;
use core::num::NonZeroU8;
use core::num::NonZeroUsize;
use crate::int::catenate;
use crate::int::lower;
use crate::int::upper;
use crate::int::widening_mul;

/// A high performance non-cryptographic random number generator.
#[derive(Clone, Eq, PartialEq)]
pub struct Rng { state: NonZeroU128 }

/// A sealed trait for sampling from the uniform distribution over all possible
/// values of the type.
///
/// See [`Rng::uniform`] and [`Rng::fill`].
#[allow(private_bounds)]
pub trait RandomUniform: private::RandomUniform {
}

/// A sealed trait for sampling from the uniform distribution over a range from
/// zero to an inclusive upper bound.
///
/// See [`Rng::bounded`].
#[allow(private_bounds)]
pub trait RandomBounded: private::RandomBounded {
}

/// A sealed trait for sampling from the uniform distribution over a range
/// between inclusive lower and upper bounds. The range is permitted to wrap
/// around from the maximum to the minimum value of the type.
///
/// See [`Rng::between`].
#[allow(private_bounds)]
pub trait RandomBetween: private::RandomBetween {
}

/// A sealed trait for sampling floating point numbers from various
/// distributions.
///
/// See [`Rng::float`] and [`Rng::float_biunit`].
#[allow(private_bounds)]
pub trait RandomFloat: private::RandomFloat {
}

// NOTE: We do not include blanket implementations like
//
//   impl<T: private::RandomUniform> RandomUniform for T { }
//
// and instead do each marker impl by itself so that the documentation for the
// trait has a full list of impls.

// Our hash for initializing the generator state uses the multiplier
//
//   HASH_MULT = round_nearest_odd(EULER_MASCHERONI * 2¹²⁸)
//
// The Euler-Mascheroni constant was selected because it is a well-known
// number in the range (0.5, 1.0).
const HASH_MULT: u128 = 0x93c4_67e3_7db0_c7a4_d1be_3f81_0152_cb57;

impl Rng {
  /// Creates a random number generator with an initial state derived by
  /// hashing the given seed.
  pub const fn new(seed: NonZeroU128) -> Self {
    let x = seed.get();
    let x = x.wrapping_mul(HASH_MULT);
    let x = x.swap_bytes();
    let x = x.wrapping_mul(HASH_MULT);
    let x = x.swap_bytes();
    let x = x.wrapping_mul(HASH_MULT);
    Self { state: unsafe { NonZeroU128::new_unchecked(x) } }
  }

  /// Creates a random number generator with an initial state derived by
  /// hashing the given `u64` seed.
  pub const fn from_u64(seed: u64) -> Self {
    let s = (1u128 << 64) | (seed as u128);
    Self::new(unsafe { NonZeroU128::new_unchecked(s) })
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
  ///
  /// # Panics
  ///
  /// Panics if `getrandom` fails to retrieve random bytes from the operating
  /// system.
  #[cfg(feature = "getrandom")]
  #[inline(never)]
  pub fn from_operating_system() -> Self {
    let mut buf = [0; 16];
    getrandom::fill(&mut buf).expect("getrandom::fill failed!");
    let s = 1 | u128::from_le_bytes(buf);
    Self { state: unsafe { NonZeroU128::new_unchecked(s) } }
  }

  /// Generates the next random number. This is the core operation of the
  /// random number generator from which other sampling routines are derived.
  #[inline(always)]
  pub fn next(&mut self) -> u64 {
    let s = self.state.get();
    let x = lower(s);
    let y = upper(s);
    let u = y ^ (x << 7);
    let v = x ^ (y.cast_signed() >> 4).cast_unsigned();
    let s = catenate(u, v);
    self.state = unsafe { NonZeroU128::new_unchecked(s) };
    let z = widening_mul(x, x);
    y.wrapping_add(lower(z)) ^ upper(z)
  }

  /// Samples a `T` from the uniform distribution over all possible values of
  /// type `T`.
  #[inline]
  pub fn uniform<T: RandomUniform>(&mut self) -> T {
    T::random_uniform(self)
  }

  /// Fills a slice with `T`s sampled from the uniform distribution over all
  /// possible values of type `T`.
  ///
  /// Some types, e.g. `u16`, are generated more efficiently in bulk than one
  /// at a time.
  #[inline]
  pub fn fill<T: RandomUniform>(&mut self, buf: &mut [T]) {
    T::random_uniform_fill(self, buf)
  }

  /// Samples a `T` from the uniform distribution over a range from zero
  /// to an inclusive upper bound `n`.
  #[inline]
  pub fn bounded<T: RandomBounded>(&mut self, n: T) -> T {
    T::random_bounded(self, n)
  }

  /// Samples a `T` from the uniform distribution over a range from an
  /// inclusive lower bound `a` to an inclusive upper bound `b`. The range is
  /// permitted to wrap around from `T::MAX` to `T::MIN`.
  #[inline]
  pub fn between<T: RandomBetween>(&mut self, a: T, b: T) -> T {
    T::random_between(self, a, b)
  }

  /// Samples a floating-point number of type `T` from a distribution that
  /// approximates the uniform distribution over the real interval [0, 1].
  ///
  /// The distribution is the same as the one produced by the following
  /// procedure:
  ///
  /// - Sample a real number from the uniform distribution on [0, 1].
  /// - Round to the nearest multiple of 2⁻⁶³.
  /// - Round to a `T` using the default rounding mode.
  ///
  /// Every output, including zero, has a positive sign bit.
  #[inline]
  pub fn float<T: RandomFloat>(&mut self) -> T {
    T::random_float(self)
  }

  /// Samples a floating-point number of type `T` from a distribution that
  /// approximates the uniform distribution over the real interval [-1, 1].
  ///
  /// The distribution is the same as the one produced by the following
  /// procedure:
  ///
  /// - Sample a real number from the uniform distribution on [-1, 1].
  /// - Round to the nearest multiple of 2⁻⁶².
  /// - Round to a `T` using the default rounding mode.
  #[inline]
  pub fn float_biunit<T: RandomFloat>(&mut self) -> T {
    T::random_float_biunit(self)
  }

  /// Samples a `bool` from the Bernoulli distribution where `true` appears
  /// with probability approximately equal to `p`.
  ///
  /// A probability `p` <= 0 or NaN is treated as 0, and a probability `p` >= 1
  /// is treated as 1.
  #[inline(always)]
  pub fn bernoulli(&mut self, p: f64) -> bool {
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
    let x = self.next();
    x < n || n == u64::MAX
  }

  /// Shuffles a mutable slice in place with a random permutation.
  pub fn shuffle<T>(&mut self, buf: &mut [T]) {
    let p = buf.as_mut_ptr();
    let n = buf.len();
    for i in 1 .. n {
      let j = self.bounded(i);
      unsafe { p.add(i).swap(p.add(j)) };
    }
  }

  #[inline(always)]
  fn next_8p(&mut self) -> [bool; 8] {
    let x = self.next();
    let x = x & 0x01010101_01010101;
    let x = x.to_le_bytes().map(|b| b != 0);
    x
  }

  #[inline(always)]
  fn next_8b(&mut self) -> [u8; 8] {
    u64::to_le_bytes(self.next())
  }

  #[inline(always)]
  fn next_4h(&mut self) -> [u16; 4] {
    let x = self.next();
    [x as u16, (x >> 16) as u16, (x >> 32) as u16, (x >> 48) as u16]
  }

  #[inline(always)]
  fn next_2w(&mut self) -> [u32; 2] {
    let x = self.next();
    [lower(x), upper(x)]
  }

  #[inline(always)]
  unsafe fn fill_unchecked__<const N: usize, T, F>(&mut self, buf: *mut T, len: usize, f: F)
  where
    T: Copy,
    F: Fn(&mut Self) -> [T; N]
  {
    const { assert!(N <= 8) };
    let mut p = buf;
    let mut i = len;
    if i == 0 { return }
    while i > N {
      let x = f(self);
      unsafe { p.copy_from_nonoverlapping(x.as_ptr(), N) };
      p = unsafe { p.add(N) };
      i = i - N;
    }
    let x = f(self);
    match i {
      1 => unsafe { p.copy_from_nonoverlapping(x.as_ptr(), 1) },
      2 => unsafe { p.copy_from_nonoverlapping(x.as_ptr(), 2) },
      3 => unsafe { p.copy_from_nonoverlapping(x.as_ptr(), 3) },
      4 => unsafe { p.copy_from_nonoverlapping(x.as_ptr(), 4) },
      5 => unsafe { p.copy_from_nonoverlapping(x.as_ptr(), 5) },
      6 => unsafe { p.copy_from_nonoverlapping(x.as_ptr(), 6) },
      7 => unsafe { p.copy_from_nonoverlapping(x.as_ptr(), 7) },
      8 => unsafe { p.copy_from_nonoverlapping(x.as_ptr(), 8) },
      _ => unreachable!()
    }
  }

  #[inline(always)]
  unsafe fn fill_unchecked_b(&mut self, buf: *mut u8, len: usize) {
    // the byte version is unrolled more than the others
    const N: usize = 8;
    let mut p = buf;
    let mut i = len;
    if i == 0 { return }
    while i > 2 * N {
      let x = self.next_8b();
      let y = self.next_8b();
      unsafe { p.copy_from_nonoverlapping(x.as_ptr(), N) };
      p = unsafe { p.add(N) };
      i = i - N;
      unsafe { p.copy_from_nonoverlapping(y.as_ptr(), N) };
      p = unsafe { p.add(N) };
      i = i - N;
    }
    if i > N {
      let x = self.next_8b();
      unsafe { p.copy_from_nonoverlapping(x.as_ptr(), N) };
      p = unsafe { p.add(N) };
      i = i - N;
    }
    let x = self.next_8b();
    match i {
      1 => unsafe { p.copy_from_nonoverlapping(x.as_ptr(), 1) },
      2 => unsafe { p.copy_from_nonoverlapping(x.as_ptr(), 2) },
      3 => unsafe { p.copy_from_nonoverlapping(x.as_ptr(), 3) },
      4 => unsafe { p.copy_from_nonoverlapping(x.as_ptr(), 4) },
      5 => unsafe { p.copy_from_nonoverlapping(x.as_ptr(), 5) },
      6 => unsafe { p.copy_from_nonoverlapping(x.as_ptr(), 6) },
      7 => unsafe { p.copy_from_nonoverlapping(x.as_ptr(), 7) },
      8 => unsafe { p.copy_from_nonoverlapping(x.as_ptr(), 8) },
      _ => unreachable!()
    }
  }
}

impl RandomUniform for bool {
}

impl private::RandomUniform for bool {
  #[inline(always)]
  fn random_uniform(g: &mut Rng) -> Self {
    g.next().cast_signed() < 0
  }

  #[inline(always)]
  fn random_uniform_array<const N: usize>(g: &mut Rng) -> [Self; N] {
    let mut buf = [false; N];
    unsafe { g.fill_unchecked__(buf.as_mut_ptr(), buf.len(), Rng::next_8p) };
    buf
  }

  fn random_uniform_fill(g: &mut Rng, buf: &mut [Self]) {
    unsafe { g.fill_unchecked__(buf.as_mut_ptr(), buf.len(), Rng::next_8p) };
  }
}

impl RandomUniform for usize {
}

impl private::RandomUniform for usize {
  #[inline(always)]
  fn random_uniform(g: &mut Rng) -> Self {
    cfg_select! {
      target_pointer_width = "16" => { u16::random_uniform(g) as _ }
      target_pointer_width = "32" => { u32::random_uniform(g) as _ }
      target_pointer_width = "64" => { u64::random_uniform(g) as _ }
      _ => { unimplemented!() }
    }
  }
}

impl RandomUniform for u8 {
}

impl private::RandomUniform for u8 {
  #[inline(always)]
  fn random_uniform(g: &mut Rng) -> Self {
    g.next() as _
  }

  #[inline(always)]
  fn random_uniform_array<const N: usize>(g: &mut Rng) -> [Self; N] {
    let mut buf = [0; N];
    unsafe { g.fill_unchecked_b(buf.as_mut_ptr(), buf.len()) };
    buf
  }

  fn random_uniform_fill(g: &mut Rng, buf: &mut [Self]) {
    unsafe { g.fill_unchecked_b(buf.as_mut_ptr(), buf.len()) };
  }
}

impl RandomUniform for u16 {
}

impl private::RandomUniform for u16 {
  #[inline(always)]
  fn random_uniform(g: &mut Rng) -> Self {
    g.next() as _
  }

  #[inline(always)]
  fn random_uniform_array<const N: usize>(g: &mut Rng) -> [Self; N] {
    let mut buf = [0; N];
    unsafe { g.fill_unchecked__(buf.as_mut_ptr(), buf.len(), Rng::next_4h) };
    buf
  }

  fn random_uniform_fill(g: &mut Rng, buf: &mut [Self]) {
    unsafe { g.fill_unchecked__(buf.as_mut_ptr(), buf.len(), Rng::next_4h) };
  }
}

impl RandomUniform for u32 {
}

impl private::RandomUniform for u32 {
  #[inline(always)]
  fn random_uniform(g: &mut Rng) -> Self {
    g.next() as _
  }

  #[inline(always)]
  fn random_uniform_array<const N: usize>(g: &mut Rng) -> [Self; N] {
    let mut buf = [0; N];
    unsafe { g.fill_unchecked__(buf.as_mut_ptr(), buf.len(), Rng::next_2w) };
    buf
  }

  fn random_uniform_fill(g: &mut Rng, buf: &mut [Self]) {
    unsafe { g.fill_unchecked__(buf.as_mut_ptr(), buf.len(), Rng::next_2w) };
  }
}

impl RandomUniform for u64 {
}

impl private::RandomUniform for u64 {
  #[inline(always)]
  fn random_uniform(g: &mut Rng) -> Self {
    g.next()
  }
}

impl RandomUniform for u128 {
}

impl private::RandomUniform for u128 {
  #[inline(always)]
  fn random_uniform(g: &mut Rng) -> Self {
    catenate(g.next(), g.next())
  }
}

macro_rules! int_uniform_impls {
  ($($sint:ty, $uint:ty, $nzsint:ty, $nzuint:ty;)*) => {
    $(
    impl RandomUniform for $sint {
    }

    impl private::RandomUniform for $sint {
      #[inline(always)]
      fn random_uniform(g: &mut Rng) -> Self {
        <$uint>::random_uniform(g).cast_signed()
      }

      #[inline(always)]
      fn random_uniform_array<const N: usize>(g: &mut Rng) -> [Self; N] {
        <$uint>::random_uniform_array(g).map(<$uint>::cast_signed)
      }

      #[inline]
      fn random_uniform_fill(g: &mut Rng, buf: &mut [Self]) {
        // SAFETY:
        // - iX and uX have compatible representations
        // - Rust does not do TBAA
        // - int_uniform_impls! is not exposed, and is used correctly at its
        //   only usage site
        <$uint>::random_uniform_fill(g, unsafe { transmute::<_, &'_ mut [$uint]>(buf) });
      }
    }

    impl RandomUniform for $nzsint {
    }

    impl private::RandomUniform for $nzsint {
      #[inline(always)]
      fn random_uniform(g: &mut Rng) -> Self {
        loop {
          if let Some(x) = Self::new(<$sint>::random_uniform(g)) {
            break x
          }
        }
      }
    }

    impl RandomUniform for $nzuint {
    }

    impl private::RandomUniform for $nzuint {
      #[inline(always)]
      fn random_uniform(g: &mut Rng) -> Self {
        loop {
          if let Some(x) = Self::new(<$uint>::random_uniform(g)) {
            break x
          }
        }
      }
    }
    )*
  };
}

int_uniform_impls! {
  isize, usize, NonZeroIsize, NonZeroUsize;
  i8, u8, NonZeroI8, NonZeroU8;
  i16, u16, NonZeroI16, NonZeroU16;
  i32, u32, NonZeroI32, NonZeroU32;
  i64, u64, NonZeroI64, NonZeroU64;
  i128, u128, NonZeroI128, NonZeroU128;
}

impl<const N: usize, T: RandomUniform> RandomUniform for [T; N] {
}

impl<const N: usize, T: RandomUniform> private::RandomUniform for [T; N] {
  #[inline]
  fn random_uniform(g: &mut Rng) -> Self {
    T::random_uniform_array(g)
  }
}

impl RandomBounded for usize {
}

impl private::RandomBounded for usize {
  #[inline(always)]
  fn random_bounded(g: &mut Rng, n: Self) -> Self {
    cfg_select! {
      target_pointer_width = "16" => { u16::random_bounded(g, n as _) as _ }
      target_pointer_width = "32" => { u32::random_bounded(g, n as _) as _ }
      target_pointer_width = "64" => { u64::random_bounded(g, n as _) as _ }
      _ => { unimplemented!() }
    }
  }
}

impl RandomBounded for u8 {
}

impl private::RandomBounded for u8 {
  #[inline(always)]
  fn random_bounded(g: &mut Rng, n: Self) -> Self {
    upper(widening_mul(g.next(), n as u64 + 1)) as _
  }
}

impl RandomBounded for u16 {
}

impl private::RandomBounded for u16 {
  #[inline(always)]
  fn random_bounded(g: &mut Rng, n: Self) -> Self {
    upper(widening_mul(g.next(), n as u64 + 1)) as _
  }
}

impl RandomBounded for u32 {
}

impl private::RandomBounded for u32 {
  #[inline(always)]
  fn random_bounded(g: &mut Rng, n: Self) -> Self {
    let mut h = g.clone();
    let x = h.next();
    let n = n as u64;
    let m = n + 1;
    let u = widening_mul(x, m);
    let mut a = upper(u);
    let mut b = lower(u);
    if b.overflowing_add(n).1 {
      loop {
        let x = h.next();
        let u = widening_mul(x, m);
        let c = b.overflowing_add(upper(u));
        a = a + c.1 as u64;
        b = lower(u);
        if c.0 != u64::MAX { break }
        cold_path();
      }
    }
    *g = h;
    a as u32
  }
}

impl RandomBounded for u64 {
}

impl private::RandomBounded for u64 {
  #[inline(always)]
  fn random_bounded(g: &mut Rng, n: Self) -> Self {
    // A modified version of Canon's unbiased method.
    let mut h = g.clone();
    let x = h.next();
    let m = n.wrapping_add(1);
    let u = widening_mul(x, m);
    let mut a = upper(u);
    let mut b = lower(u);
    let v = select_unpredictable(m == 0, x, a);
    if b.overflowing_add(n).1 {
      loop {
        let x = h.next();
        let u = widening_mul(x, m);
        let c = b.overflowing_add(upper(u));
        a = a + c.1 as u64;
        b = lower(u);
        if c.0 != u64::MAX { break }
        cold_path();
        // NOTE: We get here with negligible probability and don't claim that
        // looping increases the fidelity of our sampled distribution. However,
        // the inclusion of the loop inhibits the potential pessimization of
        // control flow getting if-converted away
      }
    } else {
      a = v;
    }
    *g = h;
    a
  }
}

macro_rules! int_between_impls {
  ($($sint:ty, $uint:ty;)*) => {
    $(
    impl RandomBetween for $sint {
    }

    impl private::RandomBetween for $sint {
      #[inline(always)]
      fn random_between(g: &mut Rng, a: Self, b: Self) -> Self {
        <$uint>::random_between(g, a as $uint, b as $uint).cast_signed()
      }
    }

    impl RandomBetween for $uint {
    }

    impl private::RandomBetween for $uint {
      #[inline(always)]
      fn random_between(g: &mut Rng, a: Self, b: Self) -> Self {
        <$uint as private::RandomBounded>::random_bounded(g, b.wrapping_sub(a)).wrapping_add(a)
      }
    }
    )*
  };
}

int_between_impls! {
  isize, usize;
  i8, u8;
  i16, u16;
  i32, u32;
  i64, u64;
}

impl RandomFloat for f32 {
}

impl private::RandomFloat for f32 {
  #[inline(always)]
  fn random_float(g: &mut Rng) -> Self {
    let x = g.next().cast_signed();
    let x = f32::from_bits(0x2000_0000) * (x as f32);
    x.abs()
  }

  #[inline(always)]
  fn random_float_biunit(g: &mut Rng) -> Self {
    let x = g.next().cast_signed();
    let x = (x & 1) + (x >> 1);
    f32::from_bits(0x2080_0000) * (x as f32)
  }
}

impl RandomFloat for f64 {
}

impl private::RandomFloat for f64 {
  #[inline(always)]
  fn random_float(g: &mut Rng) -> Self {
    // The conversion into a `f64` is two instructions on aarch64:
    //
    //   scvtf d0, x8, #63
    //   fabs d0, d0
    //
    let x = g.next().cast_signed();
    let x = f64::from_bits(0x3c00_0000_0000_0000) * (x as f64);
    x.abs()
  }

  #[inline(always)]
  fn random_float_biunit(g: &mut Rng) -> Self {
    // The conversion into a `f64` is three instructions on aarch64:
    //
    //   and x1, x0, #0x1
    //   add x2, x1, x0, asr #1
    //   scvtf d0, x2, #62
    //
    let x = g.next().cast_signed();
    let x = (x & 1) + (x >> 1);
    f64::from_bits(0x3c10_0000_0000_0000) * (x as f64)
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
    Ok(self.next() as u32)
  }

  #[inline(always)]
  fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
    Ok(self.next())
  }

  #[inline]
  fn try_fill_bytes(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
    self.fill(buf);
    Ok(())
  }
}

#[cfg(feature = "rand_core")]
impl rand_core::SeedableRng for Rng {
  type Seed = [u8; 16];

  #[inline(always)]
  fn from_seed(seed: Self::Seed) -> Self {
    let s = 1 | u128::from_le_bytes(seed);
    Self::from_state(unsafe { NonZeroU128::new_unchecked(s) })
  }

  #[inline]
  fn seed_from_u64(seed: u64) -> Self {
    Self::from_u64(seed)
  }

  #[inline]
  fn from_rng<T: rand_core::Rng + ?Sized>(g: &mut T) -> Self {
    let Ok(x) = Self::try_from_rng(g);
    x
  }

  fn try_from_rng<T: rand_core::TryRng + ?Sized>(g: &mut T) -> Result<Self, T::Error> {
    let x = g.try_next_u64()?;
    let y = g.try_next_u64()?;
    let s = 1 | catenate(x, y);
    Ok(Self::from_state(unsafe { NonZeroU128::new_unchecked(s) }))
  }

  // We keep the default implementations of `fork` and `try_fork`.
}

#[cfg(feature = "thread_local")]
pub mod thread_local {
  //! Access a thread-local random number generator.

  use std::cell::Cell;
  use std::num::NonZeroU128;
  use std::thread_local;
  use super::RandomBetween;
  use super::RandomBounded;
  use super::RandomFloat;
  use super::RandomUniform;
  use super::Rng;

  thread_local! {
    static RNG: (Cell<Option<NonZeroU128>>, Cell<bool>) = const {
      (Cell::new(None), Cell::new(false))
    };
  }

  /// Provides temporary access to a thread-local random number generator
  /// instance.
  ///
  /// This function is *not* re-entrant!
  ///
  /// # Panics
  ///
  /// A call to `with` panics if
  /// - initializing the thread-local state with `getrandom` fails,
  /// - during the dynamic extent of a call to `with` a re-entrant access is
  ///   attempted, or
  /// - a previous call to `with` panicked.
  #[inline(always)]
  pub fn with<T>(f: impl FnOnce(&mut Rng) -> T) -> T {
    #[inline(never)]
    #[cold]
    fn init_thread_local(is_init: &Cell<bool>) -> Rng {
      assert!(! is_init.get());
      is_init.set(true);
      Rng::from_operating_system()
    }
    RNG.with(|&(ref state, ref is_init)| {
      let mut g =
        if let Some(s) = state.get() {
          Rng::from_state(s)
        } else {
          init_thread_local(is_init)
        };
      state.set(None); // This write is often elided.
      let x = f(&mut g);
      state.set(Some(g.state()));
      x
    })
  }

  /// See [Rng::uniform].
  #[inline]
  pub fn uniform<T: RandomUniform>() -> T {
    with(|g| g.uniform())
  }

  /// See [Rng::fill].
  #[inline]
  pub fn fill<T: RandomUniform>(buf: &mut [T]) {
    with(|g| g.fill(buf))
  }

  /// See [Rng::bounded].
  #[inline]
  pub fn bounded<T: RandomBounded>(n: T) -> T {
    with(|g| g.bounded(n))
  }

  /// See [Rng::between].
  #[inline]
  pub fn between<T: RandomBetween>(a: T, b: T) -> T {
    with(|g| g.between(a, b))
  }

  /// See [Rng::float].
  #[inline]
  pub fn float<T: RandomFloat>() -> T {
    with(|g| g.float_biunit())
  }

  /// See [Rng::float_biunit].
  #[inline]
  pub fn float_biunit<T: RandomFloat>() -> T {
    with(|g| g.float_biunit())
  }

  /// See [Rng::bernoulli].
  #[inline]
  pub fn bernoulli(p: f64) -> bool {
    with(|g| g.bernoulli(p))
  }

  /// See [Rng::shuffle].
  #[inline]
  pub fn shuffle<T>(buf: &mut [T]) {
    with(|g| g.shuffle(buf))
  }
}

mod private {
  use core::array;
  use super::Rng;

  pub(crate) trait RandomUniform: Sized {
    fn random_uniform(_: &mut Rng) -> Self;

    #[inline]
    fn random_uniform_array<const N: usize>(g: &mut Rng) -> [Self; N] {
      array::from_fn(|_| Self::random_uniform(g))
    }

    fn random_uniform_fill(g: &mut Rng, buf: &mut [Self]) {
      buf.iter_mut().for_each(|a| *a = Self::random_uniform(g));
    }
  }

  pub(crate) trait RandomBounded {
    fn random_bounded(_: &mut Rng, _: Self) -> Self;
  }

  pub(crate) trait RandomBetween {
    fn random_between(_: &mut Rng, _: Self, _: Self) -> Self;
  }

  pub(crate) trait RandomFloat {
    fn random_float(_: &mut Rng) -> Self;

    fn random_float_biunit(_: &mut Rng) -> Self;
  }
}

pub mod miniature {
  //! unstable experimental smaller-state versions of the random number
  //! generator

  #![allow(missing_docs)]

  use core::num::NonZeroU32;
  use core::num::NonZeroU64;
  use crate::int::catenate;
  use crate::int::lower;
  use crate::int::upper;
  use crate::int::widening_mul;
  use super::HASH_MULT;

  #[derive(Clone, Eq, PartialEq)]
  pub struct Rng16x2 { state: NonZeroU32 }

  impl Rng16x2 {
    pub const fn new(seed: NonZeroU32) -> Self {
      const M: u32 = 1 | ((HASH_MULT >> 96) as u32);
      let x = seed.get();
      let x = x.wrapping_mul(M);
      let x = x.swap_bytes();
      let x = x.wrapping_mul(M);
      let x = x.swap_bytes();
      let x = x.wrapping_mul(M);
      Self { state: unsafe { NonZeroU32::new_unchecked(x) } }
    }

    #[inline(always)]
    pub fn next(&mut self) -> u16 {
      let s = self.state.get();
      let x = lower(s);
      let y = upper(s);
      let u = y ^ (x << 5);
      let v = x ^ (y.cast_signed() >> 3).cast_unsigned();
      let s = catenate(u, v);
      self.state = unsafe { NonZeroU32::new_unchecked(s) };
      let z = widening_mul(x, x);
      y.wrapping_add(lower(z)) ^ upper(z)
    }
  }

  #[derive(Clone, Eq, PartialEq)]
  pub struct Rng32x2 { state: NonZeroU64 }

  impl Rng32x2 {
    pub const fn new(seed: NonZeroU64) -> Self {
      const M: u64 = 1 | ((HASH_MULT >> 64) as u64);
      let x = seed.get();
      let x = x.wrapping_mul(M);
      let x = x.swap_bytes();
      let x = x.wrapping_mul(M);
      let x = x.swap_bytes();
      let x = x.wrapping_mul(M);
      Self { state: unsafe { NonZeroU64::new_unchecked(x) } }
    }

    #[inline(always)]
    pub fn next(&mut self) -> u32 {
      let s = self.state.get();
      let x = lower(s);
      let y = upper(s);
      let u = y ^ (x << 10);
      let v = x ^ (y.cast_signed() >> 7).cast_unsigned();
      let s = catenate(u, v);
      self.state = unsafe { NonZeroU64::new_unchecked(s) };
      let z = widening_mul(x, x);
      y.wrapping_add(lower(z)) ^ upper(z)
    }
  }
}
