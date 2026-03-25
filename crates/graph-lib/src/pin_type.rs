//! Logical datatype for each [`Pin`](crate::model::Pin); used when validating [`Graph::connect`].

/// Data-type tag for a pin. [`Self::Any`] is a wildcard on either end of a link (`output → input`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PinType {
    #[default]
    Any,
    Bool,
    Int,
    Float,
    Bang,
    Symbol,
    List,
}

impl PinType {
    /// Whether an output of type `from` may connect to an input of type `to`.
    #[inline]
    #[must_use]
    pub fn compatible_link(from_output: Self, to_input: Self) -> bool {
        from_output == to_input
            || from_output == Self::Any
            || to_input == Self::Any
    }
}
