#[doc(hidden)]
#[macro_export]
macro_rules! binomial_tree_map {
    ($N:expr) => {{ $crate::binomial_tree_map::r#static::StaticBinomialTreeMap::with_capacity($N) }};
}
/// Evaluates a binomial tree for an option with specified parameters.
///
/// Automatically uses dynamic storage for trees with >128 steps, static storage otherwise.
/// Returns a type-erased `Box<dyn EvaluatedBinomialTree>` implementing Display and providing
/// access to value, greeks, and tree visualization.
///
/// # Arguments
/// - `steps` - Number of binomial tree steps
/// - `option_type` - AmericanOption or EuropeanOption
/// - `option` - Call or Put
/// - `strike` - Strike price
/// - `spot` - Current spot price
/// - `expiry` - Time to expiration (years)
/// - `volatility` - Volatility (annualized)
/// - `interest_rate` - Risk-free rate
/// - `dividend_rate` - Continuous dividend yield
///
/// # Example
/// ```
/// use binominal_tree_model::{eval_binomial_tree_with_steps, instruments::AmericanOption};
///
/// let tree = eval_binomial_tree_with_steps!(100, AmericanOption, Call, 95.0, 100.0, 0.5, 0.3, 0.05, 0.0);
/// assert!(tree.value().0 > 0.0);
/// // Display the tree
/// println!("{}", tree);
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! eval_binomial_tree {
    ($N:expr, $option:ty, $option_type:ident, $strike:expr, $spot:expr, $expiry:expr, $volatility:expr, $interest_rate:expr, $dividend_rate:expr) => {{
        use $crate::binomial_tree_map::r#static::{MAX_TREE_SIZE, StaticBinomialTreeMap};
        use $crate::instruments::{OptionContract, OptionType, $option};
        use $crate::model::{CoxRossRubenstein, erase_type, smoothing, truncation};
        use $crate::model::{Expiry, Spot};

        if $N > MAX_TREE_SIZE {
            let tree_map = $crate::binomial_tree_map::dynamic::DynamicBinomialTreeMap::new($N);
            let binom_tree: CoxRossRubenstein<
                $crate::binomial_tree_map::dynamic::DynamicBinomialTreeMap,
                smoothing::Black,
                truncation::Black,
            > = CoxRossRubenstein::new(
                tree_map,
                Spot($spot),
                $N,
                Expiry($expiry),
                $volatility,
                $interest_rate,
                $dividend_rate,
            );

            erase_type(binom_tree.eval(<$option>::new(OptionType::$option_type, $strike, $expiry)))
        } else {
            let tree_map = $crate::binomial_tree_map!($N);
            let binom_tree: CoxRossRubenstein<
                StaticBinomialTreeMap,
                smoothing::Black,
                truncation::Black,
            > = CoxRossRubenstein::new(
                tree_map,
                Spot($spot),
                $N,
                Expiry($expiry),
                $volatility,
                $interest_rate,
                $dividend_rate,
            );

            erase_type(binom_tree.eval(<$option>::new(OptionType::$option_type, $strike, $expiry)))
        }
    }};
}

/// Alias for `eval_binomial_tree!` with explicit step count.
#[doc(hidden)]
#[macro_export]
macro_rules! eval_binomial_tree_with_steps {
    ($N:expr, $($y:tt),+) => {
        {
            $crate::eval_binomial_tree!($N, $($y),+)
        }
    };
}

/// Calculates American option value (100-step binomial tree).
///
/// # Arguments
/// - `option` - Call or Put
/// - `strike, spot, expiry, volatility, interest_rate, dividend_rate` - Option parameters
///
/// # Example
/// ```
/// use binominal_tree_model::american_value;
///
/// let value = american_value!(Call, 95.0, 100.0, 0.5, 0.3, 0.05, 0.0);
/// assert!(value.0 > 10.0 && value.0 < 15.0);
/// ```
#[macro_export]
macro_rules! american_value {
    (impl $option_type:ident, $($y:expr),+) => {
        {
            $crate::eval_binomial_tree_with_steps!(100, AmericanOption, $option_type, $($y),+).value()
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

/// Calculates European option value (100-step binomial tree).
///
/// # Arguments
/// - `option` - Call or Put
/// - `strike, spot, expiry, volatility, interest_rate, dividend_rate` - Option parameters
///
/// # Example
/// ```
/// use binominal_tree_model::european_value;
///
/// let value = european_value!(Put, 105.0, 100.0, 0.5, 0.3, 0.05, 0.0);
/// assert!(value.0 > 8.0 && value.0 < 12.0);
/// ```
#[macro_export]
macro_rules! european_value {
    ($option_type:ident, $($y:expr),+) => {
        {
            $crate::eval_binomial_tree_with_steps!(100, EuropeanOption, $option_type, $($y),+).value()
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

/// Calculates American option Greeks (100-step binomial tree).
///
/// Returns Value, Delta, Gamma, and Theta.
///
/// # Arguments
/// - `option` - Call or Put
/// - `strike, spot, expiry, volatility, interest_rate, dividend_rate` - Option parameters
///
/// # Example
/// ```
/// use binominal_tree_model::american_greeks;
///
/// let greeks = american_greeks!(Call, 95.0, 100.0, 0.5, 0.3, 0.05, 0.0);
/// assert!(greeks.delta.0 > 0.0 && greeks.delta.0 < 1.0);
/// assert!(greeks.value.0 > 10.0);
/// ```
#[macro_export]
macro_rules! american_greeks {
    (impl $option_type:ident, $($y:expr),+) => {
        {
            $crate::eval_binomial_tree_with_steps!(100, AmericanOption, $option_type, $($y),+).greeks()
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

/// Calculates European option Greeks (100-step binomial tree).
///
/// Returns Value, Delta, Gamma, and Theta.
///
/// # Arguments
/// - `option` - Call or Put
/// - `strike, spot, expiry, volatility, interest_rate, dividend_rate` - Option parameters
///
/// # Example
/// ```
/// use binominal_tree_model::european_greeks;
///
/// let greeks = european_greeks!(Put, 105.0, 100.0, 0.5, 0.3, 0.05, 0.0);
/// assert!(greeks.delta.0 < 0.0 && greeks.delta.0 > -1.0);
/// assert!(greeks.value.0 > 8.0);
/// ```
#[macro_export]
macro_rules! european_greeks {
    (impl $option_type:ident, $($y:expr),+) => {
        {
            $crate::eval_binomial_tree_with_steps!(100, EuropeanOption, $option_type, $($y),+).greeks()
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
