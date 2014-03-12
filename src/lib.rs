// I have no idea what I'm doing with these attributes. Are we using
// semantic versioning? Some packages include their full github URL.
// Documentation for this stuff is extremely scarce.
#[crate_id = "quickcheck#0.1.0"];
#[crate_type = "lib"];
#[license = "UNLICENSE"];
#[doc(html_root_url = "http://burntsushi.net/rustdoc/quickcheck")];

//! This crate is a port of
//! [Haskell's QuickCheck](http://hackage.haskell.org/package/QuickCheck).
//!
//! For detailed examples, please see the
//! [README](https://github.com/BurntSushi/quickcheck).

extern crate collections;
extern crate rand;

pub use arbitrary::{Arbitrary, Gen, StdGen, ObjIter, gen};
pub use tester::{Testable, TestResult, Config};
pub use tester::{quickcheck, quickcheck_config, quicktest, quicktest_config};
pub use tester::{DEFAULT_CONFIG, DEFAULT_SIZE};

mod arbitrary;

mod tester {
    use std::fmt::Show;
    use std::iter;
    use std::task;
    use rand::task_rng;
    use super::{Arbitrary, Gen, ObjIter, gen};

    /// Default size hint used in `quickcheck` for sampling from a random
    /// distribution.
    pub static DEFAULT_SIZE: uint = 20;

    /// Default configuration used in `quickcheck`.
    pub static DEFAULT_CONFIG: Config = Config{
        tests: 100,
        max_tests: 10000,
    };

    /// Does randomized testing on `f` and produces a possibly minimal
    /// witness for test failures.
    ///
    /// This function is equivalent to calling `quickcheck_config` with
    /// `DEFAULT_CONFIG` and a `Gen` with size `DEFAULT_SIZE`.
    ///
    /// As of now, it is intended for `quickcheck` to be used inside Rust's
    /// unit testing system. For example, to check if
    /// `reverse(reverse(xs)) == xs`, you could use:
    ///
    /// ```rust
    /// fn prop_reverse_reverse() {
    ///     fn revrev(xs: ~[uint]) -> bool {
    ///         let rev = xs.clone().move_rev_iter().to_owned_vec();
    ///         let revrev = rev.move_rev_iter().to_owned_vec();
    ///         xs == revrev
    ///     }
    ///     check(revrev);
    /// }
    /// ```
    ///
    /// In particular, `quickcheck` will call `fail!` if it finds a
    /// test failure. The failure message will include a witness to the
    /// failure.
    pub fn quickcheck<A: Testable>(f: A) {
        let g = &mut gen(task_rng(), DEFAULT_SIZE);
        quickcheck_config(DEFAULT_CONFIG, g, f)
    }

    /// Does randomized testing on `f` with the given config and produces a 
    /// possibly minimal witness for test failures.
    pub fn quickcheck_config<A: Testable, G: Gen>(c: Config, g: &mut G, f: A) {
        match quicktest_config(c, g, f) {
            Ok(ntests) => debug!("[quickcheck] Passed {:u} tests.", ntests),
            Err(err) => fail!(err),
        }
    }

    /// Like `quickcheck`, but returns either the number of tests passed
    /// or a witness of failure.
    pub fn quicktest<A: Testable>(f: A) -> Result<uint, ~str> {
        let g = &mut gen(task_rng(), DEFAULT_SIZE);
        quicktest_config(DEFAULT_CONFIG, g, f)
    }

    /// Like `quickcheck_config`, but returns either the number of tests passed
    /// or a witness of failure.
    pub fn quicktest_config<A: Testable, G: Gen>
                           (c: Config, g: &mut G, f: A) -> Result<uint, ~str> {
        let mut ntests: uint = 0;
        for _ in iter::range(0, c.max_tests) {
            if ntests >= c.tests {
                break
            }
            let r = f.result(g);
            match r.status {
                Pass => ntests = ntests + 1,
                Discard => continue,
                Fail => {
                    return Err(format!(
                        "[quickcheck] TEST FAILED. Arguments: ({})",
                        r.arguments.connect(", ")));
                }
            }
        }
        Ok(ntests)
    }

