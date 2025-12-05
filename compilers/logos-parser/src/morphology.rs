use logos_protocol::{Dictionary, MorphFlags, LemmaId};
use rkyv::Archived;

#[derive(Debug, Clone)]
pub struct MorphAnalysis {
    pub flags: MorphFlags,
    pub lemma_id: Option<LemmaId>,
    pub debug_msg: String,
    pub stem: String,
    pub kind: String,
}

impl MorphAnalysis {
    pub fn unknown(debug: String) -> Self {
        Self {
            flags: MorphFlags::empty(),
            lemma_id: None,
            debug_msg: debug,
            stem: String::new(),
            kind: "Unknown".to_string(),
        }
    }
}

pub fn resolve_morphology(
    dict: &Archived<Dictionary>, 
    token_text: &str, 
    known_lemma_id: Option<u32>
) -> MorphAnalysis {
    // 1. Try to find semantic matches via suffix analysis
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
                        let flags = MobileFlags::from_bits_truncate(*flags_bits);
                        
                        // We found a match!
                        return MorphAnalysis {
                            flags,
                            lemma_id: Some(LemmaId(lemma.id.0)),
                            debug_msg: format!("Matched! Stem: '{}', Suffix: '{}', Lemma: '{}'", candidate_stem, suffix_str, lemma.text),
                            stem: candidate_stem.to_string(),
                            kind: "Word".to_string(), // Or Word(Recovered) if known_lemma_id was None? 
                                                      // Let's keep it simple "Word"
                        };
                    }
                }
            }
        }
    }

    // 2. If no match found but we had a known ID (Lexer found it exact match or prefix)
    // We should still return that ID but maybe empty morphology?
    if let Some(id) = known_lemma_id {
        return MorphAnalysis {
            flags: MorphFlags::empty(),
            lemma_id: Some(LemmaId(id)),
            debug_msg: "Lexer matched lemma, but no inflectional rule applied (Indeclinable?)".to_string(),
            stem: token_text.to_string(),
            kind: "Word".to_string(),
        };
    }

    // 3. Last Resort: Check if we can recover unknown words by simple prefix match 
    // (This was part of the recover logic in WASM)
    // Actually, robust resolve above usually handles "recover" if suffix matches.
    // If not, we check for raw lemma starts_with
    if let Some(lemma) = dict.lemmas.iter().find(|l| token_text.starts_with(l.text.as_str()) || l.text.starts_with(token_text)) {
         return MorphAnalysis {
            flags: MorphFlags::empty(),
            lemma_id: Some(LemmaId(lemma.id.0)),
            debug_msg: format!("Recovered via raw prefix match against '{}'", lemma.text),
            stem: token_text.to_string(),
            kind: "Word (Recovered)".to_string(),
        };
    }

    MorphAnalysis::unknown(format!("No match found for '{}'", token_text))
}

// Helper alias to avoid import issues if names collide
use logos_protocol::MorphFlags as MobileFlags;
