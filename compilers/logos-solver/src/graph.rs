use petgraph::graph::{Graph, NodeIndex};
use petgraph::Directed;
use petgraph::visit::EdgeRef;
use logos_protocol::LemmaId;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Relation {
    IsA,                // Inheritance (e.g., Apple IsA Fruit)
    RequiresAttribute,  // Constraint (e.g., Eat RequiresAttribute Edible)
    HasAttribute,       // Property (e.g., Fruit HasAttribute Edible)
}

pub struct SemanticGraph {
    graph: Graph<LemmaId, Relation, Directed>,
    index_map: HashMap<LemmaId, NodeIndex>,
}

impl SemanticGraph {
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            index_map: HashMap::new(),
        }
    }

    pub fn add_concept(&mut self, lemma: LemmaId) {
        if !self.index_map.contains_key(&lemma) {
            let idx = self.graph.add_node(lemma);
            self.index_map.insert(lemma, idx);
        }
    }

    pub fn add_relation(&mut self, from: LemmaId, to: LemmaId, rel: Relation) {
        let from_idx = *self.index_map.entry(from).or_insert_with(|| self.graph.add_node(from));
        let to_idx = *self.index_map.entry(to).or_insert_with(|| self.graph.add_node(to));
        
        self.graph.add_edge(from_idx, to_idx, rel);
    }

    /// Check if 'subject' satisfies a constraint required by 'verb'.
    /// Logic:
    /// 1. Verb requires 'AttributeX'.
    /// 2. Subject must have 'AttributeX' (directly or via IsA inheritance).
    pub fn satisfies_constraint(&self, subject: LemmaId, attribute: LemmaId) -> bool {
        let start_idx = match self.index_map.get(&subject) {
            Some(idx) => *idx,
            None => return false, // Unknown concept
        };

        // BFS traversal to find if subject or its parents have the attribute
        let mut stack = vec![start_idx];
        let mut visited = vec![];

        while let Some(current_idx) = stack.pop() {
            if visited.contains(&current_idx) { continue; }
            visited.push(current_idx);

            // Check outgoing edges
            for edge in self.graph.edges(current_idx) {
                let target = edge.target();
                let relation = edge.weight();

                // If we found the attribute directly
                if *relation == Relation::HasAttribute {
                     if self.graph[target] == attribute {
                         return true;
                     }
                }

                // If IsA, add parent to stack to check *their* attributes
                if *relation == Relation::IsA {
                    stack.push(target);
                }
            }
        }

        false
    }

    pub fn get_required_attributes(&self, subject: LemmaId) -> Vec<LemmaId> {
        let mut reqs = Vec::new();
        if let Some(idx) = self.index_map.get(&subject) {
            for edge in self.graph.edges(*idx) {
                if *edge.weight() == Relation::RequiresAttribute {
                    reqs.push(self.graph[edge.target()]);
                }
            }
        }
        reqs
    }
}
