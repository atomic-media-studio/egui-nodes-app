//! Bidirectional mapping between [`SemanticNodeId`](nodes_core::SemanticNodeId) and [`Snarl`](egui_snarl::Snarl) slab ids.

use std::collections::HashMap;
use std::fmt;

use egui_snarl::{InPinId, NodeId as SnarlNodeId, OutPinId, Snarl};

use nodes_core::{GraphError, SemanticEdge, SemanticEdgeId, SemanticGraph, SemanticNode, SemanticNodeId};

use crate::layout_map::{layout_to_pos2, pos2_to_layout};

/// Keeps semantic ids and Snarl [`SnarlNodeId`] in sync while you mutate both stores.
#[derive(Clone, Debug, Default)]
pub struct SemanticSnarlBridge {
    semantic_to_snarl: HashMap<SemanticNodeId, SnarlNodeId>,
    snarl_to_semantic: HashMap<SnarlNodeId, SemanticNodeId>,
}

/// Failures when applying the same operation to graph + Snarl.
#[derive(Debug)]
pub enum BridgeError {
    Graph(GraphError),
    UnmappedSemantic(SemanticNodeId),
    UnmappedSnarl(SnarlNodeId),
    /// Snarl rejected a wire that the semantic graph accepted (rolled back semantic edge).
    SnarlRejectedWire,
}

impl fmt::Display for BridgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Graph(e) => write!(f, "{e}"),
            Self::UnmappedSemantic(id) => write!(f, "no Snarl node for semantic id {:?}", id.0),
            Self::UnmappedSnarl(id) => write!(f, "no semantic id for Snarl node index {}", id.0),
            Self::SnarlRejectedWire => write!(f, "Snarl rejected wire (duplicate or invalid)"),
        }
    }
}

impl std::error::Error for BridgeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Graph(e) => Some(e),
            _ => None,
        }
    }
}

impl From<GraphError> for BridgeError {
    fn from(value: GraphError) -> Self {
        Self::Graph(value)
    }
}

