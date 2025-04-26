#[macro_export]
macro_rules! static_binomial_tree {
    ($N:literal) => {
        {
            $crate::static_binomial_tree_map::StaticBinomialTree::new::<{ $N+1 }>()
        }
    };
}

pub use static_binomial_tree;

macro_rules! value {
    () => {};
}

macro_rules! greeks {
    () => {};
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate() {
        let x = static_binomial_tree!(2);
        println!("{:?}", x);
    }
}