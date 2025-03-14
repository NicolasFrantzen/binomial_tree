use std::cell::{Cell, RefCell};
use std::fmt::{Display, Formatter};
use std::iter::once;
use std::rc::Rc;

use crate::tree::{NodeName, Tree, TreeNode, TreeNodeType, UpDown};
use crate::instruments::Option_;

pub struct BinomialTree {
    tree: Tree,
    params: VolatilityParameters,
    discount_factor: f32,
}

impl BinomialTree {
    // TODO: Newtypes
    fn new(initial_price: f32, num_steps: u32, expiry: f32, volatility: f32, interest_rate: f32, dividends: f32) -> Self {
        let timestep = expiry / num_steps as f32;
        let vol_params = VolatilityParameters::new(volatility, interest_rate, dividends, timestep);

        let tree = Tree {
            root: Rc::new(RefCell::new(TreeNode {
                parent: None,
                up: None,
                down: None,
                price: initial_price,
                value: Cell::new(0.0),
                name: NodeName{name: vec![UpDown::Initial]},
            })),
        };
        Self::add_branches(tree.root.clone(), &vol_params, 1, num_steps);

        Self {
            tree,
            params: vol_params,
            discount_factor: (-interest_rate * timestep).exp(),
        }
    }

    fn add_branches(node: TreeNodeType, vol_params: &VolatilityParameters, level: u32, max_level: u32) {
        if level > max_level {
            return;
        }

        let name = node.borrow().name.name.clone();
        let branch_price = node.borrow().price;

        if let Some(up) = TreeNode::is_duplicate(&node) {
            node.borrow_mut().up = Some(up);
        }
        else {
            node.borrow_mut().up = Some(Rc::new(RefCell::new(TreeNode {
                parent: Some(node.clone()),
                up: None,
                down: None,
                price: branch_price * vol_params.u,
                value: Cell::new(0.0),
                name: NodeName{name: name.iter().clone().chain(once(&UpDown::Up)).cloned().collect()}
            })));
        }

        node.borrow_mut().down = Some(Rc::new(RefCell::new(TreeNode {
            parent: Some(node.clone()),
            up: None,
            down: None,
            price: branch_price * vol_params.d,
            value: Cell::new(0.0),
            name: NodeName{name: name.iter().chain(once(&UpDown::Down)).cloned().collect()}
        })));

        Self::add_branches(node.borrow().up.clone().unwrap(), vol_params,level + 1, max_level);
        Self::add_branches(node.borrow().down.clone().unwrap(), vol_params, level + 1, max_level);
    }

    fn value<T: Option_>(&self, option: T) -> Value {
        let p = self.params.p();
        let iter = self.tree.clone().into_iter();
        let mut last_node: Option<TreeNodeType> = None; // TODO: Use peekable and find the last one
        for node in iter { // TODO: Maybe use iter trait, to avoid extra clone?
            let branch_price = node.borrow().price;

            if let Some(up) = &node.borrow().up {
                if let Some(down) = &node.borrow().down {
                    let value = ((up.borrow().value.get() * p) + (down.borrow().value.get() * (1.0 - p))) * self.discount_factor;
                    node.borrow().value.set(option.value(value, branch_price));

                    //println!("{} -> {}/{}", &node.borrow().name, &node.borrow().value.get(), &node.borrow().price);
                    last_node = Some(node.clone());
                }
                else {
                    panic!("Tree incomplete")
                }
            }
            else {
                if let Some(down) = &node.borrow().down {
                    panic!("Tree incomplete");
                }

                let payoff = option.payoff(branch_price);
                node.borrow().value.set(payoff);

                //println!("{} -> {}/{}", &node.borrow().name, &node.borrow().value.get(), &node.borrow().price);
                last_node = Some(node.clone());
            }
        }
        if let Some(last_node) = last_node {
            println!("{}", self.params.p());
            Value { value: last_node.borrow().value.get(), delta: 0.0, risk_free_probability: p }
        }
        else {
            Value { value: 0.0, delta: 0.0, risk_free_probability: p }
        }
    }
}

