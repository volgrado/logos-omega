use rkyv::{Archive, Deserialize, Serialize};

#[cfg(feature = "serde")]
use serde::{Deserialize as SerdeDeserialize, Serialize as SerdeSerialize};

use bitflags::bitflags;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Archive, Serialize, Deserialize)]
#[cfg_attr(feature = "serde", derive(SerdeDeserialize, SerdeSerialize))]
#[archive(check_bytes)]
#[repr(u8)]
pub enum Case {
    Nominative = 0,
    Genitive = 1,
    Accusative = 2,
    Vocative = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Archive, Serialize, Deserialize)]
#[cfg_attr(feature = "serde", derive(SerdeDeserialize, SerdeSerialize))]
#[archive(check_bytes)]
#[repr(u8)]
pub enum Gender {
    Masculine = 0,
    Feminine = 1,
    Neuter = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Archive, Serialize, Deserialize)]
#[cfg_attr(feature = "serde", derive(SerdeDeserialize, SerdeSerialize))]
#[archive(check_bytes)]
#[repr(u8)]
pub enum Number {
    Singular = 0,
    Plural = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Archive, Serialize, Deserialize)]
#[cfg_attr(feature = "serde", derive(SerdeDeserialize, SerdeSerialize))]
#[archive(check_bytes)]
#[repr(u8)]
pub enum Person {
    First = 1,
    Second = 2,
    Third = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Archive, Serialize, Deserialize)]
#[cfg_attr(feature = "serde", derive(SerdeDeserialize, SerdeSerialize))]
#[archive(check_bytes)]
#[repr(u8)]
pub enum Voice {
    Active = 0,
    Passive = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Archive, Serialize, Deserialize)]
#[cfg_attr(feature = "serde", derive(SerdeDeserialize, SerdeSerialize))]
#[archive(check_bytes)]
#[repr(u8)]
pub enum Tense {
    Present = 0,
    Future = 1,
    Aorist = 2,
    Imperfect = 3,
    Perfect = 4,
    Pluperfect = 5,
    FuturePerfect = 6,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Archive, Serialize, Deserialize)]
#[cfg_attr(feature = "serde", derive(SerdeDeserialize, SerdeSerialize))]
#[archive(check_bytes)]
#[repr(u8)]
pub enum Mood {
    Indicative = 0,
    Subjunctive = 1,
    Imperative = 2,
    Participle = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Archive, Serialize, Deserialize)]
#[cfg_attr(feature = "serde", derive(SerdeDeserialize, SerdeSerialize))]
#[archive(check_bytes)]
#[repr(u8)]
pub enum PartOfSpeech {
    Noun = 0,
    Adjective = 1,
    Verb = 2,
    Adverb = 3,
    Article = 4,
    Preposition = 5,
    Conjunction = 6,
    Pronoun = 7,
    Particle = 8,
    Numeral = 9,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[cfg_attr(feature = "serde", derive(SerdeDeserialize, SerdeSerialize))]
    pub struct MorphFlags: u32 {
        // Case (Bits 0-3)
        const NOMINATIVE = 1;
        const GENITIVE = 2;
        const ACCUSATIVE = 4;
        const VOCATIVE = 8;

        // Gender (Bits 4-6)
        const MASCULINE = 16;
        const FEMININE = 32;
        const NEUTER = 64;

        // Number (Bits 7-8)
        const SINGULAR = 128;
        const PLURAL = 256;

        // Person (Bits 9-11)
        const FIRST_PERSON = 512;
        const SECOND_PERSON = 1024;
        const THIRD_PERSON = 2048;

        // Voice (Bits 12-13)
        const ACTIVE = 4096;
        const PASSIVE = 8192;
        
        // Tense (Bits 14-16)
        const PRESENT = 16384;
        const PAST = 32768;
        const FUTURE = 65536;
    }
}

// rkyv support for MorphFlags
impl Archive for MorphFlags {
    type Archived = u32;
    type Resolver = ();

    unsafe fn resolve(&self, _pos: usize, _resolver: Self::Resolver, out: *mut Self::Archived) {
        out.write(self.bits());
    }
}

impl<S: rkyv::ser::Serializer + ?Sized> Serialize<S> for MorphFlags {
    fn serialize(&self, _serializer: &mut S) -> Result<Self::Resolver, S::Error> {
        Ok(())
    }
}

impl<D: rkyv::Fallible + ?Sized> Deserialize<MorphFlags, D> for u32 {
    fn deserialize(&self, _deserializer: &mut D) -> Result<MorphFlags, D::Error> {
        Ok(MorphFlags::from_bits(*self).unwrap_or_else(|| {
            MorphFlags::from_bits_truncate(*self)
        }))
    }
}
