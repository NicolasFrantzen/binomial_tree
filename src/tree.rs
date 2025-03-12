use std::cell::{Cell, RefCell};
use std::iter::Rev;
use std::rc::Rc;
use std::vec::IntoIter;

pub(crate) struct Tree {
    pub root: Rc<RefCell<TreeNode>>,
}

impl Tree {
    pub fn new() -> Tree {
        Tree {
            root: Rc::new(RefCell::new(TreeNode {
                parent: None,
                up: None,
                down: None,
                price: 0.0,
                value: Cell::new(0.0),
            })),
        }
    }
}

fn is_duplicate(node: &Rc<RefCell<TreeNode>>) -> Option<Rc<RefCell<TreeNode>>> {
    if let Some(parent) = &node.borrow().parent {
        if let Some(parent_up) = &parent.borrow().up {
            if !Rc::ptr_eq(parent_up, &node) {
                return parent_up.borrow().down.clone();
            }
        }
    }
    None
}

impl IntoIterator for Tree {
    type Item = Rc<RefCell<TreeNode>>;
    type IntoIter = Rev<IntoIter<Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        let mut stack: Vec<Self::Item> = Vec::new();
        let mut vec: Vec<Self::Item> = Vec::new();
        stack.push(self.root.clone());
        vec.push(self.root.clone());
        while let Some(node) = stack.pop() {
            if let Some(up) = &node.borrow().up {
                if is_duplicate(&node).is_none() {
                    println!("Push up");
                    stack.push(up.clone());
                    vec.push(up.clone())
                }
            }

            if let Some(down) = &node.borrow().down {
                println!("Push down");
                stack.push(down.clone());
                vec.push(down.clone())
            }
        }

        vec.into_iter().rev()
    }
}

#[derive(PartialEq)]
pub(crate) struct TreeNode {
    pub parent: Option<Rc<RefCell<TreeNode>>>,
    pub up: Option<Rc<RefCell<TreeNode>>>, // TODO: Replace RefCell with a macro that constructs a tree in one go
    pub down: Option<Rc<RefCell<TreeNode>>>,
    pub price: f32,
    pub value: Cell<f32>,
}

impl TreeNode {
    pub fn call_european_value(&self, strike: f32) -> f32 { // TODO: Use traits?
        (self.price - strike).max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn add_branches(node: Rc<RefCell<TreeNode>>, level: i32, max_level: i32) {
        if level > max_level {
            return;
        }

        if let Some(up) = is_duplicate(&node) {
            node.borrow_mut().up = Some(up);
        }
        else {
            node.borrow_mut().up = Some(Rc::new(RefCell::new(TreeNode {
                parent: Some(node.clone()),
                up: None,
                down: None,
                price: level as f32,
                value: Cell::new(0.2),
            })));
        }

        node.borrow_mut().down = Some(Rc::new(RefCell::new(TreeNode {
            parent: Some(node.clone()),
            up: None,
            down: None,
            price: (level as f32) + 0.5,
            value: Cell::new(0.2),
        })));

        add_branches(node.borrow().up.clone().unwrap(), level + 1, max_level);
        add_branches(node.borrow().down.clone().unwrap(), level + 1, max_level);
    }



    #[test]
    fn test_tree_zero_level() {
        let tree = Tree::new();
        add_branches(tree.root.clone(), 1, 0);
        let mut iter = tree.into_iter();

        assert_eq!(iter.size_hint(), (1, Some(1)));
        assert_eq!(iter.next().unwrap().borrow().price, 0.0);
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_tree_one_level() {
        let tree = Tree::new();
        add_branches(tree.root.clone(), 1, 1);
        let mut iter = tree.into_iter();

        assert_eq!(iter.size_hint(), (3, Some(3)));
        assert_eq!(iter.next().unwrap().borrow().price, 1.5);
        assert_eq!(iter.next().unwrap().borrow().price, 1.0);
        assert_eq!(iter.next().unwrap().borrow().price, 0.0);
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_tree_two_level() {
        let tree = Tree::new();
        add_branches(tree.root.clone(), 1, 2);
        let mut iter = tree.into_iter();

        //assert_eq!(iter.size_hint(), (6, Some(6)));
        assert_eq!(iter.next().unwrap().borrow().price, 2.5);
        assert_eq!(iter.next().unwrap().borrow().price, 2.0);
        assert_eq!(iter.next().unwrap().borrow().price, 2.5); // There is too many downs
        //assert_eq!(iter.next().unwrap().borrow().price, 2.5);
        assert_eq!(iter.next().unwrap().borrow().price, 1.5);
        assert_eq!(iter.next().unwrap().borrow().price, 1.0);
        assert_eq!(iter.next().unwrap().borrow().price, 0.0);
        assert!(iter.next().is_none());
    }
}