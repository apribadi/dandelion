//! Finds RNG parameters with full period.

mod bmatrix;

/*
use bmatrix::BM128;

fn main() {
  print!("dandelion: (x, y) => (y ^ asr(x, a), x ^ lsl(y, b))\n");
  check(|x, y, a, b| (y ^ asr(x, a), x ^ lsl(y, b)));
  // print!("\n");
  // print!("orig: (x, y) => (y ^ lsr(y, a), x ^ ror(y, b))\n");
  // check(|x, y, a, b| (y ^ lsr(y, a), x ^ ror(y, b)));
  // print!("\n");
  // print!("seiran: (x, y) => (x ^ rol(y, a), x ^ lsl(y, b))\n");
  // check(|x, y, a, b| (x ^ rol(y, a), x ^ lsl(y, b)));
  // print!("\n");
  // print!("shioi: (x, y) => (y, y ^ (asr(x, a) ^ lsl(x, b)))\n");
  // check(|x, y, a, b| (y, y ^ (asr(x, a) ^ lsl(x, b))));
}

const FULL_PERIOD: u128 = u128::MAX;

const PRIME_FACTORS: &'static [u128] = &[
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

const _: () = {
  let mut x = 1;
  let mut i = 0;
  while i < PRIME_FACTORS.len() {
    x *= PRIME_FACTORS[i];
    i += 1;
  }
  assert!(x == FULL_PERIOD);
};

fn concat(x: u64, y: u64) -> u128 {
  x as u128 ^ (y as u128) << 64
}

fn lo(x: u128) -> u64 {
  x as u64
}

fn hi(x: u128) -> u64 {
  (x >> 64) as u64
}

fn asr(x: u64, a: usize) -> u64 {
  ((x as i64) >> a) as u64
}

fn lsl(x: u64, a: usize) -> u64 {
  x << a
}

#[allow(unused)]
fn lsr(x: u64, a: usize) -> u64 {
  x >> a
}

#[allow(unused)]
fn rol(x: u64, a: usize) -> u64 {
  x.rotate_left(a as u32)
}

#[allow(unused)]
fn ror(x: u64, a: usize) -> u64 {
  x.rotate_right(a as u32)
}

fn make_mat<F: Fn(u64, u64) -> (u64, u64)>(f: F) -> BM128 {
  let mut a = BM128::ZERO;
  for j in 0 .. 128 {
    let x = 1 << j;
    let y = f(lo(x), hi(x));
    let y = concat(y.0, y.1);
    for i in 0 .. 128 {
      if y & (1 << i) != 0 {
        a.set(i, j, true);
      }
    }
  }
  a
}

fn pow_2_128(x: BM128) -> BM128 {
  let mut x = x;
  for _ in 0 .. 128 {
    x = x * x;
  }
  x
}

fn pow(x: BM128, n: u128) -> BM128 {
  if n == 0 { return BM128::I; }
  let mut n = n;
  let mut x = x;
  let mut y = BM128::I;
  while n != 1 {
    if n & 1 != 0 { y = x * y; }
    x = x * x;
    n = n / 2;
  }
  x * y
}

fn check<F: Fn(u64, u64, usize, usize) -> (u64, u64)>(f: F) {
  for a in 1 .. 64 {
    for b in 1 .. 64 {
      let x = make_mat(|x, y| f(x, y, a, b));
      if pow_2_128(x) == x && pow(x, u128::MAX) == BM128::I {
        'fail: {
          for &p in PRIME_FACTORS.iter() {
            if pow(x, FULL_PERIOD / p) == BM128::I {
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
*/

use bmatrix::BM64;

fn main() {
  print!("dandelion: (x, y) => (y ^ asr(x, a), x ^ lsl(y, b))\n");
  check(|x, y, a, b| (y ^ asr(x, a), x ^ lsl(y, b)));
}

const FULL_PERIOD: u64 = u64::MAX;

const PRIME_FACTORS: &'static [u64] = &[
  3,
  5,
  17,
  257,
  65_537,
  641,
  6_700_417,
];

const _: () = {
  let mut x = 1;
  let mut i = 0;
  while i < PRIME_FACTORS.len() {
    x *= PRIME_FACTORS[i];
    i += 1;
  }
  assert!(x == FULL_PERIOD);
};

fn concat(x: u32, y: u32) -> u64 {
  x as u64 ^ (y as u64) << 32
}

fn lo(x: u64) -> u32 {
  x as u32
}

