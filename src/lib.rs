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
pub trait Uniform: private::Uniform {
}

/// ???
#[allow(private_bounds)]
pub trait Bounded: private::Bounded {
}

/// ???
#[allow(private_bounds)]
pub trait Between: private::Between {
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

  #[inline(always)]
  const fn next(&mut self) -> u64 {
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
  pub fn uniform<T: Uniform>(&mut self) -> T {
    T::uniform(self)
  }

  /// Samples a `T` from the uniform distribution over the range `0 ..= n`.
  #[inline(always)]
  pub fn bounded<T: Bounded>(&mut self, n: T) -> T {
    // TODO: make a statement about total variation distance
    T::bounded(self, n)
  }

  /// Samples a `T` from the uniform distribution over the range `a ..= b`. The
  /// range is permitted to wrap around from `T::MAX` to `T::MIN`.
  #[inline(always)]
  pub fn between<T: Between>(&mut self, a: T, b: T) -> T {
    T::between(self, a, b)
  }

  /// ???
  #[inline(always)]
  pub fn sample<T>(&mut self, d: impl Distribution<T>) -> T {
    <_ as private::Distribution<T>>::sample(self, d)
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
  pub fn f32(&mut self) -> f32 {
    let x = self.next() as i64;
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
  pub fn f64(&mut self) -> f64 {
    // The conversion into a `f64` is two instructions on aarch64:
    //
    //   scvtf d0, x8, #63
    //   fabs d0, d0
    //
    let x = self.next() as i64;
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
  pub fn biunit_f32(&mut self) -> f32 {
    let x = self.next() as i64;
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
  pub fn biunit_f64(&mut self) -> f64 {
    // The conversion into a `f64` is three instructions on aarch64:
    //
    //   and x1, x0, #0x1
    //   add x2, x1, x0, asr #1
    //   scvtf d0, x2, #62
    //
    let x = self.next() as i64;
    let x = (x & 1) + (x >> 1);
    f64::from_bits(0x3c10_0000_0000_0000) * (x as f64)
  }

  #[inline(always)]
  const unsafe fn fill_unchecked_inlined(&mut self, dst: *mut u8, len: usize) {
    let mut p = dst;
    let mut n = len;
    if n == 0 { return }
    while n >= 17 {
      let x = self.next().to_le_bytes();
      let y = self.next().to_le_bytes();
      unsafe { p.copy_from_nonoverlapping(&raw const x as _, 8) };
      p = unsafe { p.add(8) };
      n = n - 8;
      unsafe { p.copy_from_nonoverlapping(&raw const y as _, 8) };
      p = unsafe { p.add(8) };
      n = n - 8;
    }
    if n >= 9 {
      let x = self.next().to_le_bytes();
      unsafe { p.copy_from_nonoverlapping(&raw const x as _, 8) };
      p = unsafe { p.add(8) };
      n = n - 8;
    }
    let x = self.next().to_le_bytes();
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
  pub fn fill(&mut self, dst: &mut [u8]) {
    let n = dst.len();
    unsafe { self.fill_unchecked(&raw mut *dst as _, n) };
  }

  /// Fills the provided buffer with independent uniformly distributed bytes.
  ///
  /// # Safety
  ///
  /// It must be valid to write `len` arbitrary bytes at `dst`.
  pub unsafe fn fill_unchecked(&mut self, dst: *mut u8, len: usize) {
    unsafe { self.fill_unchecked_inlined(dst, len) };
  }

  /// Fills the provided buffer with independent uniformly distributed bytes,
  /// returning a reference to the initialized buffer. The returned buffer is a
  /// reference the same memory and has the same length as the input buffer.
  pub fn fill_uninit<'a>(&mut self, dst: &'a mut [MaybeUninit<u8>]) -> &'a mut [u8] {
    let n = dst.len();
    unsafe { self.fill_unchecked(&raw mut *dst as _, n) };
    unsafe { dst.assume_init_mut() }
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
}

impl Uniform for bool {
}

impl private::Uniform for bool {
  #[inline(always)]
  fn uniform(g: &mut Rng) -> Self {
    (g.next() as i64) < 0
  }
}

impl private::Uniform for usize {
  #[inline(always)]
  fn uniform(g: &mut Rng) -> Self {
    cfg_select! {
      target_pointer_width = "16" => { u16::uniform(g) as _ }
      target_pointer_width = "32" => { u32::uniform(g) as _ }
      target_pointer_width = "64" => { u64::uniform(g) as _ }
      _ => { unimplemented!() }
    }
  }
}

impl private::Uniform for u8 {
  #[inline(always)]
  fn uniform(g: &mut Rng) -> Self {
    g.next() as _
  }

  #[inline(always)]
  fn uniform_array<const N: usize>(g: &mut Rng) -> [Self; N] {
    let mut a = [0; N];
    unsafe { g.fill_unchecked_inlined(&raw mut a as _, N) };
    a
  }
}

impl private::Uniform for u16 {
  #[inline(always)]
  fn uniform(g: &mut Rng) -> Self {
    g.next() as _
  }
}

impl private::Uniform for u32 {
  #[inline(always)]
  fn uniform(g: &mut Rng) -> Self {
    g.next() as _
  }
}

impl private::Uniform for u64 {
  #[inline(always)]
  fn uniform(g: &mut Rng) -> Self {
    g.next()
  }
}

impl private::Uniform for u128 {
  #[inline(always)]
  fn uniform(g: &mut Rng) -> Self {
    concat(g.next(), g.next())
  }
}

impl<const N: usize, T: Uniform> Uniform for [T; N] {
}

impl<const N: usize, T: Uniform> private::Uniform for [T; N] {
  #[inline(always)]
  fn uniform(g: &mut Rng) -> Self {
    T::uniform_array(g)
  }
}

impl Uniform for () {
}

impl private::Uniform for () {
  #[inline(always)]
  fn uniform(_: &mut Rng) -> Self {
  }
}

impl<A: Uniform, B: Uniform> Uniform for (A, B) {
}

impl<A: Uniform, B: Uniform> private::Uniform for (A, B) {
  #[inline(always)]
  fn uniform(g: &mut Rng) -> Self {
    (A::uniform(g), B::uniform(g))
  }
}

impl<A: Uniform, B: Uniform, C: Uniform> Uniform for (A, B, C) {
}

impl<A: Uniform, B: Uniform, C: Uniform> private::Uniform for (A, B, C) {
  #[inline(always)]
  fn uniform(g: &mut Rng) -> Self {
    (A::uniform(g), B::uniform(g), C::uniform(g))
  }
}

impl Bounded for usize {
}

impl private::Bounded for usize {
  #[inline(always)]
  fn bounded(g: &mut Rng, n: Self) -> Self {
    cfg_select! {
      target_pointer_width = "16" => { u16::bounded(g, n as _) as _ }
      target_pointer_width = "32" => { u32::bounded(g, n as _) as _ }
      target_pointer_width = "64" => { u64::bounded(g, n as _) as _ }
      _ => { unimplemented!() }
    }
  }
}

impl Bounded for u8 {
}

impl private::Bounded for u8 {
  #[inline(always)]
  fn bounded(g: &mut Rng, n: Self) -> Self {
    mulhi(g.next(), n as u64 + 1) as _
  }
}

impl Bounded for u16 {
}

impl private::Bounded for u16 {
  #[inline(always)]
  fn bounded(g: &mut Rng, n: Self) -> Self {
    mulhi(g.next(), n as u64 + 1) as _
  }
}

impl Bounded for u32 {
}

impl private::Bounded for u32 {
  #[inline(always)]
  fn bounded(g: &mut Rng, n: Self) -> Self {
    let mut h = g.clone();
    let u = h.next();
    let n = n as u64;
    let m = n + 1;
    let mut x = mulhi(u, m);
    let mut y = u.wrapping_mul(m);
    if y.overflowing_add(n).1 {
      loop {
        let v = h.next();
        let w = y.overflowing_add(mulhi(v, m));
        x = x + w.1 as u64;
        y = v.wrapping_mul(m);
        if w.0 != u64::MAX { break }
        cold_path();
      }
    }
    *g = h;
    x as u32
  }
}

impl Bounded for u64 {
}

impl private::Bounded for u64 {
  #[inline(always)]
  fn bounded(g: &mut Rng, n: Self) -> Self {
    let mut h = g.clone();
    let u = h.next();
    let m = n.wrapping_add(1);
    let mut x = mulhi(u, m);
    let mut y = u.wrapping_mul(m);
    let a = select_unpredictable(m == 0, u, x);
    if y.overflowing_add(n).1 {
      loop {
        let v = h.next();
        let w = y.overflowing_add(mulhi(v, m));
        x = x + w.1 as u64;
        y = v.wrapping_mul(m);
        if w.0 != u64::MAX { break }
        cold_path();
        // NB: We get here with negligible probability and don't claim that
        // looping increases the fidelity of our sampled distribution.
        // However, including the loop inhibits the potential pessimization of
        // the control flow getting if-converted away
      }
    } else {
      x = a;
    }
    *g = h;
    x
  }
}

impl Bounded for u128 {
}

impl private::Bounded for u128 {
  #[inline(always)]
  fn bounded(g: &mut Rng, n: Self) -> Self {
    let _ = g;
    let _ = n;
    unimplemented!()
  }
}

macro_rules! int_impls {
  ($($sint:ty, $uint:ty, $nzsint:ty, $nzuint:ty;)*) => {
    $(
    impl Uniform for $uint {
    }

    impl Uniform for $sint {
    }

    impl Uniform for $nzsint {
    }

    impl Uniform for $nzuint {
    }

    impl Between for $sint {
    }

    impl Between for $uint {
    }

    impl private::Uniform for $sint {
      #[inline(always)]
      fn uniform(g: &mut Rng) -> Self {
        <$uint>::uniform(g) as _
      }
    }

    impl private::Uniform for $nzsint {
      #[inline(always)]
      fn uniform(g: &mut Rng) -> Self {
        loop {
          if let Some(x) = Self::new(<$sint>::uniform(g)) {
            break x
          }
        }
      }
    }

    impl private::Uniform for $nzuint {
      #[inline(always)]
      fn uniform(g: &mut Rng) -> Self {
        loop {
          if let Some(x) = Self::new(<$uint>::uniform(g)) {
            break x
          }
        }
      }
    }

    impl private::Between for $sint {
      #[inline(always)]
      fn between(g: &mut Rng, a: Self, b: Self) -> Self {
        <$uint>::between(g, a as $uint, b as $uint) as _
      }
    }

    impl private::Between for $uint {
      #[inline(always)]
      fn between(g: &mut Rng, a: Self, b: Self) -> Self {
        <$uint as private::Bounded>::bounded(g, b.wrapping_sub(a)).wrapping_add(a)
      }
    }
    )*
  };
}

impl<T: Uniform> Distribution<T> for RangeFull {
}

impl<T: Uniform> private::Distribution<T> for RangeFull {
  fn sample(g: &mut Rng, _: Self) -> T {
    T::uniform(g)
  }
}

impl<T: Bounded> Distribution<T> for core::ops::RangeToInclusive<T> {
}

impl<T: Bounded> private::Distribution<T> for core::ops::RangeToInclusive<T> {
  fn sample(g: &mut Rng, range: Self) -> T {
    T::bounded(g, range.end)
  }
}

impl<T: Bounded> Distribution<T> for RangeToInclusive<T> {
}

impl<T: Bounded> private::Distribution<T> for RangeToInclusive<T> {
  fn sample(g: &mut Rng, range: Self) -> T {
    T::bounded(g, range.last)
  }
}

impl<T: Between> Distribution<T> for core::ops::RangeInclusive<T> {
}

impl<T: Between> private::Distribution<T> for core::ops::RangeInclusive<T> {
  fn sample(g: &mut Rng, range: Self) -> T {
    let (a, b) = range.into_inner();
    T::between(g, a, b)
  }
}

impl<T: Between> Distribution<T> for RangeInclusive<T> {
}

impl<T: Between> private::Distribution<T> for RangeInclusive<T> {
  fn sample(g: &mut Rng, range: Self) -> T {
    T::between(g, range.start, range.last)
  }
}

int_impls! {
  isize, usize, NonZeroIsize, NonZeroUsize;
  i8, u8, NonZeroI8, NonZeroU8;
  i16, u16, NonZeroI16, NonZeroU16;
  i32, u32, NonZeroI32, NonZeroU32;
  i64, u64, NonZeroI64, NonZeroU64;
  i128, u128, NonZeroI128, NonZeroU128;
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
  use std::thread_local;
  use super::Between;
  use super::Bounded;
  use super::Distribution;
  use super::Rng;
  use super::Uniform;

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
  pub fn uniform<T: Uniform>() -> T {
    with(|g| g.uniform())
  }

  /// See [Rng::bounded].
  pub fn bounded<T: Bounded>(n: T) -> T {
    with(|g| g.bounded(n))
  }

  /// See [Rng::between].
  pub fn between<T: Between>(a: T, b: T) -> T {
    with(|g| g.between(a, b))
  }

  /// See [Rng::sample].
  pub fn sample<T>(d: impl Distribution<T>) -> T {
    with(|g| g.sample(d))
  }

  /// See [Rng::bernoulli].
  pub fn bernoulli(p: f64) -> bool {
    with(|g| g.bernoulli(p))
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
}

mod private {
  use core::array;

  pub(crate) trait Uniform {
    fn uniform(_: &mut super::Rng) -> Self;

    #[inline(always)]
    fn uniform_array<const N: usize>(g: &mut super::Rng) -> [Self; N] where Self: Sized {
      array::from_fn(|_| Self::uniform(g))
    }
  }

  pub(crate) trait Bounded {
    fn bounded(_: &mut super::Rng, _: Self) -> Self;
  }

  pub(crate) trait Between {
    fn between(_: &mut super::Rng, _: Self, _: Self) -> Self;
  }

  pub(crate) trait Distribution<T> {
    fn sample(_: &mut super::Rng, _: Self) -> T;
  }
}
