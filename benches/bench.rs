use HullBinominalModel::binomial_tree::{BinomialTree, Expiry, Spot};
use HullBinominalModel::binomial_tree_model::BinomialTreeModel;
use HullBinominalModel::instruments::{EuropeanOption, OptionType, Option_, AmericanOption};

use criterion::{criterion_group, criterion_main, Criterion};
//use std::hint::black_box;
use std::time::Duration;

fn european_call(n: u32) {
    let binom_tree = BinomialTree::new(Spot(100.0), n, Expiry(0.5), 0.3, 0.05, 0.0);
    let _val = binom_tree.value(EuropeanOption::new(OptionType::Call, 95.0, 0.5));
}

fn european_call2(n: u32) {
    let binom_tree = BinomialTreeModel::new(Spot(100.0), n as usize, Expiry(0.5), 0.3, 0.05, 0.0);
    let _val = binom_tree.value(EuropeanOption::new(OptionType::Call, 95.0, 0.5));
}

fn american_call2(n: u32) {
    let binom_tree = BinomialTreeModel::new(Spot(100.0), n as usize, Expiry(0.5), 0.3, 0.05, 0.0);
    let _val = binom_tree.value(AmericanOption::new(OptionType::Call, 95.0, 0.5));
}

#[allow(dead_code)]
fn american_call(n: u32) {
    let binom_tree = BinomialTree::new(Spot(100.0), n, Expiry(0.5), 0.3, 0.05, 0.0);
    let _val = binom_tree.value(AmericanOption::new(OptionType::Call, 95.0, 0.5));
}

fn criterion_benchmark_method1(c: &mut Criterion) {
    let mut group = c.benchmark_group("Method 1");

    group.bench_function("european call method 1", |b| b.iter(|| european_call(15)));
    //group.bench_function("american call method 1", |b| b.iter(|| american_call(10)));
    group.finish();
}

fn criterion_benchmark_method2(c: &mut Criterion) {
    let mut group = c.benchmark_group("Method 2");

    group.bench_function("european call method 2 single threaded", |b| b.iter(|| european_call2(100)));
    group.bench_function("american call method 2 single threaded", |b| b.iter(|| american_call2(100)));
    group.finish();
}

fn alternate_measurement() -> Criterion<> {
    Criterion::default().measurement_time(Duration::from_millis(200)).sample_size(40)
}

criterion_group!(
    name = method1;
    config = alternate_measurement();
    targets = criterion_benchmark_method1
);

criterion_group!(
    name = method2;
    config = alternate_measurement();
    targets = criterion_benchmark_method2
);
criterion_main!(/*method1,*/ method2);