# Logos Omega Architecture (Monorepo)

## Directory Structure
*   'core/': The Kernel (Pure Rust, #![no_std] friendly)
    *   'logos-ecs': Data Model
    *   'logos-morph': FST Logic
    *   'logos-protocol': Shared schemas
*   'compilers/': Translators (Parser, Solver)
*   'platforms/': Adapters (WASM, CLI, Server)
*   'tools/': Data Ingestion
