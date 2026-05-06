//! Writes random bytes to stdout.

use dandelion::Rng;
use std::io::Write;
use std::io::stdout;
use std::mem::MaybeUninit;
use std::num::NonZeroU128;

fn main() {
  let mut rng = Rng::new(NonZeroU128::MIN);
  let mut buf = [MaybeUninit::uninit(); 1 << 16];
  let mut out = stdout().lock();

  loop {
    let buf = rng.fill_uninit(&mut buf);
    if let Err(_) = out.write_all(buf) { break }
  }
}
