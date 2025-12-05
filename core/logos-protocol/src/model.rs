use rkyv::{Archive, Deserialize, Serialize};
use crate::ids::{LemmaId, ParadigmId};
use crate::morphology::{Gender, PartOfSpeech};
use alloc::string::String;
use alloc::vec::Vec;

#[cfg(feature = "serde")]
use serde::{Deserialize as SerdeDeserialize, Serialize as SerdeSerialize};

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[cfg_attr(feature = "serde", derive(SerdeDeserialize, SerdeSerialize))]
#[archive(check_bytes)]
pub struct Lemma {
    pub id: LemmaId,
    pub text: String,
    pub gender: Gender,
    pub pos: PartOfSpeech,
    // Future: pub paradigm_id: ParadigmId,
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[cfg_attr(feature = "serde", derive(SerdeDeserialize, SerdeSerialize))]
#[archive(check_bytes)]
pub struct Paradigm {
    pub id: ParadigmId,
    pub endings: Vec<(u32, String)>,
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[cfg_attr(feature = "serde", derive(SerdeDeserialize, SerdeSerialize))]
#[archive(check_bytes)]
pub struct Dictionary {
    pub version: u32,
    pub lemmas: Vec<Lemma>,
    pub paradigms: Vec<Paradigm>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Archive, Serialize, Deserialize)]
#[cfg_attr(feature = "serde", derive(SerdeDeserialize, SerdeSerialize))]
#[archive(check_bytes)]
#[repr(u8)]
pub enum Relation {
    IsA = 0,                // Inheritance (e.g., Apple IsA Fruit)
    RequiresAttribute = 1,  // Constraint (e.g., Eat RequiresAttribute Edible)
    HasAttribute = 2,       // Property (e.g., Fruit HasAttribute Edible)
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[cfg_attr(feature = "serde", derive(SerdeDeserialize, SerdeSerialize))]
#[archive(check_bytes)]
pub struct SemanticEdge {
    pub from: LemmaId,
    pub to: LemmaId,
    pub relation: Relation,
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[cfg_attr(feature = "serde", derive(SerdeDeserialize, SerdeSerialize))]
#[archive(check_bytes)]
pub struct SemanticNetwork {
    pub version: u32,
    pub edges: Vec<SemanticEdge>,
}
