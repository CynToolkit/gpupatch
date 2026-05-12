# Implementation Plan: Companion GUI Binary

## Objective
Create a minimal, stylish companion binary in Rust to provide a GUI interface for the project.

## Proposed Strategy
I recommend **egui** as the primary framework. It is pure Rust, provides a modern, clean "immediate-mode" look that fits perfectly with systems tools, and maintains an extremely small binary footprint.

### Why egui?
- **Footprint:** Single, dependency-light binary.
- **Performance:** Exceptionally fast and responsive.
- **Development:** Zero build-script or asset-compilation complexity; the UI is defined in the same Rust code as the logic.
- **Styling:** Out-of-the-box dark-mode support and clean aesthetics suitable for a "pro" tool.

## Implementation Steps

1. **Project Scaffolding**
   - Create `companion/` directory in the project root.
   - Initialize a new Rust workspace member: `cargo new companion`.
   - Add `egui` and `eframe` (the native framework for egui) to `companion/Cargo.toml`.

2. **Basic GUI Setup**
   - Implement the `eframe::App` trait in `companion/src/main.rs`.
   - Define a simple window layout that mirrors the core project's status/actions.
   - Configure a minimal entry point that starts the `eframe` event loop.

3. **Styling & Polish**
   - Customize `egui::Visuals` to match a cohesive theme (e.g., modern dark theme).
   - Ensure window responsiveness.

4. **Integration**
   - Expose core logic from the existing `src/` to the `companion/` crate (e.g., via a library crate `shared/` if necessary, or simply linking the core logic).
   - Verify communication between the GUI and core system tools.

## Verification
- **Build size check:** Ensure the compiled binary stays under 10-15MB.
- **Functional test:** Launch the binary and confirm it opens the window correctly and renders the UI elements.
- **Resource usage:** Monitor baseline RAM usage (aiming for <50MB).

---
*Does this approach align with your expectations for the companion binary, or would you prefer a more design-heavy framework like Tauri?*
