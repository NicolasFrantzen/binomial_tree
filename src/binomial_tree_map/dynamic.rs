use hashbrown::HashMap;
use std::ops::Deref;
use std::cell::OnceCell;
use itertools::Itertools;
use crate::binomial_tree_map::capacity::{calculate_capacity, calculate_step_capacity};
use crate::binomial_tree_map::{BinomTreeValueType, BinomialTreeMapImpl, BinomialTreeMapNumericType, BinomialTreeMapValue, BinomialTreeStackImpl};
use crate::nodes::{NodeName, NodeNameTrait, ALL_UPDOWNS};

#[derive(Default)]
pub(crate) struct DynamicBinomialTreeMap {
    // Map consists of sorted keys only (with U < D). For example: UUUDD. Values are OnceLock, so they can be replaced without mutable borrowing
    map: HashMap<NodeName, BinomialTreeMapValue<BinomialTreeMapNumericType>>,
    stack: Vec<Vec<NodeName>>, // TODO: Fix this, it's quite expensive to construct
}

impl DynamicBinomialTreeMap {
    #[allow(dead_code)]
    pub(crate) fn new(number_of_steps: usize) -> Self {
        let mut map = HashMap::<NodeName, BinomTreeValueType>::with_capacity(calculate_capacity(number_of_steps));
        let mut stack: Vec<Vec<NodeName>> = Vec::with_capacity(calculate_capacity(number_of_steps));

        for i in 0..=number_of_steps {
            let iter = ALL_UPDOWNS
                .iter()
                .cloned()
                .combinations_with_replacement(i)
                .map(|x| NodeName::new(x) );

            for node_name in iter.clone() {
                // NOTE: Unsafe is fine here, since we insert unique combinations
                unsafe {
                    let _ = map.insert_unique_unchecked(node_name, BinomialTreeMapValue::new());
                }
            }

            let mut vec = Vec::with_capacity(calculate_step_capacity(i));
            vec.extend(iter);

            stack.push(vec);
        }

        Self {
            map,
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
        self.map.entry(node_name.clone()).or_insert(OnceCell::new()).set(value).unwrap();
    }
}

impl BinomialTreeStackImpl for DynamicBinomialTreeMap {
    type NodeNameContainerType = DynamicBinomialTreeMap;

    fn iter(&self) -> impl DoubleEndedIterator + ExactSizeIterator<Item=&impl Deref<Target=[<<Self as BinomialTreeStackImpl>::NodeNameContainerType as BinomialTreeMapImpl>::NodeNameType]>> {
        self.stack.iter()
    }
}

#[cfg(test)]
mod tests {
    use crate::binomial_tree_map::capacity::calculate_capacity;
    use crate::binomial_tree_map::*;
    use crate::binomial_tree_map::dynamic::DynamicBinomialTreeMap;
    use crate::nodes::NodeName;

    #[test]
    fn test_stack_map_size() {
        assert_eq!(DynamicBinomialTreeMap::new(3).map.len(), calculate_capacity(3));
        assert_eq!(DynamicBinomialTreeMap::new(4).map.len(), calculate_capacity(4));
        assert_eq!(DynamicBinomialTreeMap::new(5).map.len(), calculate_capacity(5));
        assert_eq!(DynamicBinomialTreeMap::new(6).map.len(), calculate_capacity(6));
        assert_eq!(DynamicBinomialTreeMap::new(7).map.len(), calculate_capacity(7));
        assert_eq!(DynamicBinomialTreeMap::new(8).map.len(), calculate_capacity(8));
    }

    #[test]
    fn test_stack_map() {
        let tree: DynamicBinomialTreeMap = DynamicBinomialTreeMap::new(3);
        let mut stack_iter = tree.stack.iter().rev();
        assert_eq!(stack_iter.next().unwrap(),
                   &vec!["UUU".try_into().unwrap(), "UUD".try_into().unwrap(), "UDD".try_into().unwrap(), "DDD".try_into().unwrap()]);
        assert_eq!(stack_iter.next().unwrap(), &vec!["UU".try_into().unwrap(), "UD".try_into().unwrap(), "DD".try_into().unwrap()]);
        assert_eq!(stack_iter.next().unwrap(), &vec!["U".try_into().unwrap(), "D".try_into().unwrap()]);
        assert_eq!(stack_iter.next().unwrap(), &vec!["".try_into().unwrap()]);
        assert_eq!(stack_iter.next(), None);


        assert_eq!(tree.map.len(), 10);
        assert!(tree.map.contains_key(&NodeName::try_from("UUU").unwrap()));
        assert!(tree.map.contains_key(&NodeName::try_from("UUD").unwrap()));
        assert!(tree.map.contains_key(&NodeName::try_from("UDD").unwrap()));
        assert!(tree.map.contains_key(&NodeName::try_from("DDD").unwrap()));
        assert!(tree.map.contains_key(&NodeName::try_from("UU").unwrap()));
        assert!(tree.map.contains_key(&NodeName::try_from("UD").unwrap()));
        assert!(tree.map.contains_key(&NodeName::try_from("DD").unwrap()));
        assert!(tree.map.contains_key(&NodeName::try_from("D").unwrap()));
        assert!(tree.map.contains_key(&NodeName::try_from("U").unwrap()));
        assert!(tree.map.contains_key(&NodeName::try_from("").unwrap()));
    }
}