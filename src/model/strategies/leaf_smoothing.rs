/// Leaf smoothing strategies for binomial tree terminal nodes.
///
/// This module defines strategies for evaluating option values at the terminal nodes
/// (leaves) of a binomial tree. Different strategies can provide different levels
/// of accuracy or smoothing.
use crate::black_scholes::black_value;
use crate::instruments::OptionContract;
use crate::model::VolatilityParameters;

/// Trait for different leaf node smoothing strategies.
///
/// When the binomial tree reaches its terminal nodes (leaves), we need a strategy
/// for computing the option value. This trait defines the interface for different
/// smoothing approaches that can be applied at these leaf nodes.
pub trait ValueAtLeaf {
    /// Compute the option value at a terminal node (leaf).
    ///
    /// # Arguments
    ///
    /// * `option` - The option contract being evaluated
    /// * `price` - The stock price at this node
    /// * `vol_params` - Volatility and rate parameters
    /// * `expiry` - Time remaining until expiration
    fn value_at_leaf<U: OptionContract + Sync>(
        option: &U,
        price: f32,
        vol_params: &VolatilityParameters,
        expiry: f32,
    ) -> f32;
}

/// No smoothing strategy - uses only intrinsic value.
///
/// This is the simplest strategy: at terminal nodes, the option value is
/// simply its intrinsic value (payoff at that node). No analytical pricing
/// model is used.
impl ValueAtLeaf for None {
    fn value_at_leaf<U: OptionContract + Sync>(
        option: &U,
        price: f32,
        _vol_params: &VolatilityParameters,
        _expiry: f32,
    ) -> f32 {
        option.intrinsic_value(price)
    }
}

/// Black-Scholes smoothing strategy.
///
/// At terminal nodes, this strategy applies the Black-Scholes analytical
/// pricing model to smooth values across the leaves. This provides more
/// accurate pricing, especially when the number of steps is small.
impl ValueAtLeaf for Black {
    fn value_at_leaf<U: OptionContract + Sync>(
        option: &U,
        price: f32,
        vol_params: &VolatilityParameters,
        expiry: f32,
    ) -> f32 {
        let time_to_expiry = expiry; // There is one timestep left to expiry
        let black_value = black_value(
            option.option_type(),
            price,
            option.strike(),
            vol_params.volatility,
            vol_params.interest_rate,
            vol_params.dividends,
            time_to_expiry,
        );
        option.value(black_value, price)
    }
}

/// Marker type for no smoothing strategy.
pub struct None;

/// Marker type for Black-Scholes smoothing strategy.
pub struct Black;
