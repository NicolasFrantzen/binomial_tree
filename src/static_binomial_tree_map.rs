use std::cell::OnceCell;
use std::sync::OnceLock;
use hashbrown::HashMap;
use binomial_tree_macro::binomial_tree_stack;
use crate::binomial_tree_map::{calculate_capacity, BinomialTree};
use crate::binomial_tree_map::BinomialTreeMapValue;
use crate::binomial_tree_map::BinomialTreeMapNumericType;
use crate::nodes::{NodeName, NodeName2, UpDown};

const PRE_ALLOCATED_STACK: &'static [&'static [&'static [UpDown]]] = binomial_tree_stack!(128);

#[derive(Debug)]
pub struct StaticBinomialTreeMap {
    //pub(crate) map: HashMap<NodeName, OnceCell<BinomialTreeMapNumericType>>,
    pub(crate) map: HashMap<NodeName2, OnceCell<BinomialTreeMapNumericType>>,
    pub(crate) stack: &'static [&'static [&'static [UpDown]]],
}

// TODO: Impl the trait instead
impl StaticBinomialTreeMap {
    pub fn new<const N: usize>() -> StaticBinomialTreeMap
    {
        let mut map = HashMap::<NodeName2, OnceCell<f32>>::with_capacity(calculate_capacity(N));
        let stack: &'static [&'static [&'static [UpDown]]] = &PRE_ALLOCATED_STACK[..N]; // TODO: This trick could probably be used in a dynamic case as well!

        //println!("{:?}", stack);

        for stack_level in stack.iter() {
            for node_name in stack_level.iter() {
                unsafe {
                    let _ = map.insert_unique_unchecked(NodeName2::new(node_name), OnceCell::new());
                }
            }
        }

        //println!("{:?}", map);

        StaticBinomialTreeMap {
            map,
            stack,
        }
    }
}

impl BinomialTree for StaticBinomialTreeMap {
    type NodeNameType = NodeName2;
    type NodeNameContainerType = &'static [&'static [UpDown]];
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
