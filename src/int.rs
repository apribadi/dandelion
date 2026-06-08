// utilities for working with integers

pub(crate) trait Widenable {
  type Wide;

  fn catenate(_: Self, _: Self) -> Self::Wide;

  fn widen(_: Self) -> Self::Wide;
}

pub(crate) trait Narrowable {
  type Narrow;

  fn lower(_: Self) -> Self::Narrow;

  fn upper(_: Self) -> Self::Narrow;
}

#[inline(always)]
pub(crate) fn catenate<T: Widenable>(x: T, y: T) -> T::Wide {
  T::catenate(x, y)
}

#[inline(always)]
pub(crate) fn widen<T: Widenable>(x: T) -> T::Wide {
  T::widen(x)
}

#[inline(always)]
pub(crate) fn widening_mul<T: Widenable>(x: T, y: T) -> T::Wide
where
  T::Wide: core::ops::Mul<Output = T::Wide>
{
  widen(x) * widen(y)
}

pub(crate) fn lower<T: Narrowable>(x: T) -> T::Narrow {
  T::lower(x)
}

pub(crate) fn upper<T: Narrowable>(x: T) -> T::Narrow {
  T::upper(x)
}

macro_rules! int_widenable_narrowable_impls {
  ($($narrow:ty, $wide:ty;)*) => {
    $(
    impl Widenable for $narrow {
      type Wide = $wide;

      #[inline(always)]
      fn catenate(x: Self, y: Self) -> Self::Wide {
        (x as Self::Wide) ^ ((y as Self::Wide) << Self::BITS)
      }

      #[inline(always)]
      fn widen(x: Self) -> Self::Wide {
        x as Self::Wide
      }
    }

    impl Narrowable for $wide {
      type Narrow = $narrow;

      #[inline(always)]
      fn lower(x: Self) -> Self::Narrow {
        x as Self::Narrow
      }

      #[inline(always)]
      fn upper(x: Self) -> Self::Narrow {
        (x >> Self::Narrow::BITS) as Self::Narrow
      }
    }
    )*
  };
}

int_widenable_narrowable_impls! {
  u16, u32;
  u32, u64;
  u64, u128;
}
