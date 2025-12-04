# Domain Specification: Greek Language Model

## 1. Morphological Attributes (Enums)
*   **Case:** Nominative, Genitive, Accusative, Vocative.
*   **Gender:** Masculine, Feminine, Neuter.
*   **Number:** Singular, Plural.
*   **Person:** First, Second, Third.
*   **Tense:** Present, Imperfect, Future, Aorist, Perfect, Pluperfect.
*   **Voice:** Active, Passive, Medio-Passive.

## 2. ECS Mapping
*   **Entity:** A distinct word token.
*   **Component 'Token':** The raw string and Lemma ID.
*   **Component 'Morphology':** Bitmask of Case/Gender/Tense.
*   **Component 'Dependency':** ID of the syntactic parent.
