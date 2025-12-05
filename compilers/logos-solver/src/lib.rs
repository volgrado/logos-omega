pub mod graph;
pub mod solver;

pub use graph::{SemanticGraph, Relation};
pub use solver::validate_semantics;

#[cfg(test)]
mod tests {
    use super::*;
    use logos_ecs::LogosWorld;
    use logos_ecs::components::DependencyRole;
    use logos_protocol::{LemmaId, MorphFlags};

    #[test]
    fn test_semantic_validation() {
        // 1. Setup Graph
        let mut graph = SemanticGraph::new();
        let eat = LemmaId(1);
        let stone = LemmaId(2);
        let food = LemmaId(3);
        let apple = LemmaId(4);
        let edible = LemmaId(99);

        // Define World Knowledge
        // "Eat" requires "Edible"
        graph.add_relation(eat, edible, Relation::RequiresAttribute);
        
        // "Food" has attribute "Edible"
        graph.add_relation(food, edible, Relation::HasAttribute);
        
        // "Apple" IsA "Food" (Inherits Edible)
        graph.add_relation(apple, food, Relation::IsA);
        
        // "Stone" is just a Stone (Not Edible)
        graph.add_concept(stone);

        // 2. Setup Sentence: "Eat Stone"
        let mut world = LogosWorld::new();
        
        let verb_entity = world.add_token("Eat".to_string(), Some(eat), MorphFlags::empty());
        let obj_entity = world.add_token("Stone".to_string(), Some(stone), MorphFlags::empty());
        
        world.set_dependency(obj_entity, verb_entity, DependencyRole::Object);

        // 3. Validate (Should Fail)
        let errors = validate_semantics(&world, &graph);
        assert_eq!(errors.len(), 1);
        println!("Caught Error: {}", errors[0].message);

        // 4. Setup Sentence: "Eat Apple"
        let mut world2 = LogosWorld::new();
        let verb2 = world2.add_token("Eat".to_string(), Some(eat), MorphFlags::empty());
        let obj2 = world2.add_token("Apple".to_string(), Some(apple), MorphFlags::empty());
        world2.set_dependency(obj2, verb2, DependencyRole::Object);

        // 5. Validate (Should Pass)
        let errors2 = validate_semantics(&world2, &graph);
        assert_eq!(errors2.len(), 0);
    }
}
