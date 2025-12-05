pub mod parser;
pub mod token;
pub mod syntax;
pub mod morphology;

use logos_protocol::{Dictionary, LemmaId};
use rkyv::Archived;
use crate::token::{Token, TokenKind};
use crate::parser::{parse_with_spans, RawToken};

pub struct Lexer<'a> {
    dict: &'a Archived<Dictionary>,
}

impl<'a> Lexer<'a> {
    pub fn new(dict: &'a Archived<Dictionary>) -> Self {
        Self { dict }
    }

    /// Primary entry point: Text -> Structured Tokens
    pub fn tokenize(&self, input: &'a str) -> Vec<Token<'a>> {
        let raw_tokens = parse_with_spans(input);

        raw_tokens
            .into_iter()
            .map(|(span, raw)| {
                let text = &input[span.start..span.end];
                
                let kind = match raw {
                    RawToken::Punct(c) => TokenKind::Punctuation(c),
                    RawToken::Word(s) => {
                        // Lookup in Dictionary
                        if let Some(lemma_id) = self.lookup_lemma(s) {
                            TokenKind::Word(lemma_id)
                        } else {
                            TokenKind::UnknownWord
                        }
                    }
                };

                Token { span, text, kind }
            })
            .collect()
    }

    /// Linear scan lookup (O(N)) - MVP only.
    /// Phase 5 Optimization: Replace with FST or Hash lookup.
    fn lookup_lemma(&self, surface_form: &str) -> Option<LemmaId> {
        // Iterate over archived lemmas
        for lemma in self.dict.lemmas.iter() {
            let stem = lemma.text.as_str();
            
            // 1. Exact Match (e.g., indeclinable words)
            if stem == surface_form {
                let val: u32 = lemma.id.0;
                return Some(LemmaId(val));
            }

            // 2. Stem Match (e.g., "άνθρωπ" matches "άνθρωπος")
            // In a real engine, we would validate the suffix against the paradigm here.
            // For this phase, if it starts with the stem, we count it!
            if surface_form.starts_with(stem) {
                 let val: u32 = lemma.id.0;
                 return Some(LemmaId(val));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use logos_protocol::{Lemma, Gender};
    use rkyv::{to_bytes};

    #[test]
    fn test_tokenizer_integration() {
        // 1. Setup Mock Dictionary
        let dict = logos_protocol::Dictionary {
            version: 1,
            lemmas: vec![
                Lemma { 
                    id: LemmaId(10), 
                    text: "άνθρωπος".to_string(), 
                    gender: Gender::Masculine,
                    pos: logos_protocol::PartOfSpeech::Noun,
                }
            ],
            paradigms: vec![],
        };

        // Serialize to bytes (simulating loading dict.rkyv)
        let bytes = to_bytes::<_, 256>(&dict).unwrap();
        let archived = unsafe { rkyv::archived_root::<logos_protocol::Dictionary>(&bytes) };

        // 2. Run Lexer
        let lexer = Lexer::new(archived);
        let input = "Ο άνθρωπος."; // Note: "Ο" is unknown, "άνθρωπος" is known, "." is punct
        let tokens = lexer.tokenize(input);

        // 3. Assertions
        assert_eq!(tokens.len(), 3);
        
        // Token 0: "Ο" (Unknown in this tiny dict)
        assert_eq!(tokens[0].text, "Ο");
        matches!(tokens[0].kind, TokenKind::UnknownWord);

        // Token 1: "άνθρωπος" (Known ID 10)
        assert_eq!(tokens[1].text, "άνθρωπος");
        if let TokenKind::Word(id) = tokens[1].kind {
            assert_eq!(id.0, 10);
        } else {
            panic!("Expected Word(10)");
        }

        // Token 2: "."
        matches!(tokens[2].kind, TokenKind::Punctuation('.'));
    }
}
