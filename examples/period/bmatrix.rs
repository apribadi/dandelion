#![allow(missing_docs)]
#![allow(unused)]

#[derive(Clone, Copy, Eq, PartialEq)]
struct Bm8(u64);

impl Bm8 {
  const ZERO: Self = Self(0);

  const I: Self = Self(0x8040_2010_0804_0201);

  fn add(x: Self, y: Self) -> Self {
    Self(x.0 ^ y.0)
  }

  fn mul(x: Self, y: Self) -> Self {
    Self(
        (x.0 >> 0 & 0x0101_0101_0101_0101) * (y.0 >>  0 & 0xff)
      ^ (x.0 >> 1 & 0x0101_0101_0101_0101) * (y.0 >>  8 & 0xff)
      ^ (x.0 >> 2 & 0x0101_0101_0101_0101) * (y.0 >> 16 & 0xff)
      ^ (x.0 >> 3 & 0x0101_0101_0101_0101) * (y.0 >> 24 & 0xff)
      ^ (x.0 >> 4 & 0x0101_0101_0101_0101) * (y.0 >> 32 & 0xff)
      ^ (x.0 >> 5 & 0x0101_0101_0101_0101) * (y.0 >> 40 & 0xff)
      ^ (x.0 >> 6 & 0x0101_0101_0101_0101) * (y.0 >> 48 & 0xff)
      ^ (x.0 >> 7 & 0x0101_0101_0101_0101) * (y.0 >> 56 & 0xff)
    )
  }

  fn get(&self, i: usize, j: usize) -> bool {
    0 != (1 << 8 * i + j) & self.0
  }

  fn set(&mut self, i: usize, j: usize, value: bool) {
    self.0 ^= (1 << 8 * i + j) & (self.0 ^ (value as u64).wrapping_neg());
  }
}

impl std::ops::Add<Self> for Bm8 {
  type Output = Self;

  fn add(self, rhs: Self) -> Self::Output {
    Self::add(self, rhs)
  }
}

impl std::ops::Mul<Self> for Bm8 {
  type Output = Self;

  fn mul(self, rhs: Self) -> Self::Output {
    Self::mul(self, rhs)
  }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) struct Bm8n<const N: usize>([[Bm8; N]; N]);

impl<const N: usize> Bm8n<N> {
  pub(crate) const ZERO: Self = Self([[Bm8::ZERO; N]; N]);

  pub(crate) const I: Self = {
    let mut x = Self::ZERO;
    let mut i = 0;
    while i < N {
      x.0[i][i] = Bm8::I;
      i += 1;
    }
    x
  };

  #[inline(never)]
  fn mul(out: &mut Self, x: &Self, y: &Self) {
    for i in 0 .. N {
      for j in 0 .. N {
        let mut a = Bm8::ZERO;
        for k in 0 .. N {
          a = a + x.0[i][k] * y.0[k][j];
        }
        out.0[i][j] = a;
      }
    }
  }

  pub(crate) fn get(&self, i: usize, j: usize) -> bool {
    self.0[i >> 3 & N - 1][j >> 3 & N - 1].get(i & 8 - 1, j & 8 - 1)
  }

  pub(crate) fn set(&mut self, i: usize, j: usize, value: bool) {
    self.0[i >> 3 & N - 1][j >> 3 & N - 1].set(i & 8 - 1, j & 8 - 1, value)
  }
}

impl<const N: usize> std::ops::Mul<Self> for Bm8n<N> {
  type Output = Self;

  #[inline(always)]
  fn mul(self, rhs: Self) -> Self::Output {
    let mut o = Self::ZERO;
    Self::mul(&mut o, &self, &rhs);
    o
  }
}

pub(crate) type Bm32 = Bm8n<4>;

pub(crate) type Bm64 = Bm8n<8>;

pub(crate) type Bm128 = Bm8n<16>;
