use HullBinominalModel::binomial_tree_model::{BinomialTreeModel, Expiry, Spot};
use HullBinominalModel::instruments::{AmericanOption, Option_, OptionType};

fn main() {
    let number_of_steps: usize = std::env::args().nth(1).expect("No number of steps given").parse::<usize>().unwrap();
    let binom_tree = BinomialTreeModel::new(Spot(100.0), number_of_steps as usize, Expiry(0.5), 0.3, 0.05, 0.0);
    let val = binom_tree.value(AmericanOption::new(OptionType::Call, 95.0, 0.5));

    println!("Greeks: {:?}", val);
}