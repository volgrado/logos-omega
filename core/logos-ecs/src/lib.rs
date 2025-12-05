pub mod components;
pub mod systems;

use hecs::{World, Entity};
use components::{TokenData, Morphology, Syntax, DependencyRole};
use systems::agreement::{check_agreement, AgreementError};

pub struct LogosWorld {
    world: World,
}

impl LogosWorld {
    pub fn new() -> Self {
        Self { world: World::new() }
    }

    /// Expose the inner hecs World for external solvers
    pub fn inner(&self) -> &World {
        &self.world
    }

    /// Add a word to the sentence
    pub fn add_token(
        &mut self, 
        text: String, 
        lemma_id: Option<logos_protocol::LemmaId>, 
        flags: logos_protocol::MorphFlags
    ) -> Entity {
        self.world.spawn((
            TokenData { text, lemma_id },
            Morphology { flags },
        ))
    }

    /// Define the syntactic tree structure
    pub fn set_dependency(&mut self, child: Entity, head: Entity, role: DependencyRole) {
        // We use insert_one to add the Syntax component to an existing Entity
        let _ = self.world.insert_one(child, Syntax { head, role });
    }

    /// Run all validation systems
    pub fn validate(&self) -> Vec<AgreementError> {
        check_agreement(&self.world)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use logos_protocol::MorphFlags;

    #[test]
    fn test_agreement_check() {
        let mut lw = LogosWorld::new();

        // Case 1: "The kids plays" (Mismatch: Plural Subject, Singular Verb)
        
        // Verb: "plays" (Singular)
        let verb = lw.add_token(
            "plays".to_string(), 
            None, 
            MorphFlags::SINGULAR | MorphFlags::THIRD_PERSON
        );

        // Subject: "kids" (Plural)
        let subject = lw.add_token(
            "kids".to_string(), 
            None, 
            MorphFlags::PLURAL | MorphFlags::THIRD_PERSON
        );

        // Link them
        lw.set_dependency(subject, verb, DependencyRole::Subject);

        // Run validation
        let errors = lw.validate();
        
        assert_eq!(errors.len(), 1);
        assert!(errors[0].details.contains("Number mismatch"));
        println!("Caught expected error: {:?}", errors[0]);
    }
}
