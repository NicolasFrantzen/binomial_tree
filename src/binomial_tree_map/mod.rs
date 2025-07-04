use nodes::NodeNameTrait;

use std::cell::OnceCell;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Deref;

mod dynamic;
pub mod r#static;
mod capacity;
pub(crate) mod nodes;

pub(crate) type BinomialTreeMapNumericType = f32;
pub(crate) type BinomialTreeMapValue<T> = OnceCell<T>;
pub(crate) type BinomTreeValueType = BinomialTreeMapValue<BinomialTreeMapNumericType>;

pub(crate) trait BinomialTreeMapImpl {
    type NodeNameType: NodeNameTrait + Debug + Hash + Default;
    type NumericType: From<f32> + Into<f32>;
    type ValueType: GetValue;

    fn get(&self, node_name: &Self::NodeNameType) -> Option<&Self::ValueType>;
    fn get_next_step(&self, node_name: &Self::NodeNameType) -> Option<&Self::ValueType>;
    fn set(&mut self, node_name: &Self::NodeNameType, value: Self::NumericType);
}

pub(crate) trait BinomialTreeStackImpl {
    type NodeNameContainerType: BinomialTreeMapImpl + Default;

    fn iter(&self) -> impl DoubleEndedIterator + ExactSizeIterator<Item=&impl Deref<Target=[<<Self as BinomialTreeStackImpl>::NodeNameContainerType as BinomialTreeMapImpl>::NodeNameType]>>;
}

pub(crate) trait GetValue {
    fn get(&self) -> &f32;
}

impl GetValue for BinomTreeValueType {
    fn get(&self) -> &f32 {
        let value =  self.get();
        value.expect("The tree should be evaluated backwards")
    }
}

