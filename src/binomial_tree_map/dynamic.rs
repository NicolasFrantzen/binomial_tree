use crate::binomial_tree_map::capacity::{calculate_capacity, calculate_step_capacity};
use crate::binomial_tree_map::nodes::{ALL_UPDOWNS, NodeName, NodeNameTrait};
use crate::binomial_tree_map::{
    BinomialTreeMapImpl, BinomialTreeMapNumericType, BinomialTreeMapValue, BinomialTreeStackImpl,
};
use hashbrown::HashMap;
use itertools::Itertools;
use std::ops::Deref;

#[derive(Default, Debug)]
pub struct DynamicBinomialTreeMap {
    // Map consists of sorted keys only (with U < D). For example: UUUDD. Values are OnceLock, so they can be replaced without mutable borrowing
    map: HashMap<NodeName, BinomialTreeMapValue<BinomialTreeMapNumericType>>,
    stack: Vec<Vec<NodeName>>, // TODO: Fix this, it's quite expensive to construct
}

impl DynamicBinomialTreeMap {
    #[allow(dead_code)]
    pub fn new(number_of_steps: usize) -> Self {
        let mut stack: Vec<Vec<NodeName>> = Vec::with_capacity(calculate_capacity(number_of_steps));

        for i in 0..=number_of_steps {
            let iter = ALL_UPDOWNS
                .iter()
                .cloned()
                .combinations_with_replacement(i)
                .map(NodeName::new);

            let mut vec = Vec::with_capacity(calculate_step_capacity(i));
            vec.extend(iter);

            stack.push(vec);
        }

        Self {
            map: Default::default(), // TODO: Decouple these
            stack,
        }
    }
}

impl BinomialTreeMapImpl for DynamicBinomialTreeMap {
    type NodeNameType = NodeName;
    type NumericType = f32;
    type ValueType = BinomialTreeMapValue<Self::NumericType>;

    fn get(&self, node_name: &Self::NodeNameType) -> Option<&Self::ValueType> {
        self.map.get(node_name)
    }

    fn get_next_step(&self, node_name: &Self::NodeNameType) -> Option<&Self::ValueType> {
        self.map.get(&node_name.up())
    }

    fn set(&mut self, node_name: &Self::NodeNameType, value: Self::NumericType) {
        self.map
            .entry(node_name.clone())
            .or_default()
            .set(value)
            .unwrap();
    }
}

impl BinomialTreeStackImpl for DynamicBinomialTreeMap {
    type NodeNameContainerType = DynamicBinomialTreeMap;

    fn iter(&self) -> impl DoubleEndedIterator +
        ExactSizeIterator<
            Item=&impl Deref<
                Target=[<<Self as BinomialTreeStackImpl>::NodeNameContainerType as BinomialTreeMapImpl>::NodeNameType]
            >
    >{
        self.stack.iter()
    }
}

#[cfg(test)]
mod tests {
    use crate::binomial_tree_map::dynamic::DynamicBinomialTreeMap;

    #[test]
    fn test_stack_initialization() {
        // Test that the stack is correctly initialized with all node names
        let tree = DynamicBinomialTreeMap::new(3);

        // Verify stack is created with correct structure
        let stack_vec: Vec<_> = tree.stack.iter().collect();
        assert_eq!(stack_vec.len(), 4); // 0 to 3 steps

        // Verify final step has correct number of nodes (C(3+2,3) = 10 combinations for step 3)
        assert_eq!(stack_vec[3].len(), 4); // 3 steps gives UUU, UUD, UDD, DDD
    }

    #[test]
    fn test_lazy_map_population() {
        // Test that the map starts empty (lazy loading)
        let tree = DynamicBinomialTreeMap::new(3);

        // Map should be empty initially since no values have been set
        assert_eq!(tree.map.len(), 0);

        // Test with larger tree
        let tree_large = DynamicBinomialTreeMap::new(5);
        assert_eq!(tree_large.map.len(), 0);
    }

    #[test]
    fn test_stack_structure() {
        // Test that the stack contains correct node names
        let tree = DynamicBinomialTreeMap::new(3);
        let mut stack_iter = tree.stack.iter().rev();

        // Final step: all combinations of U and D with 3 steps
        assert_eq!(
            stack_iter.next().unwrap(),
            &vec![
                "UUU".try_into().unwrap(),
                "UUD".try_into().unwrap(),
                "UDD".try_into().unwrap(),
                "DDD".try_into().unwrap()
            ]
        );

        // Step 2
        assert_eq!(
            stack_iter.next().unwrap(),
            &vec![
                "UU".try_into().unwrap(),
                "UD".try_into().unwrap(),
                "DD".try_into().unwrap()
            ]
        );

        // Step 1
        assert_eq!(
            stack_iter.next().unwrap(),
            &vec!["U".try_into().unwrap(), "D".try_into().unwrap()]
        );

        // Step 0
        assert_eq!(stack_iter.next().unwrap(), &vec!["".try_into().unwrap()]);
        assert_eq!(stack_iter.next(), None);
    }
}
