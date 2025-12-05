use wasm_bindgen::prelude::*;
use logos_protocol::Dictionary;
use logos_parser::Lexer;
use logos_ecs::{LogosWorld, systems::agreement::AgreementError};
use logos_solver::{SemanticGraph, validate_semantics};
use serde::Serialize;

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

/// The structured response sent back to JavaScript/React
#[derive(Serialize)]
pub struct TokenDebug {
    pub text: String,
    pub lemma_id: Option<u32>,
    pub kind: String,
    pub morphology: String,
    pub debug: String,
}

#[derive(Serialize)]
pub struct AnalysisReport {
    pub tokens: Vec<TokenDebug>,
    pub syntax_errors: Vec<SerializableAgreementError>,
    pub semantic_errors: Vec<String>,
    pub debug_info: String,
}

#[derive(Serialize)]
pub struct SerializableAgreementError {
    pub source: String,
    pub target: String,
    pub message: String,
}

impl From<AgreementError> for SerializableAgreementError {
    fn from(e: AgreementError) -> Self {
        Self {
            source: e.source,
            target: e.target,
            message: e.details,
        }
    }
}

/// The Engine Instance running in the Browser
#[wasm_bindgen]
pub struct LogosEngine {
    // We own the raw binary of the dictionary (loaded via fetch() in JS)
    data: Vec<u8>,
}

#[wasm_bindgen]
impl LogosEngine {
    #[wasm_bindgen(constructor)]
    pub fn new(data: Vec<u8>) -> Self {
        // In a production app, we would validate the RKYV archive here using rkyv::check_archived_root
        Self { data }
    }

    /// The Main Loop: Text -> Lexer -> ECS -> Solver -> JSON
    pub fn analyze(&self, input: &str) -> JsValue {
        let report = self.analyze_core(input);
        serde_wasm_bindgen::to_value(&report).unwrap()
    }
}

impl LogosEngine {
    /// Pure Rust analysis (No WASM dependencies in return type)
    pub fn analyze_core(&self, input: &str) -> AnalysisReport {
        // 1. Zero-Copy Load of Dictionary
        // We assume the data is valid. In prod, use check_archived_root.
        let dict = unsafe { rkyv::archived_root::<Dictionary>(&self.data) };

        // 2. Lexical Analysis (Text -> Tokens)
        let lexer = Lexer::new(dict);
        let tokens = lexer.tokenize(input);
        
        let debug_tokens: Vec<TokenDebug> = tokens.iter().map(|t| {
            let mut lemma_id_opt = None;
            let mut morph_desc = "None".to_string();
            let mut kind_str = "Other".to_string();
            let mut debug_msg = String::new();
            let mut is_resolved = false;

            // Try to resolve morphology for BOTH Word and UnknownWord
            // This bypasses Lexer limitations for the playground
            if let logos_parser::token::TokenKind::Word(id) = t.kind {
                lemma_id_opt = Some(id.0);
                kind_str = "Word".to_string();
                is_resolved = true;
            } else if let logos_parser::token::TokenKind::UnknownWord = t.kind {
                kind_str = "Unknown".to_string();
                // We will try to resolve it below anyway
                is_resolved = true; 
            } else if let logos_parser::token::TokenKind::Punctuation(_) = t.kind {
                kind_str = "Punctuation".to_string();
            }

            if is_resolved {
                 let (morph, debug) = resolve_morphology(dict, &t.text, lemma_id_opt);
                 morph_desc = morph;
                 debug_msg = debug;
                 
                 // If we found a match and it was unknown, recover it
                 if morph_desc != "None" && kind_str == "Unknown" {
                     // We need to find the lemma ID again if we only have the text
                     // This is a bit inefficient but works for recovery
                     if let Some(lemma) = dict.lemmas.iter().find(|l| t.text.starts_with(&*l.text) || l.text.starts_with(&t.text)) {
                         kind_str = "Word (Recovered)".to_string();
                         lemma_id_opt = Some(lemma.id.0);
                     }
                 }
            }

            TokenDebug {
                text: t.text.to_string(),
                lemma_id: lemma_id_opt,
                kind: kind_str,
                morphology: morph_desc,
                debug: debug_msg,
            }
        }).collect();

        // 3. ECS Simulation (Tokens -> Entities)
        let mut world = LogosWorld::new();
        let mut entities = Vec::new();

        for token in &tokens {
            let mut lemma_id_opt = None;
            let mut flags = logos_protocol::MorphFlags::empty();

            if let logos_parser::token::TokenKind::Word(id) = token.kind {
                lemma_id_opt = Some(id);
                
                // Resolve Flags (Re-use logic or simplify for ECS?)
                // For now, let's just re-run resolution or trust the debug tokens?
                // The ECS needs the actual flags enum, not string.
                // Let's duplicate the lookup for now to keep ECS independent of debug view
                if let Some(lemma) = dict.lemmas.iter().find(|l| l.id.0 == id.0) {
                     for paradigm in dict.paradigms.iter() {
                        for (rule_flags, rule_suffix) in paradigm.endings.iter() {
                            if token.text.ends_with(rule_suffix.as_str()) {
                                let stem_len = token.text.len() - rule_suffix.as_str().len();
                                let candidate_stem = &token.text[..stem_len];
                                if lemma.text.starts_with(candidate_stem) {
                                    flags = logos_protocol::MorphFlags::from_bits_truncate(*rule_flags);
                                    break;
                                }
                            }
                        }
                        if !flags.is_empty() { break; }
                    }
                }
            }
            entities.push(world.add_token(token.text.to_string(), lemma_id_opt, flags));
        }

        // Mock Syntax: If 2 words, assume "Det Noun" structure
        if entities.len() == 2 {
            use logos_ecs::components::DependencyRole;
            // 0 modifies 1
            world.set_dependency(entities[0], entities[1], DependencyRole::Modifier);
        }

        let syntax_errors_raw = world.validate();
        let syntax_errors: Vec<SerializableAgreementError> = syntax_errors_raw
            .into_iter()
            .map(|e| e.into())
            .collect();

        // 5. Semantic Validation (Meaning)
        // We construct a blank graph for now. 
        // Phase 8 would load this graph from the Dictionary struct.
        let graph = SemanticGraph::new();
        let semantic_errors_raw = validate_semantics(&world, &graph);
        let semantic_errors: Vec<String> = semantic_errors_raw
            .into_iter()
            .map(|e| e.message)
            .collect();

        // 6. Serialize and Return
        AnalysisReport {
            tokens: debug_tokens,
            syntax_errors,
            semantic_errors,
            debug_info: format!("Lemmas: {}, Paradigms: {}", dict.lemmas.len(), dict.paradigms.len()),
        }
    }
}