#[derive(Copy, Clone)]
struct VolatilityParameters {
    a: f32,
    u: f32,
    d: f32,
}

impl VolatilityParameters {
    fn new(volatility: f32, interest_rate: f32, dividends: f32, timestep: f32) -> VolatilityParameters {
        let u= (volatility*timestep.sqrt()).exp();
        VolatilityParameters {
            a: ((interest_rate - dividends) * timestep).exp(),
            u,
            d: 1.0 / u,
        }
    }

    fn p(&self) -> f32 {
        (self.a - self.d)/(self.u - self.d)
    }
}

struct Value {
    value: f32,
    delta: f32,
    risk_free_probability: f32,
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}",self.value, self.delta)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instruments::{AmericanOption, EuropeanOption, OptionType};

    fn r2(num: f32) -> f32 { (num * 100.0).round() / 100.0 }
    fn r3(num: f32) -> f32 { (num * 1000.0).round() / 1000.0 }
    fn r4(num: f32) -> f32 { (num * 10000.0).round() / 10000.0 }

    #[test]
    fn test_binomial_tree_new() {
        let binom_tree = BinomialTree::new(100.0, 2, 0.5, 0.3, 0.05, 0.0);

        let mut iter =  binom_tree.tree.into_iter();

        assert_eq!(r2(iter.next().unwrap().borrow().price), 100.00);
        assert_eq!(r2(iter.next().unwrap().borrow().price), 134.99);
        assert_eq!(r2(iter.next().unwrap().borrow().price), 74.08);
        assert_eq!(r2(iter.next().unwrap().borrow().price), 86.07);
        assert_eq!(r2(iter.next().unwrap().borrow().price), 116.18);
        assert_eq!(r2(iter.next().unwrap().borrow().price), 100.00);
    }

    #[test]
    fn test_binomial_tree_european_call() {
        let binom_tree = BinomialTree::new(100.0, 2, 0.5, 0.3, 0.05, 0.0);
        let val = binom_tree.value(EuropeanOption::new(OptionType::Call, 95.0, 0.5));
        assert_eq!(r2(val.value), 12.36)
    }

    #[test]
    fn test_binomial_tree_european_call2() {
        let binom_tree = BinomialTree::new(810.0, 2, 0.5, 0.2, 0.05, 0.02);
        let val = binom_tree.value(EuropeanOption::new(OptionType::Call, 800.0, 0.5));
        assert_eq!(r2(val.value), 53.39);
    }

    #[test]
    fn test_binomial_tree_european_call3() {
        let binom_tree = BinomialTree::new(0.61, 3, 0.25, 0.12, 0.05, 0.07);
        let val = binom_tree.value(EuropeanOption::new(OptionType::Call, 0.6, 0.25));
        assert_eq!(r3(val.value), 0.019);
    }

    #[test]
    fn test_binomial_tree_european_put1() {
        let binom_tree = BinomialTree::new(50.0, 2, 2.0, 0.3, 0.05, 0.0);
        let val = binom_tree.value(EuropeanOption::new(OptionType::Put, 52.0, 2.0));
        assert_eq!(r2(val.value), 6.25);
    }

    #[test]
    fn test_binomial_tree_american_put1() {
        let binom_tree = BinomialTree::new(50.0, 2, 2.0, 0.3, 0.05, 0.0);
        let val = binom_tree.value(AmericanOption::new(OptionType::Put, 52.0, 2.0));
        assert_eq!(r2(val.value), 7.43);
    }
    
    #[test]
    fn test_binomial_tree_american_put2() {
        let binom_tree = BinomialTree::new(31.0, 3, 0.75, 0.3, 0.05, 0.05);
        let val = binom_tree.value(AmericanOption::new(OptionType::Put, 30.0, 0.75));
        assert_eq!(r2(val.value), 2.84);
        assert_eq!(r4(val.risk_free_probability), 0.4626);
    }
}