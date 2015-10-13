use rand;
use rand::{Rand, Rng, XorShiftRng};
use std::fmt::Debug;
use std::thread;
use std::fmt;
use arbitrary::{Arbitrary, Gen, Testable, StdGen};
use arbitrary::Status::{Discard, Fail, Pass};
use arbitrary::TestResult;

/// The main QuickCheck type for setting configuration and running QuickCheck.
pub struct QuickCheck<G> {
    tests: usize,
    max_tests: usize,
    max_size: usize,
    gen: G,
}

impl QuickCheck<StdGen<rand::XorShiftRng>> {
    /// Creates a new QuickCheck value.
    ///
    /// This can be used to run QuickCheck on things that implement
    /// `Testable`. You may also adjust the configuration, such as
    /// the number of tests to run.
    ///
    /// By default, the maximum number of passed tests is set to `100`,
    /// the max number of overall tests is set to `10000` and the generator
    /// is set to a `StdGen` with a default size of `100`.
    pub fn new() -> QuickCheck<StdGen<rand::XorShiftRng>> {
        QuickCheck {
            tests: 100,
            max_tests: 10000,
            max_size: 200,
            gen: StdGen::new(Rand::rand(&mut rand::thread_rng()), 0),
        }
    }
}

impl<G: Gen> QuickCheck<G> {
    /// Set the number of tests to run.
    ///
    /// This actually refers to the maximum number of *passed* tests that
    /// can occur. Namely, if a test causes a failure, future testing on that
    /// property stops. Additionally, if tests are discarded, there may be
    /// fewer than `tests` passed.
    pub fn tests(mut self, tests: usize) -> QuickCheck<G> {
        self.tests = tests;
        self
    }

    /// Set the maximum number of tests to run.
    ///
    /// The number of invocations of a property will never exceed this number.
    /// This is necessary to cap the number of tests because QuickCheck
    /// properties can discard tests.
    pub fn max_tests(mut self, max_tests: usize) -> QuickCheck<G> {
        self.max_tests = max_tests;
        self
    }

    /// Set the random number generator to be used by QuickCheck.
    pub fn gen(mut self, gen: G) -> QuickCheck<G> {
        self.gen = gen;
        self
    }

    /// Tests a property and returns the result.
    ///
    /// The result returned is either the number of tests passed or a witness
    /// of failure.
    ///
    /// (If you're using Rust's unit testing infrastructure, then you'll
    /// want to use the `quickcheck` method, which will `panic!` on failure.)
    pub fn quicktest<A>(&mut self, f: A) -> Result<usize, TestResult<A::Info>>
                    where A: Testable + Clone {
        let mut ntests: usize = 0;
        for i in 0..self.max_tests {
            if i < self.max_size {
                *(self.gen.size()) = i;
            }
            if ntests >= self.tests {
                break
            }
            let r = self.gen.run(f.clone());
            match r.status {
                Pass => ntests += 1,
                Discard => continue,
                Fail => return Err(r),
            }
        }
        Ok(ntests)
    }

    /// Tests a property and calls `panic!` on failure.
    ///
    /// The `panic!` message will include a (hopefully) minimal witness of
    /// failure.
    ///
    /// It is appropriate to use this method with Rust's unit testing
    /// infrastructure.
    ///
    /// Note that if the environment variable `RUST_LOG` is set to enable
    /// `info` level log messages for the `quickcheck` crate, then this will
    /// include output on how many QuickCheck tests were passed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use quickcheck::QuickCheck;
    ///
    /// fn prop_reverse_reverse() {
    ///     fn revrev(xs: Vec<usize>) -> bool {
    ///         let rev: Vec<_> = xs.clone().into_iter().rev().collect();
    ///         let revrev: Vec<_> = rev.into_iter().rev().collect();
    ///         xs == revrev
    ///     }
    ///     QuickCheck::new().quickcheck(revrev as fn(Vec<usize>) -> bool);
    /// }
    /// ```
    pub fn quickcheck<A>(&mut self, f: A) where A: Testable + Clone {
        // Ignore log init failures, implying it has already been done.
        let _ = ::env_logger::init();

        match self.quicktest(f) {
            Ok(ntests) => info!("(Passed {} QuickCheck tests.)", ntests),
            Err(result) => panic!(result.failed_msg()),
        }
    }
}

