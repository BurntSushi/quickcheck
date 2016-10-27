QuickCheck is a way to do property based testing using randomly generated
input. This crate comes with the ability to randomly generate and shrink
integers, floats, tuples, booleans, lists, strings, options and results.
All QuickCheck needs is a property function—it will then randomly generate
inputs to that function and call the property for each set of inputs. If the
property fails (whether by a runtime error like index out-of-bounds or by not
satisfying your property), the inputs are "shrunk" to find a smaller
counter-example.

The shrinking strategies for lists and numbers use a binary search to cover
the input space quickly. (It should be the same strategy used in
[Koen Claessen's QuickCheck for
Haskell](http://hackage.haskell.org/package/QuickCheck).)

[![Build status](https://api.travis-ci.org/BurntSushi/quickcheck.png)](https://travis-ci.org/BurntSushi/quickcheck)
[![](http://meritbadge.herokuapp.com/quickcheck)](https://crates.io/crates/quickcheck)

Dual-licensed under MIT or the [UNLICENSE](http://unlicense.org).


### Documentation

The API is fully documented:
[http://burntsushi.net/rustdoc/quickcheck/](http://burntsushi.net/rustdoc/quickcheck/).


### Simple example

Here's an example that tests a function that reverses a vector:

```rust
#[cfg(test)]
#[macro_use]
extern crate quickcheck;

fn reverse<T: Clone>(xs: &[T]) -> Vec<T> {
    let mut rev = vec!();
    for x in xs.iter() {
        rev.insert(0, x.clone())
    }
    rev
}

#[cfg(test)]
mod tests {
  quickcheck! {
      fn prop(xs: Vec<u32>) -> bool {
          xs == reverse(&reverse(&xs))
      }
  }
}
```

This example uses the `quickcheck!` macro, which is available on stable Rust.

### The `#[quickcheck]` attribute (requires Rust nightly)

To make it easier to write QuickCheck tests, the `#[quickcheck]` attribute
will convert a property function into a `#[test]` function.

To use the `#[quickcheck]` attribute, you must enable the `plugin` feature and
import the `quickcheck_macros` crate as a syntax extension:

```rust
#![feature(plugin)]
#![plugin(quickcheck_macros)]

#[cfg(test)]
extern crate quickcheck;

#[cfg(test)]
mod tests {
    fn reverse<T: Clone>(xs: &[T]) -> Vec<T> {
        let mut rev = vec!();
        for x in xs {
            rev.insert(0, x.clone())
        }
        rev
    }

    #[quickcheck]
    fn double_reversal_is_identity(xs: Vec<isize>) -> bool {
        xs == reverse(&reverse(&xs))
    }
}
```


### Installation

`quickcheck` is on `crates.io`, so you can include it in your project like so:

```toml
[dependencies]
quickcheck = "0.3"
```

If you're only using `quickcheck` in your test code, then you can add it as a
development dependency instead:

```toml
[dev-dependencies]
quickcheck = "0.3"
```

If you want to use the `#[quickcheck]` attribute, then add `quickcheck_macros`

```toml
[dev-dependencies]
quickcheck = "0.3"
quickcheck_macros = "0.2"
```

and only enable the `quickcheck_macros` plugin for the test build
```rust
#![cfg_attr(test, feature(plugin))]
#![cfg_attr(test, plugin(quickcheck_macros))]
```

Note that the `#[quickcheck]` macro will not work when Rust 1.0 stable is
released, although it will continue to work on the nightlies.

N.B. When using `quickcheck` (either directly or via the attributes),
`RUST_LOG=quickcheck` enables `info!` so that it shows useful output
(like the number of tests passed). This is **not** needed to show
witnesses for failures.

Crate features:

- `"unstable"`: Enables Arbitrary implementations that require the Rust nightly
  channel.
- `"use_logging"`: (Enabled by default.) Enables the log messages governed
  `RUST_LOG`.


### Discarding test results (or, properties are polymorphic!)

Sometimes you want to test a property that only holds for a *subset* of the
possible inputs, so that when your property is given an input that is outside
of that subset, you'd discard it. In particular, the property should *neither*
pass nor fail on inputs outside of the subset you want to test. But properties
return boolean values—which either indicate pass or fail.

To fix this, we need to take a step back and look at the type of the
`quickcheck` function:

```rust
pub fn quickcheck<A: Testable>(f: A) {
    // elided
}
```

So `quickcheck` can test any value with a type that satisfies the `Testable`
trait. Great, so what is this `Testable` business?

```rust
pub trait Testable {
    fn result<G: Gen>(&self, &mut G) -> TestResult;
}
```

This trait states that a type is testable if it can produce a `TestResult`
given a source of randomness. (A `TestResult` stores information about the
results of a test, like whether it passed, failed or has been discarded.)

Sure enough, `bool` satisfies the `Testable` trait:

```rust
impl Testable for bool {
    fn result<G: Gen>(&self, _: &mut G) -> TestResult {
        TestResult::from_bool(*self)
    }
}
```

But in the example, we gave a *function* to `quickcheck`. Yes, functions can
satisfy `Testable` too!

```rust
impl<A: Arbitrary + Debug, B: Testable> Testable for fn(A) -> B {
    fn result<G: Gen>(&self, g: &mut G) -> TestResult {
        // elided
    }
}
```

Which says that a function satisfies `Testable` if and only if it has a single
parameter type (whose values can be randomly generated and shrunk) and returns
any type (that also satisfies `Testable`). So a function with type `fn(usize)
-> bool` satisfies `Testable` since `usize` satisfies `Arbitrary` and `bool`
satisfies `Testable`.

So to discard a test, we need to return something other than `bool`. What if we
just returned a `TestResult` directly? That should work, but we'll need to
make sure `TestResult` satisfies `Testable`:

```rust
impl Testable for TestResult {
    fn result<G: Gen>(&self, _: &mut G) -> TestResult { self.clone() }
}
```

Now we can test functions that return a `TestResult` directly.

As an example, let's test our reverse function to make sure that the reverse of
a vector of length 1 is equal to the vector itself.

```rust
fn prop(xs: Vec<isize>) -> TestResult {
    if xs.len() != 1 {
        return TestResult::discard()
    }
    TestResult::from_bool(xs == reverse(&xs))
}
quickcheck(prop as fn(Vec<isize>) -> TestResult);
```

(A full working program for this example is in
[`examples/reverse_single.rs`](https://github.com/BurntSushi/quickcheck/blob/master/examples/reverse_single.rs).)

So now our property returns a `TestResult`, which allows us to encode a bit
more information. There are a few more
[convenience functions defined for the `TestResult`
type](http://burntsushi.net/rustdoc/quickcheck/struct.TestResult.html).
For example, we can't just return a `bool`, so we convert a `bool` value to a
`TestResult`.

(The ability to discard tests allows you to get similar functionality as
Haskell's `==>` combinator.)

N.B. Since discarding a test means it neither passes nor fails, `quickcheck`
will try to replace the discarded test with a fresh one. However, if your
condition is seldom met, it's possible that `quickcheck` will have to settle
for running fewer tests than usual. By default, if `quickcheck` can't find
`100` valid tests after trying `10,000` times, then it will give up.
This parameter may be changed using
[`quickcheck_config`](http://burntsushi.net/rustdoc/quickcheck/fn.quickcheck_config.html).


### Shrinking

Shrinking is a crucial part of QuickCheck that simplifies counter-examples for
your properties automatically. For example, if you erroneously defined a
function for reversing vectors as: (my apologies for the contrived example)

```rust
fn reverse<T: Clone>(xs: &[T]) -> Vec<T> {
    let mut rev = vec![];
    for i in 1..xs.len() {
        rev.insert(0, xs[i].clone())
    }
    rev
}
```

And a property to test that `xs == reverse(reverse(xs))`:

```rust
fn prop(xs: Vec<isize>) -> bool {
    xs == reverse(&reverse(&xs))
}
quickcheck(prop as fn(Vec<isize>) -> bool);
```

Then without shrinking, you might get a counter-example like:

```
[quickcheck] TEST FAILED. Arguments: ([-17, 13, -12, 17, -8, -10, 15, -19,
-19, -9, 11, -5, 1, 19, -16, 6])
```

Which is pretty mysterious. But with shrinking enabled, you're nearly
guaranteed to get this counter-example every time:

```
[quickcheck] TEST FAILED. Arguments: ([0])
```

Which is going to be much easier to debug.


### Case study: The Sieve of Eratosthenes

The [Sieve of Eratosthenes](http://en.wikipedia.org/wiki/Sieve_of_Eratosthenes)
is a simple and elegant way to find all primes less than or equal to `N`.
Briefly, the algorithm works by allocating an array with `N` slots containing
booleans. Slots marked with `false` correspond to prime numbers (or numbers
not known to be prime while building the sieve) and slots marked with `true`
are known to not be prime. For each `n`, all of its multiples in this array
are marked as true. When all `n` have been checked, the numbers marked `false`
are returned as the primes.

As you might imagine, there's a lot of potential for off-by-one errors, which
makes it ideal for randomized testing. So let's take a look at my
implementation and see if we can spot the bug:

```rust
fn sieve(n: usize) -> Vec<usize> {
    if n <= 1 {
        return vec![];
    }

    let mut marked = vec![false; n+1];
    marked[0] = true;
    marked[1] = true;
    marked[2] = true;
    for p in 2..n {
        for i in (2*p..n).filter(|&n| n % p == 0) {
            marked[i] = true;
        }
    }
    marked.iter()
          .enumerate()
          .filter_map(|(i, &m)| if m { None } else { Some(i) })
          .collect()
}
```

Let's try it on a few inputs by hand:

```
sieve(3) => [2, 3]
sieve(5) => [2, 3, 5]
sieve(8) => [2, 3, 5, 7, 8] # !!!
```

Something has gone wrong! But where? The bug is rather subtle, but it's an
easy one to make. It's OK if you can't spot it, because we're going to use
QuickCheck to help us track it down.

Even before looking at some example outputs, it's good to try and come up with
some *properties* that are always satisfiable by the output of the function. An
obvious one for the prime number sieve is to check if all numbers returned are
prime. For that, we'll need an `is_prime` function:

```rust
fn is_prime(n: usize) -> bool {
    n != 0 && n != 1 && (2..).take_while(|i| i*i <= n).all(|i| n % i != 0)
}
```

All this is doing is checking to see if any number in `[2, sqrt(n)]` divides
`n` with base cases for `0` and `1`.

Now we can write our QuickCheck property:

```rust
fn prop_all_prime(n: usize) -> bool {
    sieve(n).into_iter().all(is_prime)
}
```

And finally, we need to invoke `quickcheck` with our property:

```rust
fn main() {
    quickcheck(prop_all_prime as fn(usize) -> bool);
}
```

A fully working source file with this code is in
[`examples/sieve.rs`](https://github.com/BurntSushi/quickcheck/blob/master/examples/sieve.rs).

The output of running this program has this message:

```
[quickcheck] TEST FAILED. Arguments: (4)
```

Which says that `sieve` failed the `prop_all_prime` test when given `n = 4`.
Because of shrinking, it was able to find a (hopefully) minimal counter-example
for our property.

With such a short counter-example, it's hopefully a bit easier to narrow down
where the bug is. Since `4` is returned, it's likely never marked as being not
prime. Since `4` is a multiple of `2`, its slot should be marked as `true` when
`p = 2` on these lines:

```rust
for i in (2*p..n).filter(|&n| n % p == 0) {
    marked[i] = true;
}
```

Ah! But does the `..` (range) operator include `n`? Nope! This particular
operator is a half-open interval.

A `2*p..n` range will never yield `4` when `n = 4`. When we change this to
`2*p..n+1`, all tests pass.

In addition, if our bug happened to result in an index out-of-bounds error,
then `quickcheck` can handle it just like any other failure—including
shrinking on failures caused by runtime errors.

But hold on... we're not done yet. Right now, our property tests that all
the numbers returned by `sieve` are prime but it doesn't test if the list is
complete. It does not ensure that all the primes between `0` and `n` are found.

Here's a property that is more comprehensive:

```rust
fn prop_prime_iff_in_the_sieve(n: usize) -> bool {
    sieve(n) == (0..(n + 1)).filter(|&i| is_prime(i)).collect::<Vec<_>>()
}
```

It tests that for each number between 0 and n, inclusive, the naive primality test
yields the same result as the sieve.

Now, if we run it:

```rust
fn main() {
    quickcheck(prop_all_prime as fn(usize) -> bool);
    quickcheck(prop_prime_iff_in_the_sieve as fn(usize) -> bool);
}
```

we see that it fails immediately for value n = 2.

```
[quickcheck] TEST FAILED. Arguments: (2)
```

If we inspect `sieve()` once again, we see that we mistakenly mark `2` as
non-prime. Removing the line `marked[2] = true;` results in both properties
passing.

### What's not in this port of QuickCheck?

I think I've captured the key features, but there are still things missing:

* As of now, only functions with 4 or fewer parameters can be quickchecked.
This limitation can be lifted to some `N`, but requires an implementation
for each `n` of the `Testable` trait.
* Functions that fail because of a stack overflow are not caught by QuickCheck.
Therefore, such failures will not have a witness attached
to them. (I'd like to fix this, but I don't know how.)
* `Coarbitrary` does not exist in any form in this package. I think it's
possible; I just haven't gotten around to it yet.
