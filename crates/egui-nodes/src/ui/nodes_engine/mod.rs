//! Interactive node graph for egui: [`NodeGraph`] stores nodes, wires, and positions; [`canvas`]
//! implements [`NodesCanvas`](crate::ui::nodes_engine::canvas::NodesCanvas) / rendering and input.
//! The canvas module is a submodule so it can use `NodeGraph` internals while keeping the public API narrow.

pub mod canvas;

use std::fmt;
use std::ops::{Index, IndexMut};

use egui::{Pos2, ahash::HashSet};
use slab::Slab;

impl<T> Default for NodeGraph<T> {
    fn default() -> Self {
        NodeGraph::new()
    }
}

/// Node identifier.
///
/// This is newtype wrapper around [`usize`] that implements
/// necessary traits, but omits arithmetic operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
pub struct NodeId(pub usize);

/// Node of the graph.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct Node<T> {
    /// Node generic value.
    pub value: T,

    /// Position of the top-left corner of the node.
    /// This does not include frame margin.
    pub pos: egui::Pos2,

    /// Flag indicating that the node is open - not collapsed.
    pub open: bool,
}

/// Output pin identifier: node id and pin index.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OutPinId {
    /// Node id.
    pub node: NodeId,

    /// Output pin index.
    pub output: usize,
}

/// Input pin identifier: node id and pin index.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InPinId {
    /// Node id.
    pub node: NodeId,

    /// Input pin index.
    pub input: usize,
}

/// Error from [`NodeGraph`] when a [`NodeId`] is not valid for the operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeGraphError {
    /// No node exists with this id in the [`NodeGraph`].
    UnknownNode(NodeId),
}

impl fmt::Display for NodeGraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownNode(id) => write!(f, "unknown node {}", id.0),
        }
    }
}

impl std::error::Error for NodeGraphError {}

/// Connection between two nodes.
///
/// Nodes may support multiple connections to the same input or output.
/// But duplicate connections between same input and the same output are not allowed.
/// Attempt to insert existing connection will be ignored.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct Wire {
    out_pin: OutPinId,
    in_pin: InPinId,
}

#[derive(Clone, Debug)]
struct Wires {
    wires: HashSet<Wire>,
}

#[cfg(feature = "serde")]
impl serde::Serialize for Wires {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;

        let mut seq = serializer.serialize_seq(Some(self.wires.len()))?;
        for wire in &self.wires {
            seq.serialize_element(&wire)?;
        }
        seq.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Wires {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = HashSet<Wire>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a sequence of wires")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut wires = HashSet::with_hasher(egui::ahash::RandomState::new());
                while let Some(wire) = seq.next_element()? {
                    wires.insert(wire);
                }
                Ok(wires)
            }
        }

        let wires = deserializer.deserialize_seq(Visitor)?;
        Ok(Wires { wires })
    }
}

impl Wires {
    fn new() -> Self {
        Wires {
            wires: HashSet::with_hasher(egui::ahash::RandomState::new()),
        }
    }

    fn insert(&mut self, wire: Wire) -> bool {
        self.wires.insert(wire)
    }

    fn remove(&mut self, wire: &Wire) -> bool {
        self.wires.remove(wire)
    }

    fn drop_node(&mut self, node: NodeId) -> usize {
        let count = self.wires.len();
        self.wires
            .retain(|wire| wire.out_pin.node != node && wire.in_pin.node != node);
        count - self.wires.len()
    }

    fn drop_inputs(&mut self, pin: InPinId) -> usize {
        let count = self.wires.len();
        self.wires.retain(|wire| wire.in_pin != pin);
        count - self.wires.len()
    }

    fn drop_outputs(&mut self, pin: OutPinId) -> usize {
        let count = self.wires.len();
        self.wires.retain(|wire| wire.out_pin != pin);
        count - self.wires.len()
    }

