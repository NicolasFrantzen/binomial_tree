use binominal_tree_model::binomial_tree_model::{BinomialTreeModel, Expiry, Spot};
use binominal_tree_model::instruments::{AmericanOption, Option_, OptionType};

fn main() {
    let number_of_steps: usize = std::env::args().nth(1).expect("No number of steps given").parse::<usize>().unwrap();
    let binom_tree = BinomialTreeModel::new(Spot(100.0), number_of_steps, Expiry(0.5), 0.3, 0.05, 0.0);
    let val = binom_tree.eval(AmericanOption::new(OptionType::Call, 95.0, 0.5)).greeks();

    println!("Greeks: {:?}", val);
}