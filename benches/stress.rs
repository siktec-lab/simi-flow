// Stress-test benchmark for SIMI batch processing.
// Run with: cargo bench --bench stress

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;

use simi::batch::BatchComparator;
use simi::router::Algo;

fn gen_strings(n: usize, len: usize, seed: u64) -> Vec<String> {
    let mut state = seed;
    (0..n)
        .map(|_| {
            let mut s = String::with_capacity(len);
            for _ in 0..len {
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                let c = (state % 26) as u8 + b'a';
                s.push(c as char);
            }
            s
        })
        .collect()
}

fn bench_stress_pairs(c: &mut Criterion) {
    let mut g = c.benchmark_group("stress_pairs");
    let sizes = [10, 100, 1000, 10_000];

    for size in sizes {
        let a = gen_strings(size, 20, 42);
        let b = gen_strings(size, 20, 137);
        let cmp = BatchComparator::new(Algo::Levenshtein);

        g.bench_with_input(
            BenchmarkId::new("levenshtein", format!("{size} pairs")),
            &size,
            |bench, _| {
                let aa = &a;
                let bb = &b;
                bench.iter(|| {
                    let results = cmp.compare_pairs(black_box(aa), black_box(bb)).unwrap();
                    black_box(results)
                })
            },
        );
    }

    g.finish();
}

fn bench_stress_matrix(c: &mut Criterion) {
    let mut g = c.benchmark_group("stress_matrix");
    let sizes = [10, 50, 200];

    for size in sizes {
        let a = gen_strings(size, 15, 77);
        let b = gen_strings(size, 15, 199);
        let cmp = BatchComparator::new(Algo::JaroWinkler);

        g.bench_with_input(
            BenchmarkId::new("jaro_winkler", format!("{size}x{size}")),
            &size,
            |bench, _| {
                let aa = &a;
                let bb = &b;
                bench.iter(|| {
                    let results = cmp.compare_matrix(black_box(aa), black_box(bb)).unwrap();
                    black_box(results)
                })
            },
        );
    }

    g.finish();
}

fn bench_stress_one_to_many(c: &mut Criterion) {
    let mut g = c.benchmark_group("stress_one_to_many");
    let sizes = [100, 1000, 10_000];

    for size in sizes {
        let ref_str = "the quick brown fox jumps over the lazy dog".to_string();
        let candidates = gen_strings(size, 40, 313);
        let cmp = BatchComparator::new(Algo::Bm25);

        g.bench_with_input(
            BenchmarkId::new("bm25", format!("{size} candidates")),
            &size,
            |bench, _| {
                let r = &ref_str;
                let cand = &candidates;
                bench.iter(|| {
                    let results = cmp
                        .compare_one_to_many(black_box(r), black_box(cand))
                        .unwrap();
                    black_box(results)
                })
            },
        );
    }

    g.finish();
}

fn bench_stress_long_docs(c: &mut Criterion) {
    let mut g = c.benchmark_group("stress_long_docs");

    let doc_lens = [50, 500, 5000];
    for len in doc_lens {
        let a = gen_strings(1, len, 11)[0].clone();
        let b = gen_strings(1, len, 29)[0].clone();

        g.bench_with_input(
            BenchmarkId::new("levenshtein", format!("{} chars", len)),
            &len,
            |bench, _| {
                let aa = &a;
                let bb = &b;
                bench.iter(|| {
                    simi::algo::levenshtein::similarity(black_box(aa), black_box(bb))
                })
            },
        );

        g.bench_with_input(
            BenchmarkId::new("bm25", format!("{} chars", len)),
            &len,
            |bench, _| {
                let aa = &a;
                let bb = &b;
                bench.iter(|| simi::algo::bm25::similarity(black_box(aa), black_box(bb)))
            },
        );

        g.bench_with_input(
            BenchmarkId::new("tfidf", format!("{} chars", len)),
            &len,
            |bench, _| {
                let aa = &a;
                let bb = &b;
                bench.iter(|| simi::algo::tfidf::similarity(black_box(aa), black_box(bb)))
            },
        );

        g.bench_with_input(
            BenchmarkId::new("minhash", format!("{} chars", len)),
            &len,
            |bench, _| {
                let aa = &a;
                bench.iter(|| {
                    let sig = simi::algo::minhash::signature(black_box(aa), 3, 128);
                    black_box(sig)
                })
            },
        );

        g.bench_with_input(
            BenchmarkId::new("simhash", format!("{} chars", len)),
            &len,
            |bench, _| {
                let aa = &a;
                bench.iter(|| {
                    let fp = simi::algo::simhash::fingerprint_default(black_box(aa));
                    black_box(fp)
                })
            },
        );
    }

    g.finish();
}

criterion_group!(
    benches,
    bench_stress_pairs,
    bench_stress_matrix,
    bench_stress_one_to_many,
    bench_stress_long_docs,
);
criterion_main!(benches);