    fn wired_inputs(&self, out_pin: OutPinId) -> impl Iterator<Item = InPinId> + '_ {
        self.wires
            .iter()
            .filter(move |wire| wire.out_pin == out_pin)
            .map(|wire| wire.in_pin)
    }

    fn wired_outputs(&self, in_pin: InPinId) -> impl Iterator<Item = OutPinId> + '_ {
        self.wires
            .iter()
            .filter(move |wire| wire.in_pin == in_pin)
            .map(|wire| wire.out_pin)
    }

    fn iter(&self) -> impl Iterator<Item = Wire> + '_ {
        self.wires.iter().copied()
    }
}

/// NodeGraph is generic node-graph container.
///
/// It holds graph state - positioned nodes and wires between their pins.
/// It can be rendered using [`NodeGraph::show`].
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NodeGraph<T> {
    // #[cfg_attr(feature = "serde", serde(with = "serde_nodes"))]
    nodes: Slab<Node<T>>,
    wires: Wires,
}

impl<T> NodeGraph<T> {
    /// Create a new empty NodeGraph.
    ///
    /// # Examples
    ///
    /// ```
    /// # use egui_nodes::ui::nodes_engine::NodeGraph;
    /// let node_graph = NodeGraph::<()>::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        NodeGraph {
            nodes: Slab::new(),
            wires: Wires::new(),
        }
    }

    /// Adds a node to the NodeGraph.
    /// Returns the index of the node.
    ///
    /// # Examples
    ///
    /// ```
    /// # use egui_nodes::ui::nodes_engine::NodeGraph;
    /// let mut node_graph = NodeGraph::<()>::new();
    /// node_graph.insert_node(egui::pos2(0.0, 0.0), ());
    /// ```
    pub fn insert_node(&mut self, pos: egui::Pos2, node: T) -> NodeId {
        let idx = self.nodes.insert(Node {
            value: node,
            pos,
            open: true,
        });

        NodeId(idx)
    }

    /// Adds a node to the NodeGraph in collapsed state.
    /// Returns the index of the node.
    ///
    /// # Examples
    ///
    /// ```
    /// # use egui_nodes::ui::nodes_engine::NodeGraph;
    /// let mut node_graph = NodeGraph::<()>::new();
    /// node_graph.insert_node_collapsed(egui::pos2(0.0, 0.0), ());
    /// ```
    pub fn insert_node_collapsed(&mut self, pos: egui::Pos2, node: T) -> NodeId {
        let idx = self.nodes.insert(Node {
            value: node,
            pos,
            open: false,
        });

        NodeId(idx)
    }

    /// Returns whether `id` refers to a node currently in this graph.
    #[must_use]
    pub fn contains_node(&self, id: NodeId) -> bool {
        self.nodes.contains(id.0)
    }

    /// Opens or collapses a node.
    ///
    /// # Panics
    ///
    /// Panics if the node does not exist.
    #[track_caller]
    pub fn open_node(&mut self, node: NodeId, open: bool) {
        self.try_open_node(node, open)
            .expect("NodeGraph::open_node: node id must exist");
    }

    /// Opens or collapses a node, or returns [`NodeGraphError::UnknownNode`] if `node` is invalid.
    pub fn try_open_node(&mut self, node: NodeId, open: bool) -> Result<(), NodeGraphError> {
        if !self.nodes.contains(node.0) {
            return Err(NodeGraphError::UnknownNode(node));
        }
        self.nodes[node.0].open = open;
        Ok(())
    }

    /// Removes a node from the NodeGraph.
    /// Returns the node if it was removed.
    ///
    /// # Panics
    ///
    /// Panics if the node does not exist.
    ///
    /// # Examples
    ///
    /// ```
    /// # use egui_nodes::ui::nodes_engine::NodeGraph;
    /// let mut node_graph = NodeGraph::<()>::new();
    /// let node = node_graph.insert_node(egui::pos2(0.0, 0.0), ());
    /// node_graph.remove_node(node);
    /// ```
    #[track_caller]
    pub fn remove_node(&mut self, idx: NodeId) -> T {
        self.try_remove_node(idx)
            .expect("NodeGraph::remove_node: node id must exist")
    }

    /// Removes a node and its incident wires, or returns [`NodeGraphError::UnknownNode`].
    pub fn try_remove_node(&mut self, idx: NodeId) -> Result<T, NodeGraphError> {
        if !self.nodes.contains(idx.0) {
            return Err(NodeGraphError::UnknownNode(idx));
        }
        let value = self.nodes.remove(idx.0).value;
        self.wires.drop_node(idx);
        Ok(value)
    }

    /// Connects two nodes.
    /// Returns true if the connection was successful.
    /// Returns false if the connection already exists.
    ///
    /// # Panics
    ///
    /// Panics if either node does not exist.
    #[track_caller]
    pub fn connect(&mut self, from: OutPinId, to: InPinId) -> bool {
        self.try_connect(from, to)
            .expect("NodeGraph::connect: both node ids must exist")
    }

    /// Connects two nodes. Returns `Ok(true)` if a new wire was added, `Ok(false)` if it already
    /// existed, or [`NodeGraphError::UnknownNode`] if either endpoint node is missing.
    pub fn try_connect(&mut self, from: OutPinId, to: InPinId) -> Result<bool, NodeGraphError> {
        if !self.nodes.contains(from.node.0) {
            return Err(NodeGraphError::UnknownNode(from.node));
        }
        if !self.nodes.contains(to.node.0) {
            return Err(NodeGraphError::UnknownNode(to.node));
        }
        let wire = Wire {
            out_pin: from,
            in_pin: to,
        };
        Ok(self.wires.insert(wire))
    }

    /// Disconnects two nodes.
    /// Returns true if the connection was removed.
    ///
    /// # Panics
    ///
    /// Panics if either node does not exist.
    #[track_caller]
    pub fn disconnect(&mut self, from: OutPinId, to: InPinId) -> bool {
        self.try_disconnect(from, to)
            .expect("NodeGraph::disconnect: both node ids must exist")
    }

    /// Disconnects two nodes, or returns [`NodeGraphError::UnknownNode`] if an endpoint is invalid.
    /// Returns `Ok(true)` if a wire was removed, `Ok(false)` if none matched.
    pub fn try_disconnect(&mut self, from: OutPinId, to: InPinId) -> Result<bool, NodeGraphError> {
        if !self.nodes.contains(from.node.0) {
            return Err(NodeGraphError::UnknownNode(from.node));
        }
        if !self.nodes.contains(to.node.0) {
            return Err(NodeGraphError::UnknownNode(to.node));
        }
        let wire = Wire {
            out_pin: from,
            in_pin: to,
        };
        Ok(self.wires.remove(&wire))
    }

    /// Removes all connections to the node's pin.
    ///
    /// Returns number of removed connections.
    ///
    /// # Panics
    ///
    /// Panics if the node does not exist.
    #[track_caller]
    pub fn drop_inputs(&mut self, pin: InPinId) -> usize {
        self.try_drop_inputs(pin)
            .expect("NodeGraph::drop_inputs: node id must exist")
    }

    /// Like [`Self::drop_inputs`], but returns an error if `pin.node` is not in the graph.
    pub fn try_drop_inputs(&mut self, pin: InPinId) -> Result<usize, NodeGraphError> {
        if !self.nodes.contains(pin.node.0) {
            return Err(NodeGraphError::UnknownNode(pin.node));
        }
        Ok(self.wires.drop_inputs(pin))
    }

    /// Removes all connections from the node's pin.
    /// Returns number of removed connections.
    ///
    /// # Panics
    ///
    /// Panics if the node does not exist.
    #[track_caller]
    pub fn drop_outputs(&mut self, pin: OutPinId) -> usize {
        self.try_drop_outputs(pin)
            .expect("NodeGraph::drop_outputs: node id must exist")
    }

    /// Like [`Self::drop_outputs`], but returns an error if `pin.node` is not in the graph.
    pub fn try_drop_outputs(&mut self, pin: OutPinId) -> Result<usize, NodeGraphError> {
        if !self.nodes.contains(pin.node.0) {
            return Err(NodeGraphError::UnknownNode(pin.node));
        }
        Ok(self.wires.drop_outputs(pin))
    }

    /// Returns reference to the node.
    #[must_use]
    pub fn get_node(&self, idx: NodeId) -> Option<&T> {
        self.nodes.get(idx.0).map(|node| &node.value)
    }

    /// Returns mutable reference to the node.
    pub fn get_node_mut(&mut self, idx: NodeId) -> Option<&mut T> {
        match self.nodes.get_mut(idx.0) {
            Some(node) => Some(&mut node.value),
            None => None,
        }
    }

    /// Returns reference to the node data.
    #[must_use]
    pub fn get_node_info(&self, idx: NodeId) -> Option<&Node<T>> {
        self.nodes.get(idx.0)
    }

    /// Returns mutable reference to the node data.
    pub fn get_node_info_mut(&mut self, idx: NodeId) -> Option<&mut Node<T>> {
        self.nodes.get_mut(idx.0)
    }

    /// Iterates over shared references to each node.
    pub fn nodes(&self) -> NodesIter<'_, T> {
        NodesIter {
            nodes: self.nodes.iter(),
        }
    }

    /// Iterates over mutable references to each node.
    pub fn nodes_mut(&mut self) -> NodesIterMut<'_, T> {
        NodesIterMut {
            nodes: self.nodes.iter_mut(),
        }
    }

    /// Iterates over shared references to each node and its position.
    pub fn nodes_pos(&self) -> NodesPosIter<'_, T> {
        NodesPosIter {
            nodes: self.nodes.iter(),
        }
    }

    /// Iterates over mutable references to each node and its position.
    pub fn nodes_pos_mut(&mut self) -> NodesPosIterMut<'_, T> {
        NodesPosIterMut {
            nodes: self.nodes.iter_mut(),
        }
    }

    /// Iterates over shared references to each node and its identifier.
    pub fn node_ids(&self) -> NodesIdsIter<'_, T> {
        NodesIdsIter {
            nodes: self.nodes.iter(),
        }
    }

    /// Iterates over mutable references to each node and its identifier.
    pub fn nodes_ids_mut(&mut self) -> NodesIdsIterMut<'_, T> {
        NodesIdsIterMut {
            nodes: self.nodes.iter_mut(),
        }
    }

    /// Iterates over shared references to each node, its position and its identifier.
    pub fn nodes_pos_ids(&self) -> NodesPosIdsIter<'_, T> {
        NodesPosIdsIter {
            nodes: self.nodes.iter(),
        }
    }

    /// Iterates over mutable references to each node, its position and its identifier.
    pub fn nodes_pos_ids_mut(&mut self) -> NodesPosIdsIterMut<'_, T> {
        NodesPosIdsIterMut {
            nodes: self.nodes.iter_mut(),
        }
    }

    /// Iterates over shared references to each node data.
    pub fn nodes_info(&self) -> NodeInfoIter<'_, T> {
        NodeInfoIter {
            nodes: self.nodes.iter(),
        }
    }

    /// Iterates over mutable references to each node data.
    pub fn nodes_info_mut(&mut self) -> NodeInfoIterMut<'_, T> {
        NodeInfoIterMut {
            nodes: self.nodes.iter_mut(),
        }
    }

    /// Iterates over shared references to each node id and data.
    pub fn nodes_ids_data(&self) -> NodeIdsDataIter<'_, T> {
        NodeIdsDataIter {
            nodes: self.nodes.iter(),
        }
    }

    /// Iterates over mutable references to each node id and data.
    pub fn nodes_ids_data_mut(&mut self) -> NodeIdsDataIterMut<'_, T> {
        NodeIdsDataIterMut {
            nodes: self.nodes.iter_mut(),
        }
    }

    /// Iterates over wires.
    pub fn wires(&self) -> impl Iterator<Item = (OutPinId, InPinId)> + '_ {
        self.wires.iter().map(|wire| (wire.out_pin, wire.in_pin))
    }

    /// Returns input pin of the node.
    #[must_use]
    pub fn in_pin(&self, pin: InPinId) -> InPin {
        InPin::new(self, pin)
    }

    /// Returns output pin of the node.
    #[must_use]
    pub fn out_pin(&self, pin: OutPinId) -> OutPin {
        OutPin::new(self, pin)
    }
}

