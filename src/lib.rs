//! # SIMI -- Similarity Toolkit
//!
//! A general-purpose toolkit of similarity checks, designed to protect
//! developers from wasting compute and money on LLMs for simple tasks.
//!
//! ## Feature flags
//!
//! - `std` (default on) -- enables standard library integration.
//! - `python` -- enables Python bindings via PyO3.
//! - `nodejs` -- enables Node.js bindings via napi-rs.
//!
//! ## Quick start
//!
//! ```rust
//! use simi::algo::{levenshtein, jaro_winkler};
//!
//! let d = levenshtein::similarity("kitten", "sitting");
//! assert!((d - 0.571).abs() < 0.01);
//!
//! let j = jaro_winkler::similarity("MARTHA", "MARHTA");
//! assert!((j - 0.961).abs() < 0.01);
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

pub mod algo;
pub mod error;
pub mod prelude;

pub mod batch;
pub mod preprocess;
pub mod router;

#[cfg(feature = "python")]
pub mod python;

#[cfg(feature = "nodejs")]
pub mod nodejs;

pub use algo::*;
pub use batch::BatchComparator;
pub use error::SimiError;
pub use preprocess::Preprocessor;
pub use router::{resolve_intent, Intent, SimiFlow, Strategy, Threshold};
