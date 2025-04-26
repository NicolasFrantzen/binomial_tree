use std::iter::{Filter, Rev};
use std::mem::MaybeUninit;
use std::slice::Iter;
use std::sync::OnceLock;

use const_for::const_for;
use hashbrown::HashMap;
use itertools::Itertools;
use crate::nodes::{ALL_UPDOWNS, NodeName, UpDown};

pub(crate) trait BinomialTree {
    type NodeNameType;
    type ValueType;
    type NodeType<T>;

    fn iter(&self) -> impl Iterator;

    fn get(&self, node_name: &Self::NodeNameType) -> Option<&Self::NodeType<Self::ValueType>>;
    fn set(&self, node_name: &Self::NodeNameType, value: Self::ValueType);
    //fn get_up(&self);
    //fn get_down(&self);
}

pub(crate) type BinomialTreeMapNumericType = f32;
pub(crate) type BinomialTreeMapValue<T> = OnceLock<T>;

pub(crate) struct BinomialTreeMap<const N: usize> {
    // Map consists of sorted keys only (with U < D). For example: UUUDD. Values are OnceLock, so they can be replaced without mutable borrowing
    map: HashMap<NodeName, BinomialTreeMapValue<BinomialTreeMapNumericType>>,
    stack: [Vec<NodeName>; N], // TODO: Fix this, it's quite expensive to construct
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

impl<const N: usize> BinomialTreeMap<N> {
    pub(crate) fn new(number_of_steps: usize) -> Self {
        let mut map = HashMap::<NodeName, OnceLock<f32>>::with_capacity(calculate_capacity(number_of_steps));
        let mut stack: [MaybeUninit<Vec<NodeName>>; N] = [const { MaybeUninit::uninit() }; N];

        const_for!(i in (0..N) => {
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

            let mut vec = Vec::with_capacity(calculate_step_capacity(i));
            vec.extend(iter);
            //let hej: [Option<NodeName>; N] = iter.collect::<Vec<Option<NodeName>>>().try_into().unwrap();

            stack[i].write(vec); // StackLevel{level: hej}
        });

        let stack: [Vec<NodeName>; N] = stack.map(|elem: MaybeUninit<Vec<NodeName>>| unsafe { elem.assume_init() });
        Self {
            map,
            stack,
        }
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

impl<const N: usize> BinomialTree for BinomialTreeMap<N> {
    type NodeNameType = NodeName;
    type ValueType = f32;
    type NodeType<T> = BinomialTreeMapValue<T>;

    fn iter(&self) -> BinomialTreeMapIterator<'_> {
        BinomialTreeMapIterator{iter: self.stack.iter().rev()}
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
        assert_eq!(BinomialTreeMap::<3>::new(3).map.len(), calculate_capacity(3));
        assert_eq!(BinomialTreeMap::<4>::new(4).map.len(), calculate_capacity(4));
        assert_eq!(BinomialTreeMap::<5>::new(5).map.len(), calculate_capacity(5));
        assert_eq!(BinomialTreeMap::<6>::new(6).map.len(), calculate_capacity(6));
        assert_eq!(BinomialTreeMap::<7>::new(7).map.len(), calculate_capacity(7));
        assert_eq!(BinomialTreeMap::<8>::new(8).map.len(), calculate_capacity(8));
    }

    #[test]
    fn test_stack_map() {
        let tree: BinomialTreeMap<3> = BinomialTreeMap::new(3);
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