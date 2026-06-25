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
  write!(&mut out, "{}\n", rng.bool(0.5));
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
      true
      -1033184424
      3159222901446564760
      -165052971283198763473388613640937221248
      1728172624
      13868688463880659098
      77715832314157765587541157946095867589
      4237480633
      15396245247621893394
      123235359504960677444995738177166332757
      5
      5
      2
      2
      6
      3
      3
      5
      4
      0.11772203
      0.29332885829100935
      -0.86949503
      0.08075314980176614
      [198, 136, 131, 201, 96, 175, 248, 22, 200]
      [212, 104, 214, 245, 109, 25, 185, 173, 200]
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
  let _ = dandelion::thread_local::bool(0.5);
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
      0x0000000000004001
      0x0000000010008081
      0x0000040000004009
      0x0100080130248451
      0x0000080010028009
      0x11028815223439d5
      0x0010040490120211
      0xe317498d281508a5
      0x41122e4dd5f25bf7
  "#]].assert_eq(out.drain(..).as_str());

  let mut rng = Rng::new(NonZeroU128::MIN);
  for _ in 0 .. 10 { write!(&mut out, "{:#018x}\n", rng.uniform::<u64>()); }

  expect![[r#"
      0x9ea17ffbce96bce1
      0xe6536c9dc929dc31
      0x7776db02c26adb58
      0x2bd7d07d97005b98
      0x845a314d4c6cd380
      0x83d3f43aa03a5c4e
      0x32e3bf606701d250
      0xc0777aadae2a209a
      0x16d019024bd896c5
      0x3a77865d45056721
  "#]].assert_eq(out.drain(..).as_str());

  let mut rng = Rng::new(NonZeroU128::MIN);
  write!(&mut out, "{:?}\n", array::from_fn::<_, 25, _>(|_| rng.bounded::<u32>(5)));
  write!(&mut out, "{:?}\n", array::from_fn::<_, 25, _>(|_| rng.bounded::<u64>(5)));
  write!(&mut out, "{:?}\n", array::from_fn::<_, 25, _>(|_| rng.between::<i32>(1, 6)));
  write!(&mut out, "{:?}\n", array::from_fn::<_, 25, _>(|_| rng.between::<i64>(1, 6)));
  write!(&mut out, "{:?}\n", array::from_fn::<_, 25, _>(|_| rng.between::<u32>(1, 6)));
  write!(&mut out, "{:?}\n", array::from_fn::<_, 25, _>(|_| rng.between::<u64>(1, 6)));

  expect![[r#"
      [3, 5, 2, 1, 3, 3, 1, 4, 0, 1, 4, 5, 1, 2, 5, 5, 2, 1, 5, 2, 2, 4, 3, 5, 5]
      [3, 0, 0, 2, 4, 1, 4, 2, 0, 0, 0, 4, 0, 1, 0, 0, 0, 0, 3, 1, 3, 5, 0, 2, 0]
      [2, 5, 3, 5, 2, 6, 5, 5, 5, 2, 5, 2, 4, 1, 5, 5, 2, 4, 3, 1, 5, 4, 6, 4, 2]
      [5, 5, 1, 6, 4, 4, 4, 5, 6, 6, 5, 6, 6, 5, 1, 1, 5, 2, 5, 6, 4, 1, 3, 3, 6]
      [3, 1, 4, 1, 4, 6, 5, 5, 5, 6, 6, 1, 1, 3, 2, 4, 3, 4, 6, 1, 3, 5, 6, 1, 5]
      [3, 3, 5, 1, 4, 2, 1, 3, 3, 1, 1, 1, 4, 3, 5, 5, 4, 5, 6, 5, 1, 4, 3, 3, 2]
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
      +0.2005790919065475
      +0.9333146810531616
      +0.3425236344337463
      +0.9659975171089172
      +0.9700942039489746
      +0.3975753188133240
      +0.4963537752628326
      +0.1782256364822388
      +0.4567725956439972

      +0.4490015185500424
      +0.3307357454408720
      +0.6616692352142431
      +0.7243123445981748
      +0.1175145207670947
      +0.1782554338711681
      +0.7885185913191323
      +0.5206548789018417
      +0.0138647178463698
      +0.8251597965956485

      +0.8012244701385498
      -0.3467440009117126
      -0.7984890341758728
      -0.1177220270037651
      -0.2933288514614105
      -0.8694950342178345
      +0.0807531476020813
      +0.1794642657041550
      +0.6984339952468872
      -0.6427887082099915

      +0.4312064103475206
      -0.4986203998241978
      +0.9155396244672416
      +0.3274135787962369
      +0.2828936126950808
      +0.2873729949774235
      -0.5751770748313196
      +0.2156169867614086
      +0.6528959469751927
      +0.0822904760668782
  "#]].assert_eq(out.drain(..).as_str());
}

#[test]
fn test_fill() {
  // let mut out = String::new();
  let mut rng = Rng::new(NonZeroU128::MIN);
  let mut buf = [0u16; 100];
  rng.fill(&mut buf);
}
