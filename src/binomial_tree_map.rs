use std::iter::Rev;
use std::slice::Iter;
use std::sync::OnceLock;

use hashbrown::HashMap;
use itertools::Itertools;

use crate::nodes::{ALL_UPDOWNS, NodeName};

pub(crate) struct BinomialTreeMap {
    // Map consists of sorted keys only (with U < D). For example: UUUDD. Values are OnceLock, so they can be replaced without mutable borrowing
    pub(crate) map: HashMap<NodeName, OnceLock<f32>>, // TODO: Encapsulate this
    stack: Vec<Vec<NodeName>>, // TODO: Fix this, it's quite expensive to construct
}


// A special case of binomial formula with k = 2 and n = number_of_steps + 2
const fn calculate_capacity(number_of_steps: usize) -> usize {
    let mut res = 1;
    let n = number_of_steps + 2;
    res = (res * (n - 0)) / (n + 0);
    res = (res * (n - 1)) / (n + 1);

    res
}

impl BinomialTreeMap {
    pub(crate) fn new(number_of_steps: usize) -> Self {
        let mut map = HashMap::<NodeName, OnceLock<f32>>::with_capacity(calculate_capacity(number_of_steps));
        let mut stack: Vec<Vec<NodeName>> = Vec::with_capacity(number_of_steps);

        for i in 0..=number_of_steps {
            let iter = ALL_UPDOWNS
                .iter()
                .cloned()
                .combinations_with_replacement(i)
                .map(|x| NodeName { name: x });

            for node_name in iter.clone() {
                // NOTE: Unsafe is fine here, since we insert unique combinations
                unsafe {
                    let _ = map.insert_unique_unchecked(node_name, OnceLock::new());
                }
            }
            stack.push(iter.collect()); // TODO: These needs to be sorted
            //println!("{:?}", &stack);
            //println!("{:?}", &map);
        }

        Self {
            map,
            stack,
        }
    }

    pub(crate) fn iter(&self) -> BinomialTreeMapIterator<'_> {
        BinomialTreeMapIterator{iter: self.stack.iter().rev()}
    }
}

pub(crate) struct BinomialTreeMapIterator<'a> {
    iter: Rev<Iter<'a, Vec<NodeName>>>
}

impl<'a> Iterator for BinomialTreeMapIterator<'a> {
    type Item = &'a Vec<NodeName>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_map_size() {
        for i in 3..20 {
            assert_eq!(BinomialTreeMap::new(i).map.len(), calculate_capacity(i));
        }
    }

    #[test]
    fn test_stack_map() {
        let tree = BinomialTreeMap::new(3);
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