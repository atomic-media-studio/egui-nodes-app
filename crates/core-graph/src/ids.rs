//! Stable identifiers — not UI slab indices.

use std::num::NonZeroU32;

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
        #[repr(transparent)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[cfg_attr(feature = "serde", serde(transparent))]
        pub struct $name(NonZeroU32);

        impl $name {
            #[must_use]
            pub const fn from_raw(raw: u32) -> Option<Self> {
                match NonZeroU32::new(raw) {
                    Some(n) => Some(Self(n)),
                    None => None,
                }
            }

            #[must_use]
            pub const fn get(self) -> u32 {
                self.0.get()
            }
        }
    };
}

id_type!(NodeId);
id_type!(PinId);
id_type!(LinkId);
