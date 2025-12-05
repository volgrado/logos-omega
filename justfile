set shell := ["bash", "-c"]

# --- DEFAULT ---
default: test

# --- DEVELOPMENT ---

# Build the core kernel and compilers
build:
    @echo "Building Rust Core..."
    cargo build --workspace --lib

# Run all unit tests and property-based tests
test:
    @echo "Running Tests..."
    cargo test --workspace

# Run Clippy (Strict Mode - Fails on warnings)
lint:
    @echo "Running Clippy..."
    cargo clippy --workspace --all-targets -- -D warnings

# Format code
fmt:
    cargo fmt --all

# --- DATA PIPELINE ---

# Setup the Python environment for the Atlas Pipeline
setup-atlas:
    cd tools/atlas-pipeline && python3 -m venv venv
    @echo "Activate with: source tools/atlas-pipeline/venv/bin/activate"

# Run the ingestion (Placeholder command for Phase 2)
ingest:
    @echo "Running Atlas Pipeline..."
    # cd tools/atlas-pipeline && ./venv/bin/python main.py

# --- WASM ---

# Build the WASM package (for Phase 7)
build-wasm:
    cd platforms/logos-wasm && wasm-pack build --target web