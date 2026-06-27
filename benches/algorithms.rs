//! Criterion micro-benchmarks for the SIMI hot paths.
//!
//! Run with: `cargo bench`

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;

fn bench_levenshtein(c: &mut Criterion) {
    let mut g = c.benchmark_group("levenshtein");

    g.bench_with_input(
        BenchmarkId::new("similar", "kitten/sitting"),
        &("kitten", "sitting"),
        |b, (a, bb)| b.iter(|| simi::algo::levenshtein::similarity(black_box(a), black_box(bb))),
    );

    g.bench_with_input(
        BenchmarkId::new("identical", "short"),
        &("hello", "hello"),
        |b, (a, bb)| b.iter(|| simi::algo::levenshtein::similarity(black_box(a), black_box(bb))),
    );

    g.finish();
}

fn bench_jaro_winkler(c: &mut Criterion) {
    let mut g = c.benchmark_group("jaro_winkler");

    g.bench_with_input(
        BenchmarkId::new("names", "MARTHA/MARHTA"),
        &("MARTHA", "MARHTA"),
        |b, (a, bb)| b.iter(|| simi::algo::jaro_winkler::similarity(black_box(a), black_box(bb))),
    );

    g.finish();
}

fn bench_hamming(c: &mut Criterion) {
    let mut g = c.benchmark_group("hamming");

    g.bench_with_input(
        BenchmarkId::new("equal", "length"),
        &("karolin", "kathrin"),
        |b, (a, bb)| b.iter(|| simi::algo::hamming::similarity(black_box(a), black_box(bb))),
    );

    g.finish();
}

fn bench_jaccard(c: &mut Criterion) {
    let mut g = c.benchmark_group("jaccard");

    g.bench_with_input(
        BenchmarkId::new("bigram", "hello/world"),
        &("hello world", "hello there"),
        |b, (a, bb)| b.iter(|| simi::algo::jaccard::bigram_similarity(black_box(a), black_box(bb))),
    );

    g.finish();
}

fn bench_minhash(c: &mut Criterion) {
    let mut g = c.benchmark_group("minhash");

    let doc = "the quick brown fox jumps over the lazy dog near the river bank";
    g.bench_with_input(BenchmarkId::new("default", "128 hashes"), &doc, |b, d| {
        b.iter(|| {
            let sig = simi::algo::minhash::signature(black_box(d), 3, 128);
            black_box(sig)
        })
    });

    g.finish();
}

fn bench_simhash(c: &mut Criterion) {
    let mut g = c.benchmark_group("simhash");

    let doc = "the quick brown fox jumps over the lazy dog near the river bank";
    g.bench_with_input(BenchmarkId::new("default", "tetragrams"), &doc, |b, d| {
        b.iter(|| {
            let fp = simi::algo::simhash::fingerprint_default(black_box(d));
            black_box(fp)
        })
    });

    g.finish();
}

fn bench_bm25(c: &mut Criterion) {
    let mut g = c.benchmark_group("bm25");

    g.bench_with_input(
        BenchmarkId::new("similarity", "short docs"),
        &("the quick brown fox", "the quick blue fox"),
        |b, (a, bb)| b.iter(|| simi::algo::bm25::similarity(black_box(a), black_box(bb))),
    );

    g.finish();
}

fn bench_tfidf(c: &mut Criterion) {
    let mut g = c.benchmark_group("tfidf");

    g.bench_with_input(
        BenchmarkId::new("cosine", "short texts"),
        &("the quick brown fox", "the quick blue fox"),
        |b, (a, bb)| b.iter(|| simi::algo::tfidf::similarity(black_box(a), black_box(bb))),
    );

    g.finish();
}

fn bench_preprocess(c: &mut Criterion) {
    let mut g = c.benchmark_group("preprocess");

    let text = "  The   Quick Brown Fox   Jumps Over  the Lazy Dog!  ";
    g.bench_with_input(BenchmarkId::new("default", "clean"), &text, |b, t| {
        b.iter(|| simi::preprocess::clean(black_box(t)))
    });

    g.finish();
}

fn bench_batch(c: &mut Criterion) {
    use simi::batch::BatchComparator;
    use simi::router::Algo;

    let mut g = c.benchmark_group("batch");

    let size = 100;
    let a: Vec<String> = (0..size).map(|i| format!("string {}", i)).collect();
    let b: Vec<String> = (0..size).map(|i| format!("string {}", i + 1)).collect();

    let comparator = BatchComparator::new(Algo::Levenshtein);

    g.bench_function("compare_pairs_100", |b| {
        b.iter(|| {
            comparator
                .compare_pairs(black_box(&a), black_box(&b))
                .unwrap()
        })
    });

    g.finish();
}

criterion_group!(
    benches,
    bench_levenshtein,
    bench_jaro_winkler,
    bench_hamming,
    bench_jaccard,
    bench_minhash,
    bench_simhash,
    bench_bm25,
    bench_tfidf,
    bench_preprocess,
    bench_batch,
);
criterion_main!(benches);
