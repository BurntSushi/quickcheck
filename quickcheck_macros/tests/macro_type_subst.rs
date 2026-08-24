#![deny(trivial_casts)]
#![allow(non_upper_case_globals)]

extern crate quickcheck;
extern crate quickcheck_macros;

use quickcheck_macros::quickcheck;

// Regression for #261: #[quickcheck] inside macro_rules with a substituted
// type must not emit a trivial `as` cast.
macro_rules! my_macro {
    ($my_type:ty) => {
        #[quickcheck]
        fn fuzz_my_type(my: $my_type) {
            let _ = my;
        }
    };
}

my_macro!(u32);
