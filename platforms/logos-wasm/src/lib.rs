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
    // Optional loaded Semantic Graph
    semantic_graph: Option<SemanticGraph>,
}

#[wasm_bindgen]
impl LogosEngine {
    #[wasm_bindgen(constructor)]
    pub fn new(data: Vec<u8>) -> Self {
        // In a production app, we would validate the RKYV archive here using rkyv::check_archived_root
        Self { 
            data,
            semantic_graph: None,
        }
    }

    pub fn load_semantics(&mut self, data: Vec<u8>) {
        // Assume trusted input for now, consistent with Dictionary loading
        let archived = unsafe { rkyv::archived_root::<logos_protocol::SemanticNetwork>(&data) };
        self.semantic_graph = Some(SemanticGraph::from_archived(archived));
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
        let dict = unsafe { rkyv::archived_root::<Dictionary>(&self.data) };

        // 2. Lexical Analysis (Text -> Tokens)
        let lexer = Lexer::new(dict);
        let tokens = lexer.tokenize(input);
        
        // 3. Morphology Resolution (Unified Pipeline)
        struct AnalyzedToken<'a> {
            text: &'a str,
            analysis: logos_parser::morphology::MorphAnalysis,
        }

        let analyzed_tokens: Vec<AnalyzedToken> = tokens.iter().map(|t| {
            // Check for Punctuation first to avoid unnecessary dictionary lookup
            if let logos_parser::token::TokenKind::Punctuation(_) = t.kind {
                 return AnalyzedToken {
                    text: &t.text,
                    analysis: logos_parser::morphology::MorphAnalysis {
                        flags: logos_protocol::MorphFlags::empty(),
                        lemma_id: None,
                        debug_msg: "Punctuation".to_string(),
                        stem: String::new(),
                        kind: "Punctuation".to_string(),
                    }
                };
            }

            let mut known_id = None;
            if let logos_parser::token::TokenKind::Word(id) = t.kind {
                known_id = Some(id.0);
            }

            let analysis = logos_parser::morphology::resolve_morphology(dict, &t.text, known_id);
            
            AnalyzedToken {
                text: &t.text,
                analysis,
            }
        }).collect();

        // 4. Transform for Output (TokenDebug)
        let debug_tokens: Vec<TokenDebug> = analyzed_tokens.iter().map(|at| {
             let morph_str = if at.analysis.flags.is_empty() {
                 "None".to_string()
             } else {
                 format!("{:?}", at.analysis.flags)
             };

             TokenDebug {
                text: at.text.to_string(),
                lemma_id: at.analysis.lemma_id.map(|id| id.0),
                kind: at.analysis.kind.clone(),
                morphology: morph_str,
                debug: at.analysis.debug_msg.clone(),
            }
        }).collect();

        // 5. ECS Simulation (Tokens -> Entities)
        let mut world = LogosWorld::new();
        let mut entities = Vec::new();

        for at in &analyzed_tokens {
            entities.push(
                world.add_token(
                    at.text.to_string(), 
                    at.analysis.lemma_id, 
                    at.analysis.flags
                )
            );
        }

        // 6. Syntactic Parsing
        // Construct MorphTokens for parser input
        let parser_input: Vec<logos_parser::syntax::MorphToken> = analyzed_tokens.iter().map(|at| {
            logos_parser::syntax::MorphToken {
                text: at.text,
                flags: at.analysis.flags,
            }
        }).collect();

        let dependencies = logos_parser::syntax::parse_greedy(&parser_input);
        
        for dep in dependencies {
            if dep.dependent_index < entities.len() && dep.head_index < entities.len() {
                let child_entity = entities[dep.dependent_index];
                let head_entity = entities[dep.head_index];
                
                use logos_parser::syntax::SyntaxRole;
                use logos_ecs::components::DependencyRole;

                let role = match dep.role {
                    SyntaxRole::Subject => DependencyRole::Subject,
                    SyntaxRole::Object => DependencyRole::Object,
                    SyntaxRole::Modifier => DependencyRole::Modifier,
                    SyntaxRole::Root => DependencyRole::Root,
                    SyntaxRole::PrepositionArg => DependencyRole::PrepositionArg,
                    SyntaxRole::IndirectObject => DependencyRole::IndirectObject,
                    SyntaxRole::Coordinator => DependencyRole::Coordinator,
                    SyntaxRole::Conjunct => DependencyRole::Conjunct,
                    SyntaxRole::PassiveAgent => DependencyRole::PassiveAgent,
                    SyntaxRole::AbsoluteClause => DependencyRole::AbsoluteClause,
                    SyntaxRole::Complement => DependencyRole::Complement,
                    SyntaxRole::RelativeClause => DependencyRole::RelativeClause,
                    SyntaxRole::None => continue,
                };
                
                world.set_dependency(child_entity, head_entity, role);
            }
        }

        let syntax_errors_raw = world.validate();
        let syntax_errors: Vec<SerializableAgreementError> = syntax_errors_raw
            .into_iter()
            .map(|e| e.into())
            .collect();

