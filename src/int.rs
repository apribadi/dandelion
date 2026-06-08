// int utils

pub(crate) trait Widenable {
  type Wide;

  fn catenate(_: Self, _: Self) -> Self::Wide;

  fn widening_mul(_: Self, _: Self) -> Self::Wide;
}

pub(crate) trait Narrowable {
  type Narrow;

  fn lo(_: Self) -> Self::Narrow;

  fn hi(_: Self) -> Self::Narrow;
}

#[inline(always)]
pub(crate) fn catenate<T: Widenable>(x: T, y: T) -> T::Wide {
  T::catenate(x, y)
}

#[inline(always)]
pub(crate) fn widening_mul<T: Widenable>(x: T, y: T) -> T::Wide {
  T::widening_mul(x, y)
}

pub(crate) fn lo<T: Narrowable>(x: T) -> T::Narrow {
  T::lo(x)
}

pub(crate) fn hi<T: Narrowable>(x: T) -> T::Narrow {
  T::hi(x)
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
      fn widening_mul(x: Self, y: Self) -> Self::Wide {
        (x as Self::Wide) * (y as Self::Wide)
      }
    }

    impl Narrowable for $wide {
      type Narrow = $narrow;

      #[inline(always)]
      fn lo(x: Self) -> Self::Narrow {
        x as Self::Narrow
      }

      #[inline(always)]
      fn hi(x: Self) -> Self::Narrow {
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
