use logos_ecs::LogosWorld;
use logos_ecs::components::{Syntax, DependencyRole, TokenData};
use crate::graph::{SemanticGraph};

#[derive(Debug)]
pub struct SemanticError {
    pub verb_text: String,
    pub object_text: String,
    pub message: String,
}

pub fn validate_semantics(world: &LogosWorld, graph: &SemanticGraph) -> Vec<SemanticError> {
    let mut errors = Vec::new();
    let inner = world.inner();

    // 1. Iterate over all syntactic dependencies
    for (_id, (syntax, object_token)) in inner.query::<(&Syntax, &TokenData)>().iter() {
        
        // We only care about Verb-Object relations
        if syntax.role == DependencyRole::Object {
            
            // Get the Head (The Verb)
            if let Ok(verb_token) = inner.get::<&TokenData>(syntax.head) {
                
                // Ensure both have Lemmas (if Unknown, we can't check semantics)
                if let (Some(verb_id), Some(object_id)) = (verb_token.lemma_id, object_token.lemma_id) {
                    
                    // 2. Check Graph Constraints
                    // Get what the verb requires
                    let requirements = graph.get_required_attributes(verb_id);
                    
                    for req_attr in requirements {
                        if !graph.satisfies_constraint(object_id, req_attr) {
                            errors.push(SemanticError {
                                verb_text: verb_token.text.clone(),
                                object_text: object_token.text.clone(),
                                message: format!("Constraint Violation: Object '{}' does not satisfy requirement of '{}'.", object_token.text, verb_token.text),
                            });
                        }
                    }
                }
            }
        }
    }
    
    errors
}
