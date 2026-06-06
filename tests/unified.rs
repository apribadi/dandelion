#![allow(unused_must_use)]

//! tests

use dandelion::Rng;
use expect_test::expect;
use std::array;
use std::fmt::Write;
use std::num::NonZeroU128;
use std::num::NonZeroU32;
use std::num::NonZeroU64;

#[test]
fn test_api() {
  let mut out = String::new();
  let mut rng = Rng::new(NonZeroU128::MIN);
  let mut buf = [0u8; 9];

  write!(&mut out, "{:?}\n", Rng::from_u64(0));
  write!(&mut out, "{:?}\n", Rng::from_state(NonZeroU128::MIN));
  write!(&mut out, "{}\n", rng.state());
  write!(&mut out, "{}\n", rng.bernoulli(0.5));
  write!(&mut out, "{}\n", rng.uniform::<bool>());
  write!(&mut out, "{}\n", rng.uniform::<i32>());
  write!(&mut out, "{}\n", rng.uniform::<i64>());
  write!(&mut out, "{}\n", rng.uniform::<i128>());
  write!(&mut out, "{}\n", rng.uniform::<u32>());
  write!(&mut out, "{}\n", rng.uniform::<u64>());
  write!(&mut out, "{}\n", rng.uniform::<u128>());
  write!(&mut out, "{}\n", rng.uniform::<NonZeroU32>());
  write!(&mut out, "{}\n", rng.uniform::<NonZeroU64>());
  write!(&mut out, "{}\n", rng.uniform::<NonZeroU128>());
  write!(&mut out, "{}\n", rng.bounded::<u32>(5));
  write!(&mut out, "{}\n", rng.bounded::<u64>(5));
  write!(&mut out, "{}\n", rng.bounded::<usize>(5));
  write!(&mut out, "{}\n", rng.between::<i32>(1, 6));
  write!(&mut out, "{}\n", rng.between::<i64>(1, 6));
  write!(&mut out, "{}\n", rng.between::<isize>(1, 6));
  write!(&mut out, "{}\n", rng.between::<u32>(1, 6));
  write!(&mut out, "{}\n", rng.between::<u64>(1, 6));
  write!(&mut out, "{}\n", rng.between::<usize>(1, 6));
  write!(&mut out, "{}\n", rng.float::<f32>());
  write!(&mut out, "{}\n", rng.float::<f64>());
  write!(&mut out, "{}\n", rng.float_biunit::<f32>());
  write!(&mut out, "{}\n", rng.float_biunit::<f64>());
  write!(&mut out, "{:?}\n", { rng.fill(&mut buf); buf });
  write!(&mut out, "{:?}\n", rng.uniform::<[u8; 9]>());

  expect![[r#"
      Rng { .. }
      Rng { .. }
      5678343348967320566895565595396776803
      false
      false
      -717114844
      5104952185819808942
      120316169471476648951082652869775879076
      2247926904
      7056292569151375162
      115805744251978050665992955241888459289
      3652014633
      15645028836979063904
      82802582606552591543897730281005687440
      1
      1
      4
      2
      4
      1
      4
      2
      4
      0.49345005
      0.43060277777625683
      0.10148114
      0.13280605192768077
      [18, 216, 14, 156, 103, 65, 178, 57, 94]
      [221, 83, 241, 111, 57, 62, 163, 136, 80]
  "#]].assert_eq(out.drain(..).as_str());
}

#[cfg(feature = "getrandom")]
#[test]
fn test_api_getrandom() {
  let _ = Rng::from_operating_system();
}

#[cfg(feature = "rand_core")]
#[test]
fn test_api_rand_core() {
  let mut rng = <Rng as rand_core::SeedableRng>::from_seed([0; 16]);
  let _ = <Rng as rand_core::SeedableRng>::seed_from_u64(0);
  let _ = <Rng as rand_core::SeedableRng>::from_rng(&mut rng);
  let _ = <Rng as rand_core::Rng>::next_u32(&mut rng);
  let _ = <Rng as rand_core::Rng>::next_u64(&mut rng);
  <Rng as rand_core::Rng>::fill_bytes(&mut rng, &mut [0; 21]);
  let Ok(_) = <Rng as rand_core::TryRng>::try_next_u32(&mut rng);
  let Ok(_) = <Rng as rand_core::TryRng>::try_next_u64(&mut rng);
  let Ok(()) = <Rng as rand_core::TryRng>::try_fill_bytes(&mut rng, &mut [0; 21]);
}

#[cfg(feature = "thread_local")]
#[test]
fn test_api_thread_local() {
  let mut buf = [0u8; 21];
  let _ = dandelion::thread_local::bernoulli(0.5);
  let _ = dandelion::thread_local::uniform::<bool>();
  let _ = dandelion::thread_local::uniform::<i32>();
  let _ = dandelion::thread_local::uniform::<i64>();
  let _ = dandelion::thread_local::uniform::<i128>();
  let _ = dandelion::thread_local::uniform::<u32>();
  let _ = dandelion::thread_local::uniform::<u64>();
  let _ = dandelion::thread_local::uniform::<u128>();
  let _ = dandelion::thread_local::uniform::<NonZeroU32>();
  let _ = dandelion::thread_local::uniform::<NonZeroU64>();
  let _ = dandelion::thread_local::uniform::<NonZeroU128>();
  let _ = dandelion::thread_local::uniform::<[u8; 21]>();
  let _ = dandelion::thread_local::bounded::<u32>(5);
  let _ = dandelion::thread_local::bounded::<u64>(5);
  let _ = dandelion::thread_local::bounded::<usize>(5);
  let _ = dandelion::thread_local::between::<i32>(1, 6);
  let _ = dandelion::thread_local::between::<i64>(1, 6);
  let _ = dandelion::thread_local::between::<isize>(1, 6);
  let _ = dandelion::thread_local::between::<u32>(1, 6);
  let _ = dandelion::thread_local::between::<u64>(1, 6);
  let _ = dandelion::thread_local::between::<usize>(1, 6);
  let _ = dandelion::thread_local::float::<f32>();
  let _ = dandelion::thread_local::float::<f64>();
  let _ = dandelion::thread_local::float_biunit::<f32>();
  let _ = dandelion::thread_local::float_biunit::<f64>();
  dandelion::thread_local::fill(&mut buf);
}

#[test]
fn test_vectors() {
  let mut out = String::new();
  let mut rng = Rng::from_state(NonZeroU128::MIN);
  for _ in 0 .. 10 { write!(&mut out, "{:#018x}\n", rng.uniform::<u64>()); }

  expect![[r#"
      0x0000000000000001
      0x0000000000000001
      0x0000000000000081
      0x0000000000008001
      0x0000000010248051
      0x0000040110104009
      0x0100481631489551
      0x10034c891d376419
      0xe1264b8568380085
      0x02712a405d633118
  "#]].assert_eq(out.drain(..).as_str());

  let mut rng = Rng::new(NonZeroU128::MIN);
  for _ in 0 .. 10 { write!(&mut out, "{:#018x}\n", rng.uniform::<u64>()); }

  expect![[r#"
      0x9ea17ffbce96bce1
      0x03f517a50cda5a84
      0x70738695d541b224
      0x46d86ef73f24c4ae
      0x0fdd2ff9ac246ba4
      0x5a840d25cab9f146
      0x1965353a85fca478
      0x61ecfcb4af7f073a
      0x502e03cfeea3aa19
      0x571f6003d8b677b7
  "#]].assert_eq(out.drain(..).as_str());

  let mut rng = Rng::new(NonZeroU128::MIN);
  write!(&mut out, "{:?}\n", array::from_fn::<_, 25, _>(|_| rng.bounded::<u32>(5)));
  write!(&mut out, "{:?}\n", array::from_fn::<_, 25, _>(|_| rng.bounded::<u64>(5)));
  write!(&mut out, "{:?}\n", array::from_fn::<_, 25, _>(|_| rng.between::<i32>(1, 6)));
  write!(&mut out, "{:?}\n", array::from_fn::<_, 25, _>(|_| rng.between::<i64>(1, 6)));
  write!(&mut out, "{:?}\n", array::from_fn::<_, 25, _>(|_| rng.between::<u32>(1, 6)));
  write!(&mut out, "{:?}\n", array::from_fn::<_, 25, _>(|_| rng.between::<u64>(1, 6)));

  expect![[r#"
      [3, 0, 2, 1, 0, 2, 0, 2, 1, 2, 4, 5, 2, 1, 1, 1, 4, 1, 3, 0, 3, 1, 3, 1, 4]
      [0, 0, 1, 5, 3, 1, 1, 4, 1, 0, 3, 2, 3, 5, 4, 4, 3, 3, 0, 1, 5, 3, 5, 3, 5]
      [2, 5, 3, 2, 2, 3, 4, 2, 5, 3, 4, 2, 4, 2, 1, 1, 3, 6, 4, 2, 6, 5, 4, 1, 5]
      [5, 5, 5, 5, 1, 1, 3, 4, 6, 6, 2, 5, 4, 1, 1, 3, 5, 5, 5, 2, 6, 5, 3, 1, 5]
      [5, 2, 1, 4, 1, 6, 5, 2, 5, 1, 1, 4, 1, 3, 4, 5, 5, 1, 1, 4, 1, 2, 5, 6, 2]
      [6, 2, 1, 2, 2, 4, 6, 3, 5, 4, 4, 3, 2, 3, 3, 3, 3, 6, 5, 1, 6, 5, 3, 5, 5]
  "#]].assert_eq(out.drain(..).as_str());

  let mut rng = Rng::new(NonZeroU128::MIN);
  for _ in 0 .. 10 { write!(&mut out, "{:+.16}\n", rng.float::<f32>()); }
  write!(&mut out, "\n");
  for _ in 0 .. 10 { write!(&mut out, "{:+.16}\n", rng.float::<f64>()); }
  write!(&mut out, "\n");
  for _ in 0 .. 10 { write!(&mut out, "{:+.16}\n", rng.float_biunit::<f32>()); }
  write!(&mut out, "\n");
  for _ in 0 .. 10 { write!(&mut out, "{:+.16}\n", rng.float_biunit::<f64>()); }

  expect![[r#"
      +0.7606964111328125
      +0.0309171248227358
      +0.8785255551338196
      +0.5534800291061401
      +0.1239376068115234
      +0.7071548700332642
      +0.1984011232852936
      +0.7650447487831116
      +0.6264042854309082
      +0.6806449890136719

      +0.3479656163368836
      +0.3037625746348934
      +0.6872424324973326
      +0.4866698404374918
      +0.5788075507144995
      +0.3458461512617954
      +0.5680237446181363
      +0.4597912936779393
      +0.8903554085636458
      +0.3186114342153271

      -0.9609075784683228
      +0.6302964091300964
      -0.9044740200042725
      +0.4934500455856323
      -0.4306027889251709
      +0.1014811396598816
      +0.1328060477972031
      +0.4507524371147156
      -0.2574387490749359
      -0.9325182437896729

      +0.6646209719661166
      +0.4585849065398920
      -0.4671673539099718
      +0.4787477841330546
      +0.1773565112826935
      -0.7041121798569313
      +0.7394071293257584
      -0.9124324180972943
      -0.2232125295662452
      -0.6007147352837630
  "#]].assert_eq(out.drain(..).as_str());
}
