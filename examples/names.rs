//! Example: comparing names with Jaro-Winkler and Levenshtein.
//!
//! Run with: `cargo run --example names`

use simi::algo::{jaro_winkler, levenshtein};

fn main() {
    let pairs = [
        ("MARTHA", "MARHTA"),
        ("DWAYNE", "DUANE"),
        ("DIXON", "DICKSONX"),
        ("MICHELLE", "MICHAEL"),
        ("JULIES", "JULIUS"),
        ("TANYA", "TONYA"),
        ("SHACKLEFORD", "SHACKELFORD"),
        ("CATHERINE", "KATHERINE"),
        ("JOHN", "JON"),
        ("BRITNEY", "BRITTANY"),
    ];

    println!(
        "{:<20} {:<20} {:>12} {:>12}",
        "Name A", "Name B", "Jaro-Winkler", "Levenshtein"
    );
    println!("{}", "-".repeat(68));

    for (a, b) in pairs {
        let jw = jaro_winkler::similarity(a, b);
        let lv = levenshtein::similarity(a, b);
        println!("{:<20} {:<20} {:>12.3} {:>12.3}", a, b, jw, lv);
    }
}
