use std::sync::OnceLock;
use hashbrown::HashMap;
use binomial_tree_macro::binomial_tree_stack;
use crate::binomial_tree_map::calculate_capacity;
use crate::nodes::{NodeName, UpDown};

const PRE_ALLOCATED_STACK: &'static [&'static [&'static [UpDown]]] = binomial_tree_stack!(128);

#[derive(Debug)]
pub /*(crate)*/ struct StaticBinomialTree {
    pub /*(crate)*/ map: HashMap<NodeName, crate::binomial_tree_map::BinomialTreeMapValue<crate::binomial_tree_map::BinomialTreeMapNumericType>>,
    pub /*(crate)*/ stack: &'static [&'static [&'static [UpDown]]],
}

// TODO: Impl the trait instead
impl StaticBinomialTree {
    pub(crate) fn new<const N: usize>() -> StaticBinomialTree
    {
        let mut map = HashMap::<NodeName, OnceLock<f32>>::with_capacity(calculate_capacity(N));
        let stack: &'static [&'static [&'static [UpDown]]] = &PRE_ALLOCATED_STACK[..N]; // TODO: This trick could probably be used in a dynamic case as well!

        for stack_level in stack.iter() {
            for node_name in stack_level.iter() {
                unsafe {
                    let _ = map.insert_unique_unchecked(NodeName { name: node_name.to_vec() }, OnceLock::new());
                }
            }
        }

        StaticBinomialTree {
            map,
            stack,
        }
    }

    pub(crate) fn get(&self, node_name: &NodeName) -> Option<&crate::binomial_tree_map::BinomialTreeMapValue<f32>> {
        self.map.get(node_name)
    }

    pub(crate) fn set(&self, node_name: &NodeName, value: f32) {
        self.get(node_name).expect("Map was not initialized").set(value).unwrap()
    }
}
