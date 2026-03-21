//! Snarl as view/interaction engine: [`SnarlAdapter`] keeps [`crate::graph::Graph`] and [`Snarl`](egui_snarl::Snarl) in sync.

use std::collections::{HashMap, HashSet};
use std::fmt;

use egui_snarl::{InPinId, NodeId as SnarlNodeId, OutPinId, Snarl};

use crate::graph::{Graph, GraphError, Layout2d, LinkId, NodeId, PinId};
use crate::layout_bridge::{layout_to_pos2, pos2_to_layout};

/// Payload stored in each Snarl cell — ties slab node back to [`NodeId`] and holds user `N`.
#[derive(Clone, Debug)]
pub struct NodeData<N> {
    pub node_id: NodeId,
    pub user: N,
}

#[derive(Debug)]
pub enum AdapterError {
    Graph(GraphError),
    UnmappedNode(NodeId),
    UnmappedSnarlNode(SnarlNodeId),
    SnarlRejectedWire,
}

impl fmt::Display for AdapterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Graph(e) => write!(f, "{e}"),
            Self::UnmappedNode(id) => write!(f, "node {:?} not mapped to Snarl", id.0),
            Self::UnmappedSnarlNode(id) => write!(f, "snarl node {} not mapped", id.0),
            Self::SnarlRejectedWire => write!(f, "Snarl rejected wire"),
        }
    }
}

impl std::error::Error for AdapterError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Graph(e) => Some(e),
            _ => None,
        }
    }
}

impl From<GraphError> for AdapterError {
    fn from(value: GraphError) -> Self {
        Self::Graph(value)
    }
}

/// Owns the headless [`Graph`] and the Snarl view; maps [`NodeId`] ↔ [`SnarlNodeId`].
pub struct SnarlAdapter<N, E> {
    pub graph: Graph<N, E>,
    pub snarl: Snarl<NodeData<N>>,
    node_to_snarl: HashMap<NodeId, SnarlNodeId>,
    snarl_to_node: HashMap<SnarlNodeId, NodeId>,
}

impl<N, E> Default for SnarlAdapter<N, E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<N, E> SnarlAdapter<N, E> {
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            snarl: Snarl::new(),
            node_to_snarl: HashMap::new(),
            snarl_to_node: HashMap::new(),
        }
    }

    pub fn snarl_node(&self, graph: NodeId) -> Option<SnarlNodeId> {
        self.node_to_snarl.get(&graph).copied()
    }

    pub fn graph_node(&self, snarl: SnarlNodeId) -> Option<NodeId> {
        self.snarl_to_node.get(&snarl).copied()
    }

    fn map_snarl_pin_out(&self, pin: PinId) -> Result<(SnarlNodeId, usize), AdapterError> {
        let (nid, port, is_out) = self
            .graph
            .pin_port(pin)
            .ok_or(AdapterError::Graph(GraphError::UnknownPin(pin)))?;
        if !is_out {
            return Err(AdapterError::Graph(GraphError::NotOutputPin(pin)));
        }
        let sn = self
            .node_to_snarl
            .get(&nid)
            .copied()
            .ok_or(AdapterError::UnmappedNode(nid))?;
        Ok((sn, port))
    }

    fn map_snarl_pin_in(&self, pin: PinId) -> Result<(SnarlNodeId, usize), AdapterError> {
        let (nid, port, is_out) = self
            .graph
            .pin_port(pin)
            .ok_or(AdapterError::Graph(GraphError::UnknownPin(pin)))?;
        if is_out {
            return Err(AdapterError::Graph(GraphError::NotInputPin(pin)));
        }
        let sn = self
            .node_to_snarl
            .get(&nid)
            .copied()
            .ok_or(AdapterError::UnmappedNode(nid))?;
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
        id
    }

    /// Connect output pin → input pin in both stores.
    pub fn connect_pins(&mut self, from: PinId, to: PinId, data: E) -> Result<LinkId, AdapterError> {
        let lid = self.graph.connect(from, to, data).map_err(AdapterError::from)?;
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
            return Err(AdapterError::SnarlRejectedWire);
        }
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
        N: Clone,
        E: Default + Clone,
    {
        for (&nid, &snid) in &self.node_to_snarl {
            if let (Some(gn), Some(info)) = (self.graph.node_mut(nid), self.snarl.get_node_info(snid)) {
                gn.layout = pos2_to_layout(info.pos);
                gn.collapsed = !info.open;
                gn.data = info.value.user.clone();
            }
        }
        self.sync_links_from_snarl();
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
        node.outputs.get(p.output).copied().ok_or(())
    }

    fn graph_pin_from_snarl_in(&self, p: InPinId) -> Result<PinId, ()> {
        let nid = self.snarl_to_node.get(&p.node).copied().ok_or(())?;
        let node = self.graph.node(nid).ok_or(())?;
        node.inputs.get(p.input).copied().ok_or(())
    }

}
