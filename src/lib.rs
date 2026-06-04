#![doc = include_str!("../README.md")]
#![no_std]

#[cfg(feature = "std")]
extern crate std;

use core::fmt::Debug;
use core::fmt::Formatter;
use core::fmt;
use core::hint::cold_path;
use core::hint::select_unpredictable;
use core::mem::MaybeUninit;
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
use core::ops::RangeFull;
use core::range::RangeInclusive;
use core::range::RangeToInclusive;

/// A high performance non-cryptographic random number generator.
#[derive(Clone)]
pub struct Rng { state: NonZeroU128 }

/// ???
#[allow(private_bounds)]
pub trait RandomUniform: private::RandomUniform {
}

/// ???
#[allow(private_bounds)]
pub trait RandomBounded: private::RandomBounded {
}

/// ???
#[allow(private_bounds)]
pub trait RandomBetween: private::RandomBetween {
}

/// ???
#[allow(private_bounds)]
pub trait RandomFloat: private::RandomFloat {
}

/// ???
#[allow(private_bounds)]
pub trait Distribution<T>: private::Distribution<T> {
}

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

  /// ???
  #[inline(always)]
  pub const fn next(&mut self) -> u64 {
    let s = self.state.get();
    let x = lo(s);
    let y = hi(s);
    let s = concat(y ^ asr(x, 4), x ^ lsl(y, 7));
    let s = unsafe { NonZeroU128::new_unchecked(s) };
    self.state = s;
    y.wrapping_add(x.wrapping_mul(x)) ^ mulhi(x, x)
  }

  /// Samples a `T` from the uniform distribution over all possible values of
  /// type `T`.
  #[inline(always)]
  pub fn uniform<T: RandomUniform>(&mut self) -> T {
    T::random_uniform(self)
  }

  // TODO: statement about total variation distance from the ideal distribution

  /// Samples a `T` from the uniform distribution over `0 ..= n`, i.e. the
  /// inclusive range bounded above by `n`.
  #[inline(always)]
  pub fn bounded<T: RandomBounded>(&mut self, n: T) -> T {
    T::random_bounded(self, n)
  }

  /// Samples a `T` from the uniform distribution over `a ..= b`, i.e. the
  /// inclusive range between `a` and `b`. The range is permitted to wrap
  /// around from `T::MAX` to `T::MIN`.
  #[inline(always)]
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
  #[inline(always)]
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
  #[inline(always)]
  pub fn float_biunit<T: RandomFloat>(&mut self) -> T {
    T::random_float_biunit(self)
  }

  /// ???
  #[inline(always)]
  pub fn random<T>(&mut self, distr: impl Distribution<T>) -> T {
    distr.random(self)
  }

  /// ???
  pub fn fill<T>(&mut self, distr: impl Distribution<T>, buf: &mut [T]) {
    distr.fill(self, buf)
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
  pub fn shuffle<T>(&mut self, slice: &mut [T]) {
    let n = slice.len();
    if n >= 2 {
      let p = &raw mut *slice as *mut T;
      for i in 1 .. n {
        let j = self.bounded(i);
        unsafe { p.add(i).swap(p.add(j)) };
      }
    }
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
    [x as u32, (x >> 32) as u32]
  }

  #[inline(always)]
  unsafe fn fill_unchecked__<const N: usize, T, F>(&mut self, f: F, buf: *mut T, len: usize)
  where
    T: Copy,
    F: Fn(&mut Self) -> [T; N]
  {
    const { assert!(N <= 8) };
    let mut p = buf;
    let mut i = len;
    if i == 0 { return }
    while i > 2 * N {
      let x = f(self);
      let y = f(self);
      unsafe { p.copy_from_nonoverlapping(&raw const x as _, N) };
      p = unsafe { p.add(N) };
      i = i - N;
      unsafe { p.copy_from_nonoverlapping(&raw const y as _, N) };
      p = unsafe { p.add(N) };
      i = i - N;
    }
    if i > N {
      let x = f(self);
      unsafe { p.copy_from_nonoverlapping(&raw const x as _, N) };
      p = unsafe { p.add(N) };
      i = i - N;
    }
    let x = f(self);
    match i {
      1 => unsafe { p.copy_from_nonoverlapping(&raw const x as _, 1) },
      2 => unsafe { p.copy_from_nonoverlapping(&raw const x as _, 2) },
      3 => unsafe { p.copy_from_nonoverlapping(&raw const x as _, 3) },
      4 => unsafe { p.copy_from_nonoverlapping(&raw const x as _, 4) },
      5 => unsafe { p.copy_from_nonoverlapping(&raw const x as _, 5) },
      6 => unsafe { p.copy_from_nonoverlapping(&raw const x as _, 6) },
      7 => unsafe { p.copy_from_nonoverlapping(&raw const x as _, 7) },
      8 => unsafe { p.copy_from_nonoverlapping(&raw const x as _, 8) },
      _ => unreachable!()
    }
  }
}

