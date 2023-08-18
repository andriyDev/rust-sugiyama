use std::collections::{ HashSet, hash_set::Iter };

use petgraph::stable_graph::NodeIndex;

#[derive(Debug)]
pub(super) struct Tree {
    vertices: HashSet<Vertex>,
    edges: HashSet<(NodeIndex, NodeIndex)>,
}

impl Tree {
    pub(super) fn new() -> Self {
        Self { 
            vertices: HashSet::new(), 
            edges: HashSet::new() 
        }
    }

    pub(super) fn add_vertex(&mut self, node: NodeIndex) {
        self.vertices.insert(Vertex::new(node));
    }

    /// Adds an edge to the tree.
    /// Will panic if it doesn't contain the tail and head vertices.
    pub(super) fn add_edge(&mut self, tail: NodeIndex, head: NodeIndex) {
        assert!(self.vertices.contains(&tail.into()));
        assert!(self.vertices.contains(&head.into()));

        // update tail
        let mut v_tail = self.vertices.take(&tail.into()).unwrap();
        v_tail.outgoing.insert(head);
        self.vertices.insert(v_tail);

        // update head
        let mut v_head = self.vertices.take(&head.into()).unwrap();
        v_head.incoming.insert(tail);
        self.vertices.insert(v_head);

        self.edges.insert((tail, head));
    }

    pub(super) fn contains_vertex(&self, vertex: &NodeIndex) -> bool {
        self.vertices.contains(&(*vertex).into())
    }

    pub(super) fn contains_edge(&self, tail: NodeIndex, head: NodeIndex) -> bool {
        self.edges.contains(&(tail, head))
    }

    /// Returns true if exactly one vertex is a member of the tree.
    pub(super) fn is_incident_edge(&self, tail: &NodeIndex, head: &NodeIndex) -> bool {
        self.contains_vertex(tail) ^ self.contains_vertex(head)
    }

    pub(super) fn is_leave(&self, vertex: &NodeIndex) -> bool {
        self.vertices.get(&(*vertex).into()).unwrap().is_leave()
    }

    pub(super) fn vertices(&self) -> Iter<'_, Vertex> {
        self.vertices.iter()
    }

    pub(super) fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub(super) fn vertice_count(&self) -> usize {
        self.vertices.len()
    }

    pub(super) fn neighbor_count(&self, vertex: NodeIndex) -> usize {
        self.vertices.get(&vertex.into()).unwrap().neighbor_count()
    }

    pub(super) fn leaves(&self) -> impl Iterator<Item = NodeIndex> + '_ {
        self.vertices.iter().filter(|v| v.is_leave()).map(|v| v.id()) 
    }

    pub(super) fn incoming(&self, vertex: NodeIndex) -> Neighbors<'_> {
        match self.vertices.get(&vertex.into()) {
            Some(v) => Neighbors { items: Some(v.incoming.iter()) },
            None => Neighbors { items: None },
        }
    }

    pub(super) fn outgoing(&self, vertex: NodeIndex) -> Neighbors<'_> {
        match self.vertices.get(&vertex.into()) {
            Some(v) => Neighbors { items: Some(v.outgoing.iter()) },
            None => Neighbors { items: None },
        }
    }

    pub(super) fn connected_edges(&self, vertex: NodeIndex) -> ConnectedEdges<'_> {
        match self.vertices.get(&vertex.into()) {
            Some(v) => ConnectedEdges { vertex, incoming: Some(v.incoming.iter()), outgoing: Some(v.outgoing.iter()) },
            None => ConnectedEdges { vertex, incoming: None, outgoing: None },
        }
    }

    pub(crate) fn from_edges(edges: &[(usize, usize)]) -> Self {
        let mut tree = Self::new();
        for (tail, head) in edges {
            tree.add_vertex(NodeIndex::new(*tail));
            tree.add_vertex(NodeIndex::new(*head));
            tree.add_edge(NodeIndex::new(*tail), NodeIndex::new(*head));
        }
        tree
    }
}

pub(super) struct Neighbors<'tree> {
    items: Option<Iter<'tree, NodeIndex>>
}

impl<'tree> Iterator for Neighbors<'tree> {
    type Item = NodeIndex;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.items {
            Some(i) => i.next().copied(),
            None => None,
        }
    }
}

pub(super) struct ConnectedEdges<'tree> {
    vertex: NodeIndex,
    incoming: Option<Iter<'tree, NodeIndex>>,
    outgoing: Option<Iter<'tree, NodeIndex>>,
}

impl<'tree> Iterator for ConnectedEdges<'tree> {
    type Item = (NodeIndex, NodeIndex);

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.incoming {
            Some(iter) => iter.next()
                              .map(|inc| (*inc, self.vertex))
                              .or_else(|| self.outgoing.as_mut() // if incoming is some, outgoing will also be some
                                                       .unwrap()
                                                       .next()
                                                       .map(|out| (self.vertex, *out))
            ),
            None => None
        }
    }
}



#[derive(Eq, Debug)]
pub(crate) struct Vertex {
    id: NodeIndex,
    incoming: HashSet<NodeIndex>,
    outgoing: HashSet<NodeIndex>, 
}

impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Vertex {
    fn new(id: NodeIndex) -> Self {
        Self {
            id,
            incoming: HashSet::new(),
            outgoing: HashSet::new(),
        }
    }

    pub(super) fn id(&self) -> NodeIndex {
        self.id
    }

    #[inline(always)]
    pub(super) fn neighbor_count(&self) -> usize {
        self.incoming.len() + self.outgoing.len()
    }

    fn is_leave(&self) -> bool {
        self.neighbor_count() < 2
    }
}

impl std::hash::Hash for Vertex {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl From<NodeIndex> for Vertex {
    fn from(value: NodeIndex) -> Self {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    mod tree {
        use petgraph::adj::NodeIndex;

        use crate::graphs::p1_layering::tree::Tree;

        #[test]
        fn test_leaves() {
            let tree = Tree::from_edges(&[(0, 1), (0, 5), (5, 6), (4, 6), (1, 2), (2, 3), (3, 7)]);
            let leaves = tree.leaves().collect::<Vec<_>>();
            assert_eq!(leaves.len(), 2);
            assert!(leaves.contains(&NodeIndex::from(4)));
            assert!(leaves.contains(&NodeIndex::from(7)));
        }
    }
    mod vertex {
        use std::collections::HashSet;

        use petgraph::stable_graph::NodeIndex;

        use crate::graphs::p1_layering::tree::Vertex;

        #[test]
        fn test_vertex_id_equal_same_neighbors() {
            let v1 = Vertex::new(0.into());
            let v2 = Vertex::new(0.into());
            assert_eq!(v1, v2);
        }
        
        #[test]
        fn test_vertex_id_equal_not_same_neighbors() {
            let v1 = Vertex::new(0.into());
            let mut v2 = Vertex::new(0.into());
            v2.incoming.insert(1.into());
            assert_eq!(v1, v2);
        }

        #[test]
        fn test_vertex_store_hashset_with_neighbor() {
            let mut set = HashSet::new();
            let mut v = Vertex::new(0.into());
            v.outgoing.insert(1.into());
            set.insert(v);
            let v = set.take(&NodeIndex::new(0).into());
            assert!(v.is_some());
            assert!(v.unwrap().outgoing.contains(&1.into()));
        }

        #[test]
        fn test_vertex_store_multiple_times() {
            let mut set = HashSet::new();
            let v1 = Vertex::new(0.into());
            let v2= Vertex::new(0.into());

            assert!(set.insert(v1));
            assert!(!set.insert(v2));
        }
    }
}