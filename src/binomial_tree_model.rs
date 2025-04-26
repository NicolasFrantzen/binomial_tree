use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

use crate::binomial_tree_map::{BinomialTree};
use crate::instruments::Option_;
use crate::macros::static_binomial_tree;
use crate::nodes::{NodeName, INITIAL_NODE};
use crate::static_binomial_tree_map::StaticBinomialTree;

pub struct BinomialTreeModel {
    //tree_map: BinomialTreeMap<N>,
    tree_map: StaticBinomialTree,
    params: VolatilityParameters,
    spot: Spot,
    discount_factor: f32,
    time_step: f32,
}

impl BinomialTreeModel {
    pub fn new(tree_map: StaticBinomialTree, initial_price: Spot, number_of_steps: usize, expiry: Expiry, volatility: f32, interest_rate: f32, dividends: f32) -> Self {
        let time_step = expiry.0 / number_of_steps as f32;
        let vol_params = VolatilityParameters::new(volatility, interest_rate, dividends, time_step);

        Self {
            tree_map,
            params: vol_params,
            spot: initial_price,
            discount_factor: (-interest_rate * time_step).exp(),
            time_step,
        }
    }
    /*pub fn new(initial_price: Spot, number_of_steps: usize, expiry: Expiry, volatility: f32, interest_rate: f32, dividends: f32) -> Self {
        let time_step = expiry.0 / number_of_steps as f32;
        let vol_params = VolatilityParameters::new(volatility, interest_rate, dividends, time_step);

        Self {
            tree_map: BinomialTreeMap::new(number_of_steps),
            params: vol_params,
            spot: initial_price,
            discount_factor: (-interest_rate * time_step).exp(),
            time_step,
        }
    }*/

    /*pub fn eval<T: Option_ + Sync>(self, option: T) -> EvaluatedBinomialTreeModel<N> {
        let p = self.params.p();

        for node_level in self.tree_map.iter() {
            node_level.par_iter().rev().for_each(|node| {
                let up_value = self.tree_map.get(&node.up());
                let down_value = self.tree_map.get(&node.down());

                let price = node.value(self.spot.0, self.params.u, self.params.d);

                if let (Some(up_value), Some(down_value)) = (up_value, down_value) {
                    let up_value = up_value.wait();
                    let down_value = down_value.wait();
                    let value = (up_value * p + down_value * (1.0 - p)) * self.discount_factor;

                    let option_value = option.value(value, price);

                    self.tree_map.set(&node, option_value);
                }
                else {
                    // TODO: Handle some unwraps here
                    self.tree_map.set(&node, option.payoff(price));
                }

            });
        }

        EvaluatedBinomialTreeModel{model: self}
    }*/

    pub fn eval<T: Option_ + Sync>(self, option: T) -> EvaluatedBinomialTreeModel {
        let p = self.params.p();

        for node_level in self.tree_map.stack.iter().rev() {
            node_level.par_iter().rev().for_each(|node| {
                let node = Into::<NodeName>::into(*node);
                let up_value = self.tree_map.map.get(&node.up());
                let down_value = self.tree_map.map.get(&node.down());

                let price = node.value(self.spot.0, self.params.u, self.params.d);

                if let (Some(up_value), Some(down_value)) = (up_value, down_value) {
                    let up_value = up_value.wait();
                    let down_value = down_value.wait();
                    let value = (up_value * p + down_value * (1.0 - p)) * self.discount_factor;

                    let option_value = option.value(value, price);

                    self.tree_map.set(&node, option_value);
                }
                else {
                    // TODO: Handle some unwraps here
                    self.tree_map.set(&node, option.payoff(price));
                }
            });
        }

        EvaluatedBinomialTreeModel{model: self}
    }


}

pub struct EvaluatedBinomialTreeModel {
    model: BinomialTreeModel,
}

impl EvaluatedBinomialTreeModel {
    pub fn value(&self) -> Value
    {
        let value = self.model.tree_map.get(&INITIAL_NODE).unwrap().get().unwrap();
        Value(*value)
    }

    pub fn delta(&self) -> Delta {
        self.delta_from(&INITIAL_NODE)
    }

    fn delta_from(&self, from_node: &NodeName) -> Delta {
        let last_up = from_node.up();
        let last_up_value =  self.model.tree_map.get(&last_up).unwrap().get().unwrap();
        let last_down = from_node.down();
        let last_down_value = self.model.tree_map.get(&last_down).unwrap().get().unwrap();
        let delta = (last_up_value - last_down_value) /
            (last_up.value(self.model.spot.0, self.model.params.u, self.model.params.d) - last_down.value(
                self.model.spot.0,
                self.model.params.u,
                self.model.params.d
            ));

        Delta(delta)
    }

    pub fn gamma(&self) -> Gamma {
        let node_u = INITIAL_NODE.up();
        let node_d = INITIAL_NODE.down();
        let delta_u = self.delta_from(&node_u);
        let delta_d = self.delta_from(&node_d);
        let spot_u = self.model.tree_map.get(&node_u).unwrap().get().unwrap();
        let spot_d = self.model.tree_map.get(&node_d).unwrap().get().unwrap();

        if spot_u == spot_d {
            Gamma(0.0)
        }
        else {
            Gamma((delta_u.0 - delta_d.0) / (spot_u - spot_d))
        }
    }

