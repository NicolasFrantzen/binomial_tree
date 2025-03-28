use std::iter::Rev;
use std::slice::Iter;
use crate::tree::{UpDown, NodeName};
use hashbrown::HashMap;
use itertools::{Itertools, sorted};

use std::sync::OnceLock;

const ALL_UPDOWNS: [UpDown; 2] = [UpDown::Up, UpDown::Down];

pub(crate) struct BinomialTreeMap {
    pub(crate) map: HashMap<NodeName, OnceLock<f32>>,
    stack: Vec<Vec<NodeName>>,
}

impl BinomialTreeMap {
    pub(crate) fn new(number_of_steps: usize) -> Self {
        let mut map = HashMap::<NodeName, OnceLock<f32>>::new();
        let mut stack: Vec<Vec<NodeName>> = vec![];

        let _ = map.try_insert(NodeName{name: vec![]}, OnceLock::new());
        stack.push(vec![NodeName{name: vec![]}]);

        for i in 1..=number_of_steps {
            let iter = ALL_UPDOWNS
                .iter()
                .cloned()
                .combinations_with_replacement(i)
                .map(|x| NodeName { name: x });

            for u in sorted(iter.clone()) {
                let _ = map.try_insert(u, OnceLock::new()); // TODO: insert_unique_unchecked might be ok here
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