//! Example: using preprocessing to normalize data before comparison.
//!
//! Run with: `cargo run --example preprocess`

use simi::algo::{bm25, levenshtein};
use simi::preprocess::Preprocessor;

fn main() {
    let raw_pairs = [
        ("  The   Quick Brown Fox  ", "the quick brown fox"),
        ("Hello, world!", "hello world"),
        ("\u{0065}\u{0301}t\u{0301}udier", "\u{00e9}tudier"), // decomposed vs precomposed
        (
            "the quick brown fox jumps over the lazy dog",
            "a lazy dog jumps over the quick brown fox",
        ),
    ];

    let pre = Preprocessor::new()
        .with_lowercase(true)
        .with_collapse_whitespace(true)
        .with_trim(true)
        .with_normalize_unicode(true);

    println!(
        "{:<40} {:<40} {:>12} {:>12}",
        "Raw A", "Raw B", "Raw Score", "Preproc Score"
    );
    println!("{}", "-".repeat(110));

    for (a, b) in raw_pairs {
        let raw_score = levenshtein::similarity(a, b);
        let pa = pre.process(a);
        let pb = pre.process(b);
        let preproc_score = levenshtein::similarity(&pa, &pb);

        println!(
            "{:<40} {:<40} {:>12.3} {:>12.3}",
            truncate(a, 40),
            truncate(b, 40),
            raw_score,
            preproc_score
        );
    }

    println!();
    println!("After preprocessing, the cleaned strings:");
    for (a, b) in raw_pairs {
        println!("  \"{}\" -> \"{}\"", a, pre.process(a));
        println!("  \"{}\" -> \"{}\"", b, pre.process(b));
        println!();
    }

    // Show stopword removal
    let pre_with_sw = Preprocessor::new()
        .with_lowercase(true)
        .with_remove_stopwords(true);

    let doc = "The quick brown fox jumps over the lazy dog";
    println!("Stopword removal:");
    println!("  Input:   \"{}\"", doc);
    println!("  Cleaned: \"{}\"", pre_with_sw.process(doc));

    // Show BM25 on preprocessed text
    println!();
    println!("BM25 score after preprocessing:");
    for (a, b) in raw_pairs {
        let pa = pre.process(a);
        let pb = pre.process(b);
        let score = bm25::similarity(&pa, &pb);
        println!(
            "  {:.3}  |  \"{}\"  vs  \"{}\"",
            score,
            truncate(&pa, 35),
            truncate(&pb, 35)
        );
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}
