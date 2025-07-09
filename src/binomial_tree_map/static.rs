use std::cell::OnceCell;
use std::ops::Deref;
use hashbrown::HashMap;
use id_arena::{Arena, Id};
use binomial_tree_macro::binomial_tree_stack;
use crate::binomial_tree_map::capacity::calculate_capacity;
use crate::binomial_tree_map::{BinomTreeValueType, BinomialTreeMapImpl, BinomialTreeStackImpl};
use crate::binomial_tree_map::BinomialTreeMapNumericType;
use crate::binomial_tree_map::nodes::{NodeName2, NodeNameTrait, UpDown};

pub const MAX_TREE_SIZE: usize = 128;
const PRE_ALLOCATED_STACK: &[&[NodeName2]] = binomial_tree_stack!(128);
pub(crate) const MAX_CAPACITY: usize = calculate_capacity(MAX_TREE_SIZE);

#[derive(Debug, Default)]
pub struct StaticBinomialTreeMap {
    pub(crate) stack: &'static [&'static [NodeName2]],
}

impl StaticBinomialTreeMap {
    pub fn with_capacity(capacity: usize) -> StaticBinomialTreeMap
    {
        let stack: &'static [&'static [NodeName2]] = &PRE_ALLOCATED_STACK[..=capacity]; // TODO: This trick could probably be used in a dynamic case as well!

        StaticBinomialTreeMap {
            stack,
        }
    }
}

pub struct StaticContainer {
    arena: Arena<BinomTreeValueType>,
    pub(crate) map: HashMap<NodeName2, Id<BinomTreeValueType>>,
}

impl Default for StaticContainer {
    fn default() -> Self {
        Self {
            arena: Arena::with_capacity(MAX_CAPACITY),
            map: HashMap::<NodeName2, Id<BinomTreeValueType>>::with_capacity(MAX_CAPACITY)
        }
    }
}

impl BinomialTreeMapImpl for StaticContainer {
    type NodeNameType = NodeName2;
    type NumericType = BinomialTreeMapNumericType;
    type ValueType = BinomTreeValueType;


    fn get(&self, node_name: &Self::NodeNameType) -> Option<&Self::ValueType> {
        let id = self.map.get(node_name)?;
        Some(&self.arena[*id])
    }

    fn get_next_step(&self, node_name: &Self::NodeNameType) -> Option<&Self::ValueType> {
        let (key, _) = self.map.get_key_value(&node_name.up()).unwrap();
        self.get(&key.down())
    }

    fn set(&mut self, node_name: &Self::NodeNameType, value: Self::NumericType) {
        let id = self.arena.alloc(OnceCell::new());
        self.map.entry(node_name.clone()).or_insert(id);

        self.arena[id].set(value).unwrap();
    }
}

impl BinomialTreeStackImpl for StaticBinomialTreeMap {
    //type NodeNameType = NodeName2;
    type NodeNameContainerType = StaticContainer;

    fn iter(&self) -> impl DoubleEndedIterator + ExactSizeIterator<Item=&impl Deref<Target=[<<Self as BinomialTreeStackImpl>::NodeNameContainerType as BinomialTreeMapImpl>::NodeNameType]>> {
        self.stack.iter()
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn test_static_binomial_tree() {
    }
}