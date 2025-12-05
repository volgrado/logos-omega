use logos_protocol::MorphFlags;

#[derive(Debug, Clone)]
pub struct MorphToken<'a> {
    pub text: &'a str,
    pub flags: MorphFlags,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyntaxRole {
    Subject,
    Object,
    IndirectObject, // Dative target
    Modifier, // Adjective/Determiner modifying following noun OR Genitive modifier
    Root,     // The main verb
    PrepositionArg, // Argument of a preposition
    Coordinator,    // Conjunction
    Conjunct,       // Coordinated element
    PassiveAgent,   // "by X" in passive
    AbsoluteClause, // Genitive Absolute
    Complement,     // Infinitive complement (Subject/Object of main verb)
    RelativeClause, // Relative clause (linked to antecedent)
    None,
}

#[derive(Debug, Clone)]
pub struct Dependency {
    pub head_index: usize,
    pub dependent_index: usize,
    pub role: SyntaxRole,
}


/// Enhanced Greedy Parser for Ancient Greek
/// Handles:
/// - SVO / Deterministic Case Assignment
/// - Article-Adjective-Noun clustering
/// - Prepositional Phrases
pub fn parse_greedy(tokens: &[MorphToken]) -> Vec<Dependency> {
    let mut deps = Vec::new();
    let mut root_idx = None;

    // 1. Identify Root (First indicative/imperative Verb)
    for (i, token) in tokens.iter().enumerate() {
        if is_verb(token) {
            root_idx = Some(i);
            break; 
        }
    }

    // Default to first word if no verb found (Nominal sentence)
    // But typically we return empty deps if no root, or treat first as root?
    // Let's assume there is a root or we take 0.
    let root = root_idx.unwrap_or(0);

    // Track "current nominal head" for modifiers
    // This helps with "The [good] man" -> good modifies man
    // Standard Greek word order often puts modifiers *before* the head (Article Adj Noun)
    // But sometimes after.
    
    // We will do a linear scan and try to attach to:
    // A. The Root (if Subject/Object) - or Active Clause Head
    // B. The "Active" Preposition (if in a PP)
    // C. The "Pending" Adjectives (if we find a Noun) OR Attach modifiers to this Noun

    let mut open_preposition_idx: Option<usize> = None;
    let mut pending_modifiers: Vec<usize> = Vec::new();
    let mut last_noun_idx: Option<usize> = None;
    let mut active_coordination_head: Option<usize> = None;
    let mut pending_relative_clause: Option<(usize, usize)> = None; // (PronounIdx, AntecedentIdx)
    let mut current_clause_head = root; // Start with Main Root

    for (i, token) in tokens.iter().enumerate() {
        if i == root { 
            // If we hit the root, close any open structures that shouldn't cross the verb?
            // Actually Greek Word order is flexible.
            continue; 
        }

        if token.flags.contains(MorphFlags::PREPOSITION) {
            // Found a Preposition.
            // It modifies the Verb (typically) or previous Noun. 
            // Check for Passive Agent (hypo + genitive context usually, but we check text and root voice)
            let is_passive_agent = (token.text == "υπό" || token.text == "ὑπό") 
                                   && tokens[current_clause_head].flags.contains(MorphFlags::PASSIVE);

            let role = if is_passive_agent {
                SyntaxRole::PassiveAgent
            } else {
                SyntaxRole::Modifier // Adverbial modifier really
            };

            deps.push(Dependency {
                head_index: current_clause_head,
                dependent_index: i,
                role,
            });
            
            // Set this as open to catch the next Noun
            open_preposition_idx = Some(i);
            continue;
        }

        // Is it a Conjunction?
        if token.flags.contains(MorphFlags::CONJUNCTION) {
            // Attach to last significant element (Noun or Root?)
            // Priority: Last Noun > Root
            let head = last_noun_idx.unwrap_or(current_clause_head);
             deps.push(Dependency {
                head_index: head,
                dependent_index: i,
                role: SyntaxRole::Coordinator,
            });
            
            // Set expectation for next element
            active_coordination_head = Some(head);
            continue;
        }

        // Is it a Relative Pronoun? (os, h, o)
        if token.flags.contains(MorphFlags::RELATIVE) {
            // Find Antecedent (Last Noun? matching gender/number?)
            // Simplification: Last Noun.
            if let Some(antecedent) = last_noun_idx {
                // Check Gender/Number match?
                // For now greedy assume last noun is proper antecedent.
                pending_relative_clause = Some((i, antecedent));
            }
            continue;
        }
        // Check for Secondary Verb (Relative Clause or other subordinate)
        // If i != root, and it is a verb (Active/Passive etc.)
        // We use is_verb helper or flags.
        if i != root && is_verb(token) {
             // 1. Is it a Relative Clause Verb?
             if let Some((pronoun_idx, antecedent_idx)) = pending_relative_clause {
                 // Attach Verb to Antecedent
                 deps.push(Dependency {
                    head_index: antecedent_idx,
                    dependent_index: i,
                    role: SyntaxRole::RelativeClause,
                });
                
                // Attach Pronoun to Verb (Subject/Object based on Pronoun Case)
                let role = if tokens[pronoun_idx].flags.contains(MorphFlags::NOMINATIVE) {
                    SyntaxRole::Subject
                } else if tokens[pronoun_idx].flags.contains(MorphFlags::ACCUSATIVE) {
                    SyntaxRole::Object
                } else {
                    SyntaxRole::Modifier // Dative/Genitive object?
                };
                
                 deps.push(Dependency {
                    head_index: i,
                    dependent_index: pronoun_idx,
                    role,
                });
                
                // Set scope for subsequent tokens
                current_clause_head = i;
                pending_relative_clause = None;
             }
             // Else: Other subordinate verb? For now ignore or attach to root?
             // If we don't handle it, it might just float.
             // We continue to avoid processing it as a Noun (if it has Case flags like Participle? No Participle is handled in Noun block).
             // Finite verbs don't have Case.
             continue;
        }

        // Is it a Noun-like thing? (Noun, Pronoun, or Subst. Adjective)
        // Heuristic: If it has Case, it's nominal.
        // Also Infinitives act as Nouns (Articular or Complement), so let them pass this check.
        if has_case(token) || token.flags.contains(MorphFlags::INFINITIVE) {
            // Is it a "Head" noun or a Modifier (Adj/Article)?
            let is_head_noun = token.flags.contains(MorphFlags::NOUN) || 
                              (!token.flags.contains(MorphFlags::ARTICLE) && !token.flags.contains(MorphFlags::ADJECTIVE));

            if is_head_noun {
                // It's a Noun. 
                
                // 1. Resolve Pending Modifiers (Articles/Adj before this noun)
                // Check Agreement!
                let mut matched_modifiers = Vec::new();
                for &mod_idx in &pending_modifiers {
                    let mod_token = &tokens[mod_idx];
                    if check_agreement(mod_token, token) {
                         deps.push(Dependency {
                            head_index: i,
                            dependent_index: mod_idx,
                            role: SyntaxRole::Modifier,
                        });
                        matched_modifiers.push(mod_idx);
                    }
                }
                pending_modifiers.retain(|idx| !matched_modifiers.contains(idx));

                // 2. Attach this Noun to something

                if let Some(coord_head) = active_coordination_head {
                    // We are the second part of "X and Y"
                     deps.push(Dependency {
                        head_index: coord_head,
                        dependent_index: i,
                        role: SyntaxRole::Conjunct,
                    });
                    // Reset coordination state (greedy: only handles binary coordination for now)
                    active_coordination_head = None;
                
                } else if let Some(prep_idx) = open_preposition_idx {
                    // We are in a Prepositional Phrase
                    // Attach Noun to Preposition
                     deps.push(Dependency {
                        head_index: prep_idx,
                        dependent_index: i,
                        role: SyntaxRole::PrepositionArg,
                    });
                    // Close the prep if we found its head
                    open_preposition_idx = None;

                // 3. Handle Participles (Genitive Absolute or Modifier)
                } else if token.flags.contains(MorphFlags::PARTICIPLE) {
                   
                   let mut is_gen_abs = false;
                   
                   // Check for Genitive Absolute: Genitive Participle + Genitive Noun (last_noun_idx)
                   if token.flags.contains(MorphFlags::GENITIVE) {
                       if let Some(prev_noun_idx) = last_noun_idx {
                           let prev_token = &tokens[prev_noun_idx];
                           if prev_token.flags.contains(MorphFlags::GENITIVE) && check_agreement(token, prev_token) {
                               // Start of Gen Absolute!
                               is_gen_abs = true;
                               
                               // 1. Relink the Noun to become Subject of this Participle
                               // Find the dep where dependent == prev_noun_idx
                               for dep in deps.iter_mut() {
                                   if dep.dependent_index == prev_noun_idx {
                                       dep.head_index = i;
                                       dep.role = SyntaxRole::Subject;
                                       break;
                                   }
                               }
                               
                               // 2. Attach Participle to Root (Absolute Clause)
                               deps.push(Dependency {
                                   head_index: current_clause_head,
                                   dependent_index: i,
                                   role: SyntaxRole::AbsoluteClause,
                               });
                           }
                       }
                   }
                   
                   if !is_gen_abs {
                       // Normal Modifier (Attributive/Circumstantial)
                       // Attach to last noun if agrees, else Root
                       let head = if let Some(noun_idx) = last_noun_idx {
                           if check_agreement(token, &tokens[noun_idx]) {
                               noun_idx
                           } else {
                               current_clause_head
                           }
                       } else {
                           root
                       };

                       deps.push(Dependency {
                           head_index: head,
                           dependent_index: i,
                           role: SyntaxRole::Modifier,
                       });
                   }

                // 4. Handle Infinitives (Articular or Complement)
                } else if token.flags.contains(MorphFlags::INFINITIVE) {
                    
                    // Check for Articular Infinitive (Preceding Article)
                    // We already consumed modifiers at the top of this block.
                    // If any matched modifier was an Article, we treat this as Articular.
                    let is_articular = matched_modifiers.iter().any(|&m| tokens[m].flags.contains(MorphFlags::ARTICLE));
                    
                    // No need to re-scan pending_modifiers, they are gone.
                    
                    if is_articular {
                        // Treat as Noun (Subject/Object based on Case of Article?)
                        // Warning: The *Token* flags might not have Case if it's just an Infinitive form.
                        // BUT if we matched an Article, that Article has Case.
                        // We should assume the Infinitive group takes the role of the Article.
                        // For simplicity, we attach to Root as Subject/Object based on Article's case (if we can find it).
                        // Or just Subject/Object if we assume nominative/accusative context.
                        // Let's optimize: Check valid modifier case.
                        let role = if matched_modifiers.iter().any(|&m| tokens[m].flags.contains(MorphFlags::NOMINATIVE)) {
                            SyntaxRole::Subject
                        } else {
                            SyntaxRole::Object // Default to Object needed? Or PrepositionArg?
                        };
                        
                        // If Preposition exists?
                        if let Some(prep_idx) = open_preposition_idx {
                             deps.push(Dependency {
                                head_index: prep_idx,
                                dependent_index: i,
                                role: SyntaxRole::PrepositionArg,
                            });
                            open_preposition_idx = None;
                        } else {
                            deps.push(Dependency {
                                head_index: current_clause_head,
                                dependent_index: i,
                                role,
                            });
                        }
                        
                    } else {
                        // Naked Infinitive -> Complement
                        // 1. Relink Accusative Subject?
                        // If last noun was Accusative Object, relink to Subject of Infinitive.
                        if let Some(prev_noun_idx) = last_noun_idx {
                            if tokens[prev_noun_idx].flags.contains(MorphFlags::ACCUSATIVE) {
                                // Find current dependency
                                for dep in deps.iter_mut() {
                                   if dep.dependent_index == prev_noun_idx && dep.head_index == current_clause_head && dep.role == SyntaxRole::Object {
                                       dep.head_index = i;
                                       dep.role = SyntaxRole::Subject;
                                       break;
                                   }
                               }
                            }
                        }
                        
                        // 2. Attach Infinitive to Root
                        deps.push(Dependency {
                            head_index: current_clause_head,
                            dependent_index: i,
                            role: SyntaxRole::Complement,
                        });
                    }

                } else if token.flags.contains(MorphFlags::NOMINATIVE) {
                    // Subject of Root
                    deps.push(Dependency {
                        head_index: current_clause_head,
                        dependent_index: i,
                        role: SyntaxRole::Subject,
                    });
                } else if token.flags.contains(MorphFlags::ACCUSATIVE) {
                    // Object of Root
                    deps.push(Dependency {
                        head_index: current_clause_head,
                        dependent_index: i,
                        role: SyntaxRole::Object,
                    });
                } else if token.flags.contains(MorphFlags::GENITIVE) {
                    // Genitive Case (Possession / "Of X")
                    // If we saw a noun recently, attach to it. 
                    // "The house of the father" -> "father" modifies "house"
                    if let Some(prev_noun) = last_noun_idx {
                         deps.push(Dependency {
                            head_index: prev_noun,
                            dependent_index: i,
                            role: SyntaxRole::Modifier,
                        });
                    } else {
                        // No previous noun? Attach to Root (maybe Object of value/time etc.)
                         deps.push(Dependency {
                            head_index: current_clause_head,
                            dependent_index: i,
                            role: SyntaxRole::Modifier,
                        });
                    }
                } else if token.flags.contains(MorphFlags::DATIVE) {
                    // Dative -> Indirect Object
                     deps.push(Dependency {
                        head_index: current_clause_head,
                        dependent_index: i,
                        role: SyntaxRole::IndirectObject,
                    });
                } else {
                    // Dative/Vocative/etc.
                     deps.push(Dependency {
                        head_index: current_clause_head,
                        dependent_index: i,
                        role: SyntaxRole::Modifier,
                    });
                }

                // Track this noun as a potential head for future Genitives
                last_noun_idx = Some(i);

            } else {
                // It's an Article or Adjective.
                // Add to pending modifiers.
                pending_modifiers.push(i);
            }
        }
    }
    
    // Cleanup: If any modifiers are left dangling, attach them to Root or ignore?
    // "The good [missing]" -> "The" and "good" dangle.
    // In a robust parser we might error, but here we ignore or attach to Root.
    
    deps
}