impl RandomUniform for bool {
}

impl private::RandomUniform for bool {
  #[inline(always)]
  fn random_uniform(g: &mut Rng) -> Self {
    (g.next() as i64) < 0
  }
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

impl private::RandomUniform for u8 {
  #[inline(always)]
  fn random_uniform(g: &mut Rng) -> Self {
    g.next() as _
  }

  #[inline(always)]
  fn random_uniform_array<const N: usize>(g: &mut Rng) -> [Self; N] {
    let mut a = MaybeUninit::uninit();
    unsafe { g.fill_unchecked__(Rng::next_8b, &raw mut a as _, N) };
    unsafe { a.assume_init() }
  }

  fn fill_uniform(g: &mut Rng, buf: &mut [Self]) {
    unsafe { g.fill_unchecked__(Rng::next_8b, &raw mut *buf as _, buf.len()) };
  }
}

impl private::RandomUniform for u16 {
  #[inline(always)]
  fn random_uniform(g: &mut Rng) -> Self {
    g.next() as _
  }

  #[inline(always)]
  fn random_uniform_array<const N: usize>(g: &mut Rng) -> [Self; N] {
    let mut buf = MaybeUninit::uninit();
    unsafe { g.fill_unchecked__(Rng::next_4h, &raw mut buf as _, N) };
    unsafe { buf.assume_init() }
  }

  fn fill_uniform(g: &mut Rng, buf: &mut [Self]) {
    unsafe { g.fill_unchecked__(Rng::next_4h, &raw mut *buf as _, buf.len()) };
  }
}

impl private::RandomUniform for u32 {
  #[inline(always)]
  fn random_uniform(g: &mut Rng) -> Self {
    g.next() as _
  }

  #[inline(always)]
  fn random_uniform_array<const N: usize>(g: &mut Rng) -> [Self; N] {
    let mut buf = MaybeUninit::uninit();
    unsafe { g.fill_unchecked__(Rng::next_2w, &raw mut buf as _, N) };
    unsafe { buf.assume_init() }
  }

