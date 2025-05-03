#[macro_export]
macro_rules! binomial_tree {
    ($N:literal) => {
        {
            $crate::static_binomial_tree_map::StaticBinomialTreeMap::new::<{ $N+1 }>()
        }
    };
    ($N:expr) => {
        {
            $crate::binomial_tree_map::BinomialTreeMap::new({ $N+1 })
        }
    };
}

pub use binomial_tree;

macro_rules! eval_binomial_tree {
    ($N:literal) => {
        {
            use $crate::binomial_tree_model::BinomialTreeModel;
            use $crate::binomial_tree_model::{Spot, Expiry};
            use $crate::instruments::{AmericanOption, OptionType, Option_};
            use $crate::static_binomial_tree_map::StaticBinomialTreeMap;

            let tree_map = binomial_tree!($N);
            let binom_tree: BinomialTreeModel<StaticBinomialTreeMap> = BinomialTreeModel::new(tree_map, Spot(100.0), $N, Expiry(0.5), 0.3, 0.05, 0.0);

            binom_tree.eval(AmericanOption::new(OptionType::Call, 95.0, 0.5))
        }
    };
    ($N:expr) => {
        {
            use $crate::binomial_tree_model::BinomialTreeModel;
            use $crate::binomial_tree_model::{Spot, Expiry};
            use $crate::instruments::{AmericanOption, OptionType, Option_};

            let tree_map = binomial_tree!($N);
            let binom_tree: BinomialTreeModel<StaticBinomialTreeMap> = BinomialTreeModel::new(tree_map, Spot(100.0), $N, Expiry(0.5), 0.3, 0.05, 0.0);

            binom_tree.eval(AmericanOption::new(OptionType::Call, 95.0, 0.5))
        }
    };
}

macro_rules! value {
    ($N:tt) => {
        {
            eval_binomial_tree!($N).value()
        }
    };
}

macro_rules! greeks {
    () => {};
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate() {
        let x = binomial_tree!(2);
        println!("{:?}", x);
    }

    #[test]
    fn test_value() {
        let x = value!(2);
        assert_eq!(x.0, 12.3578);

        /*let n = 2;
        let x = value!(n);
        assert_eq!(x.0, 12.3578);*/
    }
}