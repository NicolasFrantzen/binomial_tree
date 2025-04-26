use binominal_tree_model::binomial_tree_model::{BinomialTreeModel, Expiry, Spot};
use binominal_tree_model::instruments::{EuropeanOption, OptionType, Option_, AmericanOption};
use binominal_tree_model::macros::static_binomial_tree;

use criterion::{criterion_group, criterion_main, Criterion};
//use std::hint::black_box;
use std::time::Duration;

fn european_call_value_100() {
    let tree_map = static_binomial_tree!(100);
    let binom_tree = BinomialTreeModel::new(tree_map, Spot(100.0), 0usize /* dummy */, Expiry(0.5), 0.3, 0.05, 0.0);
    let _val = binom_tree.eval(EuropeanOption::new(OptionType::Call, 95.0, 0.5));
}

fn american_call_value_100() {
    let tree_map = static_binomial_tree!(100);
    let binom_tree = BinomialTreeModel::new(tree_map, Spot(100.0), 0usize /* dummy */, Expiry(0.5), 0.3, 0.05, 0.0);
    let _val = binom_tree.eval(AmericanOption::new(OptionType::Call, 95.0, 0.5));
}

fn european_call_greeks_100() {
    let tree_map = static_binomial_tree!(100);
    let binom_tree = BinomialTreeModel::new(tree_map, Spot(100.0), 0usize /* dummy */, Expiry(0.5), 0.3, 0.05, 0.0);
    let _val = binom_tree.eval(EuropeanOption::new(OptionType::Call, 95.0, 0.5));
}

fn american_call_greeks_100() {
    let tree_map = static_binomial_tree!(100);
    let binom_tree = BinomialTreeModel::new(tree_map, Spot(100.0), 0usize /* dummy */, Expiry(0.5), 0.3, 0.05, 0.0);
    let _val = binom_tree.eval(AmericanOption::new(OptionType::Call, 95.0, 0.5));
}

/*fn american_call_greeks_250() {
    let tree_map = static_binomial_tree!(250);
    let binom_tree = BinomialTreeModel::new(tree_map, Spot(100.0), 0usize /* dummy */, Expiry(0.5), 0.3, 0.05, 0.0);
    let _val = binom_tree.eval(AmericanOption::new(OptionType::Call, 95.0, 0.5));
}*/

fn criterion_benchmark_method(c: &mut Criterion) {
    let mut group = c.benchmark_group("Option benches");

    group.bench_function("european call value 100 steps", |b| b.iter(|| european_call_value_100()));
    group.bench_function("american call value 100 steps", |b| b.iter(|| american_call_value_100()));
    group.bench_function("european call greeks 100 steps", |b| b.iter(|| european_call_greeks_100()));
    group.bench_function("american call greeks 100 steps", |b| b.iter(|| american_call_greeks_100()));
    //group.bench_function("american call greeks 250 steps", |b| b.iter(|| american_call_greeks_250()));
    //group.bench_function("european call greeks 10 steps", |b| b.iter(|| european_call_greeks(10)));
    //group.bench_function("american call greeks 10 steps", |b| b.iter(|| american_call_greeks(10)));
    group.finish();
}

fn alternate_measurement() -> Criterion<> {
    Criterion::default().measurement_time(Duration::from_millis(300)).sample_size(40)
}

criterion_group!(
    name = method2;
    config = alternate_measurement();
    targets = criterion_benchmark_method
);
criterion_main!(method2);