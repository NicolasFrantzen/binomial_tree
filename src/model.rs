use crate::binomial_tree_map::{BinomialTreeMapImpl, BinomialTreeStackImpl, GetValue};
use crate::instruments::Option_;
use crate::binomial_tree_map::nodes::NodeNameTrait;

pub struct BinomialTreeModel<Stack> {
    stack: Stack,
    params: VolatilityParameters,
    spot: Spot,
    discount_factor: f32,
    time_step: f32,
}

#[allow(private_bounds)]
impl<Stack: BinomialTreeStackImpl> BinomialTreeModel<Stack> {
    pub fn new(stack: Stack, initial_price: Spot, number_of_steps: usize, expiry: Expiry, volatility: f32, interest_rate: f32, dividends: f32) -> Self {
        let time_step = expiry.0 / number_of_steps as f32;
        let vol_params = VolatilityParameters::new(volatility, interest_rate, dividends, time_step);

        Self {
            stack,
            params: vol_params,
            spot: initial_price,
            discount_factor: (-interest_rate * time_step).exp(),
            time_step,
        }
    }

    pub fn eval<T: Option_ + Sync>(self, option: T) -> EvaluatedBinomialTreeModelImpl<Stack>
    {
        let p = self.params.p();

        let mut tree_map = <Stack as BinomialTreeStackImpl>::NodeNameContainerType::default();

        for (i, node_level) in self.stack.iter().enumerate().rev() {
            node_level.iter().rev().enumerate().for_each(|(j, node)| {

                let up_value = tree_map.get(&node.up());
                let down_value = tree_map.get(&node.down());

                // TODO: Hide details
                let price = self.spot.0 * self.params.u.powi(j as i32) * self.params.d.powi((i-j) as i32);

                //println!("{:?}{:?}", node.up(), up_value);
                //println!("{:?}{:?}", node.down(), down_value);

                if let (Some(up_value), Some(down_value)) = (up_value, down_value) {
                    let up_value = up_value.get();
                    let down_value = down_value.get(); //.expect("The tree should be evaluated backwards");

                    // TODO: Hide details
                    let value = (up_value * p + down_value * (1.0 - p)) * self.discount_factor;

                    let option_value = option.value(value, price);
                    tree_map.set(node, option_value.into());
                } else {
                    let option_value = option.intrinsic_value(price);
                    tree_map.set(node, option_value.into());
                }
            });
        }

        EvaluatedBinomialTreeModelImpl {
            model: self,
            map: tree_map,
        }
    }
}

#[allow(private_bounds)]
pub struct EvaluatedBinomialTreeModelImpl<Stack: BinomialTreeStackImpl> {
    model: BinomialTreeModel<Stack>,
    map: <Stack as BinomialTreeStackImpl>::NodeNameContainerType,
}

#[allow(private_bounds)]
impl<Stack: BinomialTreeStackImpl> EvaluatedBinomialTreeModelImpl<Stack> {
    pub fn value(&self) -> Value
    {
        let initial_node = <<Stack as BinomialTreeStackImpl>::NodeNameContainerType as BinomialTreeMapImpl>::NodeNameType::default();
        let value = self.map.get(&initial_node).unwrap().get();
        Value(*value)
    }

    pub fn delta(&self) -> Delta {
        self.delta_from(&<<Stack as BinomialTreeStackImpl>::NodeNameContainerType as BinomialTreeMapImpl>::NodeNameType::default())
    }

    fn delta_from(&self, from_node: &<<Stack as BinomialTreeStackImpl>::NodeNameContainerType as BinomialTreeMapImpl>::NodeNameType) -> Delta {
        let last_up = from_node.up();
        let last_up_value =  self.map.get(&last_up).unwrap().get();
        let last_down = from_node.down();
        let last_down_value = self.map.get(&last_down).unwrap().get();
        let h = last_up.value(self.model.spot.0, self.model.params.u, self.model.params.d) - last_down.value(
            self.model.spot.0,
            self.model.params.u,
            self.model.params.d
        );

        if h != 0.0 {
            let delta = (last_up_value - last_down_value) / h;
            Delta(delta)
        } else { Delta(0.0) }
    }

    pub fn gamma(&self) -> Gamma {
        let initial_node = <<Stack as BinomialTreeStackImpl>::NodeNameContainerType as BinomialTreeMapImpl>::NodeNameType::default();
        let node_u = initial_node.up();
        let node_d = initial_node.down();
        let delta_u = self.delta_from(&node_u);
        let delta_d = self.delta_from(&node_d);
        let spot_u = self.map.get(&node_u).unwrap().get();
        let spot_d = self.map.get(&node_d).unwrap().get();

        if spot_u == spot_d {
            Gamma(0.0)
        }
        else {
            Gamma((delta_u.0 - delta_d.0) / (spot_u - spot_d))
        }
    }

    pub fn theta(&self) -> Theta {
        let initial_node = <<Stack as BinomialTreeStackImpl>::NodeNameContainerType as BinomialTreeMapImpl>::NodeNameType::default();
        let val_0 = self.map.get(&initial_node).unwrap().get();
        let val_2 = self.map.get_next_step(&initial_node).unwrap().get();

        assert_ne!(self.model.time_step, 0.0);
        Theta((val_2 - val_0) / (2.0 * self.model.time_step))
    }

