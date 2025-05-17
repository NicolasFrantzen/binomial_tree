use binominal_tree_model::binomial_tree_model::{BinomialTreeModel, Expiry, Spot};
use binominal_tree_model::instruments::{EuropeanOption, OptionType, Option_, AmericanOption};
use binominal_tree_model::binomial_tree_map;
use binominal_tree_model::static_binomial_tree_map::StaticBinomialTreeMap;

use criterion::{criterion_group, criterion_main, Criterion};
//use std::hint::black_box;
use std::time::Duration;


fn european_call_value_100() {
    let tree_map = binomial_tree_map!(100);
    let binom_tree: BinomialTreeModel<StaticBinomialTreeMap> = BinomialTreeModel::new(tree_map, Spot(100.0), 100usize, Expiry(0.5), 0.3, 0.05, 0.0);
    let _val = binom_tree.eval(EuropeanOption::new(OptionType::Call, 95.0, 0.5)).value();
}

fn american_call_value_100() {
    let tree_map = binomial_tree_map!(100);
    let binom_tree: BinomialTreeModel<StaticBinomialTreeMap> = BinomialTreeModel::new(tree_map, Spot(100.0), 100usize, Expiry(0.5), 0.3, 0.05, 0.0);
    let _val = binom_tree.eval(AmericanOption::new(OptionType::Call, 95.0, 0.5)).value();
}

fn european_call_greeks_100() {
    let tree_map = binomial_tree_map!(100);
    let binom_tree: BinomialTreeModel<StaticBinomialTreeMap> = BinomialTreeModel::new(tree_map, Spot(100.0), 100usize, Expiry(0.5), 0.3, 0.05, 0.0);
    let _val = binom_tree.eval(EuropeanOption::new(OptionType::Call, 95.0, 0.5)).greeks();
}

fn american_call_greeks_100() {
    let tree_map = binomial_tree_map!(100);
    let binom_tree: BinomialTreeModel<StaticBinomialTreeMap> = BinomialTreeModel::new(tree_map, Spot(100.0), 1000usize, Expiry(0.5), 0.3, 0.05, 0.0);
    let _val = binom_tree.eval(AmericanOption::new(OptionType::Call, 95.0, 0.5)).greeks();
}

fn american_call_value_50() {
    let tree_map = binomial_tree_map!(50);
    let binom_tree: BinomialTreeModel<StaticBinomialTreeMap> = BinomialTreeModel::new(tree_map, Spot(100.0), 100usize, Expiry(0.5), 0.3, 0.05, 0.0);
    let _val = binom_tree.eval(AmericanOption::new(OptionType::Call, 95.0, 0.5)).value();
}

fn american_call_greeks_50() {
    let tree_map = binomial_tree_map!(50);
    let binom_tree: BinomialTreeModel<StaticBinomialTreeMap> = BinomialTreeModel::new(tree_map, Spot(100.0), 50usize, Expiry(0.5), 0.3, 0.05, 0.0);
    let _val = binom_tree.eval(AmericanOption::new(OptionType::Call, 95.0, 0.5)).greeks();
}

fn american_call_greeks_30_1000() {
    for _ in 0..1000 {
        let tree_map = binomial_tree_map!(30);
        let binom_tree: BinomialTreeModel<StaticBinomialTreeMap> = BinomialTreeModel::new(tree_map, Spot(100.0), 30usize, Expiry(0.5), 0.3, 0.05, 0.0);
        let _val = binom_tree.eval(AmericanOption::new(OptionType::Call, 95.0, 0.5)).greeks();
    }
}

fn criterion_benchmark_method(c: &mut Criterion) {
    let mut group = c.benchmark_group("Option benches");

    group.bench_function("european call value 100 steps", |b| b.iter(|| european_call_value_100()));
    group.bench_function("american call value 100 steps", |b| b.iter(|| american_call_value_100()));
    group.bench_function("european call greeks 100 steps", |b| b.iter(|| european_call_greeks_100()));
    group.bench_function("american call greeks 100 steps", |b| b.iter(|| american_call_greeks_100()));
    group.bench_function("american call greeks 100 times 30 steps", |b| b.iter(|| american_call_greeks_30_1000()));

    // Hull states that 30 gives reasonable results, 50 seems to be almost convergence
    group.bench_function("american call value 50 steps", |b| b.iter(|| american_call_value_50()));
    group.bench_function("american call greeks 50 steps", |b| b.iter(|| american_call_greeks_50()));
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