fn hi(x: u64) -> u32 {
  (x >> 32) as u32
}

fn asr(x: u32, a: usize) -> u32 {
  ((x as i32) >> a) as u32
}

fn lsl(x: u32, a: usize) -> u32 {
  x << a
}

fn make_mat<F: Fn(u32, u32) -> (u32, u32)>(f: F) -> BM64 {
  let mut a = BM64::ZERO;
  for j in 0 .. 64 {
    let x = 1 << j;
    let y = f(lo(x), hi(x));
    let y = concat(y.0, y.1);
    for i in 0 .. 64 {
      if y & (1 << i) != 0 {
        a.set(i, j, true);
      }
    }
  }
  a
}

fn pow_2_64(x: BM64) -> BM64 {
  let mut x = x;
  for _ in 0 .. 64 {
    x = x * x;
  }
  x
}

fn pow(x: BM64, n: u64) -> BM64 {
  if n == 0 { return BM64::I; }
  let mut n = n;
  let mut x = x;
  let mut y = BM64::I;
  while n != 1 {
    if n & 1 != 0 { y = x * y; }
    x = x * x;
    n = n / 2;
  }
  x * y
}

fn check<F: Fn(u32, u32, usize, usize) -> (u32, u32)>(f: F) {
  for a in 1 .. 32 {
    for b in 1 .. 32 {
      let x = make_mat(|x, y| f(x, y, a, b));
      if pow_2_64(x) == x && pow(x, u64::MAX) == BM64::I {
        'fail: {
          for &p in PRIME_FACTORS.iter() {
            if pow(x, FULL_PERIOD / p) == BM64::I {
              print!("FAIL a={} b={} p={}\n", a, b, p);
              break 'fail;
            }
          }
          print!("OK a={} b={}\n", a, b);
        }
      }
    }
  }
}

/*
use bmatrix::BM32;

fn main() {
  print!("dandelion: (x, y) => (y ^ asr(x, a), x ^ lsl(y, b))\n");
  check(|x, y, a, b| (y ^ asr(x, a), x ^ lsl(y, b)));
}

const FULL_PERIOD: u32 = u32::MAX;

const PRIME_FACTORS: &'static [u32] = &[
  3,
  5,
  17,
  257,
  65_537,
];

const _: () = {
  let mut x = 1;
  let mut i = 0;
  while i < PRIME_FACTORS.len() {
    x *= PRIME_FACTORS[i];
    i += 1;
  }
  assert!(x == FULL_PERIOD);
};

fn concat(x: u16, y: u16) -> u32 {
  x as u32 ^ (y as u32) << 16
}

fn lo(x: u32) -> u16 {
  x as u16
}

fn hi(x: u32) -> u16 {
  (x >> 16) as u16
}

fn asr(x: u16, a: usize) -> u16 {
  ((x as i16) >> a) as u16
}

fn lsl(x: u16, a: usize) -> u16 {
  x << a
}

fn make_mat<F: Fn(u16, u16) -> (u16, u16)>(f: F) -> BM32 {
  let mut a = BM32::ZERO;
  for j in 0 .. 32 {
    let x = 1 << j;
    let y = f(lo(x), hi(x));
    let y = concat(y.0, y.1);
    for i in 0 .. 32 {
      if y & (1 << i) != 0 {
        a.set(i, j, true);
      }
    }
  }
  a
}

fn pow_2_32(x: BM32) -> BM32 {
  let mut x = x;
  for _ in 0 .. 32 {
    x = x * x;
  }
  x
}

fn pow(x: BM32, n: u32) -> BM32 {
  if n == 0 { return BM32::I; }
  let mut n = n;
  let mut x = x;
  let mut y = BM32::I;
  while n != 1 {
    if n & 1 != 0 { y = x * y; }
    x = x * x;
    n = n / 2;
  }
  x * y
}

fn check<F: Fn(u16, u16, usize, usize) -> (u16, u16)>(f: F) {
  for a in 1 .. 16 {
    for b in 1 .. 16 {
      let x = make_mat(|x, y| f(x, y, a, b));
      if pow_2_32(x) == x && pow(x, u32::MAX) == BM32::I {
        'fail: {
          for &p in PRIME_FACTORS.iter() {
            if pow(x, FULL_PERIOD / p) == BM32::I {
              print!("FAIL a={} b={} p={}\n", a, b, p);
              break 'fail;
            }
          }
          print!("OK a={} b={}\n", a, b);
        }
      }
    }
  }
}
*/
