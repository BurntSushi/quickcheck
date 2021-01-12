use std::cmp;
use std::env;
use std::fmt::Debug;
use std::panic;

use crate::{
    tester::Status::{Discard, Fail, Pass},
    Arbitrary, Gen,
};

/// The main QuickCheck type for setting configuration and running QuickCheck.
pub struct QuickCheck {
    tests: u64,
    max_tests: u64,
    min_tests_passed: u64,
    gen: Gen,
}

fn qc_tests() -> u64 {
    let default = 100;
    match env::var("QUICKCHECK_TESTS") {
        Ok(val) => val.parse().unwrap_or(default),
        Err(_) => default,
    }
}

fn qc_max_tests() -> u64 {
    let default = 10_000;
    match env::var("QUICKCHECK_MAX_TESTS") {
        Ok(val) => val.parse().unwrap_or(default),
        Err(_) => default,
    }
}

fn qc_gen_size() -> usize {
    let default = 100;
    match env::var("QUICKCHECK_GENERATOR_SIZE") {
        Ok(val) => val.parse().unwrap_or(default),
        Err(_) => default,
    }
}

fn qc_min_tests_passed() -> u64 {
    let default = 0;
    match env::var("QUICKCHECK_MIN_TESTS_PASSED") {
        Ok(val) => val.parse().unwrap_or(default),
        Err(_) => default,
    }
}

impl QuickCheck {
    /// Creates a new QuickCheck value.
    ///
    /// This can be used to run QuickCheck on things that implement `Testable`.
    /// You may also adjust the configuration, such as the number of tests to
    /// run.
    ///
    /// By default, the maximum number of passed tests is set to `100`, the max
    /// number of overall tests is set to `10000` and the generator is created
    /// with a size of `100`.
    pub fn new() -> QuickCheck {
        let gen = Gen::new(qc_gen_size());
        let tests = qc_tests();
        let max_tests = cmp::max(tests, qc_max_tests());
        let min_tests_passed = qc_min_tests_passed();

        QuickCheck { tests, max_tests, min_tests_passed, gen }
    }

    /// Set the random number generator to be used by QuickCheck.
    pub fn gen(self, gen: Gen) -> QuickCheck {
        QuickCheck { gen, ..self }
    }

    /// Set the number of tests to run.
    ///
    /// This actually refers to the maximum number of *passed* tests that
    /// can occur. Namely, if a test causes a failure, future testing on that
    /// property stops. Additionally, if tests are discarded, there may be
    /// fewer than `tests` passed.
    pub fn tests(mut self, tests: u64) -> QuickCheck {
        self.tests = tests;
        self
    }

    /// Set the maximum number of tests to run.
    ///
    /// The number of invocations of a property will never exceed this number.
    /// This is necessary to cap the number of tests because QuickCheck
    /// properties can discard tests.
    pub fn max_tests(mut self, max_tests: u64) -> QuickCheck {
        self.max_tests = max_tests;
        self
    }

    /// Set the minimum number of tests that needs to pass.
    ///
    /// This actually refers to the minimum number of *valid* *passed* tests
    /// that needs to pass for the property to be considered successful.
    pub fn min_tests_passed(mut self, min_tests_passed: u64) -> QuickCheck {
        self.min_tests_passed = min_tests_passed;
        self
    }

    /// Tests a property and returns the result.
    ///
    /// The result returned is either the number of tests passed or a witness
    /// of failure.
    ///
    /// (If you're using Rust's unit testing infrastructure, then you'll
    /// want to use the `quickcheck` method, which will `panic!` on failure.)
    pub fn quicktest<A>(&mut self, f: A) -> Result<u64, TestResult>
    where
        A: Testable,
    {
        let mut n_tests_passed = 0;
        for _ in 0..self.max_tests {
            if n_tests_passed >= self.tests {
                break;
            }
            match f.result(&mut self.gen) {
                TestResult { status: Pass, .. } => n_tests_passed += 1,
                TestResult { status: Discard, .. } => continue,
                r @ TestResult { status: Fail, .. } => return Err(r),
            }
        }
        Ok(n_tests_passed)
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
    pub fn quickcheck<A>(&mut self, f: A)
    where
        A: Testable,
    {
        // Ignore log init failures, implying it has already been done.
        let _ = crate::env_logger_init();

        let n_tests_passed = match self.quicktest(f) {
            Ok(n_tests_passed) => n_tests_passed,
            Err(result) => panic!(result.failed_msg()),
        };

        if n_tests_passed >= self.min_tests_passed {
            info!("(Passed {} QuickCheck tests.)", n_tests_passed)
        } else {
            panic!(
                "(Unable to generate enough tests, {} not discarded.)",
                n_tests_passed
            )
        }
    }
}

/// Convenience function for running QuickCheck.
///
/// This is an alias for `QuickCheck::new().quickcheck(f)`.
pub fn quickcheck<A: Testable>(f: A) {
    QuickCheck::new().quickcheck(f)
}

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
enum Status {
    Pass,
    Fail,
    Discard,
}

impl TestResult {
    /// Produces a test result that indicates the current test has passed.
    pub fn passed() -> TestResult {
        TestResult::from_bool(true)
    }

