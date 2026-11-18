//! Writes random bytes to stdout using a miniaturized version of the dandelion
//! algorithm with a 32 bit state, operating on 16 bit words.

use std::io::Write as _;

struct Rng(u16, u16);

const fn widening_mul(x: u16, y: u16) -> u32 {
  x as u32 * y as u32
}

const fn lower(x: u32) -> u16 {
  x as u16
}

const fn upper(x: u32) -> u16 {
  (x >> 16) as u16
}

impl Rng {
  const fn new() -> Self {
    Self(0x67e3, 0x93c4)
  }

  const fn next(&mut self) -> u16 {
    let x = self.0;
    let y = self.1;
    self.0 = y ^ (x << 5);
    self.1 = x ^ (y.cast_signed() >> 3).cast_unsigned();
    let z = widening_mul(x, x);
    lower(z).wrapping_add(y) ^ upper(z)
  }

  fn fill<const N: usize>(&mut self, buf: &mut [u8; N]) {
    const { assert!(N % 2 == 0) };
    let mut a: &mut [u8] = buf;
    while a.len() >= 2 {
      a[.. 2].copy_from_slice(&self.next().to_le_bytes());
      a = &mut a[2 ..];
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