    /// Config contains various parameters for controlling automated testing.
    ///
    /// Note that the distribution of random values is controlled by the
    /// generator passed to `quickcheck_config`.
    pub struct Config {
        /// The number of tests to run on a function where the result is
        /// either a pass or a failure. (i.e., This doesn't include discarded
        /// test results.)
        tests: uint,

        /// The maximum number of tests to run for each function including
        /// discarded test results.
        max_tests: uint,
    }

    /// Describes the status of a single instance of a test.
    ///
    /// All testable things must be capable of producing a `~TestResult`.
    #[deriving(Clone, Show)]
    pub struct TestResult {
        priv status: Status,
        priv arguments: ~[~str],
    }

    /// Whether a test has passed, failed or been discarded.
    #[deriving(Clone, Show)]
    priv enum Status { Pass, Fail, Discard }

    impl TestResult {
        /// Produces a test result that indicates the current test has passed.
        pub fn passed() -> ~TestResult { TestResult::from_bool(true) }

        /// Produces a test result that indicates the current test has failed.
        pub fn failed() -> ~TestResult { TestResult::from_bool(false) }

        /// Produces a test result that instructs `quickcheck` to ignore it.
        /// This is useful for restricting the domain of your properties.
        /// When a test is discarded, `quickcheck` will replace it with a
        /// fresh one (up to a certain limit).
        pub fn discard() -> ~TestResult {
            ~TestResult { status: Discard, arguments: ~[] }
        }

        /// Converts a `bool` to a `~TestResult`. A `true` value indicates that
        /// the test has passed and a `false` value indicates that the test
        /// has failed.
        pub fn from_bool(b: bool) -> ~TestResult {
            ~TestResult { status: if b { Pass } else { Fail }, arguments: ~[] }
        }

        /// Returns `true` if and only if this test result describes a failing
        /// test.
        pub fn is_failure(&self) -> bool {
            match self.status {
                Fail => true,
                Pass|Discard => false,
            }
        }
    }

    /// `Testable` describes types (e.g., a function) whose values can be 
    /// tested.
    ///
    /// Anything that can be tested must be capable of producing a `TestResult`
    /// given a random number generator. This is trivial for types like `bool`,
    /// which are just converted to either a passing or failing test result.
    ///
    /// For functions, an implementation must generate random arguments
    /// and potentially shrink those arguments if they produce a failure.
    ///
    /// It's unlikely that you'll have to implement this trait yourself.
    /// This comes with a caveat: currently, only functions with 3 parameters 
    /// or fewer (both `fn` and `||` types) satisfy `Testable`. If you have
    /// functions to test with more than 3 parameters, please
    /// [file a bug](https://github.com/BurntSushi/quickcheck/issues) and
    /// I'll hopefully add it. (As of now, it would be very difficult to
    /// add your own implementation outside of `quickcheck`, since the
    /// functions that do shrinking are not public.)
    pub trait Testable {
        fn result<G: Gen>(&self, &mut G) -> ~TestResult;
    }

    impl Testable for bool {
        fn result<G: Gen>(&self, _: &mut G) -> ~TestResult {
            TestResult::from_bool(*self)
        }
    }

    impl Testable for ~TestResult {
        fn result<G: Gen>(&self, _: &mut G) -> ~TestResult { self.clone() }
    }

    impl<A: Testable> Testable for Result<A, ~str> {
        fn result<G: Gen>(&self, _: &mut G) -> ~TestResult {
            match *self {
                Ok(_) => TestResult::passed(),
                Err(_) => TestResult::failed(),
            }
        }
    }

    // I should really figure out how to use macros. This is painful.
    // N.B. This isn't needed in Haskell because it's currying by default!
    // Perhaps there is a way to circumvent this in Rust too (without macros),
    // but I'm not sure.

