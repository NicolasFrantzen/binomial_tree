use crate::binomial_tree::{Expiry, Spot, VolatilityParameters};
use crate::binomial_tree_map::BinomialTreeMap;
use crate::instruments::Option_;
use crate::tree::NodeName;

use hashbrown::hash_map::rayon::*;
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator};
use rayon::iter::*;
use rayon::prelude::*;

pub struct BinomialTreeModel {
    tree_map: BinomialTreeMap,
    params: VolatilityParameters,
    spot: Spot,
    discount_factor: f32,
}

impl BinomialTreeModel {
    pub fn new(initial_price: Spot, number_of_steps: usize, expiry: Expiry, volatility: f32, interest_rate: f32, dividends: f32) -> Self {
        let timestep = expiry.0 / number_of_steps as f32;
        let vol_params = VolatilityParameters::new(volatility, interest_rate, dividends, timestep);

        Self {
            tree_map: BinomialTreeMap::new(number_of_steps),
            params: vol_params,
            spot: initial_price,
            discount_factor: (-interest_rate * timestep).exp(),
        }
    }

    pub fn value<T: Option_>(&self, option: T) -> Greeks {
        let mut iter = self.tree_map.stack.iter();

        let first_node_level = &iter.last();

        // TODO: Here we have assumed that the length is bigger than 1

        for node in first_node_level.unwrap() {
            let price = node.value(self.spot.0, self.params.u, self.params.d);

            // TODO: Handle some unwraps here
            self.tree_map.map.get(node).unwrap().set(option.payoff(price)).unwrap();
        }

        let p = self.params.p();

        self.tree_map.stack.par_iter().for_each(|_| () );

        //for node_level in self.tree_map.stack.iter().rev() {
        self.tree_map.stack.par_iter().for_each(|node_level|
            // let value = ((up.borrow().value.get() * p) + (down.borrow().value.get() * (1.0 - p))) * self.discount_factor;
            //                     node.borrow().value.set(option.value(value, branch_price));
            for node in node_level {
                let mut up_value: f32 = 0.0; // TODO: Put in function
                if let Some(up) = self.tree_map.map.get(&node.up2()) {
                    up_value = *up.get().unwrap();
                }
                else { panic!("Incomplete tree. Missing: {:?} ", node.up2()); }

                //let up_value = self.tree_map.map[&node.up()].get().expect("Previous level was evaluated"); // TODO: Fix unwrap
                let down_value = self.tree_map.map[&node.down()].get().expect("Previous level was evaluated"); // TODO: Fix unwrap

                let value = (up_value * p + down_value * (1.0 - p)) * self.discount_factor;
                let price = node.value(self.spot.0, self.params.u, self.params.d);
                let option_value = option.value(value, price);

                self.tree_map.map[node].set(value).unwrap();
            }
        );

        let value = self.tree_map.map[&NodeName{ name: vec![] }].get().unwrap();

        Greeks {
            value: Value(*value),
            delta: Delta(0.0),
            gamma: Gamma(0.0),
        }
    }

}

#[derive(Debug, PartialEq)]
pub struct Greeks {
    value: Value,
    delta: Delta,
    gamma: Gamma,
}

#[derive(Debug, PartialEq)]
pub struct Value(pub f32);

#[derive(Debug, PartialEq)]
pub struct Delta(pub f32);

#[derive(Debug, PartialEq)]
pub struct Gamma(pub f32);

#[cfg(test)]
mod tests {
    use crate::instruments::{EuropeanOption, OptionType};
    use super::*;

    #[test]
    fn test_stack_map() {

    }

    #[test]
    fn test_binomial_tree_european_call2() {
        let model = BinomialTreeModel::new(Spot(810.0), 2, Expiry(0.5), 0.2, 0.05, 0.02);
        let option = EuropeanOption::new(OptionType::Call, 800.0, 0.5);
        assert_eq!(model.value(option).value, Value(53.39472));
    }
}