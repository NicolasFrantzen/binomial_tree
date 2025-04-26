use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::iter::once;

pub /*(crate)*/ static ALL_UPDOWNS: [UpDown; 2] = [UpDown::Up, UpDown::Down];
pub /*(crate)*/ static INITIAL_NODE: NodeName = NodeName{ name: vec![] };

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub /*(crate)*/ enum UpDown {
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


#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Hash, Clone)]
pub /*(crate)*/ struct NodeName {
    pub name: Vec<UpDown>,
}

impl NodeName {
    pub(crate) fn value(&self, initial_value: f32, up_value: f32, down_value: f32) -> f32
    {
        let mut value = initial_value;
        for i in self.iter() {
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
        // NOTE: Prepending is equivalent with sorting if downs are appended
        NodeName{ name: once(UpDown::Up).chain(self.name.iter().cloned()).collect() }
    }

    pub(crate) fn down(&self) -> NodeName {
        // NOTE: Appending is equivalent with sorting if ups are prepended
        NodeName{name: self.name.iter().chain(once(&UpDown::Down)).cloned().collect()}
    }

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

pub(crate) struct NodeName2 {
    pub(crate) name: &'static [UpDown]
}

#[cfg(test)]
mod tests {
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