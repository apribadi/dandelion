//! Finds RNG parameters with full period.

mod bmatrix;

use bmatrix::M128;

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

const PRIME_FACTORS: [u128; 9] = [
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

fn make_mat<F: Fn(u64, u64) -> (u64, u64)>(f: F) -> M128 {
  let mut a = M128::ZERO;
  for j in 0 .. 128 {
    let x = 1 << j;
    let y = f(lo(x), hi(x));
    let y = concat(y.0, y.1);
    for i in 0 .. 128 {
      if y >> i & 1 != 0 {
        a.set(i, j, true);
      }
    }
  }
  a
}

fn pow_2_128(x: M128) -> M128 {
  let mut x = x;
  for _ in 0 .. 128 {
    x = x * x;
  }
  x
}

fn pow(x: M128, n: u128) -> M128 {
  if n == 0 { return M128::I; }
  let mut n = n;
  let mut x = x;
  let mut y = M128::I;
  while n != 1 {
    if n & 1 != 0 { y = x * y; }
    x = x * x;
    n = n / 2;
  }
  x * y
}

fn check<F: Fn(u64, u64, usize, usize) -> (u64, u64)>(f: F) {
  for a in 1 ..= 63 {
    for b in 1 ..= 63 {
      let x = make_mat(|x, y| f(x, y, a, b));
      if pow_2_128(x) == x && pow(x, u128::MAX) == M128::I {
        'fail: {
          for &p in PRIME_FACTORS.iter() {
            if pow(x, FULL_PERIOD / p) == M128::I {
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
