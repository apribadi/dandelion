//! ???

fn main() {
  let mut g = dandelion::miniature::Rng16x2::new(std::num::NonZeroU32::MIN);
  let mut i = 0usize;
  let s = g.clone();

  loop {
    let _ = g.next();
    i += 1;
    if g == s { break }
  }

  print!("{}\n", i);
}
