use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::iter::once;

pub /*(crate)*/ static ALL_UPDOWNS: [UpDown; 2] = [UpDown::Up, UpDown::Down];
//pub /*(crate)*/ static INITIAL_NODE: NodeName = NodeName{ name: vec![] };

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
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

pub(crate) trait NodeNameTrait {
    type NameType;
    fn new(name: Self::NameType) -> Self;
    fn up(&self) -> Self;
    fn down(&self) -> Self;

    fn value(&self, initial_value: f32, up_probability: f32, down_probability: f32) -> f32;
}

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Hash, Clone, Default)]
pub(crate) struct NodeName {
    name: Vec<UpDown>,
}

impl NodeNameTrait for NodeName {
    type NameType = Vec<UpDown>;

    fn new(name: Self::NameType) -> Self {
        Self { name: name }
    }

    fn up(&self) -> Self {
        // NOTE: Prepending is equivalent with sorting if downs are appended
        NodeName{ name: once(UpDown::Up).chain(self.name.iter().cloned()).collect() }
    }

    fn down(&self) -> Self {
        // NOTE: Appending is equivalent with sorting if ups are prepended
        NodeName{name: self.name.iter().chain(once(&UpDown::Down)).cloned().collect()}
    }

    fn value(&self, initial_value: f32, up_probability: f32, down_probability: f32) -> f32
    {
        let mut value = initial_value;

        for i in self.iter() {
            match i {
                UpDown::Initial => {}
                UpDown::Up => {
                    value *= up_probability;
                }
                UpDown::Down => {
                    value *= down_probability;
                }
            }
        }

        value
    }
}

impl NodeName {
    fn iter(&self) -> impl Iterator<Item = &UpDown> {
        self.name.iter()
    }
}

impl Display for NodeName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s: String = self.name.iter().map(char::from).collect();
        write!(f, "{}", s)
    }
}

impl TryFrom<&str> for NodeName {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let updowns: Result<Vec<_>, _> = value.chars().map(UpDown::try_from).collect();

        Ok(NodeName{name: updowns?})
    }
}

impl From<NodeName2> for NodeName {
    fn from(value: NodeName2) -> Self {
        NodeName{ name: value.name.into() }
    }
}

impl From<&[UpDown]> for NodeName {
    fn from(value: &[UpDown]) -> Self {
        NodeName { name: value.into() }
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct NodeName2 {
    pub(crate) name: &'static [UpDown],
    pub(crate) direction: Option<UpDown>,
}

impl NodeNameTrait for NodeName2 {
    type NameType = &'static [UpDown];

    fn new(name: Self::NameType) -> Self {
        Self { name, direction: None }
    }

    fn up(&self) -> Self {
        Self { name: self.name, direction: Some(UpDown::Up) }
    }

    fn down(&self) -> Self {
        Self { name: self.name, direction: Some(UpDown::Down) }
    }

    fn value(&self, initial_value: f32, up_value: f32, down_value: f32) -> f32
    {
        let mut value = initial_value;

        for i in self.iter() {
            match i {
                UpDown::Initial => {}
                UpDown::Up => {
                    value *= up_value;
                }
                UpDown::Down => {
                    value *= down_value;
                }
            }
        }

        value
    }
}

impl NodeName2 {
    pub(crate) fn iter(&self) -> impl Iterator<Item = &UpDown> {
        self.name.iter().chain(std::iter::from_fn(|| self.direction.as_ref()).take(1))
    }
}

impl Hash for NodeName2 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if let Some(direction) = self.direction {
            if direction == UpDown::Up {
                direction.hash(state);
            }
        }
        for i in self.name.iter() {
            i.hash(state)
        }
        if let Some(direction) = self.direction {
            if direction == UpDown::Down {
                direction.hash(state);
            }
        }
    }
}

impl PartialEq for NodeName2 {
    fn eq(&self, other: &Self) -> bool {
        if self.direction == other.direction {
            return self.name == other.name
        }
        if let Some(direction) = other.direction {
            if self.name.len() == 0usize {
                return other.name.len() == 0usize && self.direction == other.direction
            }
            match direction {
                UpDown::Initial => {}
                UpDown::Up => {
                    return &self.name[1..] == other.name && self.name[0] == direction;
                }
                UpDown::Down => {
                    return &self.name[..self.name.len() - 1] == other.name && self.name[self.name.len() - 1] == direction;
                }
            }
        }
        if let Some(direction) = self.direction {
            if other.name.len() == 0usize {
                return self.name.len() == 0usize && self.direction == other.direction
            }
            match direction {
                UpDown::Initial => {}
                UpDown::Up => {
                    return &other.name[1..] == self.name && other.name[0] == direction;
                }
                UpDown::Down => {
                    return &other.name[..other.name.len() - 1] == self.name && other.name[other.name.len() - 1] == direction;
                }
            }
        }

        false
    }
}

impl Eq for NodeName2 {
}

/*impl NodeNameTrait for &'static [UpDown] {
    type NameType = <NodeName2 as NodeNameTrait>::NameType;

    fn new(name: Self::NameType) -> Self {
        name
    }

    fn up(&self) -> Self {
        todo!()
    }

    fn down(&self) -> Self {
        todo!()
    }

    fn value(&self, initial_value: f32, up_probability: f32, down_probability: f32) -> f32 {
        todo!()
    }
}*/


#[cfg(test)]
mod tests {
    use itertools::assert_equal;
    use super::*;

    #[test]
    fn test_nodename2_hash() {
        let mut hashmap = hashbrown::HashMap::new();
        hashmap.insert(NodeName2 { name: &[UpDown::Up, UpDown::Down], direction: None }, 1234);

        let first = NodeName2 { name: &[UpDown::Up, UpDown::Down], direction: None };
        let second = NodeName2 { name: &[UpDown::Up], direction: Some(UpDown::Down) };
        assert!(first.eq(&second));

        assert!(hashmap.get(&first).is_some());
        assert!(hashmap.get(&second).is_some());

        let third = NodeName2 { name: &[UpDown::Up], direction: None }.down();
        assert!(hashmap.get(&third).is_some());

        // Check that we can get the equivalent key back
        assert_eq!(hashmap.get_key_value(&second), Some((&NodeName2 { name: &[UpDown::Up, UpDown::Down], direction: None }, &1234)));

        // Check some special cases
        let mut hashmap = hashbrown::HashMap::new();
        hashmap.insert(NodeName2 { name: &[UpDown::Down], direction: None }, 1234);
        assert!(hashmap.get(&NodeName2::default().down()).is_some());

        assert_eq!(NodeName2 { name: &[UpDown::Down], direction: None }, NodeName2::default().down());
        assert_ne!(NodeName2::default(), NodeName2::default().down());
    }

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

    #[test]
    fn test_node_name_up_down() {
        let name = NodeName { name: vec![] };

        assert_eq!(name.up(), NodeName { name: vec![UpDown::Up] });
        assert_eq!(name.down(), NodeName { name: vec![UpDown::Down] });

        let up_name = name.up();
        assert_eq!(up_name.up(), NodeName { name: vec![UpDown::Up, UpDown::Up] });
        assert_eq!(up_name.down(), NodeName { name: vec![UpDown::Up, UpDown::Down] });

        let down_name = name.down();
        assert_eq!(down_name.up(), NodeName { name: vec![UpDown::Up, UpDown::Down] });
        assert_eq!(down_name.down(), NodeName { name: vec![UpDown::Down, UpDown::Down] });
    }
}