use binominal_tree_model::american_greeks;

fn main() {
    let number_of_steps: usize = std::env::args().nth(1).expect("No number of steps given").parse::<usize>().unwrap();

    for _ in 1..1000 {
        american_greeks!(Call, 95.0, 100.0, 0.5, 0.3, 0.05, 0.0)
    }
}