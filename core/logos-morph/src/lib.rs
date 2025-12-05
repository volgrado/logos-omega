#![no_std]

#[macro_use]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

use alloc::string::String;
use alloc::format;

use logos_protocol::{Lemma, Paradigm, MorphFlags};

use core::fmt;

#[derive(Debug)]
pub enum MorphError {
    FormNotFound(MorphFlags),
}

impl fmt::Display for MorphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MorphError::FormNotFound(flags) => write!(f, "Form not found for flags: {:?}", flags),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for MorphError {}

/// Generates a word form by combining Lemma stem and Paradigm suffix.
pub fn generate(
    lemma: &Lemma,
    paradigm: &Paradigm,
    flags: MorphFlags,
) -> Result<String, MorphError> {
    // In rkyv, vectors are ArchivedVec, but for now we use standard types 
    // assuming we deserialized or are using the unarchived version.
    // If using Archived version, the signature would change. 
    // For Phase 3 MVP, we work with the standard structs.

    for (rule_flags, suffix) in &paradigm.endings {
        // Exact match check
        if *rule_flags == flags.bits() {
            return Ok(format!("{}{}", lemma.text, suffix));
        }
    }

    Err(MorphError::FormNotFound(flags))
}

#[cfg(test)]
mod tests {
    use super::*;
    use logos_protocol::{LemmaId, Gender, ParadigmId};
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_safe_generation(stem in "[a-z]+", suffix in "[a-z]+") {
            let lemma = Lemma { 
                id: LemmaId(1), 
                text: stem.clone(), 
                gender: Gender::Neuter 
            };
            
            // Arbitrary flags
            let flags = MorphFlags::NOMINATIVE;
            
            let paradigm = Paradigm {
                id: ParadigmId(1),
                endings: vec![(flags.bits(), suffix.clone())]
            };
            
            let result = generate(&lemma, &paradigm, flags);
            assert_eq!(result.unwrap(), format!("{}{}", stem, suffix));
        }
    }
}
