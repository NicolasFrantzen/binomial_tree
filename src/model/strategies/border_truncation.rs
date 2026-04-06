/// Border truncation strategies for binomial tree edges.
///
/// This module defines strategies for handling nodes at the edges of the
/// binomial tree that would extend beyond the computational domain.
/// Different strategies can decide whether to compute a value or exclude
/// nodes from evaluation.
use crate::black_scholes::black_value;
use crate::instruments::OptionContract;
use crate::model::VolatilityParameters;

/// Trait for different border truncation strategies.
///
/// As a binomial tree expands, extreme nodes at the borders can be outside
/// the typical price range. This trait defines strategies for handling these
/// boundary cases - either by excluding them or by applying special pricing.
pub trait ValueAtBorder {
    /// Create a new truncation strategy instance.
    fn new(spot: f32, expiry: f32, volatility: f32, rate: f32, dividends: f32) -> Self;

    /// Compute the option value at a border node, if it should be included.
    ///
    /// # Arguments
    ///
    /// * `option` - The option contract being evaluated
    /// * `value` - The computed value from child nodes
    /// * `price` - The stock price at this node
    /// * `vol_params` - Volatility and rate parameters
    /// * `expiry` - Time remaining until expiration
    ///
    /// # Returns
    ///
    /// `Some(value)` if the node should be included, `None` if it should be excluded.
    fn value<U: OptionContract + Sync>(
        &self,
        option: &U,
        value: f32,
        price: f32,
        vol_params: &VolatilityParameters,
        expiry: f32,
    ) -> Option<f32>;

    /// Returns whether this strategy is a non-trivial truncation (not no-op).
    fn not_none() -> bool;
}

/// No truncation strategy - all nodes are included.
///
/// This strategy does not filter any nodes at the borders.
/// All nodes in the tree are evaluated using intrinsic values.
impl ValueAtBorder for None {
    fn new(_spot: f32, _expiry: f32, _volatility: f32, _rate: f32, _dividends: f32) -> Self {
        Self {}
    }

    fn value<U: OptionContract + Sync>(
        &self,
        option: &U,
        value: f32,
        price: f32,
        _vol_params: &VolatilityParameters,
        _expiry: f32,
    ) -> Option<f32> {
        Some(option.value(value, price))
    }

    fn not_none() -> bool {
        false
    }
}

/// Black-Scholes based truncation strategy.
///
/// This strategy excludes nodes that fall outside a confidence band
/// calculated using the Black-Scholes framework. Under the risk-neutral
/// measure, the stock price is expected to stay within these bounds
/// with high probability. This is more computationally efficient than
/// evaluating all extreme nodes.
impl ValueAtBorder for Black {
    fn new(spot: f32, expiry: f32, volatility: f32, rate: f32, dividends: f32) -> Self {
        const NUM_OF_STD: usize = 6;
        Self {
            price_bounds: PriceBounds::new(spot, expiry, volatility, rate, dividends, NUM_OF_STD),
        }
    }

    fn value<U: OptionContract + Sync>(
        &self,
        option: &U,
        _value: f32,
        price: f32,
        vol_params: &VolatilityParameters,
        current_expiry: f32,
    ) -> Option<f32> {
        if self.price_bounds.is_out_of_range(price) {
            return Option::None;
        }

        let black_value = black_value(
            option.option_type(),
            price,
            option.strike(),
            vol_params.volatility,
            vol_params.interest_rate,
            vol_params.dividends,
            current_expiry,
        );
        Some(option.value(black_value, price))
    }

    fn not_none() -> bool {
        true
    }
}

/// Marker type for no truncation strategy.
pub struct None;

/// Marker type for Black-Scholes based truncation strategy.
pub struct Black {
    price_bounds: PriceBounds,
}

/// Price bounds for boundary checking.
///
/// Calculates and stores the upper and lower price bounds within which
/// tree nodes are evaluated, under the risk-neutral measure.
struct PriceBounds {
    lower_bound: f32,
    upper_bound: f32,
}

impl PriceBounds {
    /// Compute log-space and standard-space boundaries at num_std deviations under Q measure.
    ///
    /// Using the lognormal distribution of stock prices under the risk-neutral measure,
    /// this calculates confidence bounds based on standard deviations.
    fn new(
        spot: f32,
        expiry: f32,
        volatility: f32,
        rate: f32,
        dividends: f32,
        number_of_std: usize,
    ) -> PriceBounds {
        let mean_log = spot.ln() + (rate - dividends - 0.5 * volatility.powi(2) * expiry);
        let std_log = volatility * expiry.sqrt();

        let lower_log = mean_log - (number_of_std as f32) * std_log;
        let upper_log = mean_log + (number_of_std as f32) * std_log;

        let lower_price = lower_log.exp();
        let upper_price = upper_log.exp();

        PriceBounds {
            lower_bound: lower_price,
            upper_bound: upper_price,
        }
    }

    fn is_out_of_range(&self, price: f32) -> bool {
        let in_range = (self.lower_bound..=self.upper_bound).contains(&price);
        !in_range
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_price_bounds() {
        let bounds = PriceBounds::new(100.0, 0.5, 0.3, 0.05, 0.0, 6);

        assert_eq!(bounds.lower_bound, 28.78568);
        assert_eq!(bounds.upper_bound, 367.03702);

        assert!(!bounds.is_out_of_range(100.0));
        assert!(bounds.is_out_of_range(0.0));
        assert!(bounds.is_out_of_range(400.0));
        assert!(!bounds.is_out_of_range(367.03702));
    }
}
