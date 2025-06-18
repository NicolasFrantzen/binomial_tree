use binominal_tree_model::eval_binomial_tree_with_steps;
use rayon::iter::ParallelIterator;

use criterion::{criterion_group, criterion_main, Criterion};
use rayon::iter::IntoParallelIterator;
//use std::hint::black_box;
use std::time::Duration;

fn european_call_value_100() {
    eval_binomial_tree_with_steps!(100, EuropeanOption, Call, 95.0, 100.0, 0.5, 0.3, 0.05, 0.0).value();
}

fn american_call_value_100() {
    eval_binomial_tree_with_steps!(100, AmericanOption, Call, 95.0, 100.0, 0.5, 0.3, 0.05, 0.0).value();
}

fn european_call_greeks_100() {
    eval_binomial_tree_with_steps!(100, EuropeanOption, Call, 95.0, 100.0, 0.5, 0.3, 0.05, 0.0).greeks();
}

fn american_call_greeks_100() {
    eval_binomial_tree_with_steps!(100, AmericanOption, Call, 95.0, 100.0, 0.5, 0.3, 0.05, 0.0).greeks();
}

fn american_call_greeks_30_1000() {
    (1..1000).into_par_iter().for_each(|_| {
        let _ = eval_binomial_tree_with_steps!(30, AmericanOption, Call, 95.0, 100.0, 0.5, 0.3, 0.05, 0.0).greeks();
    });
}

fn criterion_benchmark_method(c: &mut Criterion) {
    let mut group = c.benchmark_group("Option benches");

    group.bench_function("european call value 100 steps", |b| b.iter(|| european_call_value_100()));
    group.bench_function("american call value 100 steps", |b| b.iter(|| american_call_value_100()));
    group.bench_function("european call greeks 100 steps", |b| b.iter(|| european_call_greeks_100()));
    group.bench_function("american call greeks 100 steps", |b| b.iter(|| american_call_greeks_100()));
    group.bench_function("american call greeks 100 times 30 steps", |b| b.iter(|| american_call_greeks_30_1000()));
    group.finish();
}

fn alternate_measurement() -> Criterion<> {
    Criterion::default().measurement_time(Duration::from_millis(500)).sample_size(40)
}

criterion_group!(
    name = method2;
    config = alternate_measurement();
    targets = criterion_benchmark_method
);
criterion_main!(method2);