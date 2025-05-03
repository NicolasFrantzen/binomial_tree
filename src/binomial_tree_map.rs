use std::cell::OnceCell;
use std::iter::Rev;
use std::slice::Iter;
use std::sync::OnceLock;

use crate::nodes::{NodeName, ALL_UPDOWNS};
use const_for::const_for;
use hashbrown::HashMap;
use itertools::Itertools;

pub(crate) trait BinomialTree {
    type NodeNameType;
    type NodeNameContainerType;
    type ValueType;
    type NodeType<T>;

    fn iter(&self) -> impl Iterator<Item = &Self::NodeNameContainerType>;
    fn get(&self, node_name: &Self::NodeNameType) -> Option<&Self::NodeType<Self::ValueType>>;
    fn set(&self, node_name: &Self::NodeNameType, value: Self::ValueType);
    //fn get_up(&self);
    //fn get_down(&self);
}

pub(crate) type BinomialTreeMapNumericType = f32;
pub(crate) type BinomialTreeMapValue<T> = OnceCell<T>;

pub(crate) struct BinomialTreeMap {
    // Map consists of sorted keys only (with U < D). For example: UUUDD. Values are OnceLock, so they can be replaced without mutable borrowing
    map: HashMap<NodeName, BinomialTreeMapValue<BinomialTreeMapNumericType>>,
    stack: Vec<Vec<NodeName>>, // TODO: Fix this, it's quite expensive to construct
}

const fn binom(n: usize, k: usize) -> usize {
    let mut res = 1;
    const_for!(i in 0..k => {
        res = res * (n - i) /
            (i + 1);
    });
    res
}

// A special case of binomial formula with k = 2 and n = number_of_steps + 2
pub(crate) const fn calculate_capacity(number_of_steps: usize) -> usize {
    binom(number_of_steps + 2usize, 2usize)
}

const fn calculate_step_capacity(step_number: usize) -> usize {
    if step_number > 1 {
        calculate_capacity(step_number) - calculate_capacity(step_number-1)
    }
    else
    {
        1usize
    }
}

impl BinomialTreeMap {
    pub(crate) fn new(number_of_steps: usize) -> Self {
        let mut map = HashMap::<NodeName, OnceCell<f32>>::with_capacity(calculate_capacity(number_of_steps));
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
                    let _ = map.insert_unique_unchecked(node_name, OnceCell::new());
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

impl BinomialTree for BinomialTreeMap {
    type NodeNameType = NodeName;
    type NodeNameContainerType = Vec<Self::NodeNameType>;
    type ValueType = f32;
    type NodeType<T> = BinomialTreeMapValue<T>;

    fn iter(&self) -> impl Iterator<Item = &Self::NodeNameContainerType> {
        self.stack.iter().rev()
    }

    fn get(&self, node_name: &Self::NodeNameType) -> Option<&Self::NodeType<Self::ValueType>> {
        self.map.get(node_name)
    }

    fn set(&self, node_name: &Self::NodeNameType, value: Self::ValueType) {
        self.get(node_name).expect("Map was not initialized").set(value).unwrap()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_map_size() {
        assert_eq!(BinomialTreeMap::new(3).map.len(), calculate_capacity(3));
        assert_eq!(BinomialTreeMap::new(4).map.len(), calculate_capacity(4));
        assert_eq!(BinomialTreeMap::new(5).map.len(), calculate_capacity(5));
        assert_eq!(BinomialTreeMap::new(6).map.len(), calculate_capacity(6));
        assert_eq!(BinomialTreeMap::new(7).map.len(), calculate_capacity(7));
        assert_eq!(BinomialTreeMap::new(8).map.len(), calculate_capacity(8));
    }

    #[test]
    fn test_stack_map() {
        let tree: BinomialTreeMap = BinomialTreeMap::new(3);
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