//! # negative-space-testing
//!
//! A testing framework where you define what your system does NOT do,
//! and the framework verifies the negative space.
//!
//! Inspired by the insight that the pattern lives in the holes — the Jacquard
//! card specifies where the thread must rise by being absent where it must not.
//! A meteorologist who knows the cloud names too quickly loses the dragon.
//! Test the absences. Leave room for the imagination to move.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod cathedral;
pub mod conservation;
pub mod crackle;
pub mod negative_test;
pub mod space_map;

pub use cathedral::{CathedralProbe, ProbeResult, RelationshipViolation};
pub use conservation::{ConservationChecker, ConservationResult, Monotonicity, QuantityViolation};
pub use crackle::{CrackleOutcome, CracklePhase, CrackleResult};
pub use negative_test::{NegativeTest, NegativeViolation};
pub use space_map::{SpaceMap, SpaceResult};
