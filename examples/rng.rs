//! Writes random bytes to stdout.

use std::io::Write;
use dandelion::Rng;

fn main() {
  let mut rng = Rng::new([0; 15]);
  let mut out = std::io::stdout().lock();
  let buf = &mut [0u8; 65_536];

  loop {
    rng.bytes(buf);
    if let Err(_) = out.write_all(buf) { break; }
  }
}
