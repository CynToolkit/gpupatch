# Plan: Update Cargo.toml Metadata

## Objective
Add missing metadata fields (`documentation`, `homepage`, `repository`) to `Cargo.toml`.

## Changes
- Update `Cargo.toml` in the root directory.
- Add the following fields to the `[package]` section:
    - `documentation = "https://github.com/CynToolkit/gpupatch"`
    - `homepage = "https://github.com/CynToolkit/gpupatch"`
    - `repository = "https://github.com/CynToolkit/gpupatch"`

## Verification
- Run `cargo metadata --format-version 1` to verify the fields are correctly recognized.
