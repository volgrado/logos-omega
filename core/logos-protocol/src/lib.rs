#![no_std] // Critical for WASM/Embedded compatibility

extern crate alloc;

// Enable std if the feature is active (for tests/tools)
#[cfg(feature = "std")]
extern crate std;

pub mod ids;
pub mod morphology;

// Re-export core types for convenience
pub use ids::{LemmaId, TokenId, SentenceId, ParadigmId};
pub use morphology::*;

pub mod model;
pub use model::*;

#[cfg(test)]
mod tests {
    use super::*;
    use rkyv::{to_bytes, from_bytes};

    #[test]
    fn test_enum_serialization() {
        // Test basic enum round-trip
        let original = Case::Accusative;
        
        // Serialize
        let bytes = to_bytes::<_, 256>(&original).expect("Failed to serialize Case");
        
        // Deserialize (Simulate loading from disk)
        let deserialized: Case = from_bytes(&bytes).expect("Failed to deserialize Case");
        
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_id_serialization() {
        // Test NewType ID round-trip
        let original = LemmaId::new(42);
        
        let bytes = to_bytes::<_, 256>(&original).expect("Failed to serialize LemmaId");
        let deserialized: LemmaId = from_bytes(&bytes).expect("Failed to deserialize LemmaId");
        
        assert_eq!(original, deserialized);
    }
    
    #[test]
    fn test_id_layout() {
        // Verify Zero-Cost abstraction: LemmaId(u32) should be exactly 4 bytes
        assert_eq!(core::mem::size_of::<LemmaId>(), 4);
        assert_eq!(core::mem::size_of::<Option<LemmaId>>(), 8); // u32 + tag (padding)
    }
}
