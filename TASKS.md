# 🗺️ Master Execution Roadmap: Logos Omega

## 🟢 Phase 1: The Protocol (The Common Tongue)
*Goal: Define the binary contract that allows Python (Ingestion), Rust (Kernel), and WASM (Client) to speak without serialization overhead.*

- [ ] **Crate Setup**: Initialize `core/logos-protocol` library.
- [ ] **Linguistic Enums**: Define `Case`, `Gender`, `Number`, `Person`, `Tense`, `Voice`, `Mood`.
    - [ ] Derive: `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`.
    - [ ] Derive: `rkyv::Archive`, `rkyv::Serialize`, `rkyv::Deserialize`.
- [ ] **Entity Structs**: Define `LemmaId(u32)` and `ParadigmId(u32)` using the "Newtype" pattern.
- [ ] **The Dictionary Schema**: Define the `DictionaryBlob` struct that will hold the entire language map in the `rkyv` binary format.
- [ ] **Validation**: Write unit tests to ensure `rkyv` serialization/deserialization works byte-perfectly.

## 🔵 Phase 2: The Atlas Pipeline (Data Factory)
*Goal: Ingest raw chaos (Wiktionary/Text) and refine it into a pristine, zero-copy binary artifact.*

- [ ] **Environment**: Set up `tools/atlas-pipeline` with Python environment (`polars`, `pydantic`, `lxml`).
- [ ] **Scraper**: Implement `wiktionary_parser.py` to extract Greek Lemmata, Parts of Speech, and Inflection Tables.
- [ ] **Normalizer**: Implement `cleaner.py`.
    - [ ] Enforce `Unicode NFC` normalization.
    - [ ] Strip accents for "Phonetic Hashing" (building the lookup key).
- [ ] **Paradigm Inference**: Write logic to group verbs with identical conjugation patterns into `Paradigm` IDs.
- [ ] **The Compiler**: Implement `compiler.py` (or a Rust CLI tool) to map the cleaned JSON into the `logos-protocol` binary format (`dict.rkyv`).

## 🟣 Phase 3: The Morphology Engine (FST)
*Goal: Deterministic generation and analysis of word forms. "If I have the stem, I can generate the universe."*

- [ ] **Crate Setup**: Initialize `core/logos-morph`.
- [ ] **Paradigm Logic**: Implement the transformation logic (Suffix Stripping / Stem Mutation).
- [ ] **Generator**: Implement `fn generate(lemma_id, tags) -> String`.
- [ ] **Analyzer**: Implement `fn analyze(surface_form) -> Vec<(LemmaId, Tags)>`.
- [ ] **Fuzzing (Crucial)**: Set up `proptest`.
    - [ ] Property: `analyze(generate(lemma)) == lemma`.
- [ ] **Benchmarks**: Measure lookup speed (Target: < 100ns).

## 🟠 Phase 4: The Parser & Lexer (Nom)
*Goal: Turn a raw string into a stream of potential grammatical meanings.*

- [ ] **Crate Setup**: Initialize `compilers/logos-parser`.
- [ ] **Tokenizer**: Use `nom` to handle Greek punctuation, enclitics (words merging like `σ'αγαπώ`), and spacing.
- [ ] **The Lexer**:
    - [ ] Load `dict.rkyv` (via `logos-protocol`).
    - [ ] Map tokens to `LemmaId` candidates.
    - [ ] Handle "Unknown Tokens" (Out-of-Vocabulary) gracefully.
- [ ] **Resilience**: Ensure the parser recovers from errors and doesn't panic on malformed input.

## 🔴 Phase 5: The ECS Kernel (The Runtime)
*Goal: The state machine. Where words become "Entities" and rules become "Systems."*

- [ ] **Crate Setup**: Initialize `core/logos-ecs`.
- [ ] **ECS Stack**: Integrate `hecs` (or `bevy_ecs`).
- [ ] **Components**:
    - [ ] `TokenComponent`: Stores string slice and position.
    - [ ] `MorphComponent`: Stores the grammatical tags.
    - [ ] `SyntaxComponent`: Stores `HeadID` (Dependency parsing).
- [ ] **System: Agreement**: Write the logic to check Subject-Verb agreement.
- [ ] **System: Case Governance**: Write the logic to check Preposition-Case governance (e.g., "με" takes Accusative).
- [ ] **The VM**: Create a `World` struct that initializes the ECS for a given sentence.

## 🟤 Phase 6: The Semantic Solver (Neuro-Symbolic)
*Goal: Disambiguation and Fact-Checking.*

- [ ] **Crate Setup**: Initialize `compilers/logos-solver`.
- [ ] **Constraint Solver**:
    - [ ] Input: A Parse Forest (multiple possible valid syntax trees).
    - [ ] Logic: Apply semantic weights to pick the most likely tree.
- [ ] **Knowledge Graph**: Define a minimal `Graph` trait using `petgraph`.
- [ ] **Integration**: Connect Entities to Wikidata IDs (if data allows).

## ⚪ Phase 7: The Interface (Delivery)
*Goal: Putting it in the user's hands.*

- [ ] **WASM Binding**: Create `platforms/logos-wasm`.
    - [ ] Expose `fn validate_text(input: &str) -> JsValue` (JSON trace).
    - [ ] Implement Memory-Mapping for the `dict.rkyv` file in the browser.
- [ ] **Server API**: Create `platforms/logos-server` (Axum).
    - [ ] Endpoint: `POST /analyze`.
- [ ] **CI/CD**:
    - [ ] GitHub Actions to build WASM and release to GitHub Pages.
    - [ ] Automated regression testing suite.

