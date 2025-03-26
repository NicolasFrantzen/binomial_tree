use std::cell::{Cell, RefCell};
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::iter::{once, Rev};
//use std::path::Iter;
use std::rc::Rc;
use std::vec::IntoIter;
use itertools::sorted;

#[derive(Clone)]
pub(crate) struct Tree {
    pub root: TreeNodeType,
}

impl Tree {
    pub(crate) fn iter(&self) -> TreeIterator
    {
        let mut stack: Vec<TreeNodeType> = Vec::new();
        let mut vec: Vec<TreeNodeType> = Vec::new();
        stack.push(self.root.clone());
        vec.push(self.root.clone());
        while let Some(node) = stack.pop() {
            if let Some(up) = &node.borrow().up {
                if TreeNode::is_duplicate(&node).is_none() {
                    //dbg!("Push up {:?}", &up.borrow().name);
                    stack.push(up.clone());
                    vec.push(up.clone())
                }
            }

            if let Some(down) = &node.borrow().down {
                //dbg!("Push up {:?}", &down.borrow().name);
                stack.push(down.clone());
                vec.push(down.clone())
            }
        }

        TreeIterator{ iter: vec.into_iter().rev() }
    }
}

impl Default for Tree {
    fn default() -> Self {
        Self {
            root: Rc::new(RefCell::new(TreeNode {
                parent: None,
                up: None,
                down: None,
                price: 0.0,
                value: Cell::new(0.0),
                name: NodeName{name: vec![UpDown::Initial]},
            })),
        }
    }
}

pub(crate) type TreeNodeType = Rc<RefCell<TreeNode>>;

pub(crate) struct TreeIterator {
    iter: Rev<IntoIter<TreeNodeType>>,
}

impl Iterator for TreeIterator {
    type Item = TreeNodeType;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub(crate) enum UpDown {
    Initial,
    Up,
    Down,
}

impl From<&UpDown> for char {
    fn from(value: &UpDown) -> Self {
        let s = match value {
            UpDown::Initial => 'I',
            UpDown::Up => 'U',
            UpDown::Down => 'D',
        };
        s
    }
}

impl std::fmt::Display for UpDown {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {

        write!(f, "{}", char::try_from(self).unwrap())
    }
}

impl TryFrom<char> for UpDown {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'I' => Ok(UpDown::Initial),
            'U' => Ok(UpDown::Up),
            'D' => Ok(UpDown::Down),
            _ => Err(())
        }
    }
}

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub(crate) struct NodeName {
    pub name: Vec<UpDown>,
}

impl NodeName {
    pub(crate) fn value(&self, initial_value: f32, up_value: f32, down_value: f32) -> f32 {
        let mut value = initial_value;
        for i in self.name.iter() {
            match i {
                UpDown::Initial => {}
                UpDown::Up => {
                    value = value * up_value;
                }
                UpDown::Down => {
                    value = value * down_value;
                }
            }
        }

        value
    }

    pub(crate) fn up(&self) -> NodeName {
        NodeName{name: self.name.iter().chain(once(&UpDown::Up)).cloned().collect()}
    }

    pub(crate) fn up2(&self) -> NodeName {
        NodeName{name: sorted(self.name.iter().chain(once(&UpDown::Up)).cloned()).collect()}
    }

    pub(crate) fn down(&self) -> NodeName {
        NodeName{name: self.name.iter().chain(once(&UpDown::Down)).cloned().collect()}
    }
}

impl Display for NodeName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s: String = self.name.iter().map(char::from).collect();
        write!(f, "{}", s)
    }
}

/*impl From<Iter<'_>> for NodeName {
    fn from(value: Iter) -> Self {
        NodeName{ name: value.collect() }
    }
}*/

impl TryFrom<&str> for NodeName {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let updowns: Result<Vec<_>, _> = value.chars().map(UpDown::try_from).collect();

        Ok(NodeName{name: updowns?})
    }
}

#[derive(Debug)]
pub(crate) struct TreeNode {
    pub parent: Option<TreeNodeType>,
    pub up: Option<TreeNodeType>, // TODO: Replace RefCell with a macro that constructs a tree in one go
    pub down: Option<TreeNodeType>,
    pub price: f32,
    pub value: Cell<f32>,
    pub name: NodeName, //Vec<UpDown>,
}

