//! Bridges [`graph_lib::Graph`] and [`NodeGraph`]: bidirectional
//! id mapping, wire sync, and [`GraphChanges`] so evaluation runs only when something changed.
//!
//! This layer stays in **egui-nodes** (not **graph-lib**): it maps slab canvas indices ↔ [`PinId`]
//! and merges the two wire sets. Pure topology and DAG checks live in [`graph_lib`].

pub mod shell_viewer;

use std::collections::{HashMap, HashSet};
use std::fmt;

use egui::Pos2;

use graph_lib::{Graph, GraphError, Layout2d, LinkId, NodeId, PinId, PinType};

use crate::ui::nodes_engine::{InPinId, NodeGraph, NodeId as ViewNodeId, OutPinId};

/// Converts [`Layout2d`] to egui [`Pos2`] (node position on the canvas).
#[inline]
pub fn layout_to_pos2(layout: Layout2d) -> Pos2 {
    Pos2::new(layout.x, layout.y)
}

/// Converts egui [`Pos2`] back to [`Layout2d`].
#[inline]
pub fn pos2_to_layout(pos: Pos2) -> Layout2d {
    Layout2d::new(pos.x, pos.y)
}

/// Payload stored in each NodeGraph cell — ties slab node back to [`NodeId`] and holds user `N`.
#[derive(Clone, Debug)]
pub struct NodeData<N> {
    pub node_id: NodeId,
    pub user: N,
}

/// Accumulated edits since the last [`NodesEditor::take_graph_changes`].
///
/// Use this to drive [`graph_lib::Executor::recompute_topology`] / evaluation only when something
/// actually changed, instead of every frame.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GraphChanges {
    /// Node/link connectivity changed (add/remove wire or node).
    pub topology_changed: bool,
    /// Node payload, [`graph_lib::Layout2d`], or collapse state changed (NodeGraph ↔ graph sync).
    pub payload_or_layout_changed: bool,
}

impl GraphChanges {
    #[must_use]
    pub fn any(&self) -> bool {
        self.topology_changed || self.payload_or_layout_changed
    }
}

#[derive(Debug)]
pub enum NodesEditorError {
    Graph(GraphError),
    UnmappedNode(NodeId),
    UnmappedViewNode(ViewNodeId),
    ViewRejectedWire,
}

impl fmt::Display for NodesEditorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Graph(e) => write!(f, "{e}"),
            Self::UnmappedNode(id) => write!(f, "node {} not mapped to NodeGraph", id.get()),
            Self::UnmappedViewNode(id) => write!(f, "view node {} not mapped", id.0),
            Self::ViewRejectedWire => write!(f, "node graph rejected wire"),
        }
    }
}

impl std::error::Error for NodesEditorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Graph(e) => Some(e),
            _ => None,
        }
    }
}

impl From<GraphError> for NodesEditorError {
    fn from(value: GraphError) -> Self {
        Self::Graph(value)
    }
}

/// Owns the headless [`Graph`] and the [`NodeGraph`] view; maps [`NodeId`] ↔ [`ViewNodeId`].
///
/// After [`crate::NodesView::show`](crate::ui::view::NodesView::show), call [`Self::take_graph_changes`]
/// once to see whether topology or payloads changed — then refresh a [`graph_lib::Executor`] only when
/// needed (e.g. [`GraphChanges::topology_changed`] ⇒ [`graph_lib::Executor::recompute_topology`]).
pub struct NodesEditor<N, E> {
    pub graph: Graph<N, E>,
    pub node_graph: NodeGraph<NodeData<N>>,
    core_to_view: HashMap<NodeId, ViewNodeId>,
    view_to_core: HashMap<ViewNodeId, NodeId>,
    pending_changes: GraphChanges,
}

