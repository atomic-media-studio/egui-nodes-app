//! Editor session: keeps [`core_graph::Graph`] and [`Snarl`](crate::ui::nodes_engine::Snarl) in sync.

pub mod shell_viewer;

use std::collections::{HashMap, HashSet};
use std::fmt;

use core_graph::{Graph, GraphError, Layout2d, LinkId, NodeId, PinId};

use crate::layout_bridge::{layout_to_pos2, pos2_to_layout};
use crate::ui::nodes_engine::{InPinId, NodeId as SnarlNodeId, OutPinId, Snarl};

/// Payload stored in each Snarl cell — ties slab node back to [`NodeId`] and holds user `N`.
#[derive(Clone, Debug)]
pub struct NodeData<N> {
    pub node_id: NodeId,
    pub user: N,
}

/// Accumulated edits since the last [`NodesEditor::take_graph_changes`].
///
/// Use this to drive [`core_graph::Executor::recompute_topology`] / evaluation only when something
/// actually changed, instead of every frame.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GraphChanges {
    /// Node/link connectivity changed (add/remove wire or node).
    pub topology_changed: bool,
    /// Node payload, [`core_graph::Layout2d`], or collapse state changed (Snarl ↔ graph sync).
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
    UnmappedSnarlNode(SnarlNodeId),
    SnarlRejectedWire,
}

impl fmt::Display for NodesEditorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Graph(e) => write!(f, "{e}"),
            Self::UnmappedNode(id) => write!(f, "node {} not mapped to Snarl", id.get()),
            Self::UnmappedSnarlNode(id) => write!(f, "snarl node {} not mapped", id.0),
            Self::SnarlRejectedWire => write!(f, "Snarl rejected wire"),
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

