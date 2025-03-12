use std::cell::{Cell, RefCell};
use std::iter::once;
use std::rc::Rc;
use crate::Greeks;
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

    fn value<T: Option_>(&self, option: T, underlying_price: f32) -> Greeks {
        let iter = self.tree.clone().into_iter();
        let mut last_node: Option<TreeNodeType> = None; // TODO: Use peekable and find the last one
        for node in iter { // TODO: Maybe use iter trait, to avoid extra clone?
            let p = self.params.p();
            if let Some(up) = &node.borrow().up {
                if let Some(down) = &node.borrow().down {
                    let payoff = (up.borrow().value.get() * p) + (down.borrow().value.get() * (1.0 - p)) * self.discount_factor;
                    node.borrow().value.set(payoff);
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

                let branch_price = node.borrow().price;
                let payoff = (branch_price - option.strike()).max(0.0); // NOTE: Assumed european call!
                node.borrow().value.set(payoff);
                last_node = Some(node.clone());
            }
        }
        if let Some(last_node) = last_node {
            Greeks{ value: last_node.borrow().value.get(), delta: 0.0 }
        }
        else {
            Greeks{ value: 0.0, delta: 0.0 }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instruments::EuropeanOption;

    fn r2(num: f32) -> f32
    {
        (num * 100.0).round() / 100.0
    }

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

        let val = binom_tree.value(EuropeanOption::new(95.0, 0.5), 100.0);
        println!("{}", val);
    }

    #[test]
    fn test_binomial_tree_european_call2() {
        let binom_tree = BinomialTree::new(810.0, 2, 0.5, 0.2, 0.05, 0.02);

        let val = binom_tree.value(EuropeanOption::new(800.0, 0.5), 810.0);
        println!("{}", val);

        assert_eq!(r2(val.value), 53.39);
    }
}