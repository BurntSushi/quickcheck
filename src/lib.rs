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

pub use arbitrary::{Arbitrary, Gen, StdGen, arbitrary, default_gen, gen};
pub use shrink::{ObjIter, Shrink};
pub use tester::{Testable, TestResult, Status};

mod arbitrary;
mod shrink;

mod tester {
    use std::fmt::Show;

    use super::{Arbitrary, Gen};

    fn arby<A: Arbitrary, G: Gen>(g: &mut G) -> A { Arbitrary::arbitrary(g) }

    #[deriving(Clone, Show)]
    pub struct TestResult {
        status: Status,
        arguments: ~[~str],
    }

    #[deriving(Clone, Show)]
    pub enum Status { Pass, Fail, Discard }

    impl TestResult {
        pub fn passed() -> ~TestResult { TestResult::from_bool(true) }
        pub fn failed() -> ~TestResult { TestResult::from_bool(false) }
        pub fn discard() -> ~TestResult {
            ~TestResult { status: Discard, arguments: ~[] }
        }
        pub fn from_bool(b: bool) -> ~TestResult {
            ~TestResult { status: if b { Pass } else { Fail }, arguments: ~[] }
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
            (*self)().result(g)
        }
    }

    impl<A: Arbitrary + Show, B: Testable> Testable for 'static |A| -> B {
        fn result<G: Gen>(&self, g: &mut G) -> ~TestResult {
            let a: A = Arbitrary::arbitrary(g);
            let arg: ~str = a.to_str();
            let mut r: ~TestResult = (*self)(a).result(g);
            r.arguments.unshift(arg);
            r
        }
    }

    impl<A: Arbitrary + Show, B: Arbitrary + Show, C: Testable>
        Testable for 'static |A, B| -> C {
        fn result<G: Gen>(&self, g: &mut G) -> ~TestResult {
            let (a, b): (A, B) = (arby(g), arby(g));
            let (a_arg, b_arg) = (a.to_str(), b.to_str());
            let mut r: ~TestResult = (*self)(a, b).result(g);
            r.arguments.unshift(b_arg);
            r.arguments.unshift(a_arg);
            r
        }
    }

    impl<A: Arbitrary + Show,
         B: Arbitrary + Show,
         C: Arbitrary + Show,
         D: Testable>
        Testable for 'static |A, B, C| -> D {
        fn result<G: Gen>(&self, g: &mut G) -> ~TestResult {
            let (a, b, c): (A, B, C) = (arby(g), arby(g), arby(g));
            let (a_arg, b_arg, c_arg) = (a.to_str(), b.to_str(), c.to_str());
            let mut r: ~TestResult = (*self)(a, b, c).result(g);
            r.arguments.unshift(c_arg);
            r.arguments.unshift(b_arg);
            r.arguments.unshift(a_arg);
            r
        }
    }

    impl<A: Testable> Testable for fn() -> A {
        fn result<G: Gen>(&self, g: &mut G) -> ~TestResult {
            (*self)().result(g)
        }
    }

    impl<A: Arbitrary + Show, B: Testable> Testable for fn(A) -> B {
        fn result<G: Gen>(&self, g: &mut G) -> ~TestResult {
            let a: A = Arbitrary::arbitrary(g);
            let arg: ~str = a.to_str();
            let mut r: ~TestResult = (*self)(a).result(g);
            r.arguments.unshift(arg);
            r
        }
    }

    impl<A: Arbitrary + Show, B: Arbitrary + Show, C: Testable>
        Testable for fn(A, B) -> C {
        fn result<G: Gen>(&self, g: &mut G) -> ~TestResult {
            let (a, b): (A, B) = (arby(g), arby(g));
            let (a_arg, b_arg) = (a.to_str(), b.to_str());
            let mut r: ~TestResult = (*self)(a, b).result(g);
            r.arguments.unshift(b_arg);
            r.arguments.unshift(a_arg);
            r
        }
    }

    impl<A: Arbitrary + Show,
         B: Arbitrary + Show,
         C: Arbitrary + Show,
         D: Testable>
        Testable for fn(A, B, C) -> D {
        fn result<G: Gen>(&self, g: &mut G) -> ~TestResult {
            let (a, b, c): (A, B, C) = (arby(g), arby(g), arby(g));
            let (a_arg, b_arg, c_arg) = (a.to_str(), b.to_str(), c.to_str());
            let mut r: ~TestResult = (*self)(a, b, c).result(g);
            r.arguments.unshift(c_arg);
            r.arguments.unshift(b_arg);
            r.arguments.unshift(a_arg);
            r
        }
    }
}

#[cfg(test)]
mod test {
    use std::iter;

    use super::{Testable, TestResult, default_gen};

    #[test]
    fn reverse_reverse() {
        let g = &mut default_gen();
        fn revrev(xs: ~[uint]) -> bool {
            let rev = xs.clone().move_rev_iter().to_owned_vec()
                        .move_rev_iter().to_owned_vec();
            xs == rev
        }
        for _ in iter::range(0, 10) {
            debug!("{}", (revrev).result(g));
        }
    }

    #[test]
    fn reverse_single() {
        let g = &mut default_gen();
        fn rev_single(xs: ~[uint]) -> ~TestResult {
            if xs.len() != 1 {
                return TestResult::discard()
            }
            return TestResult::from_bool(
                xs.clone().move_rev_iter().to_owned_vec()
                ==
                xs.clone().move_rev_iter().to_owned_vec()
            )
        }
        for _ in iter::range(0, 10) {
            debug!("{}", (rev_single).result(g));
        }
    }

    #[test]
    fn reverse_app() {
        let g = &mut default_gen();
        fn revapp(xs: ~[uint], ys: ~[uint]) -> bool {
            let app = ::std::vec::append(xs.clone(), ys);
            let app_rev = app.move_rev_iter().to_owned_vec();

            let rxs = xs.clone().move_rev_iter().to_owned_vec();
            let rys = ys.clone().move_rev_iter().to_owned_vec();
            let rev_app = ::std::vec::append(rys, rxs);

            app_rev == rev_app
        }
        for _ in iter::range(0, 10) {
            debug!("{}", (revapp).result(g));
        }
    }
}
