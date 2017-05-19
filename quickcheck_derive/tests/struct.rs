#[macro_use]
extern crate quickcheck;
#[macro_use]
extern crate quickcheck_derive;

#[derive(Arbitrary, Clone, Debug)]
struct Foo {
    alpha: i32,
    bravo: isize,
}

#[derive(Arbitrary, Clone, Debug)]
struct Bar(bool, u32, char);

#[derive(Arbitrary, Clone, Debug)]
struct Baz();

#[derive(Arbitrary, Clone, Debug)]
struct Quux;

quickcheck! {
    fn ensure_arbitrary_is_impld_for_foo(_foo: Foo) -> bool {
        true
    }
}

quickcheck! {
    fn ensure_arbitrary_is_impld_for_bar(_bar: Bar) -> bool {
        true
    }
}

quickcheck! {
    fn ensure_arbitrary_is_impld_for_baz(_baz: Baz) -> bool {
        true
    }
}

quickcheck! {
    fn ensure_arbitrary_is_impld_for_quux(_quux: Quux) -> bool {
        true
    }
}