impl<T> Index<NodeId> for NodeGraph<T> {
    type Output = T;

    #[inline]
    #[track_caller]
    fn index(&self, idx: NodeId) -> &Self::Output {
        &self.nodes[idx.0].value
    }
}

impl<T> IndexMut<NodeId> for NodeGraph<T> {
    #[inline]
    #[track_caller]
    fn index_mut(&mut self, idx: NodeId) -> &mut Self::Output {
        &mut self.nodes[idx.0].value
    }
}

/// Iterator over shared references to nodes.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct NodesIter<'a, T> {
    nodes: slab::Iter<'a, Node<T>>,
}

impl<'a, T> Iterator for NodesIter<'a, T> {
    type Item = &'a T;

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.nodes.size_hint()
    }

    fn next(&mut self) -> Option<&'a T> {
        let (_, node) = self.nodes.next()?;
        Some(&node.value)
    }

    fn nth(&mut self, n: usize) -> Option<&'a T> {
        let (_, node) = self.nodes.nth(n)?;
        Some(&node.value)
    }
}

/// Iterator over mutable references to nodes.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct NodesIterMut<'a, T> {
    nodes: slab::IterMut<'a, Node<T>>,
}

impl<'a, T> Iterator for NodesIterMut<'a, T> {
    type Item = &'a mut T;

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.nodes.size_hint()
    }

    fn next(&mut self) -> Option<&'a mut T> {
        let (_, node) = self.nodes.next()?;
        Some(&mut node.value)
    }

    fn nth(&mut self, n: usize) -> Option<&'a mut T> {
        let (_, node) = self.nodes.nth(n)?;
        Some(&mut node.value)
    }
}

