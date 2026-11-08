# Karnaugh Studio

**Karnaugh Studio** is an end-to-end native Windows desktop application written in **Rust** using **`eframe` / `egui` 0.31**. It provides an interactive environment for digital logic analysis, truth table synthesis, Gray-code Karnaugh Map visualization with toroidal boundary wrapping, and exact Boolean logic minimization for 2, 3, and 4 variables.

---

## Features

### 1. Interactive Truth Table Editor
- **Multi-Variable Support**: Supports 2-variable ($A, B$), 3-variable ($A, B, C$), and 4-variable ($A, B, C, D$) boolean functions.
- **Manual Cell Editing**: Click to toggle inputs ($0, 1$) and outputs ($0, 1, X$ for Don't Care).
- **Strict Validation Engine**:
  - Detects duplicate minterm rows with exact conflict highlighting.
  - Identifies missing minterm combinations.
  - Accommodates unordered arbitrary row sequences.
  - Blocks minimization computation and surfaces clear inline warnings when invalid.
- **Batch Quick Actions**: One-click presets for *All 0*, *All 1*, *All X*, and *Canonical Rows (Gray / Natural)*.

### 2. Toroidal Karnaugh Map Visualization
- **Gray Code Matrix**: Cell indexing follows Gray code progression ($00, 01, 11, 10$) along both axes for rigorous Hamming distance adjacency.
- **Toroidal Boundary Wrap Decomposition**:
  - Groups that wrap across map edges or the 4 corners are decomposed into split open-edge rounded segments.
  - Visually illustrates cyclic boundary adjacency without graphical distortion.
- **Subtle Minterm Badges**: Each cell clearly displays its minterm decimal index alongside its boolean value.

### 3. Exact Boolean Minimization (SoP & PoS)
- **Petrick's Exact Cover Algorithm**: Finds the absolute minimum implicant set using branch-and-bound optimization with minimum literals and deterministic tie-breaking.
- **Sum of Products (SoP)**: Minimizes minterms ($1$) while leveraging don't-cares ($X$).
- **Product of Sums (PoS)**: Minimizes maxterms ($0$) using duality with inverted don't-cares.
- **Educational Explanations**:
  - Step-by-step implicant derivation.
  - Lists essential prime implicants vs optional primes.
  - Details exact eliminated variables for each grouped implicant.

### 4. Cross-Panel Synchronized Linking
- Hovering or selecting an implicant group in the **Groups Panel** highlights:
  - Corresponding grouped cells and wrap loops on the **Karnaugh Map**.
  - All contributing input rows in the **Truth Table**.
  - The associated product/sum term in the **Function Panel**.
- Consistent 8-color accessible palette shared across loops, badges, and formula terms.

### 5. Windows Native Integration & Deliverables
- **Embedded Application Icon**: Multi-resolution Windows PE icon (`.ico` containing 16, 32, 48, 64, 128, and 256px frames) embedded directly into the PE header via `winres`.
- **Light & Dark Theme Engine**: High-contrast, clean visual styling adhering to modern Windows desktop ergonomics.
- **Standalone Release Binary**: Zero external runtime dependencies; compiled directly to native x64 machine code (`dist/KarnaughStudio.exe`).
- **WiX MSI Installer**: Standalone setup package (`dist/KarnaughStudio-Setup.msi`) configured for per-user and per-machine deployments.

---

## Project Structure

```
├── assets/
│   ├── icon-256.png           # 256x256 PNG raster asset
│   ├── icon.ico               # Multi-resolution Windows icon (16..256px)
│   └── logo.svg               # Vector geometric K-map logo
├── dist/
│   ├── KarnaughStudio.exe     # Release binary (5.29 MB)
│   ├── KarnaughStudio-Setup.msi # Windows Installer package (2.17 MB)
│   ├── RELEASE_NOTES.md       # Release documentation & changelog
│   └── SHA256SUMS.txt         # Cryptographic checksums
├── installer/
│   └── karnaugh-studio.wxs    # WiX Toolset v5 setup definition
├── src/
│   ├── app.rs                 # KarnaughStudioApp top-level UI controller
│   │   ├── expression.rs      # AST Boolean evaluator for formal verification
│   │   ├── implicant.rs       # Implicant bitmask models and term formatting
│   │   ├── kmap.rs            # Gray-code coordinate mapping & torus segmenter
│   │   ├── minimizer.rs       # Petrick exact-cover minimizer
│   │   ├── model.rs           # Truth table and cell data structures
│   │   └── validation.rs      # Table validation & row error detection
│   ├── main.rs                # Windows PE entrypoint and eframe bootstrapper

│   └── ui/                    # Presentation components
│       ├── function_panel.rs  # Minimized formula presentation & clipboard
│       ├── groups_panel.rs    # Implicant group cards & educational details
│       ├── kmap_view.rs       # Interactive grid & loop painter
│       ├── theme.rs           # Palettes and egui Visuals configuration
│       ├── toolbar.rs         # Navigation, variable switcher, theme toggle
│       └── truth_table.rs     # Editable truth table & inline error tooltips
└── tests/
    ├── expression_equivalence_tests.rs # Automated truth-table equivalence checks
    ├── kmap_mapping_tests.rs           # Gray code, adjacency, and torus geometry
    └── minimizer_tests.rs              # 2/3/4-variable solver validation
```

---

## Building from Source

### Prerequisites
- **Rust Toolchain**: 1.80.0 or later with `x86_64-pc-windows-msvc` target.
- **WiX Toolset**: (Optional, for building the installer) WiX v5 (`dotnet tool install --global wix`).

### Compilation
To compile and run the application in release mode:
```powershell
cargo run --release
```

To build release deliverables into `dist/`:
```powershell
cargo build --release
Copy-Item target\release\karnaugh-studio.exe dist\KarnaughStudio.exe
```

To compile the MSI setup installer:
```powershell
wix build installer\karnaugh-studio.wxs -arch x64 -o dist\KarnaughStudio-Setup.msi
```

---

## Quality & Verification

The codebase adheres to zero compiler warnings and zero clippy lints:

```powershell
# Run the complete test suite (25 automated tests)
cargo test --all-targets

# Verify code formatting
cargo fmt --check

# Enforce strict clippy linter passes
cargo clippy --all-targets --all-features -- -D warnings
```

### Automated Test Coverage
- **`kmap_mapping_tests`**: Validates Gray code ordering, cell coordinate projection, cyclic boundary distance, segment decomposition for corner/edge wraps, and closed-outline preservation for full-span and full-map groups.
- **`minimizer_tests`**: Tests single-literal functions, 4-corner wraps, full tables, don't-care optimization, all-don't-care edge cases, unordered inputs, and duplicate row detection.
- **`expression_equivalence_tests`**: Uses the formal `src/expression.rs` evaluator to mathematically verify that synthesized SoP and PoS boolean expressions evaluate identically to their truth table specifications across all possible input permutations (tested on Half Adder, 3-Variable Majority, 4-Variable Multiplexer, 4-variable parity and prime checkers, all 16 2-variable functions, and exhaustive check across all 256 3-variable functions).

---

## License
Distributed under the [MIT License](LICENSE).
