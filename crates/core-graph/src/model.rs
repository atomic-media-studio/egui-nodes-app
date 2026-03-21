//! Graph, nodes, links — headless model.

use std::collections::{HashMap, HashSet};

use crate::error::GraphError;
use crate::ids::{LinkId, NodeId, PinId};
use crate::layout::Layout2d;

/// Logical pin handle — alias of [`PinId`] for APIs that prefer the word “pin”.
pub type Pin = PinId;

/// Whether a pin is an input or output port on a node.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PinKind {
    Input,
    Output,
}

/// One node: domain `data`, layout, and stable [`PinId`] lists for inputs / outputs.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(
        serialize = "N: serde::Serialize",
        deserialize = "N: serde::de::Deserialize<'de>"
    ))
)]
pub struct Node<N> {
    pub id: NodeId,
    pub data: N,
    pub layout: Layout2d,
    /// UI collapse (e.g. open == !collapsed in a node editor).
    pub collapsed: bool,
    pub inputs: Vec<PinId>,
    pub outputs: Vec<PinId>,
}

/// Directed link from an **output** pin to an **input** pin.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(
        serialize = "E: serde::Serialize",
        deserialize = "E: serde::de::Deserialize<'de>"
    ))
)]
pub struct Link<E> {
    pub id: LinkId,
    pub from: PinId,
    pub to: PinId,
    pub data: E,
}

/// Foundational graph — portable, testable, no UI.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(
        serialize = "N: serde::Serialize, E: serde::Serialize",
        deserialize = "N: serde::de::Deserialize<'de>, E: serde::de::Deserialize<'de>"
    ))
)]
pub struct Graph<N, E> {
    /// All nodes (order not significant; use [`Graph::node`](Graph::node) by id).
    pub nodes: Vec<Node<N>>,
    pub links: Vec<Link<E>>,
    node_index: HashMap<NodeId, usize>,
    pin_kind: HashMap<PinId, (NodeId, usize, PinKind)>,
    next_node: u64,
    next_pin: u64,
    next_link: u64,
}

impl<N, E> Default for Graph<N, E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<N, E> Graph<N, E> {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            links: Vec::new(),
            node_index: HashMap::new(),
            pin_kind: HashMap::new(),
            next_node: 1,
            next_pin: 1,
            next_link: 1,
        }
    }

    fn alloc_node_id(&mut self) -> NodeId {
        let id = NodeId(self.next_node);
        self.next_node = self.next_node.saturating_add(1);
        id
    }

    fn alloc_pin_id(&mut self) -> PinId {
        let id = PinId(self.next_pin);
        self.next_pin = self.next_pin.saturating_add(1);
        id
    }

    fn alloc_link_id(&mut self) -> LinkId {
        let id = LinkId(self.next_link);
        self.next_link = self.next_link.saturating_add(1);
        id
    }

    /// Add a node with `inputs` input pins and `outputs` output pins.
    pub fn add_node(
        &mut self,
        data: N,
        layout: Layout2d,
        inputs: usize,
        outputs: usize,
    ) -> NodeId {
        let id = self.alloc_node_id();
        let mut in_pins = Vec::with_capacity(inputs);
        let mut out_pins = Vec::with_capacity(outputs);
        for i in 0..inputs {
            let p = self.alloc_pin_id();
            in_pins.push(p);
            self.pin_kind.insert(p, (id, i, PinKind::Input));
        }
        for i in 0..outputs {
            let p = self.alloc_pin_id();
            out_pins.push(p);
            self.pin_kind.insert(p, (id, i, PinKind::Output));
        }
        let idx = self.nodes.len();
        self.nodes.push(Node {
            id,
            data,
            layout,
            collapsed: false,
            inputs: in_pins,
            outputs: out_pins,
        });
        self.node_index.insert(id, idx);
        id
    }

    pub fn node(&self, id: NodeId) -> Option<&Node<N>> {
        self.node_index.get(&id).map(|&i| &self.nodes[i])
    }

    pub fn node_mut(&mut self, id: NodeId) -> Option<&mut Node<N>> {
        self.node_index.get(&id).copied().map(|i| &mut self.nodes[i])
    }

    pub fn nodes_iter(&self) -> impl Iterator<Item = &Node<N>> {
        self.nodes.iter()
    }

    pub fn pin_port(&self, pin: PinId) -> Option<(NodeId, usize, bool)> {
        self.pin_kind
            .get(&pin)
            .map(|&(n, i, k)| (n, i, matches!(k, PinKind::Output)))
    }

    /// `from` must be an output pin; `to` must be an input pin.
    pub fn connect(&mut self, from: PinId, to: PinId, data: E) -> Result<LinkId, GraphError> {
        if from == to {
            return Err(GraphError::SelfLoop);
        }
        self.pin_port(from)
            .ok_or(GraphError::UnknownPin(from))
            .and_then(|(_, _, is_out)| {
                if !is_out {
                    Err(GraphError::NotOutputPin(from))
                } else {
                    Ok(())
                }
            })?;
        self.pin_port(to)
            .ok_or(GraphError::UnknownPin(to))
            .and_then(|(_, _, is_out)| {
                if is_out {
                    Err(GraphError::NotInputPin(to))
                } else {
                    Ok(())
                }
            })?;

        for l in &self.links {
            if l.from == from && l.to == to {
                return Err(GraphError::DuplicateLink { from, to });
            }
        }

        let id = self.alloc_link_id();
        self.links.push(Link {
            id,
            from,
            to,
            data,
        });
        Ok(id)
    }

    pub fn disconnect_link(&mut self, id: LinkId) -> Option<Link<E>> {
        let pos = self.links.iter().position(|l| l.id == id)?;
        Some(self.links.remove(pos))
    }

    /// Remove a node and all links touching its pins.
    pub fn remove_node(&mut self, id: NodeId) -> Option<Node<N>> {
        let idx = self.node_index.remove(&id)?;
        let node = self.nodes.remove(idx);
        let dead: HashSet<PinId> = node.inputs.iter().chain(node.outputs.iter()).copied().collect();
        for p in &dead {
            self.pin_kind.remove(p);
        }
        self.links
            .retain(|l| !dead.contains(&l.from) && !dead.contains(&l.to));
        self.node_index.clear();
        for (i, n) in self.nodes.iter().enumerate() {
            self.node_index.insert(n.id, i);
        }
        Some(node)
    }
}

impl<N, E> Graph<N, E> {
    /// Remove links whose endpoint pins are missing (internal consistency).
    pub fn prune_stale_links(&mut self) {
        self.links.retain(|l| {
            self.pin_kind.contains_key(&l.from) && self.pin_kind.contains_key(&l.to)
        });
    }

    pub fn links_iter(&self) -> impl Iterator<Item = &Link<E>> {
        self.links.iter()
    }

    /// Edges keyed by (output pin, input pin) for adapter sync.
    pub fn link_key_set(&self) -> HashSet<(PinId, PinId)> {
        self.links.iter().map(|l| (l.from, l.to)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_connect() {
        let mut g = Graph::<&str, ()>::new();
        let a = g.add_node("a", Layout2d::new(0.0, 0.0), 0, 1);
        let b = g.add_node("b", Layout2d::new(1.0, 0.0), 1, 0);
        let out_a = g.node(a).unwrap().outputs[0];
        let in_b = g.node(b).unwrap().inputs[0];
        let _ = g.connect(out_a, in_b, ()).unwrap();
        assert_eq!(g.links.len(), 1);
    }
}