    pub fn greeks(&self) -> Greeks {
        Greeks {
            value: self.value(),
            delta: self.delta(),
            gamma: self.gamma(),
            theta: self.theta(),
            // TODO: Implement vega and rho
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
    pub value: Value,
    pub delta: Delta,
    pub gamma: Gamma,
    pub theta: Theta,
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
    use crate::binomial_tree_map;
    use crate::instruments::{AmericanOption, EuropeanOption, OptionType};
    use crate::binomial_tree_map::r#static::StaticBinomialTreeMap;
    use super::*;

    #[test]
    fn test_binomial_tree_european_call() {
        let tree_map = binomial_tree_map!(2);
        let model: BinomialTreeModel<StaticBinomialTreeMap> = BinomialTreeModel::new(tree_map, Spot(100.0), 2, Expiry(0.5), 0.3, 0.05, 0.0);
        let option = EuropeanOption::new(OptionType::Call, 95.0, 0.5);
        let greeks = model.eval(option);
        assert_eq!(greeks.value(), Value(12.3578));
        assert_eq!(greeks.delta(), Delta(0.6599607));
    }

    #[test]
    fn test_binomial_tree_european_call2() {
        let tree_map = binomial_tree_map!(2);
        let model: BinomialTreeModel<StaticBinomialTreeMap> = BinomialTreeModel::new(tree_map, Spot(810.0), 2, Expiry(0.5), 0.2, 0.05, 0.02);
        let option = EuropeanOption::new(OptionType::Call, 800.0, 0.5);
        let greeks = model.eval(option);
        assert_eq!(greeks.value(), Value(53.394733));
        assert_eq!(greeks.delta(), Delta(0.5891357));
    }

    #[test]
    fn test_binomial_tree_european_call3() {
        let tree_map = binomial_tree_map!(3);
        let model: BinomialTreeModel<StaticBinomialTreeMap> = BinomialTreeModel::new(tree_map, Spot(0.61), 3, Expiry(0.25), 0.12, 0.05, 0.07);
        let option = EuropeanOption::new(OptionType::Call, 0.6, 0.25);
        let greeks = model.eval(option);
        assert_eq!(greeks.value(), Value(0.018597357));
        assert_eq!(greeks.delta(), Delta(0.6000447));
    }

    #[test]
    fn test_binomial_tree_european_put1() {
        let tree_map = binomial_tree_map!(2);
        let model: BinomialTreeModel<StaticBinomialTreeMap> = BinomialTreeModel::new(tree_map, Spot(50.0), 2, Expiry(2.0), 0.3, 0.05, 0.0);
        let option = EuropeanOption::new(OptionType::Put, 52.0, 2.0);
        let greeks = model.eval(option);
        assert_eq!(greeks.value(), Value(6.2457113));
        assert_eq!(greeks.delta(), Delta(-0.37732533));
    }

    #[test]
    fn test_binomial_tree_american_put1() {
        let tree_map = binomial_tree_map!(2);
        let model: BinomialTreeModel<StaticBinomialTreeMap> = BinomialTreeModel::new(tree_map, Spot(50.0), 2, Expiry(2.0), 0.3, 0.05, 0.0);
        let option = AmericanOption::new(OptionType::Put, 52.0, 2.0);
        let eval = model.eval(option);

        assert_eq!(eval.greeks(), Greeks{
            value: Value(7.428405),
            delta: Delta(-0.4606061),
            gamma: Gamma(-0.0), // Hmm
            theta: Theta(-2.7142024),
        })
    }

    #[test]
    fn test_binomial_tree_american_put2() {
        let tree_map = binomial_tree_map!(3);
        let model: BinomialTreeModel<StaticBinomialTreeMap> = BinomialTreeModel::new(tree_map, Spot(31.0), 3, Expiry(0.75), 0.3, 0.05, 0.05);
        let option = AmericanOption::new(OptionType::Put, 30.0, 0.75);
        let greeks = model.eval(option);
        assert_eq!(greeks.value(), Value(2.8356347));
        assert_eq!(greeks.delta(), Delta(-0.38601997));
        //assert_eq!(val.risk_free_probability, 0.4626);
    }

    #[test]
    fn test_binomial_tree_american_put3() {
        let tree_map = binomial_tree_map!(3);
        let model: BinomialTreeModel<StaticBinomialTreeMap> = BinomialTreeModel::new(tree_map, Spot(60.0), 3, Expiry(3.0/12.0), 0.45, 0.1, 0.00);
        let option = AmericanOption::new(OptionType::Put, 60.0, 3.0/12.0);
        let greeks = model.eval(option);
        assert_eq!(greeks.value(), Value(5.1627836));
        assert_eq!(greeks.delta(), Delta(-0.43557432));
        //println!("{:?}", greeks.model.tree_map.map);
    }

    #[test]
    fn test_binomial_tree_american_fut_call1() {
        let tree_map = binomial_tree_map!(3);
        // Notice r = q for futs
        let model: BinomialTreeModel<StaticBinomialTreeMap> = BinomialTreeModel::new(tree_map, Spot(400.0), 3, Expiry(9.0/12.0), 0.35, 0.06, 0.06);
        let option = AmericanOption::new(OptionType::Call, 420.0, 9.0/12.0);
        let greeks = model.eval(option);
        assert_eq!(greeks.value(), Value(42.06769));
        assert_eq!(greeks.delta(), Delta(0.48716724));
        //println!("{:?}", greeks.model.tree_map.map);
    }

    #[test]
    fn test_binomial_tree_american_put2_100steps() {
        let tree_map = binomial_tree_map!(100);
        let model: BinomialTreeModel<StaticBinomialTreeMap> = BinomialTreeModel::new(tree_map, Spot(31.0), 100, Expiry(0.75), 0.3, 0.05, 0.05);
        let option = AmericanOption::new(OptionType::Put, 30.0, 0.75);
        let greeks = model.eval(option);
        assert_eq!(greeks.value(), Value(2.6043036));
        assert_eq!(greeks.delta(), Delta(-0.38875455));
    }
}