    pub fn theta(&self) -> Theta {
        let val_0 = self.model.tree_map.get(&INITIAL_NODE).unwrap().get().unwrap();
        let val_2 = self.model.tree_map.get(&INITIAL_NODE.up().down()).unwrap().get().unwrap();

        Theta((val_2 - val_0) / (2.0 * self.model.time_step))
    }

    pub fn greeks(&self) -> Greeks {
        Greeks {
            value: self.value(),
            delta: self.delta(),
            gamma: self.gamma(),
            theta: self.theta(),
        }
    }
}


pub struct Spot(pub f32);
pub struct Expiry(pub f32);

#[derive(Copy, Clone)]
pub struct VolatilityParameters {
    a: f32,
    pub(crate) u: f32,
    pub(crate) d: f32,
}

impl VolatilityParameters {
    pub fn new(volatility: f32, interest_rate: f32, dividends: f32, timestep: f32) -> VolatilityParameters {
        let u= (volatility*timestep.sqrt()).exp();
        VolatilityParameters {
            a: ((interest_rate - dividends) * timestep).exp(),
            u,
            d: 1.0 / u,
        }
    }

    pub(crate) fn p(&self) -> f32 {
        (self.a - self.d)/(self.u - self.d)
    }
}

#[derive(Debug, PartialEq)]
pub struct Greeks {
    value: Value,
    delta: Delta,
    gamma: Gamma,
    theta: Theta,
}

#[derive(Debug, PartialEq)]
pub struct Value(pub f32);

#[derive(Debug, PartialEq)]
pub struct Delta(pub f32);

#[derive(Debug, PartialEq)]
pub struct Gamma(pub f32);

#[derive(Debug, PartialEq)]
pub struct Theta(pub f32);

#[cfg(test)]
mod tests {
    use crate::instruments::{AmericanOption, EuropeanOption, OptionType};

    use super::*;

    #[test]
    fn test_binomial_tree_european_call() {
        let tree_map = static_binomial_tree!(2);
        let model = BinomialTreeModel::new(tree_map, Spot(100.0), 2, Expiry(0.5), 0.3, 0.05, 0.0);
        let option = EuropeanOption::new(OptionType::Call, 95.0, 0.5);
        let greeks = model.eval(option);
        assert_eq!(greeks.value(), Value(12.357803));
        assert_eq!(greeks.delta(), Delta(0.6599609));
    }

    #[test]
    fn test_binomial_tree_european_call2() {
        let tree_map = static_binomial_tree!(2);
        let model  = BinomialTreeModel::new(tree_map, Spot(810.0), 2, Expiry(0.5), 0.2, 0.05, 0.02);
        let option = EuropeanOption::new(OptionType::Call, 800.0, 0.5);
        let greeks = model.eval(option);
        assert_eq!(greeks.value(), Value(53.39472));
        assert_eq!(greeks.delta(), Delta(0.58913547));
    }

    #[test]
    fn test_binomial_tree_european_call3() {
        let tree_map = static_binomial_tree!(3);
        let model = BinomialTreeModel::new(tree_map, Spot(0.61), 3, Expiry(0.25), 0.12, 0.05, 0.07);
        let option = EuropeanOption::new(OptionType::Call, 0.6, 0.25);
        let greeks = model.eval(option);
        assert_eq!(greeks.value(), Value(0.018597351));
        assert_eq!(greeks.delta(), Delta(0.6000445));
    }

    #[test]
    fn test_binomial_tree_european_put1() {
        let tree_map = static_binomial_tree!(2);
        let model = BinomialTreeModel::new(tree_map, Spot(50.0), 2, Expiry(2.0), 0.3, 0.05, 0.0);
        let option = EuropeanOption::new(OptionType::Put, 52.0, 2.0);
        let greeks = model.eval(option);
        assert_eq!(greeks.value(), Value(6.2457113));
        assert_eq!(greeks.delta(), Delta(-0.37732533));
    }

    #[test]
    fn test_binomial_tree_american_put1() {
        let tree_map = static_binomial_tree!(2);
        let model = BinomialTreeModel::new(tree_map, Spot(50.0), 2, Expiry(2.0), 0.3, 0.05, 0.0);
        let option = AmericanOption::new(OptionType::Put, 52.0, 2.0);
        let greeks = model.eval(option);
        assert_eq!(greeks.value(), Value(7.428405));
        assert_eq!(greeks.delta(), Delta(-0.4606061));
    }

    #[test]
    fn test_binomial_tree_american_put2() {
        let tree_map = static_binomial_tree!(3);
        let model = BinomialTreeModel::new(tree_map, Spot(31.0), 3, Expiry(0.75), 0.3, 0.05, 0.05);
        let option = AmericanOption::new(OptionType::Put, 30.0, 0.75);
        let greeks = model.eval(option);
        assert_eq!(greeks.value(), Value(2.8356347));
        assert_eq!(greeks.delta(), Delta(-0.38601997));
        //assert_eq!(val.risk_free_probability, 0.4626);
    }

    #[test]
    fn test_binomial_tree_american_put2_100steps() {
        let tree_map = static_binomial_tree!(100);
        let model = BinomialTreeModel::new(tree_map, Spot(31.0), 100, Expiry(0.75), 0.3, 0.05, 0.05);
        let option = AmericanOption::new(OptionType::Put, 30.0, 0.75);
        let greeks = model.eval(option);
        assert_eq!(greeks.value(), Value(2.604315));
        assert_eq!(greeks.delta(), Delta(-0.38875514));
    }
}