use binominal_tree_model::eval_binomial_tree_with_steps;

fn main() {
    println!("=== Greeks Calculation Example ===\n");

    // American Call Option
    println!("American Call Option");
    println!("Strike: 95.0, Spot: 100.0, Expiry: 0.5 years, Vol: 0.3, Rate: 0.05\n");

    let american_tree = eval_binomial_tree_with_steps!(
        100,
        AmericanOption,
        Call,
        95.0,  // strike
        100.0, // spot
        0.5,   // expiry
        0.3,   // volatility
        0.05,  // interest_rate
        0.0    // dividend_rate
    );

    let american_greeks = american_tree.greeks();
    println!("Greeks:");
    println!("  Value: {:.4}", american_greeks.value.0);
    println!("  Delta: {:.4}", american_greeks.delta.0);
    println!("  Gamma: {:.4}", american_greeks.gamma.0);
    println!("  Theta: {:.4}\n", american_greeks.theta.0);

    // European Put Option
    println!("European Put Option");
    println!("Strike: 105.0, Spot: 100.0, Expiry: 0.5 years, Vol: 0.3, Rate: 0.05\n");

    let european_tree = eval_binomial_tree_with_steps!(
        100,
        EuropeanOption,
        Put,
        105.0, // strike
        100.0, // spot
        0.5,   // expiry
        0.3,   // volatility
        0.05,  // interest_rate
        0.0    // dividend_rate
    );

    let european_greeks = european_tree.greeks();
    println!("Greeks:");
    println!("  Value: {:.4}", european_greeks.value.0);
    println!("  Delta: {:.4}", european_greeks.delta.0);
    println!("  Gamma: {:.4}", european_greeks.gamma.0);
    println!("  Theta: {:.4}", european_greeks.theta.0);
}
