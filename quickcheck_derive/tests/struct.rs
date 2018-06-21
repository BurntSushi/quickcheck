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

#[derive(Arbitrary, Clone, Debug)]
struct Far(i32);


#[derive(Arbitrary, Clone, Debug)]
struct ReallyBigStruct {
    alpha: i32,
    bravo: isize,
    charlie: usize,
    delta: u32,
    echo: i8,
    foxtrot: u8,
    golf: isize,
    hotel: usize,
    india: i32
}

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

quickcheck! {
    fn ensure_arbitrary_is_impld_for_far(_far: Far) -> bool {
        true
    }
}

quickcheck! {
    fn ensure_arbitrary_is_impld_for_really_big_struct(_really_big_struct: ReallyBigStruct) -> bool {
        true
    }
}
