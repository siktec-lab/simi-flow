//! Example: the SimiFlow pipeline with LLM fallback.
//!
//! Run with: `cargo run --example flow`

use simi::router::{Algo, SimiFlow, Strategy, Threshold};

fn main() {
    let pairs = [
        ("hello world", "hello world"),
        ("hello world", "hello there"),
        ("the quick brown fox", "the quick brown fox"),
        ("the quick brown fox", "the quick lazy dog"),
        ("MARTHA", "MARHTA"),
        ("kitten", "sitting"),
        ("abc", "xyz"),
    ];

    let flow = SimiFlow::new()
        .preprocess(true)
        .strategy(Strategy::Cascade)
        .tier_1(
            Algo::JaroWinkler,
            Threshold::GreaterThan(0.95),
            Threshold::LessThan(0.10),
        )
        .tier_2(Algo::Bm25, Threshold::Between(0.30, 0.94))
        .fallback(|a, b| {
            let score = if a.len() == b.len()
                && a.chars().zip(b.chars()).filter(|(c, d)| c == d).count() as f64 / a.len() as f64
                    > 0.5
            {
                0.75
            } else {
                0.25
            };
            (score, Some("simulated_llm".into()))
        });

    println!(
        "{:<32} {:<32} {:>8} {:>6} {:>20}",
        "String A", "String B", "Score", "Tier", "Algorithm"
    );
    println!("{}", "-".repeat(102));

    for (a, b) in pairs {
        let result = flow.compare(a, b).unwrap();
        println!(
            "{:<32} {:<32} {:>8.3} {:>6} {:>20}",
            a, b, result.score, result.tier, result.algorithm
        );
    }
}
