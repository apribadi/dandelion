//! Finds RNG parameters with full period.

mod bmatrix;

use bmatrix::Bm32;
use bmatrix::Bm64;
use bmatrix::Bm128;

trait W
  : Copy
  + std::ops::BitXor<Output = Self>
  + std::ops::Shl<usize, Output = Self>
{
  const BITS: u32;

  const I: Self::M;

  const PRIMES: &'static [u128];

  const FULL_PERIOD: u128;

  type M
    : Copy
    + Eq
    + std::ops::Mul<Output = Self::M>;

  fn asr(_: Self, _: usize) -> Self;

  fn lsl(x: Self, a: usize) -> Self {
    x << a
  }

  fn make_mat<F: Fn(Self, Self) -> (Self, Self)>(_: F) -> Self::M;

  fn pow(x: Self::M, n: u128) -> Self::M {
    if n == 0 { return Self::I; }
    let mut n = n;
    let mut x = x;
    let mut y = Self::I;
    while n != 1 {
      if n & 1 != 0 { y = x * y; }
      x = x * x;
      n = n / 2;
    }
    x * y
  }

  fn pow_full(x: Self::M) -> Self::M {
    let mut x = x;
    for _ in 0 .. 2 * Self::BITS {
      x = x * x;
    }
    x
  }

  fn check() {
    let _: () = const {
      let mut a = 1u128;
      let mut i = 0;
      while i < Self::PRIMES.len() {
        a *= Self::PRIMES[i];
        i += 1;
      }
      assert!(a == Self::FULL_PERIOD);
    };
    for a in 1 .. Self::BITS as usize {
      for b in 1 .. Self::BITS as usize {
        let m = Self::make_mat(|x, y| (y ^ Self::lsl(x, a), x ^ Self::asr(y, b)));
        if Self::pow_full(m) == m && Self::pow(m, Self::FULL_PERIOD) == Self::I {
          'fail: {
            for &p in Self::PRIMES.iter() {
              if Self::pow(m, Self::FULL_PERIOD / p) == Self::I {
                // print!("FAIL a={} b={} p={}\n", a, b, p);
                break 'fail;
              }
            }
            print!("OK a={} b={}\n", a, b);
          }
        }
      }
    }
  }
}

impl W for u16 {
  const BITS: u32 = Self::BITS;

  const I: Self::M = Bm32::I;

  const PRIMES: &'static [u128] = &[
    3,
    5,
    17,
    257,
    65_537,
  ];

  const FULL_PERIOD: u128 = u32::MAX as u128;

  type M = Bm32;

  fn asr(x: Self, a: usize) -> Self {
    (x.cast_signed() >> a).cast_unsigned()
  }

  fn make_mat<F: Fn(Self, Self) -> (Self, Self)>(f: F) -> Self::M {
    let mut a = Bm32::ZERO;
    for j in 0 .. 2 * Self::BITS {
      let x = 1u32 << j;
      let y = f(x as Self, (x >> Self::BITS) as Self);
      let y = (y.0 as u32) ^ ((y.1 as u32) << Self::BITS);
      for i in 0 .. 2 * Self::BITS {
        if y & (1 << i) != 0 {
          a.set(i as usize, j as usize, true);
        }
      }
    }
    a
  }
}

impl W for u32 {
  const BITS: u32 = Self::BITS;

  const I: Self::M = Bm64::I;

  const PRIMES: &'static [u128] = &[
    3,
    5,
    17,
    257,
    65_537,
    641,
    6_700_417,
  ];

  const FULL_PERIOD: u128 = u64::MAX as u128;

  type M = Bm64;

  fn asr(x: Self, a: usize) -> Self {
    (x.cast_signed() >> a).cast_unsigned()
  }

  fn make_mat<F: Fn(Self, Self) -> (Self, Self)>(f: F) -> Self::M {
    let mut a = Bm64::ZERO;
    for j in 0 .. 2 * Self::BITS {
      let x = 1u64 << j;
      let y = f(x as Self, (x >> Self::BITS) as Self);
      let y = (y.0 as u64) ^ ((y.1 as u64) << Self::BITS);
      for i in 0 .. 2 * Self::BITS {
        if y & (1 << i) != 0 {
          a.set(i as usize, j as usize, true);
        }
      }
    }
    a
  }
}

impl W for u64 {
  const BITS: u32 = Self::BITS;

  const I: Self::M = Bm128::I;

  const PRIMES: &'static [u128] = &[
    3,
    5,
    17,
    257,
    65_537,
    641,
    6_700_417,
    274_177,
    67_280_421_310_721,
  ];

  const FULL_PERIOD: u128 = u128::MAX;

  type M = Bm128;

  fn asr(x: Self, a: usize) -> Self {
    (x.cast_signed() >> a).cast_unsigned()
  }

  fn make_mat<F: Fn(Self, Self) -> (Self, Self)>(f: F) -> Self::M {
    let mut a = Bm128::ZERO;
    for j in 0 .. 2 * Self::BITS {
      let x = 1u128 << j;
      let y = f(x as Self, (x >> Self::BITS) as Self);
      let y = (y.0 as u128) ^ ((y.1 as u128) << Self::BITS);
      for i in 0 .. 2 * Self::BITS {
        if y & (1 << i) != 0 {
          a.set(i as usize, j as usize, true);
        }
      }
    }
    a
  }
}

fn main() {
  print!("u16x2: (x, y) => (y ^ lsl(x, a), x ^ asr(y, b))\n");
  u16::check();
  print!("\n");
  print!("u32x2: (x, y) => (y ^ lsl(x, a), x ^ asr(y, b))\n");
  u32::check();
  print!("\n");
  print!("u64x2: (x, y) => (y ^ lsl(x, a), x ^ asr(y, b))\n");
  u64::check();
}
