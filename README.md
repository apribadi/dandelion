Dandelion is a high performance non-cryptographic random number generator
suitable for algorithms applied to simulation, testing, statistics, and data
structures. It has a state of 128 bits and a cycle length of 2¹²⁸ - 1.

# Example

```
use dandelion::Rng;
let mut rng = Rng::from_u64(0);
let x = rng.u64();
let y = rng.between_u64(1, 6);
let z = rng.f64();
```

# Cargo Features

- `getrandom`: Adds `Rng::from_operating_system`, which uses the `getrandom`
  crate to get a seed from the operating system.

- `rand_core`: Implements the traits `RngCore` and `SeedableRng` from the
  `rand_core` crate.

- `thread_local`: Adds a thread-local random number generator. Requires `std`.

# Algorithm

The core of Dandelion's random number generator has two parts: a state
transition function `F: (u64, u64) -> (u64, u64)` and an output function `G:
(u64, u64) -> u64`, such that the `k`th output is

```text
G(F(F( ... F(x₀, y₀) ... )))
  \________/
   k times
```

where the initial state is `(x₀, y₀)`.

The state transition function `F` is defined

```text
F(x, y) = (y ^ shr(y, 19), x ^ ror(y, 7))
```

and the output function `G` is defined

```text
G(x, y) = y + ((x * x) ^ ((x * x) >> 64))
```

where `x` and `y` are `u64`s, and we use both the lower and upper halves of a
`(u64, u64) -> u128` full multiply. We further require the state to not be all
zeros, i.e. `(x, y) ≠ (0, 0)`.

There are two things to note about these functions. First, `F` can be thought
of as an invertible linear transformation on the space F₂¹²⁸ of vectors of
128 bits, i.e. `F` is an element of GL(128, F₂). Second, `G` can be thought
of as a permutation on F₂¹²⁸ / { 0 } followed by a truncation.

# The Period of the State Transition Function

We will demonstrate that the state transition function has full period 2¹²⁸ - 1.
We can do the necessary computations on binary matrices in GL(128, F₂).  Let
`A` be the binary matrix representing our transition. Then it is sufficient to
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
feasible. Actually, it is a little faster to check the equivalent `pow(A,
2¹²⁸) = A` for the first condition.

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
(x, y) ⇒ (y ^ shr(y, α), x ^ ror(y, β))
```

has full period is

```text
α=19 β=7
α=29 β=23 
α=33 β=29 
```

and we use `(19, 7)` because it has the least sparse transition matrix.

# Maximal Equidistribution

The fact that the random number generator is maximally 1-equidistributed
follows immediately from the fact that the state transition function has full
period and that the output function is a truncated permutation.

Indeed, a Feistel round

```text
(x, y) ⇒ (y + H(x), x)
```

is a permutation irrespective of any properties of `H`.

The output multiset is F₂¹²⁸ / 0 truncated to 64 bits, so 0 occurs 2⁶⁴ - 1
times and every other value occurs 2⁶⁴ times.

# Design Considerations

A state size of 128 bits was selected because 64 bits might be too small for
some non-cryptographic applications, but 128 bits should be enough for almost
everything.

The state transition function is similar in spirit to those in the xorshift
extended family, but is weakened as much as possible while retaining a full
period.

The output function contains a mixer which does a `(u64, u64) -> u128` full
multiply and then xors the lower and upper halves. This does a lot of mixing
while being relatively cheap on modern CPUs in phones, laptops, and servers.
This kind of mixer was inspired by similar constructions in the mum-hash and
wyhash libraries.

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

The benchmarks were run on an Apple M1.

```text
dandelion
1.018 ns/word - u64
2.364 ns/word - u64 noinline
1.994 ns/word - between_u64
3.504 ns/word - between_u64 noinline
1.042 ns/word - f64
2.448 ns/word - f64 noinline
0.916 ns/word - bytes large fill
2.679 ns/word - bytes small fill
2.978 ns/word - bytes small fill noinline

pcgdxsm128
1.647 ns/word - u64
4.072 ns/word - u64 noinline
2.394 ns/word - between_u64
4.897 ns/word - between_u64 noinline
1.657 ns/word - f64
4.071 ns/word - f64 noinline
1.633 ns/word - bytes large fill
3.624 ns/word - bytes small fill
5.078 ns/word - bytes small fill noinline

xoroshiro128++
1.472 ns/word - u64
3.304 ns/word - u64 noinline
3.975 ns/word - between_u64
3.848 ns/word - between_u64 noinline
1.495 ns/word - f64
3.423 ns/word - f64 noinline
1.466 ns/word - bytes large fill
3.465 ns/word - bytes small fill
4.843 ns/word - bytes small fill noinline
```

# Statistical Tests

Dandelion passes PractRand with 2 TB of output, and there is no reason to
believe that the test is particularly close to failing.

The `examples/rng` executable writes random bytes to stdout, which you can use
to run your own statistical tests.
