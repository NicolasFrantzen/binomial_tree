use HullBinominalModel::binomial_tree::{BinomialTree, Expiry, Spot};
use HullBinominalModel::instruments::{EuropeanOption, OptionType, Option_, AmericanOption};

use criterion::{criterion_group, criterion_main, Criterion};
//use std::hint::black_box;
use std::time::Duration;

fn european_call(n: u32) {
    let binom_tree = BinomialTree::new(Spot(100.0), n, Expiry(0.5), 0.3, 0.05, 0.0);
    let _val = binom_tree.value(EuropeanOption::new(OptionType::Call, 95.0, 0.5));
}

#[allow(dead_code)]
fn american_call(n: u32) {
    let binom_tree = BinomialTree::new(Spot(100.0), n, Expiry(0.5), 0.3, 0.05, 0.0);
    let _val = binom_tree.value(AmericanOption::new(OptionType::Call, 95.0, 0.5));
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Options");

    group.bench_function("european call", |b| b.iter(|| european_call(15)));
    //group.bench_function("american call", |b| b.iter(|| american_call(black_box(2))));
    group.finish();
}

fn alternate_measurement() -> Criterion<> {
    Criterion::default().measurement_time(Duration::from_millis(200)).sample_size(50)
}

criterion_group!(
    name = benches;
    config = alternate_measurement();
    targets = criterion_benchmark
);
criterion_main!(benches);