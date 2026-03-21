//! Directed multigraph with labeled nodes and edges, pin indices, and 2D layout.

use std::collections::{HashMap, HashSet};

use crate::error::GraphError;
use crate::id::{SemanticEdgeId, SemanticNodeId};
use crate::layout::Layout2d;

/// Payload and editor-facing layout for one semantic node.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(
        serialize = "N: serde::Serialize",
        deserialize = "N: serde::de::Deserialize<'de>"
    ))
)]
pub struct SemanticNode<N> {
    pub payload: N,
    pub layout: Layout2d,
    /// When `true`, the UI node is collapsed (maps to Snarl `open == false`).
    pub collapsed: bool,
}

impl<N> SemanticNode<N> {
    pub fn new(payload: N, layout: Layout2d) -> Self {
        Self {
            payload,
            layout,
            collapsed: false,
        }
    }
}

/// One directed edge: output pin on `from` → input pin on `to`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(
        serialize = "E: serde::Serialize",
        deserialize = "E: serde::de::Deserialize<'de>"
    ))
)]
pub struct SemanticEdge<E> {
    pub from: SemanticNodeId,
    pub out_port: usize,
    pub to: SemanticNodeId,
    pub in_port: usize,
    pub payload: E,
}

/// Domain graph: evaluation, serialization, and tools operate here; Snarl mirrors layout + wires for UI.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(
        serialize = "N: serde::Serialize, E: serde::Serialize",
        deserialize = "N: serde::de::Deserialize<'de>, E: serde::de::Deserialize<'de>"
    ))
)]
pub struct SemanticGraph<N, E> {
    nodes: HashMap<SemanticNodeId, SemanticNode<N>>,
    edges: HashMap<SemanticEdgeId, SemanticEdge<E>>,
    next_node_id: u64,
    next_edge_id: u64,
}

impl<N, E> Default for SemanticGraph<N, E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<N, E> SemanticGraph<N, E> {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            next_node_id: 1,
            next_edge_id: 1,
        }
    }

    fn alloc_node_id(&mut self) -> SemanticNodeId {
        let id = SemanticNodeId(self.next_node_id);
        self.next_node_id = self.next_node_id.saturating_add(1);
        id
    }

    fn alloc_edge_id(&mut self) -> SemanticEdgeId {
        let id = SemanticEdgeId(self.next_edge_id);
        self.next_edge_id = self.next_edge_id.saturating_add(1);
        id
    }

    /// Insert a node; returns its stable id.
    pub fn insert_node(&mut self, node: SemanticNode<N>) -> SemanticNodeId {
        let id = self.alloc_node_id();
        self.nodes.insert(id, node);
        id
    }

    pub fn node(&self, id: SemanticNodeId) -> Option<&SemanticNode<N>> {
        self.nodes.get(&id)
    }

    pub fn node_mut(&mut self, id: SemanticNodeId) -> Option<&mut SemanticNode<N>> {
        self.nodes.get_mut(&id)
    }

    pub fn nodes_iter(&self) -> impl Iterator<Item = (SemanticNodeId, &SemanticNode<N>)> {
        self.nodes.iter().map(|(&k, v)| (k, v))
    }

    pub fn nodes_iter_mut(&mut self) -> impl Iterator<Item = (SemanticNodeId, &mut SemanticNode<N>)> {
        self.nodes.iter_mut().map(|(&k, v)| (k, v))
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Remove a node and all incident edges.
    pub fn remove_node(&mut self, id: SemanticNodeId) -> Option<SemanticNode<N>> {
        let n = self.nodes.remove(&id)?;
        self.edges.retain(|_, e| e.from != id && e.to != id);
        Some(n)
    }

    pub fn edge(&self, id: SemanticEdgeId) -> Option<&SemanticEdge<E>> {
        self.edges.get(&id)
    }

    pub fn edges_iter(&self) -> impl Iterator<Item = (SemanticEdgeId, &SemanticEdge<E>)> {
        self.edges.iter().map(|(&k, v)| (k, v))
    }

    /// Drop every edge (keeps nodes). Used when re-deriving topology from the UI graph.
    pub fn clear_edges(&mut self) {
        self.edges.clear();
    }

    /// Connect `from.out_port` → `to.in_port`.
    pub fn connect(
        &mut self,
        from: SemanticNodeId,
        out_port: usize,
        to: SemanticNodeId,
        in_port: usize,
        payload: E,
    ) -> Result<SemanticEdgeId, GraphError> {
        if from == to {
            return Err(GraphError::SelfConnection { node: from });
        }
        if !self.nodes.contains_key(&from) {
            return Err(GraphError::UnknownNode(from));
        }
        if !self.nodes.contains_key(&to) {
            return Err(GraphError::UnknownNode(to));
        }
        for e in self.edges.values() {
            if e.from == from
                && e.out_port == out_port
                && e.to == to
                && e.in_port == in_port
            {
                return Err(GraphError::DuplicateEdge {
                    from,
                    out_port,
                    to,
                    in_port,
                });
            }
        }
        let id = self.alloc_edge_id();
        self.edges.insert(
            id,
            SemanticEdge {
                from,
                out_port,
                to,
                in_port,
                payload,
            },
        );
        Ok(id)
    }

    pub fn disconnect_edge(&mut self, id: SemanticEdgeId) -> Option<SemanticEdge<E>> {
        self.edges.remove(&id)
    }

    /// Remove an edge matching the pin pair, if present.
    pub fn disconnect_pins(
        &mut self,
        from: SemanticNodeId,
        out_port: usize,
        to: SemanticNodeId,
        in_port: usize,
    ) -> Option<SemanticEdgeId> {
        let found = self.edges.iter().find_map(|(&eid, e)| {
            if e.from == from && e.out_port == out_port && e.to == to && e.in_port == in_port {
                Some(eid)
            } else {
                None
            }
        });
        found.and_then(|eid| self.edges.remove(&eid).map(|_| eid))
    }

    /// Neighbors reachable along outgoing edges from `node`.
    pub fn outgoing_neighbors(&self, node: SemanticNodeId) -> Vec<SemanticNodeId> {
        let mut out = Vec::new();
        let mut seen: HashSet<SemanticNodeId> = HashSet::new();
        for e in self.edges.values() {
            if e.from == node && seen.insert(e.to) {
                out.push(e.to);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_connect_remove() {
        let mut g = SemanticGraph::<&str, ()>::new();
        let a = g.insert_node(SemanticNode::new("a", Layout2d::new(0.0, 0.0)));
        let b = g.insert_node(SemanticNode::new("b", Layout2d::new(100.0, 0.0)));
        let e = g.connect(a, 0, b, 0, ()).unwrap();
        assert_eq!(g.edge_count(), 1);
        assert!(g.disconnect_edge(e).is_some());
        assert_eq!(g.edge_count(), 0);
        g.remove_node(a).unwrap();
        assert_eq!(g.node_count(), 1);
    }
}
