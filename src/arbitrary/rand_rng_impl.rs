//! Implement the `RngCore` trait from `rand_core` for `quickcheck::Gen`.
//! This allows `quickcheck` to interoperate with other crates that rely on `rand_core`/`rand` as
//! their interface for sources of randomness.
//!
//! The `RngCore` implementations are gated behind opt-in feature flags that are explicitly tied to a
//! pinned version of `rand_core`.
//! If a new version of `rand_core` is released, `quickcheck` can add a new `use_rand_core_X_X`
//! feature flag to enable interoperability without compromising its API stability guarantees.

#[cfg(feature = "use_rand_core_0_6")]
mod rand_core_0_6 {
    use crate::Gen;
    use rand::Error;

    impl rand_core_0_6::RngCore for Gen {
        fn next_u32(&mut self) -> u32 {
            self.rng.next_u32()
        }

        fn next_u64(&mut self) -> u64 {
            self.rng.next_u64()
        }

        fn fill_bytes(&mut self, dest: &mut [u8]) {
            self.rng.fill_bytes(dest)
        }

        fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
            self.rng.try_fill_bytes(dest)
        }
    }
}
