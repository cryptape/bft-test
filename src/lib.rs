//! A common test framework for BFT consensus algorithm.
//!
//!

#![deny(missing_docs)]

/// Blackbox testing module.
pub mod blackbox;
///
pub mod error;
/// WhiteBox testing module.
pub mod whitebox;

pub use crate::whitebox::correctness::test_case;
