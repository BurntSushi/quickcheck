// I have no idea what I'm doing with these attributes. Are we using
// semantic versioning? Some packages include their full github URL.
// Documentation for this stuff is extremely scarce.
#[crate_id = "quickcheck#0.1.0"];
#[crate_type = "lib"];
#[license = "UNLICENSE"];
#[doc(html_root_url = "http://burntsushi.net/rustdoc/quickcheck")];

//! This crate is a port of
//! [Haskell's QuickCheck](http://hackage.haskell.org/package/QuickCheck).

extern crate collections;

pub use arbitrary::{Arbitrary, Gen, StdGen, ObjIter, gen};
pub use tester::{Testable, TestResult, Config};
pub use tester::{quickcheck, quickcheckConfig, quicktest, quicktestConfig};
pub use tester::{DEFAULT_CONFIG, DEFAULT_SIZE};

mod arbitrary;
// mod shrink; 

mod tester {
    use std::fmt::Show;
    use std::iter;
    use std::rand::task_rng;
    use super::{Arbitrary, Gen, gen};

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
    /// This function is equivalent to calling `quickcheckConfig` with
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
        quickcheckConfig(DEFAULT_CONFIG, g, f)
    }

    /// Does randomized testing on `f` with the given config and produces a 
    /// possibly minimal witness for test failures.
    pub fn quickcheckConfig<A: Testable, G: Gen>(c: Config, g: &mut G, f: A) {
        match quicktestConfig(c, g, f) {
            Ok(ntests) => debug!("[quickcheck] Passed {:u} tests.", ntests),
            Err(err) => fail!(err),
        }
    }

    /// Like `quickcheck`, but returns either the number of tests passed
    /// or a witness of failure.
    pub fn quicktest<A: Testable>(f: A) -> Result<uint, ~str> {
        let g = &mut gen(task_rng(), DEFAULT_SIZE);
        quicktestConfig(DEFAULT_CONFIG, g, f)
    }

    /// Like `quickcheckConfig`, but returns either the number of tests passed
    /// or a witness of failure.
    pub fn quicktestConfig<A: Testable, G: Gen>
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
    /// generator passed to `quickcheckConfig`.
    pub struct Config {
        /// The number of tests to run on a function where the result is
        /// either a pass or a failure. (i.e., This doesn't include discarded
        /// test results.)
        tests: uint,

        /// The maximum number of tests to run for each function including
        /// discarded test results.
        max_tests: uint,
    }

    #[deriving(Clone, Show)]
    pub struct TestResult {
        priv status: Status,
        priv arguments: ~[~str],
    }

    #[deriving(Clone, Show)]
    priv enum Status { Pass, Fail, Discard }

    impl TestResult {
        pub fn passed() -> ~TestResult { TestResult::from_bool(true) }
        pub fn failed() -> ~TestResult { TestResult::from_bool(false) }
        pub fn discard() -> ~TestResult {
            ~TestResult { status: Discard, arguments: ~[] }
        }
        pub fn from_bool(b: bool) -> ~TestResult {
            ~TestResult { status: if b { Pass } else { Fail }, arguments: ~[] }
        }

        pub fn is_failure(&self) -> bool {
            match self.status {
                Fail => true,
                Pass|Discard => false,
            }
        }
    }

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

    // I should really figure out how to use macros. This is painful.
    // N.B. This isn't needed in Haskell because it's currying by default!
    // Perhaps there is a way to circumvent this in Rust too (without macros),
    // but I'm not sure.

    impl<A: Testable> Testable for 'static || -> A {
        fn result<G: Gen>(&self, g: &mut G) -> ~TestResult {
            apply0(g, || (*self)())
        }
    }

    impl<A: Arbitrary + Show, B: Testable> Testable for 'static |A| -> B {
        fn result<G: Gen>(&self, g: &mut G) -> ~TestResult {
            apply1(g, |a| (*self)(a))
        }
    }

    impl<A: Arbitrary + Show, B: Arbitrary + Show, C: Testable>
        Testable for 'static |A, B| -> C {
        fn result<G: Gen>(&self, g: &mut G) -> ~TestResult {
            apply2(g, |a, b| (*self)(a, b))
        }
    }

    impl<A: Arbitrary + Show,
         B: Arbitrary + Show,
         C: Arbitrary + Show,
         D: Testable>
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

    impl<A: Arbitrary + Show, B: Testable> Testable for fn(A) -> B {
        fn result<G: Gen>(&self, g: &mut G) -> ~TestResult {
            apply1(g, |a| (*self)(a))
        }
    }

    impl<A: Arbitrary + Show, B: Arbitrary + Show, C: Testable>
        Testable for fn(A, B) -> C {
        fn result<G: Gen>(&self, g: &mut G) -> ~TestResult {
            apply2(g, |a, b| (*self)(a, b))
        }
    }

    impl<A: Arbitrary + Show,
         B: Arbitrary + Show,
         C: Arbitrary + Show,
         D: Testable>
        Testable for fn(A, B, C) -> D {
        fn result<G: Gen>(&self, g: &mut G) -> ~TestResult {
            apply3(g, |a, b, c| (*self)(a, b, c))
        }
    }

    fn apply0<A: Testable,
              G: Gen
             >(g: &mut G, fun: || -> A)
             -> ~TestResult {
        shrink(0, (), (), (), |_: (), _: (), _: ()| fun().result(g))
    }

    fn apply1<A: Arbitrary + Show,
              B: Testable,
              G: Gen
             >(g: &mut G, fun: |a: A| -> B)
             -> ~TestResult {
        let a = arby(g);
        shrink(1, a, (), (), |a: A, _: (), _: ()| {
            let mut r = fun(a.clone()).result(g);
            if r.is_failure() {
                r.arguments = ~[a.to_str()];
            }
            r
        })
    }

    fn apply2<A: Arbitrary + Show,
              B: Arbitrary + Show,
              C: Testable,
              G: Gen
             >(g: &mut G, fun: |a: A, b: B| -> C)
             -> ~TestResult {
        let (a, b): (A, B) = arby(g);
        shrink(2, a, b, (), |a: A, b: B, _: ()| {
            let mut r = fun(a.clone(), b.clone()).result(g);
            if r.is_failure() {
                r.arguments = ~[a.to_str(), b.to_str()];
            }
            r
        })
    }

    fn apply3<A: Arbitrary + Show,
              B: Arbitrary + Show,
              C: Arbitrary + Show,
              D: Testable,
              G: Gen
             >(g: &mut G, fun: |a: A, b: B, c: C| -> D)
             -> ~TestResult {
        let (a, b, c): (A, B, C) = arby(g);
        shrink(3, a, b, c, |a: A, b: B, c: C| {
            let mut r = fun(a.clone(), b.clone(), c.clone()).result(g);
            if r.is_failure() {
                r.arguments = ~[a.to_str(), b.to_str(), c.to_str()];
            }
            r
        })
    }

    fn shrink<A: Arbitrary + Show, B: Arbitrary + Show, C: Arbitrary + Show>
             (n: uint, a: A, b: B, c: C, fun: |A, B, C| -> ~TestResult)
             -> ~TestResult {
        let toshrink = (a.clone(), b.clone(), c.clone());
        let mut r: ~TestResult = fun(a, b, c);
        match r.status {
            Pass|Discard => return r, // don't care about the args here
            Fail => {
                for (a, b, c) in toshrink.shrink() {
                    let r1 = fun(a.clone(), b.clone(), c.clone());
                    match r1.status {
                        Pass|Discard => continue,
                        Fail => {
                            let r2 = shrink(n, a, b, c, |a, b, c| fun(a, b, c));
                            match r2.status {
                                Pass|Discard => r = r1,
                                Fail => r = r2,
                            }
                            break;
                        },
                    }
                }
            },
        }
        r
    }

    /// Convenient alias.
    fn arby<A: Arbitrary, G: Gen>(g: &mut G) -> A { Arbitrary::arbitrary(g) }
}

