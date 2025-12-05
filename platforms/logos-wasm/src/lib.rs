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

            if let logos_parser::token::TokenKind::Word(id) = t.kind {
                lemma_id_opt = Some(id.0);
                kind_str = "Word".to_string();

                // --- NEW LOGIC: Resolve Morphology ---
                // 1. Find the Lemma in the Dictionary (Linear scan for MVP)
                if let Some(lemma) = dict.lemmas.iter().find(|l| l.id.0 == id.0) {
                    
                    // 2. Calculate the Suffix (Input - Stem)
                    // Input: "άνθρωπος", Stem: "άνθρωπ" -> Suffix: "ος"
                    let stem = lemma.text.as_str();
                    if t.text.starts_with(stem) {
                        let suffix = &t.text[stem.len()..];
                        
                        // 3. Find the Paradigm
                        // Phase 9 Fix: Iterate ALL paradigms since we haven't linked Lemma->Paradigm yet.
                        for paradigm in dict.paradigms.iter() {
                            
                            // 4. Look for suffix match
                            for (flags_bits, rule_suffix) in paradigm.endings.iter() {
                                if rule_suffix.as_str() == suffix {
                                    // Found it! Convert bits to readable string
                                    let flags = logos_protocol::MorphFlags::from_bits_truncate(*flags_bits);
                                    morph_desc = format!("{:?}", flags); 
                                    break;
                                }
                            }
                            if morph_desc != "None" { break; }
                        }
                    }
                }
            } else if let logos_parser::token::TokenKind::UnknownWord = t.kind {
                kind_str = "Unknown".to_string();
            } else if let logos_parser::token::TokenKind::Punctuation(_) = t.kind {
                kind_str = "Punctuation".to_string();
            }

            TokenDebug {
                text: t.text.to_string(),
                lemma_id: lemma_id_opt,
                kind: kind_str,
                morphology: morph_desc,
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
                
                // Resolve Flags
                if let Some(lemma) = dict.lemmas.iter().find(|l| l.id.0 == id.0) {
                    let stem = lemma.text.as_str();
                    if token.text.starts_with(stem) {
                        let suffix = &token.text[stem.len()..];
                        for paradigm in dict.paradigms.iter() {
                            for (rule_flags, rule_suffix) in paradigm.endings.iter() {
                                if rule_suffix.as_str() == suffix {
                                    flags = logos_protocol::MorphFlags::from_bits_truncate(*rule_flags);
                                    break;
                                }
                            }
                        }
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
        let report = AnalysisReport {
            tokens: debug_tokens,
            syntax_errors,
            semantic_errors,
            debug_info: format!("Lemmas: {}, Paradigms: {}", dict.lemmas.len(), dict.paradigms.len()),
        };

        serde_wasm_bindgen::to_value(&report).unwrap()
    }
}