/// Iterator over shared references to nodes and their positions.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct NodesPosIter<'a, T> {
    nodes: slab::Iter<'a, Node<T>>,
}

impl<'a, T> Iterator for NodesPosIter<'a, T> {
    type Item = (Pos2, &'a T);

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.nodes.size_hint()
    }

    fn next(&mut self) -> Option<(Pos2, &'a T)> {
        let (_, node) = self.nodes.next()?;
        Some((node.pos, &node.value))
    }

    fn nth(&mut self, n: usize) -> Option<(Pos2, &'a T)> {
        let (_, node) = self.nodes.nth(n)?;
        Some((node.pos, &node.value))
    }
}

/// Iterator over mutable references to nodes and their positions.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct NodesPosIterMut<'a, T> {
    nodes: slab::IterMut<'a, Node<T>>,
}

impl<'a, T> Iterator for NodesPosIterMut<'a, T> {
    type Item = (Pos2, &'a mut T);

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.nodes.size_hint()
    }

    fn next(&mut self) -> Option<(Pos2, &'a mut T)> {
        let (_, node) = self.nodes.next()?;
        Some((node.pos, &mut node.value))
    }

    fn nth(&mut self, n: usize) -> Option<(Pos2, &'a mut T)> {
        let (_, node) = self.nodes.nth(n)?;
        Some((node.pos, &mut node.value))
    }
}