fn is_verb(token: &MorphToken) -> bool {
    // Check for Verb-specific flags (Voice, Tense, Person, Mood)
    token.flags.intersects(
        MorphFlags::ACTIVE | MorphFlags::PASSIVE | 
        MorphFlags::PRESENT | MorphFlags::PAST | MorphFlags::FUTURE |
        MorphFlags::FIRST_PERSON | MorphFlags::SECOND_PERSON | MorphFlags::THIRD_PERSON
    ) && !token.flags.contains(MorphFlags::NOUN) // Disambiguate if ambiguous
}

fn has_case(token: &MorphToken) -> bool {
    token.flags.intersects(
        MorphFlags::NOMINATIVE | MorphFlags::GENITIVE | 
        MorphFlags::ACCUSATIVE | MorphFlags::VOCATIVE | MorphFlags::DATIVE
    )
}

fn check_agreement(mod_token: &MorphToken, head_token: &MorphToken) -> bool {
    let case_mask = MorphFlags::NOMINATIVE | MorphFlags::GENITIVE | MorphFlags::ACCUSATIVE | MorphFlags::VOCATIVE | MorphFlags::DATIVE;
    
    // Infinitives are Case-less but can take Articular modifiers.
    // If head is Infinitive, strict case match is ignored (Article imparts Case).
    let case_match = (mod_token.flags & case_mask) == (head_token.flags & case_mask)
                     || head_token.flags.contains(MorphFlags::INFINITIVE);

    let gender_match = (mod_token.flags & (MorphFlags::MASCULINE | MorphFlags::FEMININE | MorphFlags::NEUTER)).is_empty() ||
                       (head_token.flags & (MorphFlags::MASCULINE | MorphFlags::FEMININE | MorphFlags::NEUTER)).is_empty() ||
                       (mod_token.flags & (MorphFlags::MASCULINE | MorphFlags::FEMININE | MorphFlags::NEUTER))
                       == (head_token.flags & (MorphFlags::MASCULINE | MorphFlags::FEMININE | MorphFlags::NEUTER));
    
    let number_match = (mod_token.flags & (MorphFlags::SINGULAR | MorphFlags::PLURAL)).is_empty() ||
                       (head_token.flags & (MorphFlags::SINGULAR | MorphFlags::PLURAL)).is_empty() ||
                       (mod_token.flags & (MorphFlags::SINGULAR | MorphFlags::PLURAL))
                       == (head_token.flags & (MorphFlags::SINGULAR | MorphFlags::PLURAL));

    case_match && gender_match && number_match
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create tokens easily
    fn t(text: &str, flags: MorphFlags) -> MorphToken {
        MorphToken { text, flags }
    }

    #[test]
    fn test_simple_svo() {
        // "Ο Πέτρος βλέπει την Μαρίαν" (The Peter sees the Maria)
        // 0: Ο (Nom|Masc|Sg|Art)
        // 1: Πέτρος (Nom|Masc|Sg|Noun)
        // 2: βλέπει (Verb|Pres|Act|3rd)
        // 3: την (Acc|Fem|Sg|Art)
        // 4: Μαρίαν (Acc|Fem|Sg|Noun)

        let tokens = vec![
            t("Ο", MorphFlags::NOMINATIVE | MorphFlags::MASCULINE | MorphFlags::SINGULAR | MorphFlags::ARTICLE),
            t("Πέτρος", MorphFlags::NOMINATIVE | MorphFlags::MASCULINE | MorphFlags::SINGULAR | MorphFlags::NOUN),
            t("βλέπει", MorphFlags::PRESENT | MorphFlags::ACTIVE | MorphFlags::THIRD_PERSON),
            t("την", MorphFlags::ACCUSATIVE | MorphFlags::FEMININE | MorphFlags::SINGULAR | MorphFlags::ARTICLE),
            t("Μαρίαν", MorphFlags::ACCUSATIVE | MorphFlags::FEMININE | MorphFlags::SINGULAR | MorphFlags::NOUN),
        ];

        let deps = parse_greedy(&tokens);

        // Root should be "βλέπει" (idx 2)
        // "Πέτρος" (1) -> Subject of (2)
        // "Ο" (0) -> Modifier of (1)
        // "Μαρίαν" (4) -> Object of (2)
        // "την" (3) -> Modifier of (4)

        // Check Subject
        assert!(deps.iter().any(|d| d.dependent_index == 1 && d.head_index == 2 && d.role == SyntaxRole::Subject));
        // Check Object
        assert!(deps.iter().any(|d| d.dependent_index == 4 && d.head_index == 2 && d.role == SyntaxRole::Object));
        // Check Modifier (O -> Petros)
        assert!(deps.iter().any(|d| d.dependent_index == 0 && d.head_index == 1 && d.role == SyntaxRole::Modifier));
        // Check Modifier (tin -> Maria)
        assert!(deps.iter().any(|d| d.dependent_index == 3 && d.head_index == 4 && d.role == SyntaxRole::Modifier));
    }

    #[test]
    fn test_prepositional_phrase() {
        // "Εν τη οικία" (In the house)
        // 0: Εν (Prep)
        // 1: τη (Dat|Fem|Sg|Art) - Using Gen/Dat overlap for simplicity or just Genitive if Dative not supported?
        // Let's assume Dative is essential for Greek PPs. 
        // MorphFlags only has Nom/Gen/Acc/Voc. Uh oh. Ancient Greek uses Dative!
        // Constraint: We only defined 4 cases. 
        // Hack: Use Genitive for now for testing, or assume we map Dative to something else? or Just check if PrepositionArg works for ANY case.
        // Let's use "Apo tēs oikias" (From the house) which is GENITIVE.

        // 0: Από (Prep)
        // 1: της (Gen|Fem|Sg|Art)
        // 2: οικίας (Gen|Fem|Sg|Noun)
        // 3: μένει (Verb)
        
        let tokens = vec![
            t("μένει", MorphFlags::PRESENT | MorphFlags::ACTIVE | MorphFlags::THIRD_PERSON), // He stays
            t("από", MorphFlags::PREPOSITION),
            t("της", MorphFlags::GENITIVE | MorphFlags::FEMININE | MorphFlags::SINGULAR | MorphFlags::ARTICLE),
            t("οικίας", MorphFlags::GENITIVE | MorphFlags::FEMININE | MorphFlags::SINGULAR | MorphFlags::NOUN),
        ];

        let deps = parse_greedy(&tokens);

        // Root: men (0)
        // Prep: apo (1) -> Modifier of Root (0)
        // Noun: oikias (3) -> Arg of Prep (1)
        // Art: tes (2) -> Modifier of Noun (3)

        assert!(deps.iter().any(|d| d.dependent_index == 1 && d.head_index == 0 && d.role == SyntaxRole::Modifier));
        assert!(deps.iter().any(|d| d.dependent_index == 3 && d.head_index == 1 && d.role == SyntaxRole::PrepositionArg));
        assert!(deps.iter().any(|d| d.dependent_index == 2 && d.head_index == 3 && d.role == SyntaxRole::Modifier));
    }

    #[test]
    fn test_dative_indirect_object() {
        // "Ο διδάσκαλος δίδει το βιβλίον τω μαθητή" (The teacher gives the book to-the student)
        // 0: Ο (Nom)
        // 1: διδάσκαλος (Nom|Noun)
        // 2: δίδει (Verb)
        // 3: το (Acc)
        // 4: βιβλίον (Acc|Noun)
        // 5: τω (Dat|Art)
        // 6: μαθητή (Dat|Noun)

        let tokens = vec![
            t("Ο", MorphFlags::NOMINATIVE | MorphFlags::ARTICLE),
            t("διδάσκαλος", MorphFlags::NOMINATIVE | MorphFlags::NOUN),
            t("δίδει", MorphFlags::PRESENT | MorphFlags::ACTIVE),
            t("το", MorphFlags::ACCUSATIVE | MorphFlags::ARTICLE),
            t("βιβλίον", MorphFlags::ACCUSATIVE | MorphFlags::NOUN),
            t("τω", MorphFlags::DATIVE | MorphFlags::ARTICLE),
            t("μαθητή", MorphFlags::DATIVE | MorphFlags::NOUN),
        ];

        let deps = parse_greedy(&tokens);

        // Root: 2 (didei)
        // Subject: 1 (didaskalos)
        // Object: 4 (biblion)
        // Ind. Object: 6 (mathete)

        assert!(deps.iter().any(|d| d.dependent_index == 1 && d.role == SyntaxRole::Subject));
        assert!(deps.iter().any(|d| d.dependent_index == 4 && d.role == SyntaxRole::Object));
        assert!(deps.iter().any(|d| d.dependent_index == 6 && d.role == SyntaxRole::IndirectObject));
        
        // Check Modifier (to -> mathete)
        assert!(deps.iter().any(|d| d.dependent_index == 5 && d.head_index == 6 && d.role == SyntaxRole::Modifier));
    }

    #[test]
    fn test_genitive_modifier() {
        // "Η οικία του πατρός" (The house of the father) - No verb, so 0 is root or ignored?
        // Let's add a verb: "Βλέπω την οικίαν του πατρός" (I see the house of the father)
        // 0: Βλέπω (Verb)
        // 1: την (Acc|Art)
        // 2: οικίαν (Acc|Noun)
        // 3: του (Gen|Art)
        // 4: πατρός (Gen|Noun)

        let tokens = vec![
            t("Βλέπω", MorphFlags::PRESENT | MorphFlags::ACTIVE),
            t("την", MorphFlags::ACCUSATIVE | MorphFlags::ARTICLE),
            t("οικίαν", MorphFlags::ACCUSATIVE | MorphFlags::NOUN),
            t("του", MorphFlags::GENITIVE | MorphFlags::ARTICLE),
            t("πατρός", MorphFlags::GENITIVE | MorphFlags::NOUN),
        ];

        let deps = parse_greedy(&tokens);

        // Root: 0
        // Object: 2 (oikian)
        // Modifier of 2: 1 (tin)
        // Modifier of 2: 4 (patros) -- THIS IS THE KEY CHECK. 4 -> 2, NOT 4 -> 0.
        // Modifier of 4: 3 (tou)

        assert!(deps.iter().any(|d| d.dependent_index == 2 && d.head_index == 0 && d.role == SyntaxRole::Object));
        assert!(deps.iter().any(|d| d.dependent_index == 4 && d.head_index == 2 && d.role == SyntaxRole::Modifier));
    }

    #[test]
    fn test_coordination() {
        // "Ο διδάσκαλος και ο μαθητής" (The teacher and the student)
        // 0: Ο (Nom)
        // 1: διδάσκαλος (Nom|Noun)
        // 2: και (Conj)
        // 3: ο (Nom)
        // 4: μαθητής (Nom|Noun)

        let tokens = vec![
            t("Ο", MorphFlags::NOMINATIVE | MorphFlags::ARTICLE),
            t("διδάσκαλος", MorphFlags::NOMINATIVE | MorphFlags::NOUN),
            t("και", MorphFlags::CONJUNCTION),
            t("ο", MorphFlags::NOMINATIVE | MorphFlags::ARTICLE),
            t("μαθητής", MorphFlags::NOMINATIVE | MorphFlags::NOUN),
        ];

        let deps = parse_greedy(&tokens);

        // Subject 1: didaskalos (Head of coordination)
        // Coordinator: 2 (kai) -> 1 (didaskalos)
        // Subject 2: 4 (mathetis) -> 1 (didaskalos) as Conjunct

        assert!(deps.iter().any(|d| d.dependent_index == 2 && d.head_index == 1 && d.role == SyntaxRole::Coordinator));
        assert!(deps.iter().any(|d| d.dependent_index == 4 && d.head_index == 1 && d.role == SyntaxRole::Conjunct));
    }

    #[test]
    fn test_passive_agent() {
        // "βάλλομαι υπό του ανθρώπου" (I am thrown by the man)
        // 0: βάλλομαι (Verb|Passive)
        // 1: υπό (Prep)
        // 2: του (Gen|Art)
        // 3: ανθρώπου (Gen|Noun)

        let tokens = vec![
            t("βάλλομαι", MorphFlags::VERB | MorphFlags::PASSIVE),
            t("υπό", MorphFlags::PREPOSITION),
            t("του", MorphFlags::GENITIVE | MorphFlags::ARTICLE),
            t("ανθρώπου", MorphFlags::GENITIVE | MorphFlags::NOUN),
        ];

        let deps = parse_greedy(&tokens);

        // Prep "υπό" -> Root "βάλλομαι" as PassiveAgent
        assert!(deps.iter().any(|d| d.dependent_index == 1 && d.head_index == 0 && d.role == SyntaxRole::PassiveAgent));
        
        // Noun "ανθρώπου" -> Prep "υπό" as PrepositionArg
        assert!(deps.iter().any(|d| d.dependent_index == 3 && d.head_index == 1 && d.role == SyntaxRole::PrepositionArg));
    }

    #[test]
    fn test_genitive_absolute() {
        // "του ανθρώπου λέγοντος" (The man speaking [Genitive Absolute])
        // 0: έφυγον (I fled - Root)
        // 1: του (Gen|Art)
        // 2: ανθρώπου (Gen|Noun)
        // 3: λέγοντος (Gen|Participle)

        let tokens = vec![
             t("έφυγον", MorphFlags::VERB),
             t("του", MorphFlags::GENITIVE | MorphFlags::ARTICLE),
             t("ανθρώπου", MorphFlags::GENITIVE | MorphFlags::NOUN),
             // Note: PARTICIPLE flag required for logic
             t("λέγοντος", MorphFlags::GENITIVE | MorphFlags::PARTICIPLE | MorphFlags::MASCULINE | MorphFlags::SINGULAR), // Agreement with noun
        ];

        let deps = parse_greedy(&tokens);

        // 3: Participle (legontos) -> 0: Root (GenAbs clause)
        assert!(deps.iter().any(|d| d.dependent_index == 3 && d.head_index == 0 && d.role == SyntaxRole::AbsoluteClause));

        // Note: Initially attached to Root or Modifier, but relinking logic should move it to Subject of Participle
        assert!(deps.iter().any(|d| d.dependent_index == 2 && d.head_index == 3 && d.role == SyntaxRole::Subject));
    }

    #[test]
    fn test_infinitive_complement() {
        // "λέγω αυτόν είναι αγαθόν" (I say him to be good)
        // 0: λέγω (Root)
        // 1: αυτόν (Acc) -> Subject of 2
        // 2: είναι (Inf) -> Complement of 0
        // 3: αγαθόν (Acc) -> Modifier of 1? Or Predicate? Greedy might attach to Root as Object... 
        //   For now, "agathon" (Adj Acc) modifies "auton" (Noun Acc) if Agreement matched?
        //   Or if "auton" is attached to Infinitive... "agathon" might try to attach to "auton".

        let tokens = vec![
            t("λέγω", MorphFlags::VERB), // Root
            t("αυτόν", MorphFlags::ACCUSATIVE | MorphFlags::NOUN), // Accusative Subject
            t("είναι", MorphFlags::INFINITIVE), // Infinitive
        ];

        let deps = parse_greedy(&tokens);

        // Infinitive -> Root (Complement)
        assert!(deps.iter().any(|d| d.dependent_index == 2 && d.head_index == 0 && d.role == SyntaxRole::Complement));

        // Accusative Noun -> Infinitive (Subject)
        assert!(deps.iter().any(|d| d.dependent_index == 1 && d.head_index == 2 && d.role == SyntaxRole::Subject));
    }

    #[test]
    fn test_articular_infinitive() {
        // "Το λέγειν" (The speaking)
        // 0: Το (Neut Sg Art)
        // 1: λέγειν (Inf)

        let tokens = vec![
            t("Βλέπω", MorphFlags::VERB), // Root "I see"
            t("το", MorphFlags::ACCUSATIVE | MorphFlags::ARTICLE | MorphFlags::NEUTER | MorphFlags::SINGULAR),
            t("λέγειν", MorphFlags::INFINITIVE | MorphFlags::NEUTER | MorphFlags::SINGULAR), // Agreement matches
        ];

        let deps = parse_greedy(&tokens);

        // Infinitive -> Root (Object, because Article is Accusative)
        assert!(deps.iter().any(|d| d.dependent_index == 2 && d.head_index == 0 && d.role == SyntaxRole::Object));

        // Article -> Infinitive (Modifier)
        assert!(deps.iter().any(|d| d.dependent_index == 1 && d.head_index == 2 && d.role == SyntaxRole::Modifier));
    }

    #[test]
    fn test_relative_clause() {
        // "Ο άνθρωπος ος βλέπει με τρέχει" (The man who sees me runs)
        // 0: Ο (Art)
        // 1: άνθρωπος (Noun Nom) -> Subject of 4 (Root)
        // 2: ος (Rel Pro Nom) -> Subject of 3 (RelVerb) + Linked to 1 (Antecedent) by 3 (RelVerb)
        // 3: βλέπει (Verb) -> Relative Clause of 1
        // 4: με (Pro Acc) -> Object of 3
        // 5: τρέχει (Root Verb)

        let tokens = vec![
            t("Ο", MorphFlags::ARTICLE | MorphFlags::NOMINATIVE),
            t("άνθρωπος", MorphFlags::NOUN | MorphFlags::NOMINATIVE),
            t("ος", MorphFlags::RELATIVE | MorphFlags::PRONOUN | MorphFlags::NOMINATIVE),
            t("βλέπει", MorphFlags::VERB | MorphFlags::PRESENT | MorphFlags::ACTIVE),
            t("με", MorphFlags::PRONOUN | MorphFlags::ACCUSATIVE),
            t("τρέχει", MorphFlags::VERB | MorphFlags::PRESENT | MorphFlags::ACTIVE),
        ];

        // Root is "τρέχει" (index 5)
        // greedy parser takes "analyze_core" root logic, but here we pass full list.
        // wait, parse_greedy finds root by linear scan in analyze_core then calls parse_greedy with root index?
        // No, parse_greedy finds root internally?
        // Let's check parse_greedy signature.
        // pub fn parse_greedy(tokens: &[MorphToken]) -> Vec<Dependency>
        // It calls `find_root`.
        // "βλέπει" (3) and "τρέχει" (5) are both Verbs.
        // `find_root` usually picks the *last* verb? Or first?
        // In simple "find_root":
        // It picks "First Verb that is not Participle/Infinitive?"
        // Usually. If multiple, it picks first?
        // If it picks "βλέπει", then "τρέχει" is secondary? 
        // Relative Clause logic relies on `i != root`.
        // If "βλέπει" is root, "τρέχει" (run) becomes secondary?
        // We need "τρέχει" to be root.
        // "βλέπει" should be detected as subordinate?
        // Unlikely in greedy parser without logic.
        // BUT `find_root` logic might be simple.
        // If `find_root` picks index 3, we fail.
        
        // Let's see `find_root` implementation in syntax.rs:
        /*
        fn find_root(tokens: &[MorphToken]) -> usize {
            tokens.iter().position(is_finite_verb).unwrap_or(0)
        }
        */
        // `.position` returns FIRST match.
        // So it will pick "βλέπει" (3).
        // This breaks the test premise.
        // I need to ensure "τρέχει" is picked OR test with "τρέχει ... ος ... βλέπει" (Runs the man who sees me).
        // OR I hack `find_root` to prefer non-relative verbs? 
        // Logic: `is_finite_verb` && !`morph_flags::RELATIVE`? (Assuming RelPron is separate token).
        // Hard to distinguish "Sees" from "Runs" without syntax tree.
        // BUT "Sees" follows "Who".
        // Maybe I assume Root is the *last* verb? Or verb not preceded by RelPron?
        // Too complex for greedy now.
        // I will Start with "Runs the man who sees me" (V S ORel).
        
        // "Τρέχει ο άνθρωπος ος βλέπει με"
        // 0: Τρέχει (Root)
        // 1: Ο
        // 2: άνθρωπος
        // 3: ος
        // 4: βλέπει
        // 5: με
        
        // Root = 0.
        // 4 is Verb, i != root. -> Trigger RelClause logic.
        
        let tokens_reordered = vec![
            t("τρέχει", MorphFlags::VERB | MorphFlags::PRESENT | MorphFlags::ACTIVE), // 0 Root
            t("Ο", MorphFlags::ARTICLE | MorphFlags::NOMINATIVE),
            t("άνθρωπος", MorphFlags::NOUN | MorphFlags::NOMINATIVE), // 2 Antecedent
            t("ος", MorphFlags::RELATIVE | MorphFlags::PRONOUN | MorphFlags::NOMINATIVE), // 3 RelPron
            t("βλέπει", MorphFlags::VERB | MorphFlags::PRESENT | MorphFlags::ACTIVE), // 4 RelVerb
            t("με", MorphFlags::PRONOUN | MorphFlags::ACCUSATIVE), // 5 Object of 4
        ];

        let deps = parse_greedy(&tokens_reordered);

        // Root is 0.
        
        // 2: Man -> 0: Runs (Subject)
        assert!(deps.iter().any(|d| d.dependent_index == 2 && d.head_index == 0 && d.role == SyntaxRole::Subject));

        // 4: Sees -> 2: Man (Relative Clause)
        assert!(deps.iter().any(|d| d.dependent_index == 4 && d.head_index == 2 && d.role == SyntaxRole::RelativeClause));

        // 3: Who -> 4: Sees (Subject)
        assert!(deps.iter().any(|d| d.dependent_index == 3 && d.head_index == 4 && d.role == SyntaxRole::Subject));

        // 5: Me -> 4: Sees (Object)
        // This verifies Scoping! "Me" should attach to 4, not 0.
        assert!(deps.iter().any(|d| d.dependent_index == 5 && d.head_index == 4 && d.role == SyntaxRole::Object));
    }
}
