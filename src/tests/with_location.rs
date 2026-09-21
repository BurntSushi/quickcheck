use crate::quickcheck;

// This test is in its own file because it depends on its own line number, which could
// easily break if more tests are added in the same file.
#[test]
#[should_panic(
    expected = r#"[quickcheck] TEST FAILED (runtime error) at src/tests/with_location.rs:13:9.
No Arguments Provided
Error: explicit panic"#
)]
fn panic_msg() {
    fn prop() {
        panic!("explicit panic");
    }
    quickcheck(prop as fn() -> ());
}