  fn fill_uniform(g: &mut Rng, buf: &mut [Self]) {
    unsafe { g.fill_unchecked__(Rng::next_2w, &raw mut *buf as _, buf.len()) };
  }
}

impl private::RandomUniform for u64 {
  #[inline(always)]
  fn random_uniform(g: &mut Rng) -> Self {
    g.next()
  }

}

impl private::RandomUniform for u128 {
  #[inline(always)]
  fn random_uniform(g: &mut Rng) -> Self {
    concat(g.next(), g.next())
  }
}

macro_rules! int_uniform_impls {
  ($($sint:ty, $uint:ty, $nzsint:ty, $nzuint:ty;)*) => {
    $(
    impl RandomUniform for $uint {
    }

    impl RandomUniform for $sint {
    }

    impl RandomUniform for $nzsint {
    }

    impl RandomUniform for $nzuint {
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

      // TODO: fill_uniform
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
  #[inline(always)]
  fn random_uniform(g: &mut Rng) -> Self {
    T::random_uniform_array(g)
  }
}

impl RandomUniform for () {
}

impl private::RandomUniform for () {
  #[inline(always)]
  fn random_uniform(_: &mut Rng) -> Self {
  }
}

impl<A: RandomUniform, B: RandomUniform> RandomUniform for (A, B) {
}

impl<A: RandomUniform, B: RandomUniform> private::RandomUniform for (A, B) {
  #[inline(always)]
  fn random_uniform(g: &mut Rng) -> Self {
    (A::random_uniform(g), B::random_uniform(g))
  }
}

impl<A: RandomUniform, B: RandomUniform, C: RandomUniform> RandomUniform for (A, B, C) {
}

impl<A: RandomUniform, B: RandomUniform, C: RandomUniform> private::RandomUniform for (A, B, C) {
  #[inline(always)]
  fn random_uniform(g: &mut Rng) -> Self {
    (A::random_uniform(g), B::random_uniform(g), C::random_uniform(g))
  }
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

impl private::RandomBounded for u8 {
  #[inline(always)]
  fn random_bounded(g: &mut Rng, n: Self) -> Self {
    mulhi(g.next(), n as u64 + 1) as _
  }
}

impl private::RandomBounded for u16 {
  #[inline(always)]
  fn random_bounded(g: &mut Rng, n: Self) -> Self {
    mulhi(g.next(), n as u64 + 1) as _
  }
}

impl private::RandomBounded for u32 {
  #[inline(always)]
  fn random_bounded(g: &mut Rng, n: Self) -> Self {
    let mut h = g.clone();
    let x = h.next();
    let n = n as u64;
    let m = n + 1;
    let mut a = mulhi(x, m);
    let mut b = x.wrapping_mul(m);
    if b.overflowing_add(n).1 {
      loop {
        let x = h.next();
        let c = b.overflowing_add(mulhi(x, m));
        a = a + c.1 as u64;
        b = x.wrapping_mul(m);
        if c.0 != u64::MAX { break }
        cold_path();
      }
    }
    *g = h;
    a as u32
  }
}

impl private::RandomBounded for u64 {
  #[inline(always)]
  fn random_bounded(g: &mut Rng, n: Self) -> Self {
    let mut h = g.clone();
    let x = h.next();
    let m = n.wrapping_add(1);
    let mut a = mulhi(x, m);
    let mut b = x.wrapping_mul(m);
    let u = select_unpredictable(m == 0, x, a);
    if b.overflowing_add(n).1 {
      loop {
        let x = h.next();
        let c = b.overflowing_add(mulhi(x, m));
        a = a + c.1 as u64;
        b = x.wrapping_mul(m);
        if c.0 != u64::MAX { break }
        cold_path();
        // NB: We get here with negligible probability and don't claim that
        // looping increases the fidelity of our sampled distribution.
        // However, the inclusion of the loop inhibits the potential
        // pessimization of control flow getting if-converted away
      }
    } else {
      a = u;
    }
    *g = h;
    a
  }
}

macro_rules! int_bounded_between_impls {
  ($($sint:ty, $uint:ty;)*) => {
    $(
    impl RandomBounded for $uint {
    }

    impl RandomBetween for $sint {
    }

    impl RandomBetween for $uint {
    }

    impl private::RandomBetween for $sint {
      #[inline(always)]
      fn random_between(g: &mut Rng, a: Self, b: Self) -> Self {
        <$uint>::random_between(g, a as $uint, b as $uint).cast_signed()
      }
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

int_bounded_between_impls! {
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
    // cf u64::random_float
    let x = g.next() as i64;
    let x = f32::from_bits(0x2000_0000) * (x as f32);
    x.abs()
  }

  #[inline(always)]
  fn random_float_biunit(g: &mut Rng) -> Self {
    // cf u64::random_float_biunit
    let x = g.next() as i64;
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
    let x = g.next() as i64;
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
    let x = g.next() as i64;
    let x = (x & 1) + (x >> 1);
    f64::from_bits(0x3c10_0000_0000_0000) * (x as f64)
  }
}

impl<'a, T, D: Distribution<T>> Distribution<T> for &'a D {
}

impl<'a, T, D: Distribution<T>> private::Distribution<T> for &'a D {
  #[inline(always)]
  fn random(&self, g: &mut Rng) -> T {
    (*self).random(g)
  }

  #[inline(always)]
  fn random_array<const N: usize>(&self, g: &mut Rng) -> [T; N] {
    (*self).random_array(g)
  }

  fn fill(&self, g: &mut Rng, buf: &mut [T]) {
    (*self).fill(g, buf)
  }
}

impl<T: RandomUniform> Distribution<T> for RangeFull {
}

impl<T: RandomUniform> private::Distribution<T> for RangeFull {
  #[inline(always)]
  fn random(&self, g: &mut Rng) -> T {
    T::random_uniform(g)
  }

  #[inline(always)]
  fn random_array<const N: usize>(&self, g: &mut Rng) -> [T; N] {
    T::random_uniform_array(g)
  }

  fn fill(&self, g: &mut Rng, buf: &mut [T]) {
    T::fill_uniform(g, buf)
  }
}

impl<T: Copy + RandomBounded> Distribution<T> for core::ops::RangeToInclusive<T> {
}

impl<T: Copy + RandomBounded> private::Distribution<T> for core::ops::RangeToInclusive<T> {
  #[inline(always)]
  fn random(&self, g: &mut Rng) -> T {
    T::random_bounded(g, self.end)
  }
}

impl<T: Copy + RandomBounded> Distribution<T> for RangeToInclusive<T> {
}

impl<T: Copy + RandomBounded> private::Distribution<T> for RangeToInclusive<T> {
  #[inline(always)]
  fn random(&self, g: &mut Rng) -> T {
    T::random_bounded(g, self.last)
  }
}

impl<T: Copy + PartialOrd + RandomBetween> Distribution<T> for core::ops::RangeInclusive<T> {
}

impl<T: Copy + PartialOrd + RandomBetween> private::Distribution<T> for core::ops::RangeInclusive<T> {
  #[inline(always)]
  fn random(&self, g: &mut Rng) -> T {
    assert!(! self.is_empty());
    T::random_between(g, *self.start(), *self.end())
  }
}

impl<T: Copy + PartialOrd + RandomBetween> Distribution<T> for RangeInclusive<T> {
}

impl<T: Copy + PartialOrd + RandomBetween> private::Distribution<T> for RangeInclusive<T> {
  #[inline(always)]
  fn random(&self, g: &mut Rng) -> T {
    assert!(! self.is_empty());
    T::random_between(g, self.start, self.last)
  }
}

impl<const N: usize, T, D: Distribution<T>> Distribution<[T; N]> for [D; 1] {
}

impl<const N: usize, T, D: Distribution<T>> private::Distribution<[T; N]> for [D; 1] {
  #[inline(always)]
  fn random(&self, g: &mut Rng) -> [T; N] {
    let [ref distr] = *self;
    distr.random_array(g)
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

  fn try_fill_bytes(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
    self.fill(.., buf);
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
  use std::num::NonZeroU128;
  use std::thread_local;
  use super::Distribution;
  use super::RandomBetween;
  use super::RandomBounded;
  use super::RandomFloat;
  use super::RandomUniform;
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

  /// See [Rng::uniform].
  pub fn uniform<T: RandomUniform>() -> T {
    with(|g| g.uniform())
  }

  /// See [Rng::bounded].
  pub fn bounded<T: RandomBounded>(n: T) -> T {
    with(|g| g.bounded(n))
  }

  /// See [Rng::between].
  pub fn between<T: RandomBetween>(a: T, b: T) -> T {
    with(|g| g.between(a, b))
  }

  /// See [Rng::float].
  pub fn float<T: RandomFloat>() -> T {
    with(|g| g.float_biunit())
  }

  /// See [Rng::float_biunit].
  pub fn float_biunit<T: RandomFloat>() -> T {
    with(|g| g.float_biunit())
  }

  /// See [Rng::random].
  pub fn random<T>(d: impl Distribution<T>) -> T {
    with(|g| g.random(d))
  }

  /// See [Rng::fill].
  pub fn fill<T>(distr: impl Distribution<T>, buf: &mut [T]) {
    with(|g| g.fill(distr, buf))
  }

  /// See [Rng::bernoulli].
  pub fn bernoulli(p: f64) -> bool {
    with(|g| g.bernoulli(p))
  }
}

mod private {
  use core::array;
  use super::Rng;

  pub(crate) trait RandomUniform {
    fn random_uniform(_: &mut Rng) -> Self;

    #[inline(always)]
    fn random_uniform_array<const N: usize>(g: &mut Rng) -> [Self; N] where Self: Sized {
      array::from_fn(|_| Self::random_uniform(g))
    }

    fn fill_uniform(g: &mut Rng, buf: &mut [Self]) where Self: Sized {
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

  pub(crate) trait Distribution<T> {
    fn random(&self, _: &mut Rng) -> T;

    #[inline(always)]
    fn random_array<const N: usize>(&self, g: &mut Rng) -> [T; N] {
      array::from_fn(|_| self.random(g))
    }

    fn fill(&self, g: &mut Rng, buf: &mut [T]) {
      buf.iter_mut().for_each(|a| *a = self.random(g));
    }
  }
}
