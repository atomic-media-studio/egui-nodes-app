//! Dataflow evaluation: topological order, pin values, and a pluggable [`NodeEvaluator`].

use std::collections::{HashMap, HashSet, VecDeque};
use std::marker::PhantomData;

use crate::ids::{NodeId, PinId};
use crate::model::Graph;

/// Context passed to node evaluation (e.g. sample rate, time, parameters).
pub trait EvalContext {}

impl EvalContext for () {}

/// Simple value enum; extend with buffers, matrices, etc. as needed.
#[derive(Debug, Clone)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
}

/// Node evaluation for dataflow graphs.
pub trait NodeEvaluator<N, E, C: EvalContext> {
    /// Compute this node’s outputs from resolved inputs.
    fn eval_node(
        &mut self,
        graph: &Graph<N, E>,
        node_id: NodeId,
        ctx: &C,
        input_values: &[(PinId, Value)],
    ) -> Vec<(PinId, Value)>;
}

/// Runs [`NodeEvaluator`] over a graph in topological order.
pub struct Executor<N, E, C: EvalContext, EV: NodeEvaluator<N, E, C>> {
    pub graph: Graph<N, E>,
    pub evaluator: EV,
    topo_order: Vec<NodeId>,
    _phantom: PhantomData<C>,
}

impl<N, E, C, EV> Executor<N, E, C, EV>
where
    C: EvalContext,
    EV: NodeEvaluator<N, E, C>,
{
    pub fn new(graph: Graph<N, E>, evaluator: EV) -> Self {
        let topo_order = compute_topological_order(&graph);
        Self {
            graph,
            evaluator,
            topo_order,
            _phantom: PhantomData,
        }
    }

    pub fn recompute_topology(&mut self) {
        self.topo_order = compute_topological_order(&self.graph);
    }

    /// Evaluate all nodes in order; returns per-node output pin values.
    pub fn evaluate(&mut self, ctx: &C) -> Vec<(NodeId, Vec<(PinId, Value)>)> {
        let mut results = Vec::new();
        let mut values_at_pins: HashMap<PinId, Value> = HashMap::new();

        for node_id in &self.topo_order {
            let input_values = gather_inputs_for_node(&self.graph, *node_id, &values_at_pins);
            let outputs = self.evaluator.eval_node(
                &self.graph,
                *node_id,
                ctx,
                &input_values,
            );
            for (pin_id, value) in &outputs {
                values_at_pins.insert(*pin_id, value.clone());
            }
            results.push((*node_id, outputs));
        }

        results
    }
}

/// Kahn topological sort on the node DAG implied by links (output pin → input pin).
/// If a cycle is detected, remaining nodes are appended in ascending [`NodeId`] order so evaluation
/// still terminates (outputs may be incomplete until the graph is fixed).
pub fn compute_topological_order<N, E>(graph: &Graph<N, E>) -> Vec<NodeId> {
    let mut in_deg: HashMap<NodeId, usize> = graph.nodes_iter().map(|n| (n.id, 0)).collect();
    let mut adj: HashMap<NodeId, Vec<NodeId>> = HashMap::new();

    for link in &graph.links {
        let Some((from_n, _, from_out)) = graph.pin_port(link.from) else {
            continue;
        };
        let Some((to_n, _, to_in)) = graph.pin_port(link.to) else {
            continue;
        };
        if !from_out || to_in {
            continue;
        }
        adj.entry(from_n).or_default().push(to_n);
        *in_deg.entry(to_n).or_insert(0) += 1;
    }

    let mut starters: Vec<NodeId> = in_deg
        .iter()
        .filter(|(_, d)| **d == 0)
        .map(|(&id, _)| id)
        .collect();
    starters.sort_by_key(|n| n.get());

    let mut q: VecDeque<NodeId> = starters.into_iter().collect();
    let mut out = Vec::new();

    while let Some(u) = q.pop_front() {
        out.push(u);
        if let Some(ns) = adj.get(&u) {
            for &v in ns {
                if let Some(d) = in_deg.get_mut(&v) {
                    *d = d.saturating_sub(1);
                    if *d == 0 {
                        q.push_back(v);
                    }
                }
            }
        }
    }

    if out.len() < graph.nodes.len() {
        let set: HashSet<NodeId> = out.iter().copied().collect();
        let mut rest: Vec<NodeId> = graph
            .nodes_iter()
            .map(|n| n.id)
            .filter(|id| !set.contains(id))
            .collect();
        rest.sort_by_key(|n| n.get());
        out.extend(rest);
    }

    out
}

/// Collect values for each **input** pin of `node_id` by following incoming links.
pub fn gather_inputs_for_node<N, E>(
    graph: &Graph<N, E>,
    node_id: NodeId,
    values_at_pins: &HashMap<PinId, Value>,
) -> Vec<(PinId, Value)> {
    let Some(node) = graph.node(node_id) else {
        return Vec::new();
    };
    let mut result = Vec::new();
    for pin in &node.inputs {
        for link in &graph.links {
            if link.to == pin.id {
                if let Some(v) = values_at_pins.get(&link.from) {
                    result.push((pin.id, v.clone()));
                }
                break;
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::Layout2d;
    use crate::model::Graph;

    #[test]
    fn topological_order_respects_dependencies() {
        let mut g = Graph::<(), ()>::new();
        let a = g.add_node((), Layout2d::default(), 0, 1);
        let b = g.add_node((), Layout2d::default(), 1, 1);
        let c = g.add_node((), Layout2d::default(), 1, 0);
        let oa = g.node(a).unwrap().outputs[0].id;
        let ib = g.node(b).unwrap().inputs[0].id;
        let ob = g.node(b).unwrap().outputs[0].id;
        let ic = g.node(c).unwrap().inputs[0].id;
        g.connect(oa, ib, ()).unwrap();
        g.connect(ob, ic, ()).unwrap();
        let order = compute_topological_order(&g);
        let pos_a = order.iter().position(|&n| n == a).unwrap();
        let pos_b = order.iter().position(|&n| n == b).unwrap();
        let pos_c = order.iter().position(|&n| n == c).unwrap();
        assert!(pos_a < pos_b && pos_b < pos_c);
    }
}