impl TreeNode {
    pub(crate) fn is_duplicate(node: &TreeNodeType) -> Option<TreeNodeType> {
        if let Some(parent) = &node.borrow().parent {
            if let Some(parent_up) = &parent.borrow().up {
                if !Rc::ptr_eq(parent_up, node) {
                    return parent_up.borrow().down.clone();
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use std::iter::once;
    use super::*;

    #[test]
    fn test_tree_updown_from() {
        assert_eq!(UpDown::try_from('I').unwrap(), UpDown::Initial);
        assert_eq!(UpDown::try_from('U').unwrap(), UpDown::Up);
        assert_eq!(UpDown::try_from('D').unwrap(), UpDown::Down);


        assert_eq!(NodeName::try_from("IUD").unwrap(),
                   NodeName{name: vec![UpDown::Initial, UpDown::Up, UpDown::Down]});
        assert_eq!(NodeName::try_from("IUDD").unwrap(),
                   NodeName{name: vec![UpDown::Initial, UpDown::Up, UpDown::Down, UpDown::Down]});
    }

    fn add_branches(node: TreeNodeType, level: i32, max_level: i32) {
        if level > max_level {
            return;
        }

        let name = node.borrow().name.name.clone();

        if let Some(up) = TreeNode::is_duplicate(&node) {
            node.borrow_mut().up = Some(up);
        }
        else {
            node.borrow_mut().up = Some(Rc::new(RefCell::new(TreeNode {
                parent: Some(node.clone()),
                up: None,
                down: None,
                price: level as f32,
                value: Cell::new(0.2),
                name: NodeName{name: name.iter().clone().chain(once(&UpDown::Up)).cloned().collect()}
            })));
        }

        node.borrow_mut().down = Some(Rc::new(RefCell::new(TreeNode {
            parent: Some(node.clone()),
            up: None,
            down: None,
            price: (level as f32) + 0.5,
            value: Cell::new(0.2),
            name: NodeName{name: name.iter().chain(once(&UpDown::Down)).cloned().collect()}
        })));

        add_branches(node.borrow().up.clone().unwrap(), level + 1, max_level);
        add_branches(node.borrow().down.clone().unwrap(), level + 1, max_level);
    }

    #[test]
    fn test_node_name_up_down() {
        let name = NodeName { name: vec![] };

        assert_eq!(name.up(), NodeName { name: vec![UpDown::Up] });
        assert_eq!(name.down(), NodeName { name: vec![UpDown::Down] });

        let up_name = name.up();
        assert_eq!(up_name.up(), NodeName { name: vec![UpDown::Up, UpDown::Up] });
        assert_eq!(up_name.down(), NodeName { name: vec![UpDown::Up, UpDown::Down] });

        let down_name = name.down();
        assert_eq!(down_name.up(), NodeName { name: vec![UpDown::Down, UpDown::Up] });
        assert_eq!(down_name.up2(), NodeName { name: vec![UpDown::Up, UpDown::Down] });
    }

    #[test]
    fn test_tree_zero_level() {
        let tree = Tree::default();
        add_branches(tree.root.clone(), 1, 0);
        let mut iter = tree.iter();

        assert_eq!(iter.size_hint(), (1, Some(1)));
        assert_eq!(iter.next().unwrap().borrow().price, 0.0);
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_tree_one_level() {
        let tree = Tree::default();
        add_branches(tree.root.clone(), 1, 1);
        let mut iter = tree.iter();

        assert_eq!(iter.size_hint(), (3, Some(3)));
        assert_eq!(iter.next().unwrap().borrow().name, "ID".try_into().unwrap());
        assert_eq!(iter.next().unwrap().borrow().name, "IU".try_into().unwrap());
        assert_eq!(iter.next().unwrap().borrow().name, "I".try_into().unwrap());
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_tree_two_level() {
        let tree = Tree::default();
        add_branches(tree.root.clone(), 1, 2);
        let mut iter = tree.iter();

        assert_eq!(iter.size_hint(), (6, Some(6)));
        assert_eq!(iter.next().unwrap().borrow().name, "IUD".try_into().unwrap());
        assert_eq!(iter.next().unwrap().borrow().name, "IUU".try_into().unwrap());
        assert_eq!(iter.next().unwrap().borrow().name, "IDD".try_into().unwrap());
        assert_eq!(iter.next().unwrap().borrow().name, "ID".try_into().unwrap());
        assert_eq!(iter.next().unwrap().borrow().name, "IU".try_into().unwrap());
        assert_eq!(iter.next().unwrap().borrow().name, "I".try_into().unwrap());
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_tree_three_level() {
        let tree = Tree::default();
        add_branches(tree.root.clone(), 1, 3);
        let mut iter = tree.iter();

        assert_eq!(iter.size_hint(), (10, Some(10)));
        assert_eq!(iter.next().unwrap().borrow().name, "IUUD".try_into().unwrap());
        assert_eq!(iter.next().unwrap().borrow().name, "IUUU".try_into().unwrap());
        assert_eq!(iter.next().unwrap().borrow().name, "IUDD".try_into().unwrap());
        assert_eq!(iter.next().unwrap().borrow().name, "IUD".try_into().unwrap());
        assert_eq!(iter.next().unwrap().borrow().name, "IUU".try_into().unwrap());
        assert_eq!(iter.next().unwrap().borrow().name, "IDDD".try_into().unwrap());
        assert_eq!(iter.next().unwrap().borrow().name, "IDD".try_into().unwrap());
        assert_eq!(iter.next().unwrap().borrow().name, "ID".try_into().unwrap());
        assert_eq!(iter.next().unwrap().borrow().name, "IU".try_into().unwrap());
        assert_eq!(iter.next().unwrap().borrow().name, "I".try_into().unwrap());
        assert!(iter.next().is_none());
    }

    fn binom(n: u32, k: u32) -> u32 {
        let mut res = 1;
        for i in 0..k {
            res = (res * (n - i)) /
                (i + 1);
        }
        res
    }

    #[test]
    fn test_tree_many_level() {
        for i in 1..10u32  {
            let tree = Tree::default();
            add_branches(tree.root.clone(), 1, i as i32);
            let iter = tree.iter();

            assert_eq!(iter.iter.len() as u32, binom(i + 2, 2));
        }
    }
}