impl<N, E> Default for NodesEditor<N, E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<N, E> NodesEditor<N, E> {
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            node_graph: NodeGraph::new(),
            core_to_view: HashMap::new(),
            view_to_core: HashMap::new(),
            pending_changes: GraphChanges::default(),
        }
    }

    /// Drain accumulated graph edits. After this call, the next [`Self::take_graph_changes`]
    /// returns empty flags until new edits occur.
    pub fn take_graph_changes(&mut self) -> GraphChanges {
        std::mem::take(&mut self.pending_changes)
    }

    pub fn view_node_id(&self, core: NodeId) -> Option<ViewNodeId> {
        self.core_to_view.get(&core).copied()
    }

    pub fn core_node_id(&self, view: ViewNodeId) -> Option<NodeId> {
        self.view_to_core.get(&view).copied()
    }

    fn map_view_pin_out(&self, pin: PinId) -> Result<(ViewNodeId, usize), NodesEditorError> {
        let (nid, port, is_out) = self
            .graph
            .pin_port(pin)
            .ok_or(NodesEditorError::Graph(GraphError::UnknownPin(pin)))?;
        if !is_out {
            return Err(NodesEditorError::Graph(GraphError::NotOutputPin(pin)));
        }
        let sn = self
            .core_to_view
            .get(&nid)
            .copied()
            .ok_or(NodesEditorError::UnmappedNode(nid))?;
        Ok((sn, port))
    }

    fn map_view_pin_in(&self, pin: PinId) -> Result<(ViewNodeId, usize), NodesEditorError> {
        let (nid, port, is_out) = self
            .graph
            .pin_port(pin)
            .ok_or(NodesEditorError::Graph(GraphError::UnknownPin(pin)))?;
        if is_out {
            return Err(NodesEditorError::Graph(GraphError::NotInputPin(pin)));
        }
        let sn = self
            .core_to_view
            .get(&nid)
            .copied()
            .ok_or(NodesEditorError::UnmappedNode(nid))?;
        Ok((sn, port))
    }

    /// Add a node to the graph and a matching NodeGraph cell.
    ///
    /// All pins are [`PinType::Any`]. See [`Self::insert_node_with_pin_types`] for typed ports.
    pub fn insert_node(
        &mut self,
        data: N,
        layout: Layout2d,
        inputs: usize,
        outputs: usize,
    ) -> NodeId
    where
        N: Clone,
    {
        let mut in_t = Vec::with_capacity(inputs);
        in_t.resize(inputs, PinType::Any);
        let mut out_t = Vec::with_capacity(outputs);
        out_t.resize(outputs, PinType::Any);
        self.insert_node_with_pin_types(data, layout, &in_t, &out_t)
    }

    /// Add a node with explicit per-pin types on the headless [`Graph`] (pin counts follow slice lengths).
    pub fn insert_node_with_pin_types(
        &mut self,
        data: N,
        layout: Layout2d,
        input_types: &[PinType],
        output_types: &[PinType],
    ) -> NodeId
    where
        N: Clone,
    {
        let id = self.graph.add_node_with_pin_types(
            data.clone(),
            layout,
            input_types,
            output_types,
        );
        let pos = layout_to_pos2(layout);
        let collapsed = self.graph.node(id).unwrap().collapsed;
        let payload = NodeData {
            node_id: id,
            user: data,
        };
        let sn = if collapsed {
            self.node_graph.insert_node_collapsed(pos, payload)
        } else {
            self.node_graph.insert_node(pos, payload)
        };
        self.core_to_view.insert(id, sn);
        self.view_to_core.insert(sn, id);
        self.pending_changes.topology_changed = true;
        self.pending_changes.payload_or_layout_changed = true;
        id
    }

    /// Remove nodes by their **view** ids (i.e. `NodeGraph` ids), keeping the headless [`Graph`]
    /// and the view graph in sync.
    ///
    /// Returns how many nodes were actually removed.
    ///
    /// This is the recommended way for UI code to delete the current selection, since selection
    /// comes from the canvas in view-id space.
    pub fn remove_view_nodes(
        &mut self,
        view_nodes: impl IntoIterator<Item = ViewNodeId>,
    ) -> usize {
        let mut unique: HashSet<ViewNodeId> = HashSet::new();
        unique.extend(view_nodes);

        let mut removed = 0usize;
        for view_id in unique {
            let Some(core_id) = self.view_to_core.get(&view_id).copied() else {
                continue;
            };
            if !self.node_graph.contains_node(view_id) {
                continue;
            }

            let _ = self.node_graph.try_remove_node(view_id);
            let _ = self.graph.remove_node(core_id);

            self.view_to_core.remove(&view_id);
            self.core_to_view.remove(&core_id);

            removed += 1;
        }

        if removed > 0 {
            self.pending_changes.topology_changed = true;
            self.pending_changes.payload_or_layout_changed = true;
        }

        removed
    }

    /// Connect output pin → input pin in both stores.
    pub fn connect_pins(
        &mut self,
        from: PinId,
        to: PinId,
        data: E,
    ) -> Result<LinkId, NodesEditorError> {
        let lid = self
            .graph
            .connect(from, to, data)
            .map_err(NodesEditorError::from)?;
        let (a, oi) = self.map_view_pin_out(from)?;
        let (b, ii) = self.map_view_pin_in(to)?;
        let ok = self.node_graph.connect(
            OutPinId {
                node: a,
                output: oi,
            },
            InPinId { node: b, input: ii },
        );
        if !ok {
            let _ = self.graph.disconnect_link(lid);
            return Err(NodesEditorError::ViewRejectedWire);
        }
        self.pending_changes.topology_changed = true;
        self.pending_changes.payload_or_layout_changed = true;
        Ok(lid)
    }

    pub fn sync_node_graph_payloads_from_graph(&mut self)
    where
        N: Clone,
    {
        for (&nid, &snid) in &self.core_to_view {
            if let (Some(gn), Some(n)) = (
                self.graph.node(nid),
                self.node_graph.get_node_info_mut(snid),
            ) {
                n.value.user = gn.data.clone();
                n.open = !gn.collapsed;
            }
        }
    }

    pub fn sync_graph_from_node_graph(&mut self)
    where
        N: Clone + PartialEq,
        E: Default + Clone,
    {
        let keys_before = self.graph.link_key_set();
        let mut payload_changed = false;
        for (&nid, &snid) in &self.core_to_view {
            if let (Some(gn), Some(info)) =
                (self.graph.node(nid), self.node_graph.get_node_info(snid))
            {
                let new_layout = pos2_to_layout(info.pos);
                let new_collapsed = !info.open;
                let new_data = info.value.user.clone();
                if gn.layout != new_layout || gn.collapsed != new_collapsed || gn.data != new_data {
                    payload_changed = true;
                }
            }
        }
        for (&nid, &snid) in &self.core_to_view {
            if let (Some(gn), Some(info)) = (
                self.graph.node_mut(nid),
                self.node_graph.get_node_info(snid),
            ) {
                gn.layout = pos2_to_layout(info.pos);
                gn.collapsed = !info.open;
                gn.data = info.value.user.clone();
            }
        }
        self.sync_links_from_node_graph();
        let keys_after = self.graph.link_key_set();
        let topology = keys_before != keys_after;
        if topology {
            self.pending_changes.topology_changed = true;
        }
        if topology || payload_changed {
            self.pending_changes.payload_or_layout_changed = true;
        }
    }

    /// Update only the headless [`Graph`] link list from NodeGraph wires (NodeGraph is authoritative).
    fn sync_links_from_node_graph(&mut self)
    where
        E: Default + Clone,
    {
        let mut desired: HashSet<(PinId, PinId)> = HashSet::new();
        for (outp, inp) in self.node_graph.wires() {
            let Ok(og) = self.graph_pin_from_view_out(outp) else {
                continue;
            };
            let Ok(ig) = self.graph_pin_from_view_in(inp) else {
                continue;
            };
            desired.insert((og, ig));
        }

        let mut remove = Vec::new();
        for l in &self.graph.links {
            if !desired.contains(&(l.from, l.to)) {
                remove.push(l.id);
            }
        }
        for id in remove {
            let _ = self.graph.disconnect_link(id);
        }

        let have = self.graph.link_key_set();
        for (a, b) in desired {
            if !have.contains(&(a, b)) {
                let _ = self.graph.connect(a, b, E::default());
            }
        }
    }

    fn graph_pin_from_view_out(&self, p: OutPinId) -> Result<PinId, ()> {
        let nid = self.view_to_core.get(&p.node).copied().ok_or(())?;
        let node = self.graph.node(nid).ok_or(())?;
        node.outputs.get(p.output).map(|pin| pin.id).ok_or(())
    }

    fn graph_pin_from_view_in(&self, p: InPinId) -> Result<PinId, ()> {
        let nid = self.view_to_core.get(&p.node).copied().ok_or(())?;
        let node = self.graph.node(nid).ok_or(())?;
        node.inputs.get(p.input).map(|pin| pin.id).ok_or(())
    }
}