#[cfg(test)]
mod test {
    use std::rand::task_rng;
    use super::{Config, Testable, TestResult, gen, quickcheckConfig};

    static CONFIG: Config = Config {
        tests: 100,
        max_tests: 10000,
    };

    fn check<A: Testable>(f: A) {
        quickcheckConfig(CONFIG, &mut gen(task_rng(), 50), f)
    }

    #[test]
    fn prop_reverse_reverse() {
        fn revrev(xs: ~[uint]) -> bool {
            let rev = xs.clone().move_rev_iter().to_owned_vec();
            let revrev = rev.move_rev_iter().to_owned_vec();
            xs == revrev
        }
        check(revrev);
    }

    #[test]
    fn reverse_single() {
        fn rev_single(xs: ~[uint]) -> ~TestResult {
            if xs.len() != 1 {
                return TestResult::discard()
            }
            return TestResult::from_bool(
                xs == xs.clone().move_rev_iter().to_owned_vec()
            )
        }
        check(rev_single);
    }

    #[test]
    fn reverse_app() {
        fn revapp(xs: ~[uint], ys: ~[uint]) -> bool {
            let app = ::std::vec::append(xs.clone(), ys);
            let app_rev = app.move_rev_iter().to_owned_vec();

            let rxs = xs.clone().move_rev_iter().to_owned_vec();
            let rys = ys.clone().move_rev_iter().to_owned_vec();
            let rev_app = ::std::vec::append(rys, rxs);

            app_rev == rev_app
        }
        check(revapp);
    }
}
