#[doc(hidden)]
#[macro_export]
macro_rules! binomial_tree_map {
    ($N:literal) => {
        {
            $crate::static_binomial_tree_map::StaticBinomialTreeMap::new::<{ $N+1 }>()
        }
    };
}
#[doc(hidden)]
#[macro_export]
macro_rules! eval_binomial_tree {
    ($N:literal, $option:ty, $option_type:ident, $strike:expr, $spot:expr, $expiry:expr, $volatility:expr, $interest_rate:expr, $dividend_rate:expr) => {
        {
            use $crate::binomial_tree_model::BinomialTreeModel;
            use $crate::binomial_tree_model::{Spot, Expiry};
            use $crate::instruments::{$option, OptionType, Option_};
            use $crate::static_binomial_tree_map::{StaticBinomialTreeMap, MAX_TREE_SIZE};

            if $N > MAX_TREE_SIZE {
                // We need to construct the tree dynamical

                todo!()
            }
            else {
                let tree_map = $crate::binomial_tree_map!($N);
                let binom_tree: BinomialTreeModel<StaticBinomialTreeMap> = BinomialTreeModel::new(
                    tree_map,
                    Spot($spot),
                    $N,
                    Expiry($expiry),
                    $volatility,
                    $interest_rate,
                    $dividend_rate);

                binom_tree.eval(<$option>::new(OptionType::$option_type, $strike, $expiry))
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! eval_binomial_tree_with_steps {
    ($N:literal, $($y:tt),+) => {
        {
            $crate::eval_binomial_tree!($N, $($y),+)
        }
    };
    ($($y:tt),+) => {
        {
            $crate::eval_binomial_tree_with_steps!(100, $($y),+)
        }
    };
}

#[macro_export]
macro_rules! american_value {
    (impl $option_type:ident, $($y:expr),+) => {
        {
            $crate::eval_binomial_tree_with_steps!(AmericanOption, $option_type, $($y),+).value()
        }
    };

    (Call, $($y:expr),+) => {
        {
            american_value!(impl Call, $($y),+)
        }
    };

    (Put, $($y:expr),+) => {
        {
            american_value!(impl Put, $($y),+)
        }
    };
}

#[macro_export]
macro_rules! european_value {
    ($option_type:ident, $($y:expr),+) => {
        {
            $crate::eval_binomial_tree_with_steps!(EuropeanOption, $option_type, $($y),+).value()
        }
    };

    (Call, $($y:expr),+) => {
        {
            european_value!(impl Call, $($y),+)
        }
    };

    (Put, $($y:expr),+) => {
        {
            european_value!(impl Put, $($y),+)
        }
    };
}

#[macro_export]
macro_rules! american_greeks {
    (impl $option_type:ident, $($y:expr),+) => {
        {
            $crate::eval_binomial_tree_with_steps!(AmericanOption, $option_type, $($y),+).greeks()
        }
    };

    (Call, $($y:expr),+) => {
        {
            american_greeks!(impl Call, $($y),+)
        }
    };

    (Put, $($y:expr),+) => {
        {
            american_greeks!(impl Put, $($y),+)
        }
    };
}

#[macro_export]
macro_rules! european_greeks {
    (impl $option_type:ident, $($y:expr),+) => {
        {
            $crate::eval_binomial_tree_with_steps!(EuropeanOption, $option_type, $($y),+).greeks()
        }
    };

    (Call, $($y:expr),+) => {
        {
            european_greeks!(impl Call, $($y),+)
        }
    };

    (Put, $($y:expr),+) => {
        {
            european_greeks!(impl Put, $($y),+)
        }
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_value() {
        let val = american_value!(Call, 95.0, 100.0, 0.5, 0.3, 0.05, 0.0);
        assert_eq!(val.0, 12.333031);

        let val = european_value!(Put, 105.0, 100.0, 0.5, 0.3, 0.05, 0.0);
        assert_eq!(val.0, 9.805331);

        let greeks = american_greeks!(Call, 95.0, 100.0, 0.5, 0.3, 0.05, 0.0);
        assert_eq!(greeks.delta.0, 0.6791013);

        let greeks = european_greeks!(Put, 105.0, 100.0, 0.5, 0.3, 0.05, 0.0);
        assert_eq!(greeks.delta.0, -0.50278103);
    }
}