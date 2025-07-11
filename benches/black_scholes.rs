use std::hint::black_box;
use binominal_tree_model::black_scholes::black_value;
use binominal_tree_model::instruments::OptionType;

use rayon::iter::ParallelIterator;
use criterion::{criterion_group, criterion_main, Criterion};
use rayon::iter::IntoParallelIterator;

fn bs_american_call_value_100000_par() {
    (1..100000).into_par_iter().for_each(|_| {
        let _ = black_value(OptionType::Call, 100.0, 95.0, 0.3, 0.05, 0.0, 0.5);
    });
}

fn bs_american_call_value_100000() {
    (1..100000).into_par_iter().for_each(|_| {
        let _ = black_value(OptionType::Call, 100.0, 95.0, 0.3, 0.05, 0.0, 0.5);
    });
}

fn criterion_benchmark_bs(c: &mut Criterion) {
    let mut group = c.benchmark_group("Black Scholes Option benches");

    group.bench_function("american call value 100000 parallel", |b| b.iter(|| black_box(bs_american_call_value_100000_par())));
    group.bench_function("american call value 100000 non-parallel", |b| b.iter(|| black_box(bs_american_call_value_100000())));
    group.finish();
}

criterion_group!(
    name = black_scholes;
    config = Criterion::default();
    targets = criterion_benchmark_bs
);

criterion_main!(black_scholes);

