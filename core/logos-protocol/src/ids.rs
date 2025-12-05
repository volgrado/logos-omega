use rkyv::{Archive, Deserialize, Serialize};

#[cfg(feature = "serde")]
use serde::{Deserialize as SerdeDeserialize, Serialize as SerdeSerialize};

macro_rules! define_id {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Archive, Serialize, Deserialize)]
        #[cfg_attr(feature = "serde", derive(SerdeDeserialize, SerdeSerialize))]
        #[archive(check_bytes)]
        #[repr(transparent)] // Ensure it has the same layout as u32
        pub struct $name(pub u32);

        impl $name {
            pub const fn new(id: u32) -> Self {
                Self(id)
            }
        }

        impl From<u32> for $name {
            fn from(id: u32) -> Self {
                Self(id)
            }
        }
        
        impl From<$name> for u32 {
            fn from(id: $name) -> u32 {
                id.0
            }
        }
    };
}

define_id!(LemmaId, "Unique identifier for a Lemma (dictionary headword).");
define_id!(ParadigmId, "Unique identifier for an inflectional paradigm.");
define_id!(TokenId, "Unique identifier for a parsed token in a sentence.");
define_id!(SentenceId, "Unique identifier for a processed sentence.");
