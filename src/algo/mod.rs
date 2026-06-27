//! ## The Algorithm Arsenal
//!
//! Categorized similarity algorithms, each returning a normalized `f64` score
//! where `1.0` = identical and `0.0` = completely dissimilar.
//!
//! **Short Strings & Typos**
//!
//! | Algorithm | Best for | Time |
//! |---|---|---|
//! | [`levenshtein`] | Edit distance (typos) | O(n·m) |
//! | [`jaro_winkler`] | Name matching | O(n·m) |
//! | [`hamming`] | Equal-length strings | O(n) |
//!
//! **Sets & Documents**
//!
//! | Algorithm | Best for | Time |
//! |---|---|---|
//! | [`jaccard`] | N-gram set similarity | O(n+m) |
//! | [`minhash`] | Large-document fingerprints | O(k·n) |
//! | [`simhash`] | Deduplication fingerprints | O(n) |
//!
//! **Statistical Meaning**
//!
//! | Algorithm | Best for | Time |
//! |---|---|---|
//! | [`bm25`] | Search relevance ranking | O(n·m) |
//! | [`tfidf`] | Term-weighted cosine similarity | O(n·m) |
//!
//! Each function in this module takes pre-processed inputs (strings) and
//! returns an `f64` in `[0.0, 1.0]`.

pub mod bm25;
pub mod hamming;
pub mod jaccard;
pub mod jaro_winkler;
pub mod levenshtein;
pub mod minhash;
pub mod simhash;
pub mod tfidf;
