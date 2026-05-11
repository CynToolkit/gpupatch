# gpupatch 🚀

A highly optimized, dependency-free Rust engine for hot-patching Windows Portable Executable (PE) binaries to enforce high-performance discrete GPU utilization.

[![Rust PE Patcher Verification](https://github.com/CynToolkit/gpupatch/actions/workflows/ci.yml/badge.svg)](https://github.com/CynToolkit/gpupatch/actions/workflows/ci.yml)

## Features
- ⚡ **Blazing Fast**: Native Rust implementation with zero external runtime dependencies.
- 🔬 **Ultra Reliable**: Passes 100% bitwise parity tests against reference C# implementation and performs safely even on heavyweight (100MB+) Chromium/Electron browser shells.
- 📦 **Modern PE Handling**: Complies strictly with official PE specification regarding ordinal sorting and manifest preservation.
- 💉 **Safe Injection**: Seamlessly injects `NvOptimusEnablement` and `AmdPowerXpressRequestHighPerformance` symbols into preexisting massive export tables.
- 🔥 **In-Place Patching**: Intelligently toggles symbols in-place on previously patched binaries instead of expanding executable footprint.

## Installation

### Option 1: Direct Install via Cargo (Recommended)
If you have Rust installed, you can compile and install globally with a single command:
```bash
cargo install --git https://github.com/CynToolkit/gpupatch.git
```

### Option 2: Building from Source
```bash
git clone https://github.com/CynToolkit/gpupatch.git
cd gpupatch
cargo build --release
```
The compiled binary will be available at `./target/release/gpupatch` (or `gpupatch.exe` on Windows).

## Usage

To force an executable to utilize the dedicated high-performance GPU:
```bash
gpupatch <input.exe> [<output.exe>]
```

To undo the patch:
```bash
gpupatch <input.exe> --disable
```

## Testing
The test suite includes unit test verification and deterministic parity simulation:
```bash
cargo test
```
