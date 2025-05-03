use binominal_tree_model::binomial_tree_model::{BinomialTreeModel, Expiry, Spot};
use binominal_tree_model::instruments::{AmericanOption, Option_, OptionType};
use binominal_tree_model::binomial_tree;

fn main() {
    let number_of_steps: usize = std::env::args().nth(1).expect("No number of steps given").parse::<usize>().unwrap();

    for _ in 1..1000 {
        let tree_map = binomial_tree!(100);
        let binom_tree = BinomialTreeModel::new(tree_map, Spot(100.0), number_of_steps, Expiry(0.5), 0.3, 0.05, 0.0);
        let _val = binom_tree.eval(AmericanOption::new(OptionType::Call, 95.0, 0.5)).greeks();
    }
}