    impl<A: Testable> Testable for 'static || -> A {
        fn result<G: Gen>(&self, g: &mut G) -> ~TestResult {
            apply0(g, || (*self)())
        }
    }

    impl<A: AShow, B: Testable> Testable for 'static |A| -> B {
        fn result<G: Gen>(&self, g: &mut G) -> ~TestResult {
            apply1(g, |a| (*self)(a))
        }
    }

    impl<A: AShow, B: AShow, C: Testable> Testable for 'static |A, B| -> C {
        fn result<G: Gen>(&self, g: &mut G) -> ~TestResult {
            apply2(g, |a, b| (*self)(a, b))
        }
    }

    impl<A: AShow, B: AShow, C: AShow, D: Testable>
        Testable for 'static |A, B, C| -> D {
        fn result<G: Gen>(&self, g: &mut G) -> ~TestResult {
            apply3(g, |a, b, c| (*self)(a, b, c))
        }
    }

    impl<A: Testable> Testable for fn() -> A {
        fn result<G: Gen>(&self, g: &mut G) -> ~TestResult {
            apply0(g, || (*self)())
        }
    }

    impl<A: AShow, B: Testable> Testable for fn(A) -> B {
        fn result<G: Gen>(&self, g: &mut G) -> ~TestResult {
            apply1(g, |a| (*self)(a))
        }
    }

    impl<A: AShow, B: AShow, C: Testable> Testable for fn(A, B) -> C {
        fn result<G: Gen>(&self, g: &mut G) -> ~TestResult {
            apply2(g, |a, b| (*self)(a, b))
        }
    }

    impl<A: AShow, B: AShow, C: AShow, D: Testable>
        Testable for fn(A, B, C) -> D {
        fn result<G: Gen>(&self, g: &mut G) -> ~TestResult {
            apply3(g, |a, b, c| (*self)(a, b, c))
        }
    }

    // The following apply functions are used to abstract over the different
    // function types `fn` and `||`.

    fn apply0<A: Testable, G: Gen>(g: &mut G, fun: || -> A) -> ~TestResult {
        shrink((), (), (), |_: (), _: (), _: ()| fun().result(g))
    }

    fn apply1<A: AShow, B: Testable, G: Gen>
             (g: &mut G, fun: |a: A| -> B) -> ~TestResult {
        let a = arby(g);
        shrink(a, (), (), |a: A, _: (), _: ()| {
            let mut r = fun(a.clone()).result(g);
            if r.is_failure() {
                r.arguments = ~[a.to_str()];
            }
            r
        })
    }

    fn apply2<A: AShow, B: AShow, C: Testable, G: Gen>
             (g: &mut G, fun: |a: A, b: B| -> C) -> ~TestResult {
        let (a, b): (A, B) = arby(g);
        shrink(a, b, (), |a: A, b: B, _: ()| {
            let mut r = fun(a.clone(), b.clone()).result(g);
            if r.is_failure() {
                r.arguments = ~[a.to_str(), b.to_str()];
            }
            r
        })
    }

    fn apply3<A: AShow, B: AShow, C: AShow, D: Testable, G: Gen>
             (g: &mut G, fun: |a: A, b: B, c: C| -> D) -> ~TestResult {
        let (a, b, c): (A, B, C) = arby(g);
        shrink(a, b, c, |a: A, b: B, c: C| {
            let mut r = fun(a.clone(), b.clone(), c.clone()).result(g);
            if r.is_failure() {
                r.arguments = ~[a.to_str(), b.to_str(), c.to_str()];
            }
            r
        })
    }

    fn shrink<A: AShow, B: AShow, C: AShow>
             (a: A, b: B, c: C, fun: |A, B, C| -> ~TestResult)
             -> ~TestResult {
        let r = fun(a.clone(), b.clone(), c.clone());
        match r.status {
            Pass|Discard => r,
            Fail => {
                // We've found a failing test case, so try to shrink it.
                match shrink_failure((a, b, c).shrink(), fun) {
                    Some(smaller) => smaller,
                    None => r,
                }
            }
        }
    }

    fn shrink_failure<A: AShow, B: AShow, C: AShow>
                     (mut shrinker: ~ObjIter:<(A, B, C)>,
                      fun: |A, B, C| -> ~TestResult)
                     -> Option<~TestResult> {
        for (a, b, c) in shrinker {
            let r = fun(a.clone(), b.clone(), c.clone());
            match r.status {
                // The shrunk value does not witness a failure, so
                // throw it away.
                Pass|Discard => continue,

                // The shrunk value *does* witness a failure, so keep trying
                // to shrink it.
                Fail => {
                    let shrunk = shrink_failure((a, b, c).shrink(),
                                                |a, b, c| fun(a, b, c));

                    // If we couldn't witness a failure on any shrunk value,
                    // then return the failure we already have.
                    return Some(shrunk.unwrap_or(r))
                },
            }
        }
        None
    }

    // This is my bright idea for capturing runtime errors caused by a test.
    // Haven't been able to get the compiler to accept it.
    #[allow(dead_code)]
    fn safe<T: Send>(fun: proc() -> T) -> Result<T, ~str> {
        let tryr = task::try(proc() { fun() });
        match tryr {
            Ok(t) => Ok(t),
            Err(_) => Err(~"WTF"),
        }
    }

    /// Convenient aliases.
    trait AShow : Arbitrary + Show {}
    impl<A: Arbitrary + Show> AShow for A {}
    fn arby<A: Arbitrary, G: Gen>(g: &mut G) -> A { Arbitrary::arbitrary(g) }
}

#[cfg(test)]
mod test {
    use std::iter;
    use std::vec;
    use rand::task_rng;
    use super::{Config, Testable, TestResult, gen, quickcheck_config};

    static CONFIG: Config = Config {
        tests: 100,
        max_tests: 10000,
    };

    fn check<A: Testable>(f: A) {
        quickcheck_config(CONFIG, &mut gen(task_rng(), 100), f)
    }

    #[test]
    fn prop_reverse_reverse() {
        fn prop(xs: ~[uint]) -> bool {
            let rev = xs.clone().move_rev_iter().to_owned_vec();
            let revrev = rev.move_rev_iter().to_owned_vec();
            xs == revrev
        }
        check(prop);
    }

    #[test]
    fn reverse_single() {
        fn prop(xs: ~[uint]) -> ~TestResult {
            if xs.len() != 1 {
                return TestResult::discard()
            }
            return TestResult::from_bool(
                xs == xs.clone().move_rev_iter().to_owned_vec()
            )
        }
        check(prop);
    }

    #[test]
    fn reverse_app() {
        fn prop(xs: ~[uint], ys: ~[uint]) -> bool {
            let app = ::std::vec::append(xs.clone(), ys);
            let app_rev = app.move_rev_iter().to_owned_vec();

            let rxs = xs.clone().move_rev_iter().to_owned_vec();
            let rys = ys.clone().move_rev_iter().to_owned_vec();
            let rev_app = ::std::vec::append(rys, rxs);

            app_rev == rev_app
        }
        check(prop);
    }

    #[test]
    fn max() {
        fn prop(x: int, y: int) -> ~TestResult {
            if x > y {
                return TestResult::discard()
            } else {
                return TestResult::from_bool(::std::cmp::max(x, y) == y)
            }
        }
        check(prop);
    }

    #[test]
    fn sort() {
        fn prop(mut xs: ~[int]) -> bool {
            xs.sort();
            let upto = if xs.len() == 0 { 0 } else { xs.len()-1 };
            for i in iter::range(0, upto) {
                if xs[i] > xs[i+1] {
                    return false
                }
            }
            true
        }
        check(prop);
    }

    #[test]
    #[should_fail]
    fn sieve_of_eratosthenes() {
        fn sieve(n: uint) -> ~[uint] {
            if n <= 1 {
                return ~[]
            }

            let mut marked = vec::from_fn(n+1, |_| false);
            marked[0] = true; marked[1] = true; marked[2] = false;
            for p in iter::range(2, n) {
                for i in iter::range_step(2 * p, n, p) { // whoops!
                    marked[i] = true;
                }
            }
            let mut primes = ~[];
            for (i, m) in marked.iter().enumerate() {
                if !m { primes.push(i) }
            }
            primes
        }

        fn prop(n: uint) -> bool {
            let primes = sieve(n);
            primes.iter().all(|&i| is_prime(i))
        }
        fn is_prime(n: uint) -> bool {
            if n == 0 || n == 1 {
                return false
            } else if n == 2 {
                return true
            }

            let max_possible = (n as f64).sqrt().ceil() as uint;
            for i in iter::range_inclusive(2, max_possible) {
                if n % i == 0 {
                    return false
                }
            }
            return true
        }
        check(prop);
    }
}