use rkyv::Archived;

/// Helper: Robust Morphology Resolution
fn resolve_morphology(dict: &Archived<Dictionary>, token_text: &str, known_lemma_id: Option<u32>) -> (String, String) {
    let mut morph_desc = "None".to_string();
    let mut debug_msg = String::new();

    for lemma in dict.lemmas.iter() {
        // Optimization: If we know the lemma ID, only check that one
        if let Some(id) = known_lemma_id {
            if lemma.id.0 != id { continue; }
        }

        for paradigm in dict.paradigms.iter() {
            for (flags_bits, rule_suffix) in paradigm.endings.iter() {
                let suffix_str = rule_suffix.as_str();
                if token_text.ends_with(suffix_str) {
                    let stem_len = token_text.len() - suffix_str.len();
                    let candidate_stem = &token_text[..stem_len];
                    
                    if lemma.text.starts_with(candidate_stem) {
                            let flags = logos_protocol::MorphFlags::from_bits_truncate(*flags_bits);
                            morph_desc = format!("{:?}", flags);
                            debug_msg = format!("Matched! Stem: '{}', Suffix: '{}', Lemma: '{}'", candidate_stem, suffix_str, lemma.text);
                            return (morph_desc, debug_msg);
                    }
                }
            }
        }
    }
    
    if morph_desc == "None" {
        debug_msg = format!("No match found for '{}'", token_text);
    }

    (morph_desc, debug_msg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use logos_protocol::{Dictionary, Lemma, Paradigm, Gender, LemmaId, PartOfSpeech, ParadigmId};
    use rkyv::to_bytes;

    #[test]
    fn test_robust_morphology_resolution() {
        // 1. Setup Mock Dictionary
        // Lemma: "άνθρωπος" (Full word stored)
        // Paradigm: Suffix "ος" -> Nom|Sg|Masc
        let lemma = Lemma {
            id: LemmaId(1),
            text: "άνθρωπος".to_string(),
            gender: Gender::Masculine,
            pos: PartOfSpeech::Noun,
        };

        let paradigm = Paradigm {
            id: ParadigmId(1),
            endings: vec![
                ((logos_protocol::MorphFlags::NOMINATIVE | logos_protocol::MorphFlags::SINGULAR | logos_protocol::MorphFlags::MASCULINE).bits(), "ος".to_string())
            ],
        };

        let dict = Dictionary {
            version: 1,
            lemmas: vec![lemma],
            paradigms: vec![paradigm],
        };

        // Serialize to bytes (simulating loading dict.rkyv)
        let bytes = to_bytes::<_, 256>(&dict).unwrap();
        let archived = unsafe { rkyv::archived_root::<Dictionary>(&bytes) };

        // 2. Test Case: Input "άνθρωπος"
        // Should match suffix "ος", leaving stem "άνθρωπ".
        // Lemma "άνθρωπος" starts with "άνθρωπ". -> MATCH.
        let (morph, debug) = resolve_morphology(archived, "άνθρωπος", Some(1));

        assert_ne!(morph, "None", "Should have resolved morphology");
        assert!(debug.contains("Matched!"), "Debug should indicate match");
        assert!(debug.contains("Stem: 'άνθρωπ'"), "Should identify correct stem");
    }

    #[test]
    fn test_integration_with_real_dict() {
        use std::fs;
        use std::path::PathBuf;

        // Locate the dictionary file relative to the crate root
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("www");
        path.push("dict_v10.rkyv");

        if !path.exists() {
            println!("Skipping integration test: Dictionary not found at {:?}", path);
            return;
        }

        println!("Loading dictionary from {:?}", path);
        let data = fs::read(path).expect("Failed to read dictionary file");
        
        // Initialize Engine
        let engine = LogosEngine::new(data);
        
        // Run Analysis: "απάνθρωπος" (Cruel/Inhuman)
        // We use this because "άνθρωπος" is missing from the current dict_v10.rkyv
        let report = engine.analyze_core("απάνθρωπος");
        
        // Verify Results
        let token = &report.tokens[0];
        assert_eq!(token.text, "απάνθρωπος");
        
        // We expect it to be resolved now with the robust matching
        assert_ne!(token.kind, "Unknown", "Should be resolved as Word (or Recovered)");
        assert_ne!(token.morphology, "None", "Morphology should be resolved");
        
        println!("Successfully analyzed '{}'", token.text);
        println!("Debug Info: {}", token.debug);
        println!("Morphology: {}", token.morphology);
    }
}
