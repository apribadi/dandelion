// partial implementation of an RNG just with two independent copies of the
// dandelion state

use crate::rngs::RngForBench;
use std::num::NonZeroU128;

#[derive(Clone)]
pub(crate) struct DoubleDandelion {
  u: NonZeroU128,
  v: NonZeroU128,
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

#[inline(always)]
const fn hash(x: NonZeroU128) -> NonZeroU128 {
  const M: u128 = 0x93c4_67e3_7db0_c7a4_d1be_3f81_0152_cb57;
  let x = x.get();
  let x = x.wrapping_mul(M);
  let x = x.swap_bytes();
  let x = x.wrapping_mul(M);
  let x = x.swap_bytes();
  let x = x.wrapping_mul(M);
  unsafe { NonZeroU128::new_unchecked(x) }
}

impl RngForBench for DoubleDandelion {
  fn from_u64(n: u64) -> Self {
    Self {
      u: hash(NonZeroU128::new(concat(n, 1)).unwrap()),
      v: hash(NonZeroU128::new(concat(n, 2)).unwrap()),
    }
  }

  #[inline(always)]
  fn u64(&mut self) -> u64 {
    let s = self.u.get();
    let x = lo(s);
    let y = hi(s);
    let s = concat(y ^ asr(x, 4), x ^ lsl(y, 7));
    self.u = self.v;
    self.v = unsafe { NonZeroU128::new_unchecked(s) };
    y.wrapping_add(x.wrapping_mul(x)) ^ mulhi(x, x)
  }

  fn range_u64(&mut self, _: u64, _: u64) -> u64 {
    unimplemented!()
  }

  fn f64(&mut self) -> f64 {
    unimplemented!()
  }

  #[inline(never)]
  fn fill(&mut self, buf: &mut [u8]) {
    let mut p = buf.as_mut_ptr();
    let mut n = buf.len();
    if n % 32 != 0 { unimplemented!() } // no tail handling
    while n >= 32 {
      for _ in 0 .. 4 {
        let x = self.u64().to_le_bytes();
        unsafe { p.copy_from_nonoverlapping(&raw const x as _, 8) };
        p = unsafe { p.add(8) };
        n = n - 8;
      }
    }
  }

  fn shuffle<T>(&mut self, _: &mut [T]) {
    unimplemented!()
  }

  fn bernoulli(&mut self, _: f64) -> bool {
    unimplemented!()
  }
}
