use crate::quickcheck;

// This test is in its own file because it depends on its own line number, which could
// easily break if more tests are added in the same file.

#[cfg(target_os = "windows")]
#[test]
#[should_panic(
    expected = r#"[quickcheck] TEST FAILED (runtime error) at src\tests\with_location.rs:15:9.
No Arguments Provided
Error: explicit panic"#
)]
fn panic_msg() {
    fn prop() {
        panic!("explicit panic");
    }
    quickcheck(prop as fn() -> ());
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
#[should_panic(
    expected = r#"[quickcheck] TEST FAILED (runtime error) at src/tests/with_location.rs:29:9.
No Arguments Provided
Error: explicit panic"#
)]
fn panic_msg() {
    fn prop() {
        panic!("explicit panic");
    }
    quickcheck(prop as fn() -> ());
}