/// Convenience function for running QuickCheck.
///
/// This is an alias for `QuickCheck::new().quickcheck(f)`.
pub fn quickcheck<A: Testable + Clone>(f: A) { QuickCheck::new().quickcheck(f) }

pub struct FunDesc<In, Out> {
    args: In,
    output: Out
}

impl <In: Debug, Out: Debug> Debug for FunDesc<(In,), Out> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "fn({:?}) -> {:?}", self.args.0, self.output) // Avoid trailing comma
    }
}

macro_rules! fundesc_tuple_debug {
    ($($tyvar:ident),* ) => {
        impl <$( $tyvar : Debug,)* Out: Debug> Debug for FunDesc<($( $tyvar),*), Out> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "fn{:?} -> {:?}", self.args, self.output)
            }
        }
    }
}

fundesc_tuple_debug!();
fundesc_tuple_debug!(A, B);
fundesc_tuple_debug!(A, B, C);
fundesc_tuple_debug!(A, B, C, D);
fundesc_tuple_debug!(A, B, C, D, E);
fundesc_tuple_debug!(A, B, C, D, E, F);
fundesc_tuple_debug!(A, B, C, D, E, F, G);
fundesc_tuple_debug!(A, B, C, D, E, F, G, H);
fundesc_tuple_debug!(A, B, C, D, E, F, G, H, I);
fundesc_tuple_debug!(A, B, C, D, E, F, G, H, I, J);
fundesc_tuple_debug!(A, B, C, D, E, F, G, H, I, J, K);
fundesc_tuple_debug!(A, B, C, D, E, F, G, H, I, J, K, L);

impl <T: Debug>TestResult<T> {
    /// Produces a test result that indicates the current test has passed.
    pub fn passed() -> TestResult<()> { TestResult::<()>::from_bool(true) }

    /// Produces a test result that indicates the current test has failed.
    pub fn failed() -> TestResult<()> { TestResult::<()>::from_bool(false) }

    /// Produces a test result that indicates failure from a runtime error.
    pub fn error<S: Into<String>>(msg: S) -> TestResult<()> {
        let mut r = TestResult::<()>::from_bool(false);
        r.err = Some(msg.into());
        r
    }

    pub fn map_info<U: Debug, F: FnOnce(T) -> U>(self, f: F) -> TestResult<U> {
        let TestResult {
            status: s,
            info: i,
            err: e
        } = self;
        TestResult {
            status: s,
            info: f(i),
            err: e
        }
    }

    /// Produces a test result that instructs `quickcheck` to ignore it.
    /// This is useful for restricting the domain of your properties.
    /// When a test is discarded, `quickcheck` will replace it with a
    /// fresh one (up to a certain limit).
    pub fn discard() -> TestResult<()> {
        TestResult {
            status: Discard,
            info: (),
            err: None,
        }
    }

    /// Converts a `bool` to a `TestResult`. A `true` value indicates that
    /// the test has passed and a `false` value indicates that the test
    /// has failed.
    pub fn from_bool(b: bool) -> TestResult<()> {
        TestResult {
            status: if b { Pass } else { Fail },
            info: (),
            err: None,
        }
    }

    /// Tests if a "procedure" fails when executed. The test passes only if
    /// `f` generates a task failure during its execution.
    pub fn must_fail<U, F>(f: F) -> TestResult<()>
            where F: FnOnce() -> U, F: Send + 'static, U: Send + 'static {
        TestResult::<()>::from_bool(
            thread::Builder::new()
                            .spawn(move || { let _ = f(); })
                            .unwrap()
                            .join()
                            .is_err())
    }

    /// Returns `true` if and only if this test result describes a failing
    /// test.
    pub fn is_failure(&self) -> bool {
        match self.status {
            Fail => true,
            Pass | Discard => false,
        }
    }

