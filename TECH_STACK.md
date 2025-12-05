# 🛠️ Logos Omega: Technology Stack & Constraints

This document defines the approved technologies, libraries, and patterns for the Logos Omega project. 
**Strict adherence is required** to maintain the "Zero-Latency / Zero-Copy" performance goals.

---

## 1. The Core Kernel (Rust)
*Target: `core/*` and `compilers/*` crates.*

| Category | Crate / Tool | Logic / Justification |
| :--- | :--- | :--- |
| **Language** | **Rust (2021)** | Memory safety without Garbage Collection. |
| **ECS** | **`hecs`** | Lightweight Entity-Component-System. Preferred over `bevy_ecs` for smaller WASM binary size and simpler API. |
| **Serialization** | **`rkyv`** | **Critical.** Zero-Copy deserialization. We do not parse JSON at runtime; we `mmap` binary blobs. |
| **Parsing** | **`nom`** | Parser Combinators. Used for tokenizing Greek text. Faster and more composable than Regex. |
| **Morphology** | **`fst`** | Finite State Transducers for compact storage of millions of string patterns. |
| **Graph** | **`petgraph`** | For modeling dependency trees (Syntax) and Knowledge Graphs (Semantics). |
| **Error Handling** | **`thiserror`** | For library code (structured enums). |
| **Error Handling** | **`anyhow`** | For application/binary code (top-level results). |
| **Logging** | **`tracing`** | Structured, async-aware logging. |
| **Bit Manipulation** | **`bitflags`** | For compact storage of morphological tags (e.g., `Case::Nom | Number::Sg`). |

---

## 2. The Data Factory (Python)
*Target: `tools/atlas-pipeline`. Used ONLY for offline data ingestion.*

| Category | Library | Usage |
| :--- | :--- | :--- |
| **Language** | **Python 3.10+** | Scripting speed and ecosystem richness. |
| **Dataframes** | **`polars`** | High-performance data manipulation (Rust-backed). Replaces Pandas. |
| **Scraping** | **`beautifulsoup4`** | Parsing Wiktionary HTML dumps. |
| **Validation** | **`pydantic`** | Enforcing strict schemas on raw ingested data before compilation. |
| **XML Parsing** | **`lxml`** | Fast XML processing for Wiktionary dumps. |

---

## 3. The Interface (Adapters)
*Target: `platforms/*`.*

| Category | Crate | Usage |
| :--- | :--- | :--- |
| **WASM** | **`wasm-bindgen`** | Binding Rust logic to JS/Browser. |
| **WASM Utils** | **`console_error_panic_hook`** | For debugging Rust panics in the browser console. |
| **Server** | **`axum`** | High-performance, async HTTP server (if API is needed). |
| **Runtime** | **`tokio`** | Async runtime for the Server (NOT used in the Kernel). |
| **CLI** | **`clap`** | Command-line argument parsing for developer tools. |

---

## 4. Quality Assurance (Testing)

| Category | Crate | Usage |
| :--- | :--- | :--- |
| **Unit Testing** | **`cargo test`** | Standard library testing. |
| **Fuzzing** | **`proptest`** | **Mandatory.** Property-based testing for Morphology. (e.g., `assert(analyze(generate(x)) == x)`). |
| **Benchmarking** | **`criterion`** | Statistical benchmarking to ensure sub-millisecond latency. |

---

## 5. ⛔ Constraints & Anti-Patterns (The "Do Not" List)

1.  **NO Runtime Regex for Grammar:**
    *   Do not use Regex to validate sentence structure. It is insufficiently powerful (Regular Languages cannot parse Recursive Syntax). Use `nom` or the ECS Constraint Solver.

2.  **NO `serde_json` in the Hot Path:**
    *   JSON is for logging or API boundaries only. The core engine must load data via `rkyv` (binary).

3.  **NO Garbage Collection:**
    *   Do not use Python, Node.js, or Go for the `core/` logic.

4.  **NO `unwrap()` in Production:**
    *   Core logic must handle every edge case. Use `?` propagation or distinct Error Enums.

5.  **NO Blocking I/O in WASM:**
    *   The browser main thread must never block. All heavy computation must be chunked or async.

6.  **NO Deep OOP Hierarchies:**
    *   Do not create `class Verb extends Word`. Use Composition: `Entity` has `VerbComponent`.

---

## 6. Build Tooling

*   **Task Runner:** `just` (Justfile) - Standardizes commands (`just build`, `just test`, `just bench`).
*   **Linter:** `clippy` - Must pass with no warnings.
*   **Formatter:** `rustfmt`.