/// Iterator over shared references to nodes and their identifiers.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct NodesIdsIter<'a, T> {
    nodes: slab::Iter<'a, Node<T>>,
}

impl<'a, T> Iterator for NodesIdsIter<'a, T> {
    type Item = (NodeId, &'a T);

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.nodes.size_hint()
    }

    fn next(&mut self) -> Option<(NodeId, &'a T)> {
        let (idx, node) = self.nodes.next()?;
        Some((NodeId(idx), &node.value))
    }

    fn nth(&mut self, n: usize) -> Option<(NodeId, &'a T)> {
        let (idx, node) = self.nodes.nth(n)?;
        Some((NodeId(idx), &node.value))
    }
}

/// Iterator over mutable references to nodes and their identifiers.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct NodesIdsIterMut<'a, T> {
    nodes: slab::IterMut<'a, Node<T>>,
}

impl<'a, T> Iterator for NodesIdsIterMut<'a, T> {
    type Item = (NodeId, &'a mut T);

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.nodes.size_hint()
    }

    fn next(&mut self) -> Option<(NodeId, &'a mut T)> {
        let (idx, node) = self.nodes.next()?;
        Some((NodeId(idx), &mut node.value))
    }

    fn nth(&mut self, n: usize) -> Option<(NodeId, &'a mut T)> {
        let (idx, node) = self.nodes.nth(n)?;
        Some((NodeId(idx), &mut node.value))
    }
}

