use binominal_tree_model::american_greeks;

use rayon::prelude::*;

fn main() {
    let number_of_steps: usize = std::env::args().nth(1).expect("No number of calculations given").parse::<usize>().unwrap();

    // Note number_of_steps = 1000 is good for profiling
    (1..number_of_steps).into_par_iter().for_each(|_| {
        american_greeks!(Call, 95.0, 100.0, 0.5, 0.3, 0.05, 0.0);
    });
}