# Karnaugh Studio v1.0.0 Release Notes

**Release Date:** October 20, 2026  
**Version:** 1.0.0  
**Target Architecture:** Windows x86_64 (Windows 10 / Windows 11)  

---

## Overview

**Karnaugh Studio** is a high-performance native Windows desktop application engineered in **Rust** using **`eframe` / `egui` 0.31**. It provides an integrated, educational, and mathematically rigorous environment for Boolean logic design, Gray-code Karnaugh Map visualization with toroidal boundary wrapping, and exact two-level logic minimization for 2, 3, and 4 variables.

---

## Release Artifacts

| File | Size | Description |
| :--- | :--- | :--- |
| `KarnaughStudio.exe` | 5.29 MB | Standalone portable 64-bit Windows PE executable (zero external runtime dependencies). |
| `KarnaughStudio-Setup.msi` | 2.17 MB | Standard Windows Installer package with Start Menu integration and dual per-user / per-machine scope. |
| `SHA256SUMS.txt` | ~200 B | Cryptographic SHA-256 hashes for all release artifacts. |

---

## Key Highlights & Capabilities

### 1. Interactive Truth Table Editor & Validation Engine
- Edit truth tables for 2 ($A,B$), 3 ($A,B,C$), or 4 ($A,B,C,D$) variables.
- Interactive cell toggling: input literals ($0, 1$) and output states ($0, 1, X$).
- Real-time table integrity checks:
  - Detects duplicate minterms across rows.
  - Detects incomplete tables with missing minterm configurations.
  - Fully supports unordered row input arrangements.
  - Blocks minimization and displays inline visual error callouts when the table is invalid.
- One-click preset utilities: All 0s, All 1s, All Xs, and Canonical Natural / Gray order generation.

### 2. Gray-Code Karnaugh Map with Toroidal Boundary Wrapping
- Renders 2x2, 2x4, and 4x4 matrix layouts in standard Gray code sequence ($00, 01, 11, 10$) preserving Hamming distance adjacency.
- Custom vector rendering engine that segments implicant loops that wrap across map boundaries (top-bottom, left-right, and the 4 corners) into split rounded open-edge segments.
- Clear cell minterm decimal badges alongside boolean values.

### 3. Exact Boolean Minimization (SoP & PoS)
- Implements an exact Boolean logic minimizer using prime implicant generation and Petrick's branch-and-bound exact cover solver.
- Optimizes for:
  1. Minimum number of prime implicants.
  2. Minimum total literal count.
  3. Deterministic lexical tie-breaking.
- Supports both **Sum of Products (SoP)** and **Product of Sums (PoS)** minimization modes.
- Generates educational explanations detailing:
  - Essential prime implicants vs non-essential covers.
  - Minterm membership per group.
  - Specific eliminated variables per product/sum term.

### 4. Cross-Panel Synchronized Highlighting
- Interactive hover and selection states link:
  - Implicant cards in the Groups Panel.
  - Torus loop overlays on the Karnaugh Map.
  - Corresponding minterm rows in the Truth Table.
  - Individual terms within the synthesized Boolean formula.
- Shared 8-color accessible palette engineered for high contrast in both Light and Dark themes.

### 5. Windows Native Integration
- Embedded multi-resolution application icon (`.ico` bundling 16, 32, 48, 64, 128, and 256px images) compiled into the PE header via `winres`.
- High-DPI display scaling handled natively via `egui`.
- Clean single-file deployment or full MSI setup with Start Menu shortcuts.

---

## Bug Fixes & Refinements in v1.0.0

- **K-Map Group Boundary Segmentation**: Fixed `decompose_group_segments` where full-span groups (e.g. 2-cell columns in 2-variable, or 4-cell rows/columns in 4-variable, and full-map all-1/all-0 covers) were incorrectly treated as wrapping over torus boundaries, leaving open edges or zero outlines. Non-wrapping groups now render fully enclosed rounded rectangular outlines.
- **Full-Map Implicant Essential Badging**: Ensured full-map special cases (all-1 in SoP, all-0 in PoS) are correctly flagged with `is_essential: true` so the educational "Essential" badge displays consistently.
- **Decoupled UI Event Architecture**: Replaced mutable single-action enums in `TruthTable`, `KMapView`, `GroupsPanel`, and `FunctionPanel` with typed response structs (`action`, `hovered_*`, `selected_*`). This prevents continuous frame hover states from overwriting one-shot user actions such as cell clicks and clipboard copy operations.
- **Truth Table Quick Input Toggle**: Added right-click quick toggling (`0 <-> 1`) directly on truth table input cells.

---

## Verification & Quality Assurance

- **Unit & Integration Tests:** 25 automated tests passing across 3 test suites:
  - `kmap_mapping_tests`: 7 tests validating coordinate math, Gray code bijection, wrap decomposition, and closed boundary outlines for full-span/full-map groups.
  - `minimizer_tests`: 11 tests verifying exact cover optimality, don't-care expansion, all-don't-care edge cases, unordered tables, and duplicate detection.
  - `expression_equivalence_tests`: 7 tests utilizing an AST Boolean evaluator verifying that synthesized SoP/PoS formulas evaluate identically to truth tables across all $2^N$ states (tested on Half Adder, 3-Variable Majority, 4-Variable Multiplexer, 4-variable parity & primes, all 16 2-variable functions, and exhaustive evaluation across all 256 possible 3-variable functions).
- **Code Quality:**
  - `cargo fmt --check`: 100% compliant.
  - `cargo clippy --all-targets --all-features -- -D warnings`: 0 warnings.
