use crate::binomial_tree::{Expiry, Spot, VolatilityParameters};
use crate::binomial_tree_map::BinomialTreeMap;
use crate::instruments::Option_;
use crate::tree::{NodeName, UpDown};

use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

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

    pub fn value<T: Option_ + Sync>(&self, option: T) -> Greeks {
        let mut stack_iter = self.tree_map.iter();

        for node in stack_iter.next().expect("The tree must have length at least 1") {
            let price = node.value(self.spot.0, self.params.u, self.params.d);

            // TODO: Handle some unwraps here
            self.tree_map.map.get(node).unwrap().set(option.payoff(price)).unwrap();
        }

        let p = self.params.p();

        for node_level in stack_iter {
            node_level.par_iter().rev().for_each(|node| {
                let up_value = self.tree_map.map[&node.up2()].get().expect("Previous level was not evaluated");
                let down_value = self.tree_map.map[&node.down()].get().expect("Previous level was not evaluated");

                let value = (up_value * p + down_value * (1.0 - p)) * self.discount_factor;
                let price = node.value(self.spot.0, self.params.u, self.params.d);
                let option_value = option.value(value, price);

                self.tree_map.map[node].set(option_value).unwrap();
            });
        }

        let value = self.tree_map.map[&NodeName{ name: vec![] }].get().unwrap();


        Greeks {
            value: Value(*value),
            delta: self.delta(),
            gamma: Gamma(0.0), // TODO
        }
    }

    // The tree should already be valued before calling this
    fn delta(&self) -> Delta {
        let last_up = NodeName{ name: vec![UpDown::Up] };
        let last_up_value = self.tree_map.map[&last_up].get().unwrap();
        let last_down = NodeName{ name: vec![UpDown::Down] };
        let last_down_value = self.tree_map.map[&last_down].get().unwrap();
        let delta = (last_up_value - last_down_value) /
            (last_up.value(self.spot.0, self.params.u, self.params.d) - last_down.value(self.spot.0, self.params.u, self.params.d));

        Delta(delta)
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
    use crate::instruments::{AmericanOption, EuropeanOption, OptionType};
    use super::*;


    #[test]
    fn test_binomial_tree_european_call() {
        let model = BinomialTreeModel::new(Spot(100.0), 2, Expiry(0.5), 0.3, 0.05, 0.0);
        let option = EuropeanOption::new(OptionType::Call, 95.0, 0.5);
        let greeks = model.value(option);
        assert_eq!(greeks.value, Value(12.357803));
        assert_eq!(greeks.delta, Delta(0.6599609));
    }

    #[test]
    fn test_binomial_tree_european_call2() {
        let model = BinomialTreeModel::new(Spot(810.0), 2, Expiry(0.5), 0.2, 0.05, 0.02);
        let option = EuropeanOption::new(OptionType::Call, 800.0, 0.5);
        let greeks = model.value(option);
        assert_eq!(greeks.value, Value(53.39472));
        assert_eq!(greeks.delta, Delta(0.58913547));
    }

    #[test]
    fn test_binomial_tree_european_call3() {
        let model = BinomialTreeModel::new(Spot(0.61), 3, Expiry(0.25), 0.12, 0.05, 0.07);
        let option = EuropeanOption::new(OptionType::Call, 0.6, 0.25);
        let greeks = model.value(option);
        assert_eq!(greeks.value, Value(0.018597351));
        assert_eq!(greeks.delta, Delta(0.6000445));
    }

    #[test]
    fn test_binomial_tree_european_put1() {
        let model = BinomialTreeModel::new(Spot(50.0), 2, Expiry(2.0), 0.3, 0.05, 0.0);
        let option = EuropeanOption::new(OptionType::Put, 52.0, 2.0);
        let greeks = model.value(option);
        assert_eq!(greeks.value, Value(6.2457113));
        assert_eq!(greeks.delta, Delta(-0.37732533));
    }

    #[test]
    fn test_binomial_tree_american_put1() {
        let model = BinomialTreeModel::new(Spot(50.0), 2, Expiry(2.0), 0.3, 0.05, 0.0);
        let option = AmericanOption::new(OptionType::Put, 52.0, 2.0);
        let greeks = model.value(option);
        assert_eq!(greeks.value, Value(7.428405));
        assert_eq!(greeks.delta, Delta(-0.4606061));
    }

    #[test]
    fn test_binomial_tree_american_put2() {
        let model = BinomialTreeModel::new(Spot(31.0), 3, Expiry(0.75), 0.3, 0.05, 0.05);
        let option = AmericanOption::new(OptionType::Put, 30.0, 0.75);
        let greeks = model.value(option);
        assert_eq!(greeks.value, Value(2.8356347));
        assert_eq!(greeks.delta, Delta(-0.38601997));
        //assert_eq!(val.risk_free_probability, 0.4626);
    }

    #[test]
    fn test_binomial_tree_american_put2_100steps() {
        let model = BinomialTreeModel::new(Spot(31.0), 100, Expiry(0.75), 0.3, 0.05, 0.05);
        let option = AmericanOption::new(OptionType::Put, 30.0, 0.75);
        let greeks = model.value(option);
        assert_eq!(greeks.value, Value(2.604315));
        assert_eq!(greeks.delta, Delta(-0.38875514));
    }
}