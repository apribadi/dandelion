//! Writes random bytes to stdout using a miniaturized version of the dandelion
//! algorithm using 32-bit integers.

use std::io::Write as _;

struct Rng(u32, u32);

#[inline(always)]
const fn widening_mul(x: u32, y: u32) -> u64 {
  x as u64 * y as u64
}

#[inline(always)]
const fn lower(x: u64) -> u32 {
  x as u32
}

#[inline(always)]
const fn upper(x: u64) -> u32 {
  (x >> 32) as u32
}

impl Rng {
  const fn new() -> Self {
    Self(0x7db0_c7a4, 0x93c4_67e3)
  }

  #[inline(always)]
  const fn next(&mut self) -> u32 {
    let x = self.0;
    let y = self.1;
    self.0 = y ^ (x << 10);
    self.1 = x ^ (y.cast_signed() >> 7).cast_unsigned();
    let z = widening_mul(x, x);
    lower(z).wrapping_add(y) ^ upper(z)
  }

  #[inline(never)]
  fn fill(&mut self, buf: &mut [u8]) {
    assert!(buf.len() % 8 == 0);
    let mut a = buf;
    while a.len() >= 8 {
      a[.. 4].copy_from_slice(&self.next().to_le_bytes());
      a = &mut a[4 ..];
      a[.. 4].copy_from_slice(&self.next().to_le_bytes());
      a = &mut a[4 ..];
    }
  }
}

fn main() {
  let mut rng = Rng::new();
  let mut buf = [0u8; 1 << 16];
  let mut out = std::io::stdout().lock();

  loop {
    rng.fill(&mut buf);
    if let Err(_) = out.write_all(&buf) { break }
  }
}