        // 7. Semantic Validation (Meaning)
        let default_graph = SemanticGraph::new();
        let graph = self.semantic_graph.as_ref().unwrap_or(&default_graph);
        let semantic_errors_raw = validate_semantics(&world, graph);
        let semantic_errors: Vec<String> = semantic_errors_raw
            .into_iter()
            .map(|e| e.message)
            .collect();

        AnalysisReport {
            tokens: debug_tokens,
            syntax_errors,
            semantic_errors,
            debug_info: format!("Lemmas: {}, Paradigms: {}", dict.lemmas.len(), dict.paradigms.len()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use logos_protocol::{Dictionary, Lemma, Paradigm, Gender, LemmaId, PartOfSpeech, ParadigmId};
    use rkyv::to_bytes;

    #[test]
    fn test_robust_morphology_resolution() {
        // 1. Setup Mock Dictionary
        // Lemma: "άνθρωπος" (Full word stored, stem: άνθρωπ)
        // Paradigm: 
        //  - "ος" -> Nom|Sg|Masc
        //  - "ου" -> Gen|Sg|Masc
        let lemma = Lemma {
            id: LemmaId(1),
            text: "άνθρωπος".to_string(),
            gender: Gender::Masculine,
            pos: PartOfSpeech::Noun,
        };

        let paradigm = Paradigm {
            id: ParadigmId(1),
            endings: vec![
                ((logos_protocol::MorphFlags::NOMINATIVE | logos_protocol::MorphFlags::SINGULAR | logos_protocol::MorphFlags::MASCULINE).bits(), "ος".to_string()),
                ((logos_protocol::MorphFlags::GENITIVE | logos_protocol::MorphFlags::SINGULAR | logos_protocol::MorphFlags::MASCULINE).bits(), "ου".to_string())
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

        // Case 1: "άνθρωπος" (Nom Sg)
        let analysis = logos_parser::morphology::resolve_morphology(archived, "άνθρωπος", Some(1));
        assert_ne!(analysis.kind, "Unknown", "Should resolve 'άνθρωπος'");
        assert!(format!("{:?}", analysis.flags).contains("NOMINATIVE"), "Should be Nominative");
        assert!(analysis.debug_msg.contains("Matched!"), "Debug should indicate match");

        // Case 2: "άνθρωπου" (Gen Sg)
        let analysis = logos_parser::morphology::resolve_morphology(archived, "άνθρωπου", Some(1));
        assert_ne!(analysis.kind, "Unknown", "Should resolve 'άνθρωπου'");
        assert!(format!("{:?}", analysis.flags).contains("GENITIVE"), "Should be Genitive");

        // Case 3: "άλογο" (Mismatch)
        // We pass None to simulate that Lexer didn't match it (or we are verifying scratch lookup)
        let analysis = logos_parser::morphology::resolve_morphology(archived, "άλογο", None);
        assert_eq!(analysis.kind, "Unknown", "Should NOT resolve 'άλογο'");
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

        // Run Analysis: "xyznonsense" (Unknown)
        let report_unknown = engine.analyze_core("xyznonsense");
        let token_unknown = &report_unknown.tokens[0];
        assert_eq!(token_unknown.text, "xyznonsense");
        assert_eq!(token_unknown.kind, "Unknown", "Should be Unknown");
        assert_eq!(token_unknown.morphology, "None", "Morphology should be None");
    }

    #[test]
    fn test_full_sentence_analysis() {
        use std::fs;
        use std::path::PathBuf;

        // Load Dictionary
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("www");
        path.push("dict_v10.rkyv");
        if !path.exists() { return; }
        let data = fs::read(path).expect("Failed to read dictionary file");
        let engine = LogosEngine::new(data);

        // Analyze: "Ο απάνθρωπος." (The cruel [one].)
        let report = engine.analyze_core("Ο απάνθρωπος.");
        
        assert_eq!(report.tokens.len(), 3, "Should have 3 tokens");
        
        let t1 = &report.tokens[1]; // "απάνθρωπος"
        assert_eq!(t1.text, "απάνθρωπος");
        assert_ne!(t1.kind, "Unknown", "Middle word should be resolved");
        
        let t2 = &report.tokens[2]; // "."
        assert_eq!(t2.text, ".");
        assert_eq!(t2.kind, "Punctuation", "Dot should be punctuation");
    }

    #[test]
    fn test_syntax_and_semantics_pipeline() {
        use std::fs;
        use std::path::PathBuf;

        // Load Dictionary
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("www");
        path.push("dict_v10.rkyv");
        if !path.exists() { return; }
        let data = fs::read(path).expect("Failed to read dictionary file");
        let engine = LogosEngine::new(data);

        // Analyze: "Ο απάνθρωπος" (2 tokens exactly to trigger mock syntax)
        let report = engine.analyze_core("Ο απάνθρωπος");
        
        assert_eq!(report.tokens.len(), 2, "Should have 2 tokens");

        // Verify Syntax/Semantics fields exist (Pipeline RAN)
        println!("Syntax Errors: {:?}", report.syntax_errors.len());
        println!("Semantic Errors: {:?}", report.semantic_errors.len());
        
        // Ensure we didn't crash during ECS/Solver steps
        assert!(report.debug_info.contains("Lemmas:"), "Debug info should be present");
    }
}
