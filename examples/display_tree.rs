use binominal_tree_model::eval_binomial_tree_with_steps;

fn main() {
    println!("=== Binomial Tree Visualization Example ===\n");

    // Create a small American Call option with 7 steps for clear visualization
    let tree = eval_binomial_tree_with_steps!(
        7,
        AmericanOption,
        Call,
        95.0,  // strike
        100.0, // spot
        0.5,   // expiry
        0.3,   // volatility
        0.05,  // interest_rate
        0.0    // dividend_rate
    );

    println!("American Call Option Binomial Tree");
    println!("Strike: 95.0, Spot: 100.0, Expiry: 0.5 years\n");
    println!("Option Value: {}\n", tree.value().0);

    println!("Tree Structure:\n");
    println!("{}", tree);
}
