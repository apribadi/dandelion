# Synopsis

Dandelion is a high performance non-cryptographic random number generator. Its
state consists of 128 bits and its cycle length is 2¹²⁸ - 1. It is suited to
producing random numbers for algorithms applied to simulation, testing,
statistics, data structures, etc.

Particular attention has been paid to keeping the footprint of the generator
small (e.g. small code size, small state, no large constants) so that it can be
used to generate a few random numbers at a time without negatively impacting
the performance of surrounding code.

You can initialize a `Rng` from a deterministic seed with `Rng::new` or
`Rng::from_u64`, or you can initialize a `Rng` from a platform provided source
of randomness with `Rng::from_operating_system`. You can also access a
thread-local `Rng` instance with the `dandelion::thread_local` module.

Much of the API for generating random numbers is organized around a few sealed
traits. For example, the `Rng::uniform` method can be used to generate a value
of any type that satisfies the `RandomUniform` trait.

```
let mut g = dandelion::Rng::new(std::num::NonZeroU128::MIN);
let a: u32 = g.uniform();
let b: u64 = g.between(1, 6);
let c: f64 = g.float();
let d = g.bernoulli(0.75);
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
T(x, y) = (y ^ lsl(x, 7), x ^ asr(y, 4))
```

and the output function `F` is defined

```text
F(x, y) = (x * x + y) ^ umulh(x, x)
```

where `lsl` is logical shift left, `asr` is arithmetic shift right, and `umulh`
produces the high half of an unsigned full multiply. We further require the
state to not be all zeros, i.e. `(x, y) != (0, 0)`.

There are two things to note about these functions. First, `T` can be thought
of as a linear transformation on the space F₂¹²⁸ of vectors of 128 bits.
Second, for any fixed value of `x`, the function `y => (x * x + y) ^ mulhi(x,
x)` is a permutation of F₂⁶⁴.

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
(x, y) ⇒ (y ^ lsl(x, α), x ^ asr(y, β))
```

has full period (see code in examples/period) is

```text
α=7 β=4
α=37 β=26
```

and we use `(7, 4)` because it has the less sparse transition matrix.

# Maximal Equidistribution

The generator is maximally 1-equidistributed. More precisely, over the full
cycle of 2¹²⁸ - 1 states, the zero output occurs 2⁶⁴ - 1 times and every other
output occurs 2⁶⁴ times.

This follows from the previously noted fact that for any fixed value of `x`,
the function `y => (x * x + y) ^ mulhi(x, x)` is a permutation of F₂⁶⁴, and
that the "missing" state is `(0, 0)`.

# Design Considerations

A state size of 128 bits was selected to be barely large enough to suffice for
almost all non-cryptographic applications. A 64 bit state is small enough that
it isn't too difficult to detect weaknesses by brute force, and a 192 bit state
is larger than necessary.

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
design objective. One could imagine a bulk generator design that updates
multiple copies of the state at the same time to take advantage of instruction
level parallelism or vectorization.

# Benchmarks

TODO: produce benchmark table from a script

We compare dandelion with two other non-cryptographic random number generators
that each have 128 bit states and no known statistical flaws:

- the pcg-dxsm algorithm with a 64 bit multiplier, as implemented in the
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

TODO: redo tests
TODO: test smaller state rngs

Dandelion passed the full v0.47 SmokeRand test suite, and there is no reason to
believe that the test is close to failing. In particular, it is *not* the case
that the algorithm was designed by successively adding complexity until
statistical tests passed.

The `examples/rng` executable writes random bytes to stdout, which you can use
to run your own statistical tests.

# Sampling From a Range

TODO: Canon's method for u32 and u64

```text
bounded_u64:
    ldp x8, x11, [x0]
    eor x9, x11, x8, lsl #7
    eor x10, x8, x11, asr #4
    umulh x12, x8, x8
    madd x8, x8, x8, x11
    eor x8, x8, x12
    adds x11, x1, #1
    mul x12, x8, x11
    umulh x13, x8, x11
    csel x8, x8, x13, hs
    cmn x12, x1
    b.lo L3
    mov x8, x13
L2:
    mov x13, x9
    eor x9, x10, x9, lsl #7
    madd x14, x13, x13, x10
    eor x10, x13, x10, asr #4
    umulh x13, x13, x13
    eor x13, x14, x13
    umulh x14, x13, x11
    adds x14, x12, x14
    mul x12, x13, x11
    cinc x8, x8, hs
    cmn x14, #1
    b.eq L2
L3:
    stp x9, x10, [x0]
    mov x0, x8
    ret
```

# Sampling Floating-Point Numbers

TODO: explain

```text
f64:
    ldp x8, x9, [x0]
    eor x10, x9, x8, lsl #7
    eor x11, x8, x9, asr #4
    stp x10, x11, [x0]
    umulh x10, x8, x8
    madd x8, x8, x8, x9
    eor x8, x8, x10
    scvtf d0, x8, #63
    fabs d0, d0
    ret
```

# Sampling Multiple Uniform Small Integers

TODO: explain

# Thread Local Generator

TODO: explain

# Portability for Platform Endianness and Word Size

Generator output does not depend on platform endianness.

Word-sized output (e.g. `usize`, `isize`, ...) is produced as if the
corresponding fixed size integers had been produced instead.

# Cargo Features

- `getrandom`: Adds `Rng::from_operating_system`, which uses the `getrandom`
  crate to get a seed from the operating system.

- `rand_core`: Implements the traits `TryRng`, `Rng`, and `SeedableRng` from the
  `rand_core` crate.

- `thread_local`: Adds a thread-local random number generator. Requires `std`.
