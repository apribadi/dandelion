# Synopsis

Dandelion is a high performance non-cryptographic random number generator. Its
state consists of 128 bits and its cycle length is 2¹²⁸ - 1. It is suited to
producing random numbers for algorithms applied to simulation, testing,
statistics, data structures, etc.

Particular attention has been paid to keeping the footprint of the generator
small (e.g. small code size, small state) so that it can be used to generate a
few random numbers at a time without negatively impacting the performance of
surrounding code.

You can deterministically initialize a `Rng` from a seed with `Rng::new` or
`Rng::from_u64`, or you can non-deterministically initialize a `Rng` from a
platform provided source of randomness with `Rng::from_operating_system`. You
can also access a thread-local `Rng` instance with the
`dandelion::thread_local` module.

Much of the API for generating random numbers is organized around a few sealed
traits. For example, the `Rng::uniform` method can generate a value of any type
that satisfies the `RandomUniform` trait.

```
let mut g = dandelion::Rng::new(std::num::NonZeroU128::MIN);
let a: u32 = g.uniform();
let b: u64 = g.between(1, 6);
let c: f64 = g.float();
let d = g.bool(0.75);
let e: [i16; 3] = g.uniform();
std::assert_matches!(a, 3465985249);
std::assert_matches!(b, 6);
std::assert_matches!(c, 0.9333146823381274);
std::assert_matches!(d, true);
std::assert_matches!(e, [-11392, 19564, 12621]);
```

# A Peek at the Assembly

The `aarch64` assembly for generating one `u64` looks like:

```text
next:
    ldp x8, x9, [x0]
    eor x10, x9, x8, lsl #7
    eor x11, x8, x9, asr #4
    stp x10, x11, [x0]
    madd x9, x8, x8, x9
    umulh x8, x8, x8
    eor x0, x9, x8
    ret
```

# Core Algorithm

The core of the random number generator has two parts: a state transition
function `T: (u64, u64) -> (u64, u64)` and an output function `F: (u64, u64) ->
u64`.

The state transition function `T` is defined

```text
T(x, y) = (y ^ shl(x, 7), x ^ asr(y, 4))
```

and the output function `F` is defined

```text
F(x, y) = (x * x + y) ^ mulhi(x, x)
```

where `shl` is shift left, `asr` is arithmetic shift right, and `mulhi` produces
the high half of an unsigned full multiply. We further require the state to not
be all zeros, i.e. `(x, y) != (0, 0)`.

There are two things to note about these functions. First, `T` can be thought of
as a linear transformation on the space F₂¹²⁸ of vectors of 128 bits. Second,
for any fixed value of `x`, the function `y => (x * x + y) ^ mulhi(x, x)` is a
permutation of F₂⁶⁴.

# The Period of the State Transition

We demonstrate that the state transition function has full period 2¹²⁸ - 1. We
can do the necessary computations on matrices over the field F₂. Let `A` be the
binary matrix representing our transition, and let `I` be the identity matrix.
Then it is sufficient to show that

```text
pow(A, 2¹²⁸ - 1) = I
```

and

```text
pow(A, (2¹²⁸ - 1) / p) != I
```

for all prime divisors `p` of 2¹²⁸ - 1. We can compute powers `pow(A, n)` in
`O(log(n))` time, so these computations are feasible.

The Mersenne number 2¹²⁸ - 1 factors into Fermat numbers as

```text
2¹²⁸ - 1 =
    (2¹ + 1)
    * (2² + 1)
    * (2⁴ + 1)
    * (2⁸ + 1)
    * (2¹⁶ + 1)
    * (2³² + 1)
    * (2⁶⁴ + 1)
```

and the prime factorizations of small Fermat numbers are well known. The full
prime factorization of 2¹²⁸ - 1 is

```text
2¹²⁸ - 1 =
    3
    * 5
    * 17
    * 257
    * 65_537
    * 641
    * 6_700_417
    * 274_177
    * 67_280_421_310_721
```

The complete set of `(α, β)` such that

```text
(x, y) ⇒  (y ^ shl(x, α), x ^ asr(y, β))
```

has full period (see `cargo run --release --example period`) is

```text
α=7 β=4
α=37 β=26
```

and we use `(7, 4)` because it has the less sparse transition matrix.

# Maximal Equidistribution

The generator is maximally 1-equidistributed. More precisely, over the full
cycle of 2¹²⁸ - 1 states, the zero output occurs 2⁶⁴ - 1 times and every other
output occurs 2⁶⁴ times.

This follows from the previously noted fact that for any fixed value of `x`, the
function `y => (x * x + y) ^ mulhi(x, x)` is a permutation of F₂⁶⁴, and that the
"missing" state is `(0, 0)`.

