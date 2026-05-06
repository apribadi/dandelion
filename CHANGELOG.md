# 0.3.0

- Changed both state transition and output functions in the core generator
  algorithm.
- Changed sampling algorithm for an integer range to consume a variable number
  of random words.
- Changed `Rng::new` to take a `NonZeroU128` instead of `[u8; 15]`.
- Renamed `Rng::bytes` to `Rng::fill`.
- Renamed `between_XXX` functions to `range_XXX`.
- Added `Rng::bounded_usize`.
- Added `Rng::range_isize`.
- Added `Rng::range_usize`.
- Added `Rng::i128`.
- Added `Rng::u128`.
- Added `Rng::non_zero_u32`.
- Added `Rng::non_zero_u64`.
- Added `Rng::non_zero_u128`.
- Added `Rng::biunit_f32`.
- Added `Rng::biunit_f64`.
- Added `Rng::fill_unchecked`.
- Added `Rng::fill_uninit`.
- Added `thread_local::bounded_usize`.
- Added `thread_local::range_isize`.
- Added `thread_local::range_usize`.
- Added `thread_local::i128`.
- Added `thread_local::u128`.
- Added `thread_local::non_zero_u32`.
- Added `thread_local::non_zero_u64`.
- Added `thread_local::non_zero_u128`.
- Added `thread_local::biunit_f32`.
- Added `thread_local::biunit_f64`.
- Added `thread_local::fill_unchecked`.
- Added `thread_local::fill_uninit`.
- Added implementation of `Debug` for `Rng`.
- Removed `Rng::split`.
- Removed `thread_local::split`.
- Updated `rand_core` to version 0.10.x.

# 0.2.0

- Updated `rand_core` to version 0.9.0.
- Renamed `from_entropy` to `from_operating_system`.
