use rand;
use std::fmt::Debug;
use std::thread;
use super::{Arbitrary, Gen, StdGen};
use tester::Status::{Discard, Fail, Pass};

/// The main QuickCheck type for setting configuration and running QuickCheck.
pub struct QuickCheck<G> {
    tests: usize,
    max_tests: usize,
    gen: G,
}

impl QuickCheck<StdGen<rand::ThreadRng>> {
    /// Creates a new QuickCheck value.
    ///
    /// This can be used to run QuickCheck on things that implement
    /// `Testable`. You may also adjust the configuration, such as
    /// the number of tests to run.
    ///
    /// By default, the maximum number of passed tests is set to `100`,
    /// the max number of overall tests is set to `10000` and the generator
    /// is set to a `StdGen` with a default size of `100`.
    pub fn new() -> QuickCheck<StdGen<rand::ThreadRng>> {
        QuickCheck {
            tests: 100,
            max_tests: 10000,
            gen: StdGen::new(rand::thread_rng(), 100),
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
    pub fn quicktest<A>(&mut self, f: A) -> Result<usize, TestResult>
                    where A: Testable {
        let mut ntests: usize = 0;
        for _ in 0..self.max_tests {
            if ntests >= self.tests {
                break
            }
            match f.result(&mut self.gen) {
                TestResult { status: Pass, .. } => ntests += 1,
                TestResult { status: Discard, .. } => continue,
                r @ TestResult { status: Fail, .. } => return Err(r),
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
    pub fn quickcheck<A>(&mut self, f: A) where A: Testable {
        match self.quicktest(f) {
            Ok(ntests) => info!("(Passed {} QuickCheck tests.)", ntests),
            Err(result) => panic!(result.failed_msg()),
        }
    }
}

/// Convenience function for running QuickCheck.
///
/// This is an alias for `QuickCheck::new().quickcheck(f)`.
pub fn quickcheck<A: Testable>(f: A) { QuickCheck::new().quickcheck(f) }

/// Describes the status of a single instance of a test.
///
/// All testable things must be capable of producing a `TestResult`.
#[derive(Clone, Debug)]
pub struct TestResult {
    status: Status,
    arguments: Vec<String>,
    err: Option<String>,
}

/// Whether a test has passed, failed or been discarded.
#[derive(Clone, Debug)]
enum Status { Pass, Fail, Discard }

impl TestResult {
    /// Produces a test result that indicates the current test has passed.
    pub fn passed() -> TestResult { TestResult::from_bool(true) }

    /// Produces a test result that indicates the current test has failed.
    pub fn failed() -> TestResult { TestResult::from_bool(false) }

    /// Produces a test result that indicates failure from a runtime error.
    pub fn error<S: Into<String>>(msg: S) -> TestResult {
        let mut r = TestResult::from_bool(false);
        r.err = Some(msg.into());
        r
    }

    /// Produces a test result that instructs `quickcheck` to ignore it.
    /// This is useful for restricting the domain of your properties.
    /// When a test is discarded, `quickcheck` will replace it with a
    /// fresh one (up to a certain limit).
    pub fn discard() -> TestResult {
        TestResult {
            status: Discard,
            arguments: vec![],
            err: None,
        }
    }

    /// Converts a `bool` to a `TestResult`. A `true` value indicates that
    /// the test has passed and a `false` value indicates that the test
    /// has failed.
    pub fn from_bool(b: bool) -> TestResult {
        TestResult {
            status: if b { Pass } else { Fail },
            arguments: vec![],
            err: None,
        }
    }

    /// Tests if a "procedure" fails when executed. The test passes only if
    /// `f` generates a task failure during its execution.
    pub fn must_fail<T, F>(f: F) -> TestResult
            where F: FnOnce() -> T, F: Send + 'static, T: Send + 'static {
        TestResult::from_bool(
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
            Pass|Discard => false,
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
                format!("[quickcheck] TEST FAILED. Arguments: ({})",
                        self.arguments.connect(", "))
            }
            Some(ref err) => {
                format!("[quickcheck] TEST FAILED (runtime error). \
                         Arguments: ({})\nError: {}",
                        self.arguments.connect(", "), err)
            }
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
/// This comes with a caveat: currently, only functions with 4 parameters
/// or fewer (both `fn` and `||` types) satisfy `Testable`. If you have
/// functions to test with more than 4 parameters, please
/// [file a bug](https://github.com/BurntSushi/quickcheck/issues) and
/// I'll hopefully add it. (As of now, it would be very difficult to
/// add your own implementation outside of `quickcheck`, since the
/// functions that do shrinking are not public.)
pub trait Testable : Send + 'static {
    fn result<G: Gen>(&self, &mut G) -> TestResult;
}

impl Testable for bool {
    fn result<G: Gen>(&self, _: &mut G) -> TestResult {
        TestResult::from_bool(*self)
    }
}

impl Testable for () {
    fn result<G: Gen>(&self, _: &mut G) -> TestResult {
        TestResult::passed()
    }
}

impl Testable for TestResult {
    fn result<G: Gen>(&self, _: &mut G) -> TestResult { self.clone() }
}

impl<A, E> Testable for Result<A, E> where A: Testable, E: Debug + Send + 'static {
    fn result<G: Gen>(&self, g: &mut G) -> TestResult {
        match *self {
            Ok(ref r) => r.result(g),
            Err(ref err) => TestResult::error(format!("{:?}", err)),
        }
    }
}

impl<T> Testable for fn() -> T where T: Testable {
    fn result<G: Gen>(&self, g: &mut G) -> TestResult {
        shrink::<G, T, (), (), (), (), fn() -> T>(g, self)
    }
}

impl<A, T> Testable for fn(A) -> T where A: AShow, T: Testable {
    fn result<G: Gen>(&self, g: &mut G) -> TestResult {
        shrink::<G, T, A, (), (), (), fn(A) -> T>(g, self)
    }
}

impl<A, B, T> Testable for fn(A, B) -> T
        where A: AShow, B: AShow, T: Testable {
    fn result<G: Gen>(&self, g: &mut G) -> TestResult {
        shrink::<G, T, A, B, (), (), fn(A, B) -> T>(g, self)
    }
}

impl<A, B, C, T> Testable for fn(A, B, C) -> T
        where A: AShow, B: AShow, C: AShow, T: Testable {
    fn result<G: Gen>(&self, g: &mut G) -> TestResult {
        shrink::<G, T, A, B, C, (), fn(A, B, C) -> T>(g, self)
    }
}

impl<A, B, C, D, T,> Testable for fn(A, B, C, D) -> T
        where A: AShow, B: AShow, C: AShow, D: AShow, T: Testable {
    fn result<G: Gen>(&self, g: &mut G) -> TestResult {
        shrink::<G, T, A, B, C, D, fn(A, B, C, D) -> T>(g, self)
    }
}

trait Fun<A, B, C, D, T> {
    fn call<G>(&self, g: &mut G,
               a: Option<&A>, b: Option<&B>,
               c: Option<&C>, d: Option<&D>)
              -> TestResult
              where G: Gen;
}

macro_rules! impl_fun_call {
    ($f:expr, $g:expr, $($name:ident,)+) => ({
        let ($($name,)*) = ($($name.unwrap(),)*);
        let f = $f;
        let mut r = {
            let ($($name,)*) = ($(Box::new($name.clone()),)*);
            safe(move || { f($(*$name,)*) }).result($g)
        };
        if r.is_failure() {
            r.arguments = vec![$(format!("{:?}", $name),)*];
        }
        r
    });
}

impl<A, B, C, D, T> Fun<A, B, C, D, T> for fn() -> T
    where A: AShow, B: AShow, C: AShow, D: AShow, T: Testable {
    fn call<G>(&self, g: &mut G,
               _: Option<&A>, _: Option<&B>,
               _: Option<&C>, _: Option<&D>)
              -> TestResult where G: Gen {
        let f = *self;
        safe(move || { f() }).result(g)
    }
}

impl<A, B, C, D, T> Fun<A, B, C, D, T> for fn(A) -> T
    where A: AShow, B: AShow, C: AShow, D: AShow, T: Testable {
    fn call<G>(&self, g: &mut G,
               a: Option<&A>, _: Option<&B>,
               _: Option<&C>, _: Option<&D>)
              -> TestResult where G: Gen {
        impl_fun_call!(*self, g, a,)
    }
}

impl<A, B, C, D, T> Fun<A, B, C, D, T> for fn(A, B) -> T
    where A: AShow, B: AShow, C: AShow, D: AShow, T: Testable {
    fn call<G>(&self, g: &mut G,
               a: Option<&A>, b: Option<&B>,
               _: Option<&C>, _: Option<&D>)
              -> TestResult where G: Gen {
        impl_fun_call!(*self, g, a, b,)
    }
}

impl<A, B, C, D, T> Fun<A, B, C, D, T> for fn(A, B, C) -> T
    where A: AShow, B: AShow, C: AShow, D: AShow, T: Testable {
    fn call<G>(&self, g: &mut G,
               a: Option<&A>, b: Option<&B>,
               c: Option<&C>, _: Option<&D>)
              -> TestResult where G: Gen {
        impl_fun_call!(*self, g, a, b, c,)
    }
}

impl<A, B, C, D, T> Fun<A, B, C, D, T> for fn(A, B, C, D) -> T
    where A: AShow, B: AShow, C: AShow, D: AShow, T: Testable {
    fn call<G>(&self, g: &mut G,
               a: Option<&A>, b: Option<&B>,
               c: Option<&C>, d: Option<&D>)
              -> TestResult where G: Gen {
        impl_fun_call!(*self, g, a, b, c, d,)
    }
}

fn shrink<G, T, A, B, C, D, F>(g: &mut G, fun: &F) -> TestResult
    where G: Gen, T: Testable, A: AShow, B: AShow, C: AShow, D: AShow,
          F: Fun<A, B, C, D, T> {
    let (a, b, c, d): (A, B, C, D) = Arbitrary::arbitrary(g);
    let r = fun.call(g, Some(&a), Some(&b), Some(&c), Some(&d));
    match r.status {
        Pass|Discard => r,
        Fail => shrink_failure(g, (a, b, c, d).shrink(), fun).unwrap_or(r),
    }
}

fn shrink_failure<G, T, A, B, C, D, F>
                 (g: &mut G,
                  shrinker: Box<Iterator<Item=(A, B, C, D)>>,
                  fun: &F)
                 -> Option<TestResult>
    where G: Gen, T: Testable, A: AShow, B: AShow, C: AShow, D: AShow,
          F: Fun<A, B, C, D, T> {
    for (a, b, c, d) in shrinker {
        let r = fun.call(g, Some(&a), Some(&b), Some(&c), Some(&d));
        match r.status {
            // The shrunk value does not witness a failure, so
            // throw it away.
            Pass|Discard => continue,

            // The shrunk value *does* witness a failure, so keep trying
            // to shrink it.
            Fail => {
                let shrunk = shrink_failure(g, (a, b, c, d).shrink(), fun);

                // If we couldn't witness a failure on any shrunk value,
                // then return the failure we already have.
                return Some(shrunk.unwrap_or(r))
            },
        }
    }
    None
}

// TODO(burntsushi): Use `std::thread::catch_panic` once it stabilizes.
fn safe<T, F>(fun: F) -> Result<T, String>
        where F: FnOnce() -> T, F: Send + 'static, T: Send + 'static {
    let t = ::std::thread::Builder::new().name("safe".into());
    t.spawn(fun).unwrap().join().map_err(|any_err| {
        match any_err.downcast_ref::<&Debug>() {
            Some(ref s) => format!("{:?}", s),
            None => "UNABLE TO SHOW RESULT OF PANIC.".into(),
        }
    })
}

/// Convenient aliases.
trait AShow : Arbitrary + Debug {}
impl<A: Arbitrary + Debug> AShow for A {}
