//! Writes random bytes to stdout.

use std::io::Write as _;

fn main() {
  let mut rng = dandelion::Rng::new(std::num::NonZeroU128::MIN);
  let mut buf = [0u8; 1 << 16];
  let mut out = std::io::stdout().lock();

  loop {
    rng.fill(&mut buf);
    if let Err(_) = out.write_all(&buf) { break }
  }
}
