#[macro_use]
extern crate quickcheck;
#[macro_use]
extern crate quickcheck_derive;

#[derive(Arbitrary, Clone, Debug)]
enum Xyzzy {
    Alpha,
    Bravo(char),
    Charlie(Vec<char>, Vec<u8>),
    Delta(Option<char>),
    Echo(()),
    Foxtrot {
        one: bool,
        two: (),
        three: isize,
    },
    Golf {},
    Hotel {
        foo: usize,
    },
}

quickcheck! {
    fn ensure_arbitrary_is_impld_for_xyzzy(_xyzzy: Xyzzy) -> bool {
        true
    }
}
