# Dotnet — Standalone Dev Manager (Rust Engine)

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows%2010%2F11%20x64-lightgrey.svg)](https://microsoft.com/windows)
[![Rust](https://img.shields.io/badge/Rust-1.85%2B-orange.svg)](https://www.rust-lang.org)

High-performance, native Windows local development environment manager rewritten in Rust. Replaces the original C# .NET WinForms architecture for sub-10MB RAM usage, near-instant startup, and zero runtime dependencies.

---

## Workspace Architecture

- **`crates/dotnet-windows`**: Direct Win32 FFI bindings (`iphlpapi.dll` for `GetExtendedTcpTable`, Registry `REG_EXPAND_SZ`, and `WM_SETTINGCHANGE` broadcast).
- **`crates/dotnet-core`**: Core domain logic, tool catalog with embedded manifests, environment PATH editor, atomic JSON persistence, and INI configuration engine.
- **`crates/dotnet-cli`**: Command-line developer client (`dotnet.exe`).
- **`crates/dotnet-app`**: Desktop GUI and System Tray companion.

---

## Migration Documentation

Detailed architecture specifications and community coding standards are located in the `docs/` folder:

- 📘 [**MIGRATION-SPEC.md**](docs/MIGRATION-SPEC.md) — Technical mapping from C# to Rust.
- 🦀 [**RUST-BEST-PRACTICES.md**](docs/RUST-BEST-PRACTICES.md) — Rust community best practices and performance guidelines.
- 💡 [**DOMAIN-KNOWLEDGE.md**](docs/DOMAIN-KNOWLEDGE.md) — Ported business rules, managed blocks, and security models.
- 📊 [**MIGRATION-PROGRESS.md**](docs/MIGRATION-PROGRESS.md) — Phase-by-phase task tracking.

---

## Building from Source

### Prerequisites
- Windows 10/11 64-bit
- [Rust toolchain](https://rustup.rs/) (Stable channel with MSVC target)

```powershell
# 1. Clone repository
git clone https://github.com/JackBerck/dotnet.git
cd dotnet/dotnet-rs

# 2. Check compilation across workspace
cargo check

# 3. Run unit tests
cargo test

# 4. Run CLI tool
cargo run -p dotnet-cli -- ports
```

---

## License
MIT License.
