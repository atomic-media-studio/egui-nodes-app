//! JSON load/save helpers for serializable graphs (e.g. [`core_graph::Graph`] with `serde`).

use std::path::Path;

use anyhow::Context;
use serde::Serialize;
use serde::de::DeserializeOwned;

/// Pretty JSON snapshot of any serializable value (e.g. [`core_graph::Graph`]).
pub fn save_graph<G: Serialize>(graph: &G, path: impl AsRef<Path>) -> anyhow::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(graph)?;
    std::fs::write(path, json).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub fn load_graph<G: DeserializeOwned>(path: impl AsRef<Path>) -> anyhow::Result<G> {
    let path = path.as_ref();
    let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_slice(&bytes).context("parse graph JSON")
}
