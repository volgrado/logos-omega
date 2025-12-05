# 🏛️ Logos Omega: System Architecture

## Overview
Logos Omega is designed as a **Computational Linguistics Appliance**. It follows a **Hexagonal Architecture** (Ports & Adapters) implemented within a **Rust Cargo Workspace**.

The system is divided into three "Zones":
1.  **The Factory (Offline):** Ingests raw data and compiles it into optimized binary artifacts.
2.  **The Kernel (Core):** Pure, platform-agnostic logic (Morphology, Syntax, ECS).
3.  **The Interface (Adapters):** Connects the Kernel to the outside world (Web, CLI, API).

---

## 📂 Directory Structure (The Monorepo)

```text
logos-root/
├── Cargo.toml                  # Workspace Root
├── justfile                    # Command runner (build, test, deploy)
│
├── core/                       # 🧠 THE KERNEL (Pure Rust, no I/O)
│   ├── logos-protocol/         # Shared Types, Traits & Rkyv Schemas
│   ├── logos-morph/            # Finite State Transducers (Generation/Analysis)
│   └── logos-ecs/              # The Runtime (Entity-Component-System logic)
│
├── compilers/                  # 🗣️ THE TRANSLATORS
│   ├── logos-parser/           # Text -> ECS Entities (Nom-based)
│   └── logos-solver/           # Constraint Solver & Semantic Resolver
│
├── platforms/                  # 🔌 THE ADAPTERS (Hexagonal Ports)
│   ├── logos-wasm/             # Browser Bindings (wasm-bindgen)
│   ├── logos-server/           # HTTP API (Axum)
│   └── logos-cli/              # Developer Tools
│
└── tools/                      # 🏭 THE FACTORY
    └── atlas-pipeline/         # Python/Rust ETL Pipeline
        ├── ingest/             # Scrapers (Wiktionary/UD)
        ├── clean/              # Normalization (NFC, Accent stripping)
        └── compile/            # Serializer (JSON -> .rkyv)
```

---

## 🔄 Data Flow Pipeline

### 1. Build Time (The "Atlas" Pipeline)
*Goal: Transform chaotic human data into ordered machine binary.*

1.  **Input:** Raw Wiktionary Dumps (XML/JSON) & Universal Dependencies Treebanks.
2.  **Process:** `tools/atlas-pipeline` scrapes, cleans, and structures the data.
3.  **Output:** `dict.rkyv` (A memory-mapped binary archive containing the Dictionary and Paradigms).
    *   *Constraint:* This process happens **offline**. We never parse XML at runtime.

### 2. Initialization (Runtime Boot)
*Goal: 0ms Startup Time.*

1.  **Load:** The application (WASM or Server) acts as a host.
2.  **Map:** It loads `dict.rkyv` into memory using `rkyv::Archived<Dictionary>`.
    *   *Zero-Copy:* No parsing occurs. The bytes on disk are the bytes in memory.
3.  **Inject:** The reference to the Dictionary is passed to the `LogosContext`.

### 3. Execution (The "Thought" Loop)
*Goal: Validate and Analyze.*

1.  **Input:** User types text: *"Ο Γιάννης..."*
2.  **Parser:** `logos-parser` tokenizes the string.
    *   Lookups are performed against the `dict.rkyv` blob.
3.  **ECS Instantiation:**
    *   Each token becomes an **Entity**.
    *   Properties (Case, Gender, Tense) are attached as **Components**.
4.  **Systems Execution:**
    *   `MorphologySystem`: Checks word forms.
    *   `AgreementSystem`: Validates Subject-Verb / Article-Noun agreement.
    *   `DependencySystem`: Builds the Syntax Tree.
5.  **Output:** A structured `AnalysisResult` (JSON) containing the AST, errors, and metadata.

---

## 🧩 Component Breakdown

### A. `core/logos-protocol`
*   **Role:** The "Common Tongue."
*   **Contents:** All Enums (`Case`, `Tense`), Structs (`Lemma`, `Paradigm`), and Traits (`KnowledgeBase`).
*   **Key Tech:** `rkyv`, `bitflags`.

### B. `core/logos-morph` (The FST)
*   **Role:** The Engine of Form.
*   **Logic:** It does not store every word. It stores **Stems** and **Suffix Tables** (Paradigms).
*   **Functions:**
    *   `generate(lemma_id, tags) -> String`
    *   `analyze(surface_form) -> Candidates`

### C. `core/logos-ecs` (The Logic)
*   **Role:** The State Machine.
*   **Pattern:** Entity-Component-System.
*   **Why ECS?** Greek sentences are non-linear (Free Word Order). An Object-Oriented tree hierarchy is too rigid. ECS allows us to query relationships (`Query<(&Subject, &Verb)>`) regardless of position.

### D. `compilers/logos-parser`
*   **Role:** The Bridge.
*   **Tech:** `nom` (Parser Combinators).
*   **Logic:** Handles punctuation, enclitics, and unicode normalization. It feeds the ECS.

---

## 🛡️ The "Neuro-Symbolic Firewall" Pattern

When integrating with LLMs (Phase 7), the architecture acts as a constraint filter:

1.  **LLM:** Predicts next token logits.
2.  **Logos Engine:** Analyzes the partial sentence constructed so far.
3.  **Intervention:**
    *   If the Syntax System expects a `Verb`, the Engine masks out all Non-Verb tokens from the LLM's vocabulary.
4.  **Result:** The LLM is forced to be grammatically correct.

---

## 📏 Architectural Constraints

1.  **No Cyclic Dependencies:** `core` crates must never depend on `platforms` or `compilers`.
2.  **Platform Agnostic:** `core` must be `#![no_std]` compatible (or at least avoid OS-specific calls) to ensure it runs in WASM.
3.  **Fail-Fast:** Invalid state should be unrepresentable in the Type System where possible.
```