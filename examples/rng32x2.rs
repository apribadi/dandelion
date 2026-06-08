//! Writes random bytes to stdout.

use std::io::Write as _;

fn main() {
  let mut rng = dandelion::miniature::Rng32x2::new(std::num::NonZeroU64::MIN);
  let mut buf = [0u8; 1 << 16];
  let mut out = std::io::stdout().lock();

  loop {
    for i in 0 .. buf.len() / 4 {
      buf[4 * i .. 4 * i + 4].copy_from_slice(&rng.next().to_le_bytes());
    }
    if let Err(_) = out.write_all(&buf) { break }
  }
}
