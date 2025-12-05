from enum import Enum
from pydantic import BaseModel

class Gender(str, Enum):
    Masculine = "Masculine"
    Feminine = "Feminine"
    Neuter = "Neuter"

class PartOfSpeech(str, Enum):
    Noun = "Noun"
    Adjective = "Adjective"
    Verb = "Verb"
    Adverb = "Adverb"
    Article = "Article"
    Preposition = "Preposition"
    Conjunction = "Conjunction"
    Pronoun = "Pronoun"
    Particle = "Particle"
    Numeral = "Numeral"

class Lemma(BaseModel):
    id: int
    text: str
    gender: Gender
    pos: PartOfSpeech = PartOfSpeech.Noun # Default for backward compatibility during migration

class Paradigm(BaseModel):
    id: int
    # List of (MorphFlags as int, Suffix string)
    endings: list[tuple[int, str]]

class Dictionary(BaseModel):
    version: int = 1
    lemmas: list[Lemma]
    paradigms: list[Paradigm]

# --- Configuration Models ---

class ParadigmTrigger(BaseModel):
    template: str | None = None
    suffix: str | None = None
    gender: Gender | None = None

class ParadigmEnding(BaseModel):
    flags: int
    suffix: str

class ParadigmConfig(BaseModel):
    id: int
    name: str
    pos: str | None = "Noun" # Default to Noun for now
    example: str | None = None
    triggers: list[ParadigmTrigger] = []
    endings: list[ParadigmEnding]

class MorphologyConfig(BaseModel):
    paradigms: list[ParadigmConfig]
