use hecs::World;
use crate::components::{Morphology, Syntax, DependencyRole, TokenData};
use logos_protocol::MorphFlags;

#[derive(Debug, Clone)]
pub struct AgreementError {
    pub source: String, // Renamed from subject_text
    pub target: String, // Renamed from verb_text
    pub details: String,
}

pub fn check_agreement(world: &World) -> Vec<AgreementError> {
    let mut errors = Vec::new();

    // 1. Subject-Verb Agreement (Existing)
    errors.extend(check_subject_verb_agreement(world));

    // 2. Determiner-Noun Agreement (New)
    errors.extend(check_determiner_agreement(world));

    errors
}

fn check_subject_verb_agreement(world: &World) -> Vec<AgreementError> {
    let mut errors = Vec::new();

    // Query: Get all entities that have Morphology, Syntax, and TokenData
    for (_id, (subject_morph, syntax, subject_token)) in world.query::<(&Morphology, &Syntax, &TokenData)>().iter() {
        
        // Filter: We only care about Subjects
        if syntax.role == DependencyRole::Subject {
            
            // Look up the Head (The Verb)
            // Note: In hecs, random access is O(1) via world.get
            if let Ok(verb_morph) = world.get::<&Morphology>(syntax.head) {
                
                // 1. Check Number Agreement
                let subj_num = subject_morph.flags.intersection(MorphFlags::SINGULAR | MorphFlags::PLURAL);
                let verb_num = verb_morph.flags.intersection(MorphFlags::SINGULAR | MorphFlags::PLURAL);
                
                // If both have Number defined (non-empty) and they don't match
                if !subj_num.is_empty() && !verb_num.is_empty() && subj_num != verb_num {
                    
                    // Fetch verb text for the error message
                    let verb_text = world.get::<&TokenData>(syntax.head)
                        .map(|t| t.text.clone())
                        .unwrap_or_else(|_| "Unknown Verb".to_string());

                    errors.push(AgreementError {
                        source: subject_token.text.clone(),
                        target: verb_text,
                        details: format!("Number mismatch: {:?} vs {:?}", subj_num, verb_num),
                    });
                }
                
                // 2. Check Person Agreement (Optional: Nouns are 3rd person by default)
                // If the subject is a Pronoun, it might be 1st/2nd. 
                // Nouns don't usually have Person flags in simple lexers, so we skip if empty.
                let subj_person = subject_morph.flags.intersection(MorphFlags::FIRST_PERSON | MorphFlags::SECOND_PERSON | MorphFlags::THIRD_PERSON);
                let verb_person = verb_morph.flags.intersection(MorphFlags::FIRST_PERSON | MorphFlags::SECOND_PERSON | MorphFlags::THIRD_PERSON);

                if !subj_person.is_empty() && !verb_person.is_empty() && subj_person != verb_person {
                     let verb_text = world.get::<&TokenData>(syntax.head)
                        .map(|t| t.text.clone())
                        .unwrap_or_else(|_| "Unknown Verb".to_string());

                     errors.push(AgreementError {
                        source: subject_token.text.clone(),
                        target: verb_text,
                        details: format!("Person mismatch: {:?} vs {:?}", subj_person, verb_person),
                    });
                }
            }
        }
    }
    errors
}

fn check_determiner_agreement(world: &World) -> Vec<AgreementError> {
    let mut errors = Vec::new();

    for (_id, (det_morph, syntax, det_token)) in world.query::<(&Morphology, &Syntax, &TokenData)>().iter() {
        if syntax.role == DependencyRole::Modifier {
             if let Ok(head_morph) = world.get::<&Morphology>(syntax.head) {
                 
                 // Check Number Agreement
                 let det_num = det_morph.flags.intersection(MorphFlags::SINGULAR | MorphFlags::PLURAL);
                 let head_num = head_morph.flags.intersection(MorphFlags::SINGULAR | MorphFlags::PLURAL);
                 
                 if !det_num.is_empty() && !head_num.is_empty() && det_num != head_num {
                     let head_text = world.get::<&TokenData>(syntax.head)
                        .map(|t| t.text.clone())
                        .unwrap_or_else(|_| "Head".to_string());

                     errors.push(AgreementError {
                        source: det_token.text.clone(),
                        target: head_text,
                        details: format!("Agreement Mismatch (Det-Noun): {:?} vs {:?}", det_num, head_num),
                    });
                 }
             }
        }
    }
    errors
}