/// Iterator over shared references to nodes, their positions and their identifiers.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct NodesPosIdsIter<'a, T> {
    nodes: slab::Iter<'a, Node<T>>,
}

impl<'a, T> Iterator for NodesPosIdsIter<'a, T> {
    type Item = (NodeId, Pos2, &'a T);

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.nodes.size_hint()
    }

    fn next(&mut self) -> Option<(NodeId, Pos2, &'a T)> {
        let (idx, node) = self.nodes.next()?;
        Some((NodeId(idx), node.pos, &node.value))
    }

    fn nth(&mut self, n: usize) -> Option<(NodeId, Pos2, &'a T)> {
        let (idx, node) = self.nodes.nth(n)?;
        Some((NodeId(idx), node.pos, &node.value))
    }
}

/// Iterator over mutable references to nodes, their positions and their identifiers.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct NodesPosIdsIterMut<'a, T> {
    nodes: slab::IterMut<'a, Node<T>>,
}

impl<'a, T> Iterator for NodesPosIdsIterMut<'a, T> {
    type Item = (NodeId, Pos2, &'a mut T);

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.nodes.size_hint()
    }

    fn next(&mut self) -> Option<(NodeId, Pos2, &'a mut T)> {
        let (idx, node) = self.nodes.next()?;
        Some((NodeId(idx), node.pos, &mut node.value))
    }

    fn nth(&mut self, n: usize) -> Option<(NodeId, Pos2, &'a mut T)> {
        let (idx, node) = self.nodes.nth(n)?;
        Some((NodeId(idx), node.pos, &mut node.value))
    }
}

/// Iterator over shared references to nodes.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct NodeInfoIter<'a, T> {
    nodes: slab::Iter<'a, Node<T>>,
}

impl<'a, T> Iterator for NodeInfoIter<'a, T> {
    type Item = &'a Node<T>;

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.nodes.size_hint()
    }

    fn next(&mut self) -> Option<&'a Node<T>> {
        let (_, node) = self.nodes.next()?;
        Some(node)
    }

    fn nth(&mut self, n: usize) -> Option<&'a Node<T>> {
        let (_, node) = self.nodes.nth(n)?;
        Some(node)
    }
}

/// Iterator over mutable references to nodes.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct NodeInfoIterMut<'a, T> {
    nodes: slab::IterMut<'a, Node<T>>,
}

impl<'a, T> Iterator for NodeInfoIterMut<'a, T> {
    type Item = &'a mut Node<T>;

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.nodes.size_hint()
    }

    fn next(&mut self) -> Option<&'a mut Node<T>> {
        let (_, node) = self.nodes.next()?;
        Some(node)
    }

    fn nth(&mut self, n: usize) -> Option<&'a mut Node<T>> {
        let (_, node) = self.nodes.nth(n)?;
        Some(node)
    }
}