/// Owns the headless [`Graph`] and the Snarl view; maps [`NodeId`] ↔ [`SnarlNodeId`].
///
/// After [`crate::NodesView::show`](crate::ui::view::NodesView::show), call [`Self::take_graph_changes`]
/// once to see whether topology or payloads changed — then refresh a [`core_graph::Executor`] only when
/// needed (e.g. [`GraphChanges::topology_changed`] ⇒ [`core_graph::Executor::recompute_topology`]).
pub struct NodesEditor<N, E> {
    pub graph: Graph<N, E>,
    pub snarl: Snarl<NodeData<N>>,
    node_to_snarl: HashMap<NodeId, SnarlNodeId>,
    snarl_to_node: HashMap<SnarlNodeId, NodeId>,
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
            snarl: Snarl::new(),
            node_to_snarl: HashMap::new(),
            snarl_to_node: HashMap::new(),
            pending_changes: GraphChanges::default(),
        }
    }

    /// Drain accumulated graph edits. After this call, the next [`Self::take_graph_changes`]
    /// returns empty flags until new edits occur.
    pub fn take_graph_changes(&mut self) -> GraphChanges {
        std::mem::take(&mut self.pending_changes)
    }

    pub fn snarl_node(&self, graph: NodeId) -> Option<SnarlNodeId> {
        self.node_to_snarl.get(&graph).copied()
    }

    pub fn graph_node(&self, snarl: SnarlNodeId) -> Option<NodeId> {
        self.snarl_to_node.get(&snarl).copied()
    }

    fn map_snarl_pin_out(&self, pin: PinId) -> Result<(SnarlNodeId, usize), NodesEditorError> {
        let (nid, port, is_out) = self
            .graph
            .pin_port(pin)
            .ok_or(NodesEditorError::Graph(GraphError::UnknownPin(pin)))?;
        if !is_out {
            return Err(NodesEditorError::Graph(GraphError::NotOutputPin(pin)));
        }
        let sn = self
            .node_to_snarl
            .get(&nid)
            .copied()
            .ok_or(NodesEditorError::UnmappedNode(nid))?;
        Ok((sn, port))
    }

    fn map_snarl_pin_in(&self, pin: PinId) -> Result<(SnarlNodeId, usize), NodesEditorError> {
        let (nid, port, is_out) = self
            .graph
            .pin_port(pin)
            .ok_or(NodesEditorError::Graph(GraphError::UnknownPin(pin)))?;
        if is_out {
            return Err(NodesEditorError::Graph(GraphError::NotInputPin(pin)));
        }
        let sn = self
            .node_to_snarl
            .get(&nid)
            .copied()
            .ok_or(NodesEditorError::UnmappedNode(nid))?;
        Ok((sn, port))
    }

    /// Add a node to the graph and a matching Snarl cell.
    pub fn insert_node(&mut self, data: N, layout: Layout2d, inputs: usize, outputs: usize) -> NodeId
    where
        N: Clone,
    {
        let id = self
            .graph
            .add_node(data.clone(), layout, inputs, outputs);
        let pos = layout_to_pos2(layout);
        let collapsed = self.graph.node(id).unwrap().collapsed;
        let payload = NodeData {
            node_id: id,
            user: data,
        };
        let sn = if collapsed {
            self.snarl.insert_node_collapsed(pos, payload)
        } else {
            self.snarl.insert_node(pos, payload)
        };
        self.node_to_snarl.insert(id, sn);
        self.snarl_to_node.insert(sn, id);
        self.pending_changes.topology_changed = true;
        self.pending_changes.payload_or_layout_changed = true;
        id
    }

    /// Connect output pin → input pin in both stores.
    pub fn connect_pins(&mut self, from: PinId, to: PinId, data: E) -> Result<LinkId, NodesEditorError> {
        let lid = self.graph.connect(from, to, data).map_err(NodesEditorError::from)?;
        let (a, oi) = self.map_snarl_pin_out(from)?;
        let (b, ii) = self.map_snarl_pin_in(to)?;
        let ok = self.snarl.connect(
            OutPinId {
                node: a,
                output: oi,
            },
            InPinId {
                node: b,
                input: ii,
            },
        );
        if !ok {
            let _ = self.graph.disconnect_link(lid);
            return Err(NodesEditorError::SnarlRejectedWire);
        }
        self.pending_changes.topology_changed = true;
        self.pending_changes.payload_or_layout_changed = true;
        Ok(lid)
    }

    pub fn sync_snarl_payloads_from_graph(&mut self)
    where
        N: Clone,
    {
        for (&nid, &snid) in &self.node_to_snarl {
            if let (Some(gn), Some(n)) = (self.graph.node(nid), self.snarl.get_node_info_mut(snid)) {
                n.value.user = gn.data.clone();
                n.open = !gn.collapsed;
            }
        }
    }

    pub fn sync_graph_from_snarl(&mut self)
    where
        N: Clone + PartialEq,
        E: Default + Clone,
    {
        let keys_before = self.graph.link_key_set();
        let mut payload_changed = false;
        for (&nid, &snid) in &self.node_to_snarl {
            if let (Some(gn), Some(info)) = (self.graph.node(nid), self.snarl.get_node_info(snid)) {
                let new_layout = pos2_to_layout(info.pos);
                let new_collapsed = !info.open;
                let new_data = info.value.user.clone();
                if gn.layout != new_layout || gn.collapsed != new_collapsed || gn.data != new_data {
                    payload_changed = true;
                }
            }
        }
        for (&nid, &snid) in &self.node_to_snarl {
            if let (Some(gn), Some(info)) = (self.graph.node_mut(nid), self.snarl.get_node_info(snid)) {
                gn.layout = pos2_to_layout(info.pos);
                gn.collapsed = !info.open;
                gn.data = info.value.user.clone();
            }
        }
        self.sync_links_from_snarl();
        let keys_after = self.graph.link_key_set();
        let topology = keys_before != keys_after;
        if topology {
            self.pending_changes.topology_changed = true;
        }
        if topology || payload_changed {
            self.pending_changes.payload_or_layout_changed = true;
        }
    }

    /// Update only the headless [`Graph`] link list from Snarl wires (Snarl is authoritative).
    fn sync_links_from_snarl(&mut self)
    where
        E: Default + Clone,
    {
        let mut desired: HashSet<(PinId, PinId)> = HashSet::new();
        for (outp, inp) in self.snarl.wires() {
            let Ok(og) = self.graph_pin_from_snarl_out(outp) else {
                continue;
            };
            let Ok(ig) = self.graph_pin_from_snarl_in(inp) else {
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

    fn graph_pin_from_snarl_out(
        &self,
        p: OutPinId,
    ) -> Result<PinId, ()> {
        let nid = self.snarl_to_node.get(&p.node).copied().ok_or(())?;
        let node = self.graph.node(nid).ok_or(())?;
        node.outputs.get(p.output).map(|pin| pin.id).ok_or(())
    }

    fn graph_pin_from_snarl_in(&self, p: InPinId) -> Result<PinId, ()> {
        let nid = self.snarl_to_node.get(&p.node).copied().ok_or(())?;
        let node = self.graph.node(nid).ok_or(())?;
        node.inputs.get(p.input).map(|pin| pin.id).ok_or(())
    }
}
