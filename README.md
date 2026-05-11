# 🚀 gpupatch

<div align="center">

[![Github All Releases](https://img.shields.io/github/downloads/CynToolkit/gpupatch/total.svg?style=for-the-badge&color=blue)](https://github.com/CynToolkit/gpupatch/releases)
[![Latest Release](https://img.shields.io/github/v/release/CynToolkit/gpupatch?style=for-the-badge&color=success&label=Download%20Latest)](https://github.com/CynToolkit/gpupatch/releases/latest)
[![Workflow Status](https://img.shields.io/github/actions/workflow/status/CynToolkit/gpupatch/ci.yml?branch=main&style=for-the-badge)](https://github.com/CynToolkit/gpupatch/actions)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)

**A highly optimized, dependency-free native engine for hot-patching Windows executables to force high-performance discrete GPU utilization.**

---
</div>

## 📥 Direct Download (Fastest)
For standard users, bypass the command line entirely:
1. Click the green **[Download Latest Release](https://github.com/CynToolkit/gpupatch/releases/latest)** button above.
2. Under **Assets**, download `gpupatch-windows-amd64.exe`.
3. Put it on your desktop and you're ready to go!


## Features
- ⚡ **Blazing Fast**: Native Rust implementation with no external dependencies.
- 💉 **Safe Injection**: Seamlessly injects `NvOptimusEnablement` and `AmdPowerXpressRequestHighPerformance` symbols.
- 🔥 **In-Place Patching**: Intelligently toggles symbols on previously patched binaries.

## 💡 Key Improvements Over `nvpatch`
This tool is a from-scratch rewrite built to address edge-cases in the original [nvpatch](https://github.com/toptensoftware/nvpatch) tool:
- ✅ **Strict PE Compliance**: Correctly implements **Binary Ordinal Sorting** and header alignment.
- ✅ **Stand-Alone Binary**: Entirely bypasses the heavy .NET runtime install requirement.
- ✅ **Antivirus Safe**: Cleanses corrupt digital signatures to avoid heuristic malware traps.
- ✅ **Neophyte-Friendly**: Adds graphical drag-and-drop support with persistent lock-screens.

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

### 🔥 For Standard Users (Easy Mode)
The application includes an interactive mode designed for ease of use:
1. **Double-click** the downloaded `gpupatch.exe` binary to launch the console window.
2. **Drag and drop** your target game or application executable directly onto the window.
3. Hit **Enter**. That's it! 🎉
*(Alternatively, you can simply drag and drop an executable file directly onto the `gpupatch.exe` icon in Windows Explorer).*

### 💻 For Power Users (CLI Mode)
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
