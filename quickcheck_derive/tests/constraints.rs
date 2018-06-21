#[macro_use]
extern crate quickcheck;
#[macro_use]
extern crate quickcheck_derive;

#[derive(Arbitrary, Clone, Debug)]
#[arbitrary(constraint = "self.alpha == self.bravo.is_positive()")]
struct TestStruct {
    alpha: bool,
    bravo: isize,
}

quickcheck! {
    fn struct_constraint(t: TestStruct) -> bool {
        t.alpha == t.bravo.is_positive()
    }
}