/// Iterator over shared references to nodes.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct NodeIdsDataIter<'a, T> {
    nodes: slab::Iter<'a, Node<T>>,
}

impl<'a, T> Iterator for NodeIdsDataIter<'a, T> {
    type Item = (NodeId, &'a Node<T>);

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.nodes.size_hint()
    }

    fn next(&mut self) -> Option<(NodeId, &'a Node<T>)> {
        let (id, node) = self.nodes.next()?;
        Some((NodeId(id), node))
    }

    fn nth(&mut self, n: usize) -> Option<(NodeId, &'a Node<T>)> {
        let (id, node) = self.nodes.nth(n)?;
        Some((NodeId(id), node))
    }
}

/// Iterator over mutable references to nodes.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct NodeIdsDataIterMut<'a, T> {
    nodes: slab::IterMut<'a, Node<T>>,
}

impl<'a, T> Iterator for NodeIdsDataIterMut<'a, T> {
    type Item = (NodeId, &'a mut Node<T>);

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.nodes.size_hint()
    }

    fn next(&mut self) -> Option<(NodeId, &'a mut Node<T>)> {
        let (id, node) = self.nodes.next()?;
        Some((NodeId(id), node))
    }

    fn nth(&mut self, n: usize) -> Option<(NodeId, &'a mut Node<T>)> {
        let (id, node) = self.nodes.nth(n)?;
        Some((NodeId(id), node))
    }
}

/// Node and its output pin.
#[derive(Clone, Debug)]
pub struct OutPin {
    /// Output pin identifier.
    pub id: OutPinId,

    /// List of input pins connected to this output pin.
    pub remotes: Vec<InPinId>,
}

/// Node and its output pin.
#[derive(Clone, Debug)]
pub struct InPin {
    /// Input pin identifier.
    pub id: InPinId,

    /// List of output pins connected to this input pin.
    pub remotes: Vec<OutPinId>,
}

impl OutPin {
    fn new<T>(node_graph: &NodeGraph<T>, pin: OutPinId) -> Self {
        OutPin {
            id: pin,
            remotes: node_graph.wires.wired_inputs(pin).collect(),
        }
    }
}

impl InPin {
    fn new<T>(node_graph: &NodeGraph<T>, pin: InPinId) -> Self {
        InPin {
            id: pin,
            remotes: node_graph.wires.wired_outputs(pin).collect(),
        }
    }
}

#[cfg(test)]
mod node_graph_try_api_tests {
    use egui::Pos2;

    use super::*;

    #[test]
    fn try_connect_errs_on_unknown_from_node() {
        let mut s = NodeGraph::<()>::new();
        let b = s.insert_node(Pos2::ZERO, ());
        let bad = NodeId(42);
        let out = OutPinId {
            node: bad,
            output: 0,
        };
        let inp = InPinId { node: b, input: 0 };
        assert!(matches!(
            s.try_connect(out, inp),
            Err(NodeGraphError::UnknownNode(n)) if n == bad
        ));
    }

    #[test]
    fn try_connect_ok_and_duplicate() {
        let mut s = NodeGraph::<()>::new();
        let a = s.insert_node(Pos2::ZERO, ());
        let b = s.insert_node(Pos2::ZERO, ());
        let out = OutPinId { node: a, output: 0 };
        let inp = InPinId { node: b, input: 0 };
        assert!(s.try_connect(out, inp).unwrap());
        assert!(!s.try_connect(out, inp).unwrap());
    }

    #[test]
    fn try_remove_node_unknown() {
        let mut s = NodeGraph::<()>::new();
        assert!(matches!(
            s.try_remove_node(NodeId(99)),
            Err(NodeGraphError::UnknownNode(_))
        ));
    }
}