impl SemanticSnarlBridge {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.semantic_to_snarl.is_empty()
    }

    pub fn len(&self) -> usize {
        self.semantic_to_snarl.len()
    }

    #[inline]
    pub fn semantic_id(&self, snarl: SnarlNodeId) -> Option<SemanticNodeId> {
        self.snarl_to_semantic.get(&snarl).copied()
    }

    #[inline]
    pub fn snarl_id(&self, semantic: SemanticNodeId) -> Option<SnarlNodeId> {
        self.semantic_to_snarl.get(&semantic).copied()
    }

    fn register(&mut self, semantic: SemanticNodeId, snarl: SnarlNodeId) {
        self.semantic_to_snarl.insert(semantic, snarl);
        self.snarl_to_semantic.insert(snarl, semantic);
    }

    fn unregister_pair(&mut self, semantic: SemanticNodeId, snarl: SnarlNodeId) {
        self.semantic_to_snarl.remove(&semantic);
        self.snarl_to_semantic.remove(&snarl);
    }

    /// Insert the same logical node into [`SemanticGraph`] and [`Snarl`], and record the mapping.
    pub fn insert_node<N: Clone, E>(
        &mut self,
        graph: &mut SemanticGraph<N, E>,
        snarl: &mut Snarl<N>,
        node: SemanticNode<N>,
    ) -> SemanticNodeId {
        let layout = node.layout;
        let collapsed = node.collapsed;
        let payload = node.payload.clone();
        let sid = graph.insert_node(node);
        let pos = layout_to_pos2(layout);
        let nid = if collapsed {
            snarl.insert_node_collapsed(pos, payload)
        } else {
            snarl.insert_node(pos, payload)
        };
        self.register(sid, nid);
        sid
    }

    /// Remove a semantic node everywhere (graph edges, Snarl wires, mapping).
    pub fn remove_node<N, E>(
        &mut self,
        graph: &mut SemanticGraph<N, E>,
        snarl: &mut Snarl<N>,
        semantic: SemanticNodeId,
    ) -> Option<SemanticNode<N>> {
        let snarl_id = self.snarl_id(semantic)?;
        self.unregister_pair(semantic, snarl_id);
        let _ = snarl.remove_node(snarl_id);
        graph.remove_node(semantic)
    }

    /// Add matching wires to graph and Snarl.
    pub fn connect<N, E: Default>(
        &mut self,
        graph: &mut SemanticGraph<N, E>,
        snarl: &mut Snarl<N>,
        from: SemanticNodeId,
        out_port: usize,
        to: SemanticNodeId,
        in_port: usize,
    ) -> Result<SemanticEdgeId, BridgeError> {
        let eid = graph
            .connect(from, out_port, to, in_port, E::default())
            .map_err(BridgeError::from)?;
        let a = self
            .snarl_id(from)
            .ok_or(BridgeError::UnmappedSemantic(from))?;
        let b = self
            .snarl_id(to)
            .ok_or(BridgeError::UnmappedSemantic(to))?;
        let ok = snarl.connect(
            OutPinId {
                node: a,
                output: out_port,
            },
            InPinId {
                node: b,
                input: in_port,
            },
        );
        if !ok {
            let _ = graph.disconnect_edge(eid);
            return Err(BridgeError::SnarlRejectedWire);
        }
        Ok(eid)
    }

    /// Disconnect one semantic edge in both stores.
    pub fn disconnect_edge<N, E: Clone>(
        &mut self,
        graph: &mut SemanticGraph<N, E>,
        snarl: &mut Snarl<N>,
        edge: SemanticEdgeId,
    ) -> Option<SemanticEdge<E>> {
        let e = graph.edge(edge)?.clone();
        let a = self.snarl_id(e.from)?;
        let b = self.snarl_id(e.to)?;
        let _ = snarl.disconnect(
            OutPinId {
                node: a,
                output: e.out_port,
            },
            InPinId {
                node: b,
                input: e.in_port,
            },
        );
        graph.disconnect_edge(edge)
    }

    /// Copy node positions and collapse state from Snarl into the semantic graph.
    pub fn sync_layout_from_snarl<N, E>(&self, graph: &mut SemanticGraph<N, E>, snarl: &Snarl<N>) {
        for (sem, &snid) in &self.semantic_to_snarl {
            if let Some(info) = snarl.get_node_info(snid) {
                if let Some(n) = graph.node_mut(*sem) {
                    n.layout = pos2_to_layout(info.pos);
                    n.collapsed = !info.open;
                }
            }
        }
    }

    /// Copy payloads from Snarl into the semantic graph (after widgets edited the Snarl side).
    pub fn sync_graph_payloads_from_snarl<N: Clone, E>(
        &self,
        graph: &mut SemanticGraph<N, E>,
        snarl: &Snarl<N>,
    ) {
        for (sem, &snid) in &self.semantic_to_snarl {
            if let (Some(gn), Some(v)) = (graph.node_mut(*sem), snarl.get_node(snid)) {
                gn.payload = v.clone();
            }
        }
    }

    /// Align semantic edges with Snarl wires without nuking all ids: remove stale edges, add missing ones.
    pub fn sync_edges_from_snarl<N, E: Default + Clone>(
        &self,
        graph: &mut SemanticGraph<N, E>,
        snarl: &Snarl<N>,
    ) {
        use std::collections::HashSet;

        let mut desired: HashSet<(SemanticNodeId, usize, SemanticNodeId, usize)> = HashSet::new();
        for (out_pin, in_pin) in snarl.wires() {
            let Some(s_from) = self.snarl_to_semantic.get(&out_pin.node).copied() else {
                continue;
            };
            let Some(s_to) = self.snarl_to_semantic.get(&in_pin.node).copied() else {
                continue;
            };
            desired.insert((s_from, out_pin.output, s_to, in_pin.input));
        }

        let mut remove_ids = Vec::new();
        for (eid, e) in graph.edges_iter() {
            let key = (e.from, e.out_port, e.to, e.in_port);
            if !desired.contains(&key) {
                remove_ids.push(eid);
            }
        }
        for eid in remove_ids {
            let _ = graph.disconnect_edge(eid);
        }

        let mut have: HashSet<(SemanticNodeId, usize, SemanticNodeId, usize)> = graph
            .edges_iter()
            .map(|(_, e)| (e.from, e.out_port, e.to, e.in_port))
            .collect();

        for key in desired {
            if !have.contains(&key) {
                let _ = graph.connect(key.0, key.1, key.2, key.3, E::default());
                have.insert(key);
            }
        }
    }

    /// Typical end-of-frame sync: layout, topology, payloads — Snarl is authoritative after interaction.
    pub fn sync_graph_from_snarl<N: Clone, E: Default + Clone>(
        &self,
        graph: &mut SemanticGraph<N, E>,
        snarl: &Snarl<N>,
    ) {
        self.sync_layout_from_snarl(graph, snarl);
        self.sync_edges_from_snarl(graph, snarl);
        self.sync_graph_payloads_from_snarl(graph, snarl);
    }

    /// Push semantic payloads into Snarl cells (e.g. after loading a file or batch edit).
    pub fn sync_snarl_payloads_from_graph<N: Clone, E>(
        &self,
        graph: &SemanticGraph<N, E>,
        snarl: &mut Snarl<N>,
    ) {
        for (sem, &snid) in &self.semantic_to_snarl {
            if let (Some(gn), Some(cell)) = (graph.node(*sem), snarl.get_node_mut(snid)) {
                *cell = gn.payload.clone();
            }
        }
    }
}
