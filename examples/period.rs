//! Finds RNG parameters with full period.

#[derive(Clone, Copy, Eq, PartialEq)]
struct M8(u64);

impl M8 {
  const ZERO: Self = Self(0);
  const ID: Self = Self(0x8040_2010_0804_0201);

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

  fn set(&mut self, i: usize, j: usize, value: bool) {
    self.0 ^= (1 << 8 * i + j) & (self.0 ^ (value as u64).wrapping_neg());
  }
}

impl std::ops::Add<Self> for M8 {
  type Output = Self;

  fn add(self, rhs: Self) -> Self::Output {
    Self::add(self, rhs)
  }
}

impl std::ops::Mul<Self> for M8 {
  type Output = Self;

  fn mul(self, rhs: Self) -> Self::Output {
    Self::mul(self, rhs)
  }
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct M128([[M8; 16]; 16]);

impl M128 {
  const ZERO: Self = Self([[M8::ZERO; 16]; 16]);

  const ID: Self = {
    let mut x = Self::ZERO;
    let mut i = 0;
    while i < 16 {
      x.0[i][i] = M8::ID;
      i += 1;
    }
    x
  };

  #[inline(never)]
  fn mul(out: &mut Self, x: &Self, y: &Self) {
    for i in 0 .. 16 {
      for j in 0 .. 16 {
        let mut a = M8::ZERO;
        for k in 0 .. 16 {
          a = a + x.0[i][k] * y.0[k][j];
        }
        out.0[i][j] = a;
      }
    }
  }

  fn set(&mut self, i: usize, j: usize, value: bool) {
    self.0[i >> 3 & 15][j >> 3 & 15].set(i & 7, j & 7, value)
  }
}

impl std::ops::Mul<Self> for M128 {
  type Output = Self;

  fn mul(self, rhs: Self) -> Self::Output {
    let mut o = Self::ZERO;
    Self::mul(&mut o, &self, &rhs);
    o
  }
}

const N: u128 = u128::MAX;

const FACTORS: [u128; 9] = [
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

fn next(s: u128, a: usize, b: usize) -> u128 {
  let x = s as u64;
  let y = (s >> 64) as u64;
  let u = y ^ y >> a;
  let v = x ^ y.rotate_right(b as u32);
  u as u128 ^ (v as u128) << 64
}

fn make_mat<F>(f: F) -> M128
where
  F: Fn(u128) -> u128
{
  let mut x = M128::ZERO;
  for j in 0 .. 128 {
    let y = f(1 << j);
    for i in 0 .. 128 {
      if y >> i & 1 != 0 {
        x.set(i, j, true);
      }
    }
  }
  x
}

fn pow_2_128(x: M128) -> M128 {
  let mut x = x;
  for _ in 0 .. 128 {
    x = x * x;
  }
  x
}

fn pow(x: M128, n: u128) -> M128 {
  if n == 0 { return M128::ID; }
  let mut n = n;
  let mut x = x;
  let mut y = M128::ID;
  while n != 1 {
    if n & 1 != 0 { y = x * y; }
    x = x * x;
    n = n / 2;
  }
  x * y
}

fn main() {
  assert!(FACTORS.iter().fold(1, |x, y| x * y) == N);

  for a in 1 ..= 63 {
    for b in 1 ..= 63 {
      let x = make_mat(|s| next(s, a, b));
      if x == pow_2_128(x) {
        let mut ok = true;
        for &p in FACTORS.iter() {
          if pow(x, N / p) == M128::ID {
            ok = false;
            println!("FAIL: a={a:<2} b={b:<2} p={p}");
            break;
          }
        }
        if ok {
          println!("OK:   a={a:<2} b={b:<2}");
        }
      }
    }
  }
}
