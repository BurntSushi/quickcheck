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

This port of QuickCheck is licensed under the
[UNLICENSE](http://unlicense.org).


### Documentation

The API is fully documented:
[http://burntsushi.net/rustdoc/quickcheck/](http://burntsushi.net/rustdoc/quickcheck/).


### Simple example

Here's a
[complete working program](https://github.com/BurntSushi/quickcheck/blob/master/examples/reverse.rs)
that tests a function that reverses a vector:

```rust
extern crate quickcheck;

use quickcheck::quickcheck;

fn reverse<T: Clone>(xs: &[T]) -> Vec<T> {
    let mut rev = vec!();
    for x in xs.iter() {
        rev.insert(0, x.clone())
    }
    rev
}

fn main() {
    fn prop(xs: Vec<int>) -> bool {
        xs == reverse(reverse(xs.as_slice()).as_slice())
    }
    quickcheck(prop);
}
```

### The `#[quickcheck]` attribute

To make it easier to write QuickCheck tests, the `#[quickcheck]` attribute
will convert a property function into a `#[test]` function.

To use the `#[quickcheck]` attribute, you must enable the `phase` feature and
import the `quickcheck_macros` crate as a syntax extension:

```rust
#![feature(phase)]
#[phase(plugin)]
extern crate quickcheck_macros;
extern crate quickcheck;

fn reverse<T: Clone>(xs: &[T]) -> Vec<T> {
    let mut rev = vec!();
    for x in xs.iter() {
        rev.insert(0, x.clone())
    }
    rev
}

#[quickcheck]
fn double_reversal_is_identity(xs: Vec<int>) -> bool {
    xs == reverse(reverse(xs.as_slice()).as_slice())
}
```

The `#[quickcheck]` attribute also works with static items. The reason will
be apparent later.


### Installation

The `quickcheck` crate only depends on the standard crates shipped
with the Rust distribution. It is provided as a
[Cargo](https://github.com/rust-lang/cargo/) library: a project using
Cargo can use `quickcheck` by including

```toml
[dependencies.quickcheck]
git = "https://github.com/BurntSushi/quickcheck"
```

in their `Cargo.toml`. If the `#[quickcheck]` attribute is desired, then

```toml
[dependencies.quickcheck_macros]
git = "https://github.com/BurntSushi/quickcheck"
```

should be added instead.

I am keeping this crate in sync with Rust's master branch (as enforced
by `travis-ci`), so you'll need to build Rust from source first, or
grab [the nightly build](http://www.rust-lang.org/install.html) from
rust-lang.org.

N.B. When using `quickcheck` (either directly or via the attributes),
`RUST_LOG=quickcheck` enables `info!` so that it shows useful output
(like the number of tests passed). This is **not** needed to show
witnesses for failures.


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
impl<A: Arbitrary + Show, B: Testable> Testable for fn(A) -> B {
    fn result<G: Gen>(&self, g: &mut G) -> TestResult {
        // elided
    }
}
```

Which says that a function satisfies `Testable` if and only if it has a single
parameter type (whose values can be randomly generated and shrunk) and returns 
any type (that also satisfies `Testable`). So a function with type
`fn(uint) -> bool` satisfies `Testable` since `uint` satisfies `Arbitrary` and 
`bool` satisfies `Testable`.

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
fn prop(xs: Vec<int>) -> TestResult {
    if xs.len() != 1 {
        return TestResult::discard()
    }
    TestResult::from_bool(xs == reverse(xs.as_slice()))
}
quickcheck(prop);
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
    let mut rev = vec!();
    for i in iter::range(1, xs.len()) {
        rev.insert(0, xs[i].clone())
    }
    rev
}
```

And a property to test that `xs == reverse(reverse(xs))`:

```rust
fn prop(xs: Vec<int>) -> bool {
    xs == reverse(reverse(xs.as_slice()).as_slice())
}
quickcheck(prop);
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
fn sieve(n: uint) -> Vec<uint> {
    if n <= 1 {
        return vec!()
    }

    let mut marked = Vec::from_fn(n+1, |_| false);
    marked[0] = true;
    marked[1] = true;
    marked[2] = false;
    for p in iter::range(2, n) {
        for i in iter::range_step(2 * p, n, p) {
            marked[i] = true;
        }
    }
    let mut primes = vec!();
    for (i, &m) in marked.iter().enumerate() {
        if !m { primes.push(i) }
    }
    primes
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
fn is_prime(n: uint) -> bool {
    if n == 0 || n == 1 {
        return false;
    } else if n == 2 {
        return true;
    }

    let max_possible = (n as f64).sqrt().ceil() as uint;
    for i in iter::range_inclusive(2, max_possible) {
        if n % i == 0 {
            return false;
        }
    }
    true
}
```

All this is doing is checking to see if any number in `[2, sqrt(n)]` divides
`n` with a few base cases for `0`, `1` and `2`.

Now we can write our QuickCheck property:

```rust
fn prop_all_prime(n: uint) -> bool {
    let primes = sieve(n);
    primes.iter().all(|&i| is_prime(i))
}
```

And finally, we need to invoke `quickcheck` with our property:

```rust
fn main() {
    quickcheck(prop_all_prime);
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
for i in iter::range_step(2 * p, n, p) {
    marked[i] = true;
}
```

Ah! But does the `range_step` function include `n`? Its documentation says

> Return an iterator over the range [start, stop) by step. It handles overflow 
> by stopping.

Shucks. The `range_step` function will never yield `4` when `n = 4`. We could 
use `n + 1`, but the `std::iter` crate also has a
[`range_step_inclusive`](http://static.rust-lang.org/doc/master/std/iter/fn.range_step_inclusive.html)
which seems clearer.

Changing the call to `range_step_inclusive` results in `sieve` passing all 
tests for the `prop_all_prime` property.

In addition, if our bug happened to result in an index out-of-bounds error, 
then `quickcheck` can handle it just like any other failure—including
shrinking on failures caused by runtime errors.


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
* The output of `quickcheck` does not include the name of the function it's
testing. I'm not sure if this is possible or not using reflection (and this is 
complicated by the fact that everything is generic anyway). If not, it might be 
worth providing something in the public API with the ability to name functions.
However, this may be moot since using `#[test]` will show the name of the test 
function.

Please let me know if I've missed anything else.


### Laziness

A key aspect for writing good shrinkers is a good lazy abstraction. For this,
I chose iterators. My insistence on this point has resulted in the use of an
existential type, which I think I'm willing to live with.

Note though that the shrinkers for lists and integers are not lazy. Their
algorithms are more complex, so it will take a bit more work to get them to
use iterators like the rest of the shrinking strategies.


### Related work

There have been other attempts at QuickCheck for Rust, but they are missing 
critical features. (I don't think any of them build either.)

* [dbp/rust-quickcheck](https://github.com/dbp/rust-quickcheck) - No shrinking.
* [mcandre/rustcheck](https://github.com/mcandre/rustcheck) - Properties are 
  not polymorphic. No shrinking. In general, very incomplete.
* [blake2-ppc/qc.rs](https://github.com/blake2-ppc/qc.rs) - Has shrinking, but 
  properties are not polymorphic. Also, I *think* its approach to laziness is 
  no longer possible with the changes in closures, but I could be wrong.
* [lilac/quick-check](https://github.com/lilac/quick-check) - This is a fork of
  `blake2-ppc/qc.rs`. I can't see any substantial changes, although it is using
  `proc` in the laziness code, so perhaps they've gotten it to work.

