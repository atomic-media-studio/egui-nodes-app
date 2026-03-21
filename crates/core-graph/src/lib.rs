//! **core-graph** — headless [`Graph`], [`Node`], [`Link`], and stable pin/link ids.
//! No GUI or windowing dependencies (suitable for CLIs and other frontends).

pub mod error;
pub mod ids;
pub mod layout;
pub mod model;
pub mod serde;

pub use error::GraphError;
pub use ids::{LinkId, NodeId, PinId};
pub use layout::Layout2d;
pub use model::{Graph, Link, Node, Pin, PinKind};
