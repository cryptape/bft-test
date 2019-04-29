//! A common test framework for BFT consensus algorithm.
//!
//!

#![deny(missing_docs)]

/// Blackbox testing module.
pub mod blackbox;
/// Error module.
pub mod error;
/// WhiteBox testing module.
pub mod whitebox;

/// Re-pub whitebox test cases;
pub use crate::whitebox::correctness::test_case;
