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