    /// Returns `true` if and only if this test result describes a failing
    /// test as a result of a run time error.
    pub fn is_error(&self) -> bool {
        self.is_failure() && self.err.is_some()
    }

    fn failed_msg(&self) -> String {
        match self.err {
            None => {
                format!("\n[quickcheck] TEST FAILED. \nInfo: {:?}\n",
                        self.info)
            }
            Some(ref err) => {
                format!("\n[quickcheck] TEST FAILED (runtime error). \
                         \nInfo: {:?}\nError: {}\n",
                        self.info, err)
            }
        }
    }
}

impl <'a>Testable for &'a bool {
    type Info = bool;
    fn result<G: Gen>(self, _: &mut G) -> TestResult<bool> {
        TestResult {
            status: if *self { Pass } else { Fail },
            info: *self,
            err: None,
        }
    }
}

impl Testable for bool {
    type Info = bool;
    fn result<G: Gen>(self, g: &mut G) -> TestResult<bool> {
        (&self).result(g)
    }
}

impl <'a>Testable for &'a () {
    type Info = ();
    fn result<G: Gen>(self, _: &mut G) -> TestResult<()> {
        TestResult::<()>::passed()
    }
}

impl Testable for () {
    type Info = ();
    fn result<G: Gen>(self, _: &mut G) -> TestResult<()> {
        TestResult::<()>::passed()
    }
}

impl <'a, T: Debug + Clone> Testable for &'a TestResult<T> {
    type Info = T;
    fn result<G: Gen>(self, _: &mut G) -> TestResult<T> {
        self.clone()
    }
}

impl <T: Debug + Clone> Testable for TestResult<T> {
    type Info = T;
    fn result<G: Gen>(self, _: &mut G) -> TestResult<T> {
        self.clone()
    }
}

impl<'a, A, E> Testable for &'a Result<A, E>
    where A: Debug + Clone + Testable,
          E: Debug + Clone {
    type Info = Result<A, E>;
    fn result<G: Gen>(self, _: &mut G) -> TestResult<Result<A, E>> {
        let b = self.is_ok();
        TestResult {
            status: if b { Pass } else { Fail },
            info: match self {
                           &Ok(ref x)  => Ok((*x).clone()),
                           &Err(ref e) => Err((*e).clone())
                       },
            err: None,
        }
    }
}

impl<A, E> Testable for Result<A, E>
    where A: Debug + Clone + Testable,
          E: Debug + Clone {
    type Info = Result<A, E>;
    fn result<G: Gen>(self, g: &mut G) -> TestResult<Result<A, E>> {
        (&self).result(g)
    }
}

macro_rules! testable_fn {
    ($($tyvar:ident),*) => {
        impl<T: Testable + Send + 'static + Debug,
             $( $tyvar: Arbitrary + Debug + Send + Clone + 'static),*> Testable for fn($($tyvar),*) -> T {
            type Info = thread::Result<FunDesc<($($tyvar,)*), T::Info>>;

            #[allow(non_snake_case)]
            fn result<_G: Gen>(self, g: &mut _G) -> TestResult<Self::Info> {
                let a: ($($tyvar,)*) = Arbitrary::arbitrary(g);
                let ( $($tyvar,)* ) = a.clone();
                let f = move || {self($($tyvar),*)};
                let t = ::std::thread::Builder::new().name("safe".into());
                let x = t.spawn(f).unwrap().join();
                match x {
                    Ok(out) => {
                        out.result(g).map_info(|info|
                            Ok(FunDesc {
                                args: a,
                                output: info,
                            })
                        )
                    }
                    Err(e)  => {
                        TestResult {
                            status: Fail,
                            info: Result::Err(e),
                            err: None,
                        }
                    }
                }
            }
        }
    }
}

testable_fn!();
testable_fn!(A);
testable_fn!(A, B);
testable_fn!(A, B, C);
testable_fn!(A, B, C, D);
testable_fn!(A, B, C, D, E);
testable_fn!(A, B, C, D, E, F);
testable_fn!(A, B, C, D, E, F, G);
testable_fn!(A, B, C, D, E, F, G, H);
