use std::array;
use std::fmt::Write;
use std::num::NonZeroU128;
use dandelion::Rng;
use expect_test::expect;

#[test]
fn test_api() {
  let mut rng = Rng::new([0; 15]);
  let _ = Rng::from_u64(0);
  let _ = Rng::from_state(NonZeroU128::MIN);
  let _ = rng.state();
  let _ = rng.split();
  let _ = rng.bernoulli(0.5);
  let _ = rng.bool();
  let _ = rng.i32();
  let _ = rng.i64();
  let _ = rng.u32();
  let _ = rng.u64();
  let _ = rng.bounded_u32(5);
  let _ = rng.bounded_u64(5);
  let _ = rng.between_i32(1, 6);
  let _ = rng.between_i64(1, 6);
  let _ = rng.between_u32(1, 6);
  let _ = rng.between_u64(1, 6);
  let _ = rng.f32();
  let _ = rng.f64();
  rng.bytes(&mut [0; 16]);
  let _ = rng.byte_array::<16>();
}

#[cfg(feature = "getrandom")]
#[test]
fn test_api_getrandom() {
  let _ = Rng::from_entropy();
}

#[cfg(feature = "rand_core")]
#[test]
fn test_api_rand_core() {
  let mut rng = <Rng as rand_core::SeedableRng>::from_seed([0; 16]);
  let _ = <Rng as rand_core::SeedableRng>::seed_from_u64(0);
  let _ = <Rng as rand_core::SeedableRng>::from_rng(&mut rng);
  let _ = <Rng as rand_core::RngCore>::next_u32(&mut rng);
  let _ = <Rng as rand_core::RngCore>::next_u64(&mut rng);
  <Rng as rand_core::RngCore>::fill_bytes(&mut rng, &mut [0; 16]);
  let _ = <Rng as rand_core::RngCore>::try_fill_bytes(&mut rng, &mut [0; 16]);
}

#[cfg(feature = "thread_local")]
#[test]
fn test_api_thread_local() {
  let _ = dandelion::thread_local::split();
  let _ = dandelion::thread_local::bernoulli(0.5);
  let _ = dandelion::thread_local::bool();
  let _ = dandelion::thread_local::i32();
  let _ = dandelion::thread_local::i64();
  let _ = dandelion::thread_local::u32();
  let _ = dandelion::thread_local::u64();
  let _ = dandelion::thread_local::bounded_u32(5);
  let _ = dandelion::thread_local::bounded_u64(5);
  let _ = dandelion::thread_local::between_i32(1, 6);
  let _ = dandelion::thread_local::between_i64(1, 6);
  let _ = dandelion::thread_local::between_u32(1, 6);
  let _ = dandelion::thread_local::between_u64(1, 6);
  let _ = dandelion::thread_local::f32();
  let _ = dandelion::thread_local::f64();
  dandelion::thread_local::bytes(&mut [0; 16]);
  let _ = dandelion::thread_local::byte_array::<16>();
}

#[test]
fn test_vectors() -> std::fmt::Result {
  let mut out = String::new();

  let mut rng = Rng::from_state(NonZeroU128::MIN);
  for _ in 0 .. 10 { write!(&mut out, "{:#018x}\n", rng.u64())?; }
  write!(&mut out, "\n")?;
  let mut rng = rng.split();
  for _ in 0 .. 10 { write!(&mut out, "{:#018x}\n", rng.u64())?; }

  expect![[r#"
      0x0000000000000001
      0x0000000000000001
      0x0200000000000001
      0x0008000100001001
      0x4008085100040001
      0x0405105000441012
      0x424a007521880121
      0x8845012082d51c52
      0x0842122c46a5a552
      0x0201011886721d42

      0xffaaf841aa7959b7
      0xb798630dd1377d07
      0x23f3fd8100c25d34
      0x3d75b7fb36f37583
      0x7c34a499955c8497
      0x8b722c5d1c5a2c53
      0x1d0d28f48c5f87e2
      0xe127f906c2fad5a3
      0x3e2e7ae17e539dff
      0xca480e0b98985977
  "#]].assert_eq(out.drain(..).as_str());

  let mut rng = Rng::new([0; 15]);
  write!(&mut out, "{:?}\n", array::from_fn::<_, 25, _>(|_| rng.bounded_u32(5)))?;
  write!(&mut out, "{:?}\n", array::from_fn::<_, 25, _>(|_| rng.bounded_u64(5)))?;
  write!(&mut out, "{:?}\n", array::from_fn::<_, 25, _>(|_| rng.between_i32(1, 6)))?;
  write!(&mut out, "{:?}\n", array::from_fn::<_, 25, _>(|_| rng.between_i64(1, 6)))?;
  write!(&mut out, "{:?}\n", array::from_fn::<_, 25, _>(|_| rng.between_u32(1, 6)))?;
  write!(&mut out, "{:?}\n", array::from_fn::<_, 25, _>(|_| rng.between_u64(1, 6)))?;

  expect![[r#"
      [4, 5, 3, 2, 4, 5, 2, 4, 1, 1, 2, 0, 3, 0, 3, 1, 3, 0, 3, 5, 0, 3, 3, 5, 0]
      [4, 0, 4, 4, 3, 5, 0, 2, 4, 4, 2, 0, 5, 4, 1, 0, 5, 0, 3, 3, 5, 3, 1, 0, 1]
      [5, 4, 1, 1, 3, 3, 1, 5, 2, 6, 5, 3, 1, 5, 6, 3, 4, 5, 5, 5, 2, 4, 2, 6, 3]
      [6, 5, 4, 2, 4, 2, 1, 1, 6, 5, 3, 2, 3, 3, 4, 5, 6, 5, 6, 6, 3, 1, 5, 6, 3]
      [6, 5, 3, 1, 2, 4, 6, 2, 1, 5, 6, 1, 2, 3, 5, 4, 2, 1, 5, 6, 6, 2, 3, 5, 3]
      [5, 1, 5, 2, 6, 3, 2, 6, 4, 5, 5, 2, 4, 4, 2, 2, 5, 6, 3, 5, 4, 1, 1, 6, 1]
  "#]].assert_eq(out.drain(..).as_str());

  let mut rng = Rng::new([0; 15]);
  for _ in 0 .. 10 { write!(&mut out, "{:+.16}\n", rng.f32())?; }
  write!(&mut out, "\n")?;
  for _ in 0 .. 10 { write!(&mut out, "{:+.16}\n", rng.f64())?; }

  expect![[r#"
      +0.3691386580467224
      +0.5822194218635559
      +0.1062258630990982
      +0.0308251641690731
      +0.2706349492073059
      +0.7396056056022644
      +0.2046746015548706
      +0.9753432869911194
      +0.6591350436210632
      +0.6380081772804260

      +0.7959530838734304
      +0.2868534074117475
      +0.9110512180269887
      +0.6720960569747567
      +0.7978147870855217
      +0.4164672987015255
      +0.7724244080838594
      +0.6010959683537979
      +0.5455702633086768
      +0.4945630052036953
  "#]].assert_eq(out.drain(..).as_str());

  Ok(())
}
