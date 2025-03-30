use HullBinominalModel::binomial_tree_model::{BinomialTreeModel, Expiry, Spot};
use HullBinominalModel::instruments::{EuropeanOption, OptionType, Option_, AmericanOption};

use criterion::{criterion_group, criterion_main, Criterion};
//use std::hint::black_box;
use std::time::Duration;


fn european_call(n: u32) {
    let binom_tree = BinomialTreeModel::new(Spot(100.0), n as usize, Expiry(0.5), 0.3, 0.05, 0.0);
    let _val = binom_tree.value(EuropeanOption::new(OptionType::Call, 95.0, 0.5));
}

fn american_call(n: u32) {
    let binom_tree = BinomialTreeModel::new(Spot(100.0), n as usize, Expiry(0.5), 0.3, 0.05, 0.0);
    let _val = binom_tree.value(AmericanOption::new(OptionType::Call, 95.0, 0.5));
}

fn criterion_benchmark_method(c: &mut Criterion) {
    let mut group = c.benchmark_group("Option benches");

    group.bench_function("european call threaded", |b| b.iter(|| european_call(100)));
    group.bench_function("american call threaded", |b| b.iter(|| american_call(100)));
    group.finish();
}

fn alternate_measurement() -> Criterion<> {
    Criterion::default().measurement_time(Duration::from_millis(200)).sample_size(40)
}

criterion_group!(
    name = method2;
    config = alternate_measurement();
    targets = criterion_benchmark_method
);
criterion_main!(method2);