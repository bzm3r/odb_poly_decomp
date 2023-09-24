use std::cmp::Ordering;

use crate::point::Point;

use super::edge::Edge;

pub struct Node<'a> {
    pub point: Point,
    pub in_edge: Option<&'a Edge<'a>>,
    pub out_edge: Option<&'a Edge<'a>>,
}

impl<'a> From<Point> for Node<'a> {
    fn from(point: Point) -> Self {
        Node {
            point,
            in_edge: None,
            out_edge: None,
        }
    }
}

impl<'a> PartialEq for Node<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.point == other.point
    }
}

impl<'a> PartialOrd for Node<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.point.cmp(&other.point) {
            o @ (Ordering::Less | Ordering::Greater) => Some(o),
            _ => None,
        }
    }
}

impl<'a> Node<'a> {
    pub fn x(&self) -> isize {
        self.point.x
    }

    pub fn y(&self) -> isize {
        self.point.y
    }
}

// pub struct NodeIndex(usize);
// pub type Nodes = Vec<Node>;

// impl<'a> Index<NodeIndex> for Nodes {
//     type Output = Node;
//     fn index(&self, index: NodeIndex) -> &Self::Output {
//         &self[index.0]
//     }
// }

// impl<'a> IndexMut<NodeIndex> for Nodes {
//     fn index_mut(&mut self, index: NodeIndex) -> &mut Self::Output {
//         &mut self[index.0]
//     }
// }
