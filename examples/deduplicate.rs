//! Example: batch deduplication — find near-duplicate names in a large list.
//!
//! Run with: `cargo run --example deduplicate --release`

use simi::batch::BatchComparator;
use simi::router::Algo;

fn main() {
    let names = vec![
        "John Smith",
        "Jon Smith",
        "John Smyth",
        "Jane Doe",
        "Jane Eyre",
        "MARTHA",
        "MARHTA",
        "Albert Einstein",
        "Alfred Einstein",
        "Nikola Tesla",
        "Nicola Tesla",
        "Marie Curie",
        "Maria Curie",
    ];

    let strings: Vec<String> = names.iter().map(|s| s.to_string()).collect();

    let comparator = BatchComparator::new(Algo::JaroWinkler);
    let results = comparator
        .compare_matrix(&strings, &strings)
        .expect("batch compare_matrix failed");

    println!("Near-duplicates (Jaro-Winkler > 0.90):");
    println!("{:<20} {:<20} {:>8}", "Name A", "Name B", "Score");
    println!("{}", "-".repeat(52));

    for r in &results {
        if r.score > 0.90 && r.index_a < r.index_b {
            println!(
                "{:<20} {:<20} {:>8.3}",
                names[r.index_a], names[r.index_b], r.score
            );
        }
    }

    println!();
    println!("Total pairs evaluated: {}", results.len());
    println!("Duplicates found: {}",
        results.iter().filter(|r| r.score > 0.90 && r.index_a < r.index_b).count()
    );
}
