# 📘 Domain Specification: The Greek Language Model (Advanced)

This document constitutes the **Linguistic Constitution** of Logos Omega. It defines the invariant rules, edge cases, and architectural representations of the Greek Language. 

**Strict Adherence Required:** The AI must not simplify these concepts. Greek is not English; do not force English grammar paradigms (like SVO rigidity) onto this model.

---

## 1. Morphological Attributes (The Atoms)

### A. Verbal Aspect & Tense (The Core Complexity)
Greek verb morphology relies on the intersection of **Time** and **Aspect**. Do not treat "Tense" as a flat list.

*   **Aspect (`Opsi`):**
    1.  `Imperfective` (Continuous/Repeated action) -> Stems: *grafr-*
    2.  `Perfective` (Completed/Single action) -> Stems: *graps-*
    3.  `Perfect` (Resultative state) -> Stems: *ech- graps-*

*   **Time (`Chronos`):**
    1.  `Past` (Augmented forms)
    2.  `NonPast` (Present/Future)

*   **Derived Tense System:**
    *   *Present* = NonPast + Imperfective
    *   *Imperfect* = Past + Imperfective
    *   *Aorist* = Past + Perfective
    *   *Future Continuous* = Particle `tha` + NonPast + Imperfective
    *   *Future Simple* = Particle `tha` + NonPast + Perfective

### B. Voice & Deponency
*   **Voice (`Foni`):** `Active`, `Passive` (includes Medio-Passive).
*   **Deponency Flag:** A Boolean Component.
    *   *Rule:* If `is_deponent == true`, the form is **Passive** but the Syntactic Role is **Active**.
    *   *Example:* `έρχομαι` (I come) -> Form: Passive, Meaning: Active Intransitive.

### C. Nominal Declension Classes
We do not just store "Noun." We store the **Declension Pattern** to predict behavior.
*   `Isosyllabic` (Same number of syllables in all cases).
*   `Anisosyllabic` (Adds syllables in plural, e.g., *giagiá* -> *giagiád-es*).
*   `Archaisms` (Irregular forms surviving from Ancient Greek, e.g., *to fos* -> *tou fotos*).

---

## 2. Tokenization & Orthography (The Lexer)

### A. Clitics (The Hidden Tokens)
Greek uses weak pronouns (Clitics) that attach phonologically to verbs.
*   **Proclitics:** Before the verb (*Mou* to edose).
*   **Enclitics:** After the verb/noun (*O anthropos mou*).
*   **Constraint:** The Parser must identify Clitics as distinct **Semantic Entities** even if they are phonologically bound (e.g., handling the accent shift: *ó ánthropos* -> *o ánthropós mou*).

### B. Euphonic Rules (Sandhi)
*   **Final 'N' (`Teliko Ni`):**
    *   Articles/Particles (*tin*, *ston*, *den*, *min*) retain `n` only before vowels and plosives (*k, p, t* and combinations).
    *   *Validator Rule:* Flag error if `τη γυναίκα` is written `την γυναίκα` (unless strict formal register is active).

### C. Normalization Standards
*   **Unicode:** Always `NFC`.
*   **Dialytika:** Handle `ϊ` and `ϋ` distinct from `ι` and `υ`.
*   **Sigma:** Auto-convert `σ` to `ς` at word boundaries during generation.

---

## 3. Syntax Constraints (The Physics)

### A. The Null Subject
*   **Rule:** Greek is a **Pro-Drop** language.
*   **ECS Implication:** A sentence might lack a `Subject` Entity.
*   **Logic:** If no Nominative Noun exists, the implicit subject is inferred from the Verb's Person/Number.
    *   *Sentence:* "Έρχομαι." -> *Subject:* Implicit(1st, Sg).

### B. Argument Structure (Valency)
*   **Transitive:** Requires Accusative Object.
*   **Intransitive:** Cannot take Direct Object.
*   **Ditransitive:** Requires Direct Object (Acc) + Indirect Object (Genitive OR `se` + Acc).
    *   *Example:* "Σου δίνω το βιβλίο" (Genitive Indirect) vs "Δίνω το βιβλίο σε εσένα" (Prepositional Indirect).

### C. Modification
*   **Adjectives:** Agree in Case/Gender/Number.
*   **Genitive Modifiers:**
    *   *Possessive:* Follows the noun ("Το σπίτι **του Γιάννη**").
    *   *Attributive:* Can precede in formal registers ("Ο **του Γιάννη** φίλος" - Rare/Archaic).

---

## 4. Semantics & Register (The Context)

### A. Register (`Ifos`)
Words carry a register tag to prevent mixing styles.
*   `Demotic` (Standard Modern).
*   `Katharevousa` (Archaic/Formal/Legal/Ecclesiastical).
*   `Slang` (Informal).
*   **Constraint:** A sentence should not mix `Slang` syntax with `Katharevousa` morphology without a "Style Clash" warning.

### B. Animacy
*   **Animate:** Humans/Animals (Relevant for `Accusative` rules, e.g., "whom" vs "which").
*   **Inanimate:** Objects/Concepts.

---

## 5. ECS Data Mapping (The Code)

### Component: `Morphology`
Use `bitflags` for zero-cost storage.

```rust
bitflags! {
    struct MorphFlags: u32 {
        const NOMINATIVE = 1 << 0;
        const GENITIVE   = 1 << 1;
        const ACCUSATIVE = 1 << 2;
        const MASCULINE  = 1 << 3;
        const FEMININE   = 1 << 4;
        const NEUTER     = 1 << 5;
        const SINGULAR   = 1 << 6;
        const PLURAL     = 1 << 7;
        const DEPONENT   = 1 << 8; // Form is passive, meaning is active
    }
}
```

### Component: `Dependency`
*   `head`: `EntityId` (The Governor).
*   `relation`: `EdgeLabel`.
    *   `nsubj` (Nominal Subject).
    *   `obj` (Direct Object).
    *   `iobj` (Indirect Object - Genitive Clitic).
    *   `obl` (Oblique - Prepositional Argument).
    *   `det` (Determiner - The article).
    *   `amod` (Adjectival Modifier).

---

## 6. Parsing Strategy (Ambiguity Resolution)

Greek is highly ambiguous morphologically.
*   *Example:* `οι` (Article Nom Pl Masc OR Nom Pl Fem).
*   *Example:* `διαβάζω` (Pres Ind Act 1sg OR Pres Subj Act 1sg).

**The Resolver Algorithm:**
1.  **Lexical Lookup:** Fetch all potential tags for the token.
2.  **Constraint Propagation:**
    *   If `Article` is Fem, `Noun` MUST be Fem.
    *   If `Noun` is Masc, discard the Fem option for the Article.
3.  **Syntactic Probability:** (Used in Phase 7)
    *   If ambiguity remains, prefer `Indicative` over `Subjunctive` unless the particle `να` or `ας` precedes.
```