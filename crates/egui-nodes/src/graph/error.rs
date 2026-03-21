use core::fmt;

use super::id::{LinkId, NodeId, PinId};

/// Headless graph errors (no UI).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphError {
    UnknownNode(NodeId),
    UnknownPin(PinId),
    UnknownLink(LinkId),
    PinKindMismatch { pin: PinId, expected_input: bool },
    NotOutputPin(PinId),
    NotInputPin(PinId),
    DuplicateLink {
        from: PinId,
        to: PinId,
    },
    SelfLoop,
}

impl fmt::Display for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownNode(id) => write!(f, "unknown node {:?}", id.0),
            Self::UnknownPin(id) => write!(f, "unknown pin {:?}", id.0),
            Self::UnknownLink(id) => write!(f, "unknown link {:?}", id.0),
            Self::PinKindMismatch { .. } => write!(f, "pin kind mismatch"),
            Self::NotOutputPin(id) => write!(f, "expected output pin, got {:?}", id.0),
            Self::NotInputPin(id) => write!(f, "expected input pin, got {:?}", id.0),
            Self::DuplicateLink { .. } => write!(f, "duplicate link"),
            Self::SelfLoop => write!(f, "cannot link a pin to itself"),
        }
    }
}

impl std::error::Error for GraphError {}
