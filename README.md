Dandelion is a high performance non-cryptographic pseudo random number
generator. Its state consists of 128 bits and its cycle length is 2¹²⁸ - 1.

It is suited to producing random numbers for algorithms applied to simulation,
testing, statistics, data structures, etc.

Particular attention has been paid to keeping the footprint of the generator
small (e.g. small state, small code size, no large constants) so that it can be
used to generate a few random numbers at a time without negatively impacting
the performance of surrounding code.

# Example

```
use dandelion::Rng;
use std::num::NonZeroU128;
let mut g = Rng::new(NonZeroU128::MIN);
let a: u64 = g.uniform();
let b: u64 = g.between(1, 6);
let c: f64 = g.float();
let d = g.bernoulli(0.75);
assert!(a == 11430558048722533601);
assert!(b == 1);
assert!(c == 0.8785255653006182);
assert!(d == true);
```

# A Peek at the Assembly

On `aarch64` generating one `u64` looks like:

```text
u64:
    ldp x8, x9, [x0]
    eor x10, x9, x8, asr #4
    eor x11, x8, x9, lsl #7
    stp x10, x11, [x0]
    madd x9, x8, x8, x9
    umulh x8, x8, x8
    eor x0, x9, x8
    ret
```

# Algorithm

The core of the generator has two parts: a state transition function `T: (u64,
u64) -> (u64, u64)` and an output function `F: (u64, u64) -> u64`. Then the
`k`th output is

```text
F(T(T( ... T(x₀, y₀) ... )))
  \________/
   k times
```

where `(x₀, y₀)` is the initial state.

The state transition function `T` is defined

```text
T(x, y) = (y ^ asr(x, 4), x ^ lsl(y, 7))
```

and the output function `F` is defined

```text
F(x, y) = (y + x * x) ^ mulhi(x, x)
```

where `asr` is arithmetic shift right, `lsl` is logical shift left, and `mulhi`
produces the high part of the result an unsigned 64-bit full multiplication.
We further require the state to not be all zeros, i.e. `(x, y) ≠ (0, 0)`.

There are two things to note about these functions. First, `T` can be thought
of as a linear transformation on the space F₂¹²⁸ of vectors of 128 bits.
Second, for any fixed value of `x`, the function `y ⇒ (y + x * x) ^ mulhi(x, x)`
is a permutation of F₂⁶⁴.

# The Period of the State Transition

We will demonstrate that the state transition function has full period 2¹²⁸ - 1.
We can do the necessary computations on matrices over the field F₂. Let `A`
be the binary matrix representing our transition. Then it is sufficient to
show that

```text
pow(A, 2¹²⁸ - 1) = I
```

and

```text
pow(A, (2¹²⁸ - 1) / p) ≠ I
```

for all prime divisors `p` of 2¹²⁸ - 1, where `I` is the identity matrix. We
can compute powers `pow(A, n)` in `O(log(n))` time, so these computations are
feasible.

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

and the prime factorizations of small(er) Fermat numbers are well known. The
full prime factorization of 2¹²⁸ - 1 is

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
(x, y) ⇒ (y ^ asr(x, α), x ^ lsl(y, β))
```

has full period (see code in examples/period) is

```text
α=4 β=7
α=26 β=37
```

and we use `(4, 7)` because it has the less sparse transition matrix.

# Maximal Equidistribution

The generator is maximally 1-equidistributed. More precisely, over the full
2¹²⁸ - 1 cycle, zero occurs 2⁶⁴ - 1 times and every other number occurs 2⁶⁴
times.

This follows from the previously noted fact that for any fixed value of `x`,
the function `y ⇒ (y + x * x) ^ mulhi(x, x)` is a permutation of F₂⁶⁴.

# Design Considerations

A state size of 128 bits was selected because 64 bits might be too small for
some non-cryptographic applications.

The state transition function is similar to those in the xorshift extended
family, but is weakened as much as possible while retaining a full period. It
is closest in design to those in the somewhat obscure "shioi" and "seiran"
random number generators.

The output function contains a `u64, u64 -> u128` full multiply. This does a
lot of mixing while being relatively cheap on current CPUs. This kind of mixer
was inspired by similar constructions in the "mum-hash" and "wyhash" libraries.

It would be straightforward to increase the throughput of the algorithm by
enlarging the state (whether the state is represented as scalars or as SIMD
vectors), but maximizing the throughput of bulk generation is not the primary
design objective. For example, one could imagine updating multiple copies of
the state at the same time to take advantage of instruction level parallelism
or vectorization.

# Benchmarks

We compare dandelion with two other non-cryptographic random number generators
that each have 128-bit states and no known statistical flaws:

- the pcg-dxsm algorithm with a 64-bit multiplier, as implemented in the
  `rand_pcg` crate, and

- the xoroshiro128++ algorithm, as implemented in `rand_xoshiro` crate.

Both alternatives use the `rand` crate to implement generating integers in
a range, generating floats, and filling byte buffers.

The benchmarks labeled "noinline" ensure that random number generation is not
inlined into the benchmark loop. This measures performance in a scenario where
certain parts of random number generation cannot be amortized, as can happen
when random number generation is embedded within a larger algorithm. For
instance, the compiler might not be able to do store-to-load forwarding of the
generator state or to hoist loading of large constants.

The benchmarks were run on an Apple M1 Macbook Air.

```text
dandelion
0.883 ns/word - u64
2.399 ns/word - u64 noinline
1.721 ns/word - range_u64
3.185 ns/word - range_u64 noinline
0.890 ns/word - f64
2.442 ns/word - f64 noinline
0.858 ns/word - fill large
2.348 ns/word - fill small
2.504 ns/word - fill small noinline
0.945 ns/word - shuffle

pcgdxsm128
1.643 ns/word - u64
4.066 ns/word - u64 noinline
2.483 ns/word - range_u64
4.897 ns/word - range_u64 noinline
1.630 ns/word - f64
4.054 ns/word - f64 noinline
1.654 ns/word - fill large
4.071 ns/word - fill small
4.693 ns/word - fill small noinline
2.067 ns/word - shuffle

xoroshiro128++
1.463 ns/word - u64
3.328 ns/word - u64 noinline
3.826 ns/word - range_u64 (n.b. pessimized by llvm)
3.898 ns/word - range_u64 noinline
1.463 ns/word - f64
3.458 ns/word - f64 noinline
1.474 ns/word - fill large
3.758 ns/word - fill small
4.380 ns/word - fill small noinline
1.970 ns/word - shuffle
```

# Statistical Tests

Dandelion passed the full v0.47 SmokeRand test suite, and there is no reason to
believe that the test is close to failing. In particular, it is *not* the case
that the algorithm was designed by successively adding complexity until
statistical tests passed.

The `examples/rng` executable writes random bytes to stdout, which you can use
to run your own statistical tests.

# Cargo Features

- `getrandom`: Adds `Rng::from_operating_system`, which uses the `getrandom`
  crate to get a seed from the operating system.

- `rand_core`: Implements the traits `TryRng`, `Rng`, and `SeedableRng` from the
  `rand_core` crate.

- `thread_local`: Adds a thread-local random number generator. Requires `std`.
