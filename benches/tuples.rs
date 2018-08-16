#![feature(test)]

extern crate quickcheck;
extern crate rand;
extern crate test;

use quickcheck::{Arbitrary, StdGen};
use rand::prng::hc128::Hc128Rng;
use rand::SeedableRng;
use test::Bencher;

macro_rules! bench_shrink {
    ($(($fn_name:ident, $type:ty),)*) => {
        $(
            #[bench]
            fn $fn_name(b: &mut Bencher) {
                // Use a deterministic generator to benchmark on the same data
                let mut gen = StdGen::new(Hc128Rng::from_seed([0u8; 32]), 100);
                let value: $type = Arbitrary::arbitrary(&mut gen);

                b.iter(|| {
                    for _ in value.shrink() {
                        // Do nothing
                    }
                });
            }
        )*
    };
}

bench_shrink! {
    (shrink_string_1_tuple, (String,)),
    (shrink_string_2_tuple, (String, String)),
    (shrink_string_3_tuple, (String, String, String)),
    (shrink_string_4_tuple, (String, String, String, String)),
    (shrink_string_5_tuple, (String, String, String, String, String)),
    (shrink_string_6_tuple, (String, String, String, String, String, String)),
    (shrink_string_7_tuple, (String, String, String, String, String, String, String)),
    (shrink_string_8_tuple, (String, String, String, String, String, String, String, String)),

    (shrink_vec_u8_1_tuple, (Vec<u8>,)),
    (shrink_vec_u8_2_tuple, (Vec<u8>, Vec<u8>)),
    (shrink_vec_u8_3_tuple, (Vec<u8>, Vec<u8>, Vec<u8>)),
    (shrink_vec_u8_4_tuple, (Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>)),
    (shrink_vec_u8_5_tuple, (Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>)),
    (shrink_vec_u8_6_tuple, (Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>)),
    (shrink_vec_u8_7_tuple, (Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>)),
    (shrink_vec_u8_8_tuple, (Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>)),

    (shrink_u64_1_tuple, (u64,)),
    (shrink_u64_2_tuple, (u64, u64)),
    (shrink_u64_3_tuple, (u64, u64, u64)),
    (shrink_u64_4_tuple, (u64, u64, u64, u64)),
    (shrink_u64_5_tuple, (u64, u64, u64, u64, u64)),
    (shrink_u64_6_tuple, (u64, u64, u64, u64, u64, u64)),
    (shrink_u64_7_tuple, (u64, u64, u64, u64, u64, u64, u64)),
    (shrink_u64_8_tuple, (u64, u64, u64, u64, u64, u64, u64, u64)),

    (shrink_i64_1_tuple, (i64,)),
    (shrink_i64_2_tuple, (i64, i64)),
    (shrink_i64_3_tuple, (i64, i64, i64)),
    (shrink_i64_4_tuple, (i64, i64, i64, i64)),
    (shrink_i64_5_tuple, (i64, i64, i64, i64, i64)),
    (shrink_i64_6_tuple, (i64, i64, i64, i64, i64, i64)),
    (shrink_i64_7_tuple, (i64, i64, i64, i64, i64, i64, i64)),
    (shrink_i64_8_tuple, (i64, i64, i64, i64, i64, i64, i64, i64)),

    (shrink_f64_1_tuple, (f64,)),
    (shrink_f64_2_tuple, (f64, f64)),
    (shrink_f64_3_tuple, (f64, f64, f64)),
    (shrink_f64_4_tuple, (f64, f64, f64, f64)),
    (shrink_f64_5_tuple, (f64, f64, f64, f64, f64)),
    (shrink_f64_6_tuple, (f64, f64, f64, f64, f64, f64)),
    (shrink_f64_7_tuple, (f64, f64, f64, f64, f64, f64, f64)),
    (shrink_f64_8_tuple, (f64, f64, f64, f64, f64, f64, f64, f64)),

    (shrink_unit_1_tuple, ((),)),
    (shrink_unit_2_tuple, ((), ())),
    (shrink_unit_3_tuple, ((), (), ())),
    (shrink_unit_4_tuple, ((), (), (), ())),
    (shrink_unit_5_tuple, ((), (), (), (), ())),
    (shrink_unit_6_tuple, ((), (), (), (), (), ())),
    (shrink_unit_7_tuple, ((), (), (), (), (), (), ())),
    (shrink_unit_8_tuple, ((), (), (), (), (), (), (), ())),
}