    /// Produces a test result that indicates the current test has failed.
    pub fn failed() -> TestResult {
        TestResult::from_bool(false)
    }

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
        TestResult { status: Discard, arguments: vec![], err: None }
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
    where
        F: FnOnce() -> T,
        F: 'static,
        T: 'static,
    {
        let f = panic::AssertUnwindSafe(f);
        TestResult::from_bool(panic::catch_unwind(f).is_err())
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
            None => format!(
                "[quickcheck] TEST FAILED. Arguments: ({})",
                self.arguments.join(", ")
            ),
            Some(ref err) => format!(
                "[quickcheck] TEST FAILED (runtime error). \
                 Arguments: ({})\nError: {}",
                self.arguments.join(", "),
                err
            ),
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
pub trait Testable: 'static {
    fn result(&self, _: &mut Gen) -> TestResult;
}

impl Testable for bool {
    fn result(&self, _: &mut Gen) -> TestResult {
        TestResult::from_bool(*self)
    }
}

impl Testable for () {
    fn result(&self, _: &mut Gen) -> TestResult {
        TestResult::passed()
    }
}

impl Testable for TestResult {
    fn result(&self, _: &mut Gen) -> TestResult {
        self.clone()
    }
}

impl<A, E> Testable for Result<A, E>
where
    A: Testable,
    E: Debug + 'static,
{
    fn result(&self, g: &mut Gen) -> TestResult {
        match *self {
            Ok(ref r) => r.result(g),
            Err(ref err) => TestResult::error(format!("{:?}", err)),
        }
    }
}

/// Return a vector of the debug formatting of each item in `args`
fn debug_reprs(args: &[&dyn Debug]) -> Vec<String> {
    args.iter().map(|x| format!("{:?}", x)).collect()
}

macro_rules! testable_fn {
    ($($name: ident),*) => {

impl<T: Testable,
     $($name: Arbitrary + Debug),*> Testable for fn($($name),*) -> T {
    #[allow(non_snake_case)]
    fn result(&self, g: &mut Gen) -> TestResult {
        fn shrink_failure<T: Testable, $($name: Arbitrary + Debug),*>(
            g: &mut Gen,
            self_: fn($($name),*) -> T,
            a: ($($name,)*),
        ) -> Option<TestResult> {
            for t in a.shrink() {
                let ($($name,)*) = t.clone();
                let mut r_new = safe(move || {self_($($name),*)}).result(g);
                if r_new.is_failure() {
                    {
                        let ($(ref $name,)*) : ($($name,)*) = t;
                        r_new.arguments = debug_reprs(&[$($name),*]);
                    }

                    // The shrunk value *does* witness a failure, so keep
                    // trying to shrink it.
                    let shrunk = shrink_failure(g, self_, t);

                    // If we couldn't witness a failure on any shrunk value,
                    // then return the failure we already have.
                    return Some(shrunk.unwrap_or(r_new))
                }
            }
            None
        }

        let self_ = *self;
        let a: ($($name,)*) = Arbitrary::arbitrary(g);
        let ( $($name,)* ) = a.clone();
        let mut r = safe(move || {self_($($name),*)}).result(g);

        {
            let ( $(ref $name,)* ) = a;
            r.arguments = debug_reprs(&[$($name),*]);
        }
        match r.status {
            Pass|Discard => r,
            Fail => {
                shrink_failure(g, self_, a).unwrap_or(r)
            }
        }
    }
}}}

testable_fn!();
testable_fn!(A);
testable_fn!(A, B);
testable_fn!(A, B, C);
testable_fn!(A, B, C, D);
testable_fn!(A, B, C, D, E);
testable_fn!(A, B, C, D, E, F);
testable_fn!(A, B, C, D, E, F, G);
testable_fn!(A, B, C, D, E, F, G, H);

fn safe<T, F>(fun: F) -> Result<T, String>
where
    F: FnOnce() -> T,
    F: 'static,
    T: 'static,
{
    panic::catch_unwind(panic::AssertUnwindSafe(fun)).map_err(|any_err| {
        // Extract common types of panic payload:
        // panic and assert produce &str or String
        if let Some(&s) = any_err.downcast_ref::<&str>() {
            s.to_owned()
        } else if let Some(s) = any_err.downcast_ref::<String>() {
            s.to_owned()
        } else {
            "UNABLE TO SHOW RESULT OF PANIC.".to_owned()
        }
    })
}

/// Convenient aliases.
trait AShow: Arbitrary + Debug {}
impl<A: Arbitrary + Debug> AShow for A {}

#[cfg(test)]
mod test {
    use crate::{Gen, QuickCheck};

    #[test]
    fn shrinking_regression_issue_126() {
        fn thetest(vals: Vec<bool>) -> bool {
            vals.iter().filter(|&v| *v).count() < 2
        }
        let failing_case = QuickCheck::new()
            .quicktest(thetest as fn(vals: Vec<bool>) -> bool)
            .unwrap_err();
        let expected_argument = format!("{:?}", [true, true]);
        assert_eq!(failing_case.arguments, vec![expected_argument]);
    }

    #[test]
    fn size_for_small_types_issue_143() {
        fn t(_: i8) -> bool {
            true
        }
        QuickCheck::new().gen(Gen::new(129)).quickcheck(t as fn(i8) -> bool);
    }

    #[test]
    fn regression_signed_shrinker_panic() {
        fn foo_can_shrink(v: i8) -> bool {
            let _ = crate::Arbitrary::shrink(&v).take(100).count();
            true
        }
        crate::quickcheck(foo_can_shrink as fn(i8) -> bool);
    }
}
