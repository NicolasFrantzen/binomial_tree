use std::cell::OnceCell;
use std::ops::Deref;
use hashbrown::HashMap;
use binomial_tree_macro::binomial_tree_stack;
use crate::binomial_tree_map::{calculate_capacity, BinomialTreeMap, BinomialTreeMapImpl, BinomialTreeStack, BinomialTreeStackImpl};
use crate::binomial_tree_map::BinomialTreeMapValue;
use crate::binomial_tree_map::BinomialTreeMapNumericType;
use crate::nodes::{NodeName2, NodeNameTrait, UpDown};

pub static MAX_STATIC_TREE_SIZE: usize = 128;
const PRE_ALLOCATED_STACK: &'static [&'static [NodeName2]] = binomial_tree_stack!(128);

#[derive(Debug, Default)]
pub struct StaticBinomialTreeMap {
    //pub(crate) map: HashMap<NodeName2, OnceCell<BinomialTreeMapNumericType>>,
    pub(crate) stack: &'static [&'static [NodeName2]], // TODO: DECOUPLE THIS
}

impl StaticBinomialTreeMap {
    pub fn new<const N: usize>() -> StaticBinomialTreeMap
    {
        //let mut map = HashMap::<NodeName2, OnceCell<f32>>::with_capacity(calculate_capacity(N));
        let stack: &'static [&'static [NodeName2]] = &PRE_ALLOCATED_STACK[..N]; // TODO: This trick could probably be used in a dynamic case as well!

        //println!("{:?}", stack);

        /*for stack_level in stack.iter() {
            for node_name in stack_level.iter().cloned() {
                unsafe {
                    let _ = map.insert_unique_unchecked(node_name, OnceCell::new());
                }
            }
        }*/

        //println!("{:?}", map);

        StaticBinomialTreeMap {
            //map,
            stack,
        }
    }
}

/*
    type NodeNameType = NodeName;
    type NodeNameContainerType = Vec<Self::NodeNameType>;
    type ValueType = f32;
    type NodeType = BinomialTreeMapValue<Self::ValueType>;
 */

#[derive(Default)]
pub struct StaticContainer {
    pub(crate) map: HashMap<NodeName2, OnceCell<BinomialTreeMapNumericType>>,
}

impl BinomialTreeMapImpl for StaticContainer {
    type NodeNameType = NodeName2;
    //type NodeNameContainerType = &'static [Self::NodeNameType];
    type ValueType = f32;
    type NodeType = BinomialTreeMapValue<Self::ValueType>;

    /*fn iter(&self) -> impl DoubleEndedIterator + ExactSizeIterator<Item = &impl Deref<Target = [Self::NodeNameType]>> {
        self.stack.iter()
    }*/

    fn get(&self, node_name: &Self::NodeNameType) -> Option<&Self::NodeType> {
        self.map.get(node_name)
    }

    fn get_next_step(&self, node_name: &Self::NodeNameType) -> Option<&Self::NodeType> {
        let (key, _) = self.map.get_key_value(&node_name.up()).unwrap();
        self.get(&key.down())
    }

    fn set(&mut self, node_name: &Self::NodeNameType, value: Self::ValueType) {
        self.map.entry(node_name.clone()).or_insert(OnceCell::new()).set(value).unwrap();
        //self.get(node_name).expect("Map was not initialized").set(value).unwrap()
    }
}

impl BinomialTreeMap for StaticContainer {}



impl BinomialTreeStackImpl for StaticBinomialTreeMap {
    //type NodeNameType = NodeName2;
    type NodeNameContainerType = StaticContainer;

    fn iter(&self) -> impl DoubleEndedIterator + ExactSizeIterator<Item=&impl Deref<Target=[<<Self as BinomialTreeStackImpl>::NodeNameContainerType as BinomialTreeMapImpl>::NodeNameType]>> {
        self.stack.iter()
    }
}

impl BinomialTreeStack for StaticBinomialTreeMap {}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn test_static_binomial_tree() {
    }
}