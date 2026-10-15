# Rust Coding Standards & Best Practices

> Guidelines for high-performance, idiomatic, and maintainable Rust code in the `dotnet-rs` workspace, aligned with modern community standards.

---

## 1. Core Principles

1. **Safety First**:
   - Limit `unsafe` strictly to FFI calls (`dotnet-windows`).
   - Every `unsafe` block must include a `// SAFETY:` comment explaining why invariants are preserved.
   - All `unsafe` functions must be wrapped in safe, idiomatic Rust abstractions.

2. **Zero-Cost Abstractions & Minimal Allocations**:
   - Prefer borrowed types (`&str`, `&[T]`, `&Path`) over owned types (`String`, `Vec<T>`, `PathBuf`) in function signatures.
   - Use `Cow<'a, str>` when data may or may not need allocation.
   - Avoid `.clone()` in hot loops or high-frequency polling routines.
   - For string concatenation in tight paths, allocate with `String::with_capacity(n)` or use formatted buffers.

3. **Explicit Error Handling (No Panics)**:
   - Never use `unwrap()` or `expect()` in production library code. Use `?` or explicit matching.
   - Use `thiserror` for typed library errors (`dotnet-core`, `dotnet-windows`).
   - Use `anyhow` with `.context(...)` in executable crates (`dotnet-cli`, `dotnet-app`).
   - Never discard errors silently (`let _ = ...` is forbidden unless explicitly logged or intentionally ignored with a comment).

---

## 2. Windows API & FFI Best Practices

1. **Lightweight FFI with `windows-sys`**:
   - Prefer `windows-sys` over the monolithic `windows` crate to keep compilation times low and binaries minimal.
   - Guard against buffer overruns when calling Win32 APIs returning variable-length structures (e.g., `GetExtendedTcpTable`).
   - Use `to_wide()` helper with trailing null terminator for Win32 Unicode (`*const u16`) calls.

2. **Process & Handle Hygiene (RAII)**:
   - Always implement `Drop` or use RAII guards (`CloseHandle`, `RegCloseKey`) to prevent resource leaks.
   - When launching child processes via `tokio::process::Command`, explicitly drain or redirect `stdout` and `stderr` to prevent OS pipe buffer deadlocks.

---

## 3. Concurrency & Async Rules

1. **Tokio Async Engine**:
   - Keep async tasks short-lived or cooperative.
   - Never perform long-running CPU-bound calculations on Tokio worker threads; offload to `tokio::task::spawn_blocking`.
   - Never use blocking std I/O (`std::fs`, `std::thread::sleep`) inside async functions; use `tokio::fs` and `tokio::time::sleep`.

2. **Synchronization**:
   - Prefer `parking_lot::RwLock` over `std::sync::RwLock` (faster, smaller footprint, no poisoning overhead).
   - Minimize lock contention: perform calculations outside locks, clone minimal handles, and drop locks immediately.

---

## 4. File System & Atomicity

1. **Atomic File Updates**:
   - Configuration files (`hosts`, `nginx.conf`, `settings.json`, `projects.json`) must NEVER be overwritten directly.
   - Pattern: Write to `<file>.tmp` in the same directory → flush/sync → atomic rename via `std::fs::rename`.
   - Before modifying system files (e.g. `hosts` or `User PATH`), always create a timestamped backup in `%LOCALAPPDATA%\Dotnet\backups\`.

2. **Path Sanitization & Security**:
   - Defend against Zip-Slip attacks during extraction by verifying that every extracted entry's canonical path begins with the target extraction directory.
   - Enforce non-elevated user-space operations unless administrative rights are strictly necessary.

---

## 5. Testing & Quality Assurance

1. **Unit Tests**:
   - Place unit tests in the same file within `#[cfg(test)] mod tests`.
   - Test pure logic (e.g. `PathEditor`, `IniDocument`, `HostsParser`) with zero OS dependencies.
2. **Integration Tests**:
   - Place integration tests in `crates/*/tests/`.
   - Use `tempfile::tempdir()` to isolate filesystem mutations from the host machine.
3. **Linter & Formatting**:
   - Enforce `cargo fmt --check` and `cargo clippy -- -D warnings`.