# Design Considerations

A state size of 128 bits was selected to be barely large enough to suffice for
almost all non-cryptographic applications. A 64 bit state is small enough that
it isn't too difficult to detect statistical weaknesses by brute force, and a
192 bit state is larger than necessary.

The state transition function is similar to those in the xorshift extended
family, but is weakened as much as possible while retaining a full period. It is
closest in design to those in the somewhat obscure "shioi" and "seiran" random
number generators.

The output function contains a `u64, u64 -> u128` full multiply. This does a
lot of mixing while being relatively cheap on current CPUs. This kind of mixer
was inspired by similar constructions in the "mum-hash" and "wyhash" libraries.

It would be straightforward to increase the throughput of the algorithm by
enlarging the state (whether the state is represented as a set of scalars or
SIMD vectors), but maximizing the throughput of bulk generation is not the
primary design objective. One could imagine a generator design that instead
updates multiple copies of the state at the same time to take advantage of
either instruction level parallelism or vector operations.

# Benchmarks

We compare dandelion with a few other random number generators.

- The xoroshiro128++ algorithm, as implemented in `rand_xoshiro` crate.
- The pcg-dxsm algorithm with a 64 bit multiplier, as implemented in the
  `rand_pcg` crate.
- The `rand` crate's `SmallRng`, which currently uses xoshiro256++.
- THe `rand` crate's thread-local generator, which currently uses 12-round
  chacha.

The first two algorithms are notable for each being a non-cryptographic random
number generator with a 128 bit state and no known statistical flaws. The last
two algorithms are de-facto standards in the Rust crate ecosystem.

The alternative algorithms use the `rand` crate to implement generating an
integer in a range, generating a float, filling a byte buffer, and shuffling a
slice.

Units are nanoseconds per generated number (lower is better).

```text
                        uniform  uniform-ni  between  between-ni  float  bool   fill   fill-sm  shuffle
dandelion               0.530    1.794       0.941    2.250       0.530  0.572  0.517  2.052    0.598
xoroshiro128++          0.961    2.315       2.243    2.638       0.986  0.970  0.988  2.148    1.221
pcg-dxsm                1.138    2.751       1.688    3.161       1.150  1.168  1.135  2.417    1.309
rand-small-rng          0.660    2.114       2.009    2.423       0.697  0.671  0.651  2.382    1.262
dandelion-thread-local  0.534    1.886       1.060    2.468       0.597  0.610  0.517  3.067    0.587
rand-thread-local       3.777    5.274       6.077    6.554       5.360  3.930  2.518  7.129    1.751
```

The benchmarks labeled "`-ni`" ensure that random number generation is not
inlined into the benchmark loop. This measures performance in a scenario where
certain parts of random number generation cannot be amortized, as can happen
when random number generation is embedded within a larger algorithm. For
instance, the compiler might not be able to do store-to-load forwarding of the
generator state or to hoist loading of large constants.

The benchmarks were run on an Apple M5. The data was generated with the
following command:

```text
cargo bench unified --all-features
```

# Statistical Tests

Dandelion has been run against the PractRand and SmokeRand test suites, which
have not detected any statistical flaws. Furthermore, there is no reason to
believe that the tests are particularly close to failing; it is *not* the case
that the algorithm was designed by making successive modifications until
statistical tests passed.

The `examples/dandelion` executable writes random bytes to stdout, which you can
use to run your own statistical tests.

# Portability

Generator output does not depend on platform endianness.

Word-sized output (e.g. `usize`, `isize`, ...) is produced as if the
corresponding fixed size integer had been produced instead.

# Smaller Word and State Size Variants

There exist generators with the same structure that operate on 32 bit and 16 bit
words, with 64 bit and 32 bit states, respectively. They might be useful on a
machine constrained to a small word size. Of course, the smaller state size
means that they will be distinguishable from random with fewer samples.

These shifts produce generators with full period.

```text
T(x, y) = (y ^ shl(x, 10), x ^ asr(y, 7)) // 32 bit word, 64 bit state
```

```text
T(x, y) = (y ^ shl(x, 5), x ^ asr(y, 3)) // 16 bit word, 32 bit state
```

```text
F(x, y) = (x * x + y) ^ mulhi(x, x) // same for any word size
```

The `examples/dandelion-mini64` and `examples/dandelion-mini32` executables
write random bytes with these two generators.

# Cargo Features

- `getrandom`: Adds `Rng::from_operating_system`, which uses the `getrandom`
  crate to get a seed from the operating system.

- `rand_core`: Implements the traits `TryRng`, `Rng`, and `SeedableRng` from the
  `rand_core` crate.

- `thread_local`: Adds a thread-local random number generator. Requires `std`.
