use hecs::Entity;
use logos_protocol::{LemmaId, MorphFlags};

/// Basic data about the token (Source of Truth)
#[derive(Debug, Clone)]
pub struct TokenData {
    pub text: String,
    pub lemma_id: Option<LemmaId>,
}

/// Grammatical state (Bitflags wrapper)
/// We wrap it in a struct so it can be a distinct Component in hecs
#[derive(Debug, Clone, Copy)]
pub struct Morphology {
    pub flags: MorphFlags,
}

impl Morphology {
    pub fn new(flags: MorphFlags) -> Self {
        Self { flags }
    }
}

/// The Syntactic Role of a word
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyRole {
    Root,
    Subject,
    Object,
    Modifier,
    PrepositionArg,
    IndirectObject,
    Coordinator,
    Conjunct,
    PassiveAgent,
    AbsoluteClause,
    Complement,
    RelativeClause,
    // Add more as needed (IndirectObj, etc.)
}

/// The Syntactic Tree Structure
#[derive(Debug, Clone, Copy)]
pub struct Syntax {
    pub head: Entity, // The parent node in the dependency tree
    pub role: DependencyRole,
}
