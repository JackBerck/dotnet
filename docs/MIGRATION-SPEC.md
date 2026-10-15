# MIGRATION-SPEC: C# (.NET 10) to Rust (Windows Native)

> Technical mapping, architecture decisions, and porting blueprint for migrating `JackBerck/dotnet` from .NET 10 (WinForms) to native Rust.

---

## 1. Executive Summary & Goals

The original application is a Windows developer environment manager (open-source Laragon alternative) managing PHP, Node.js, Nginx, MySQL, PostgreSQL, hosts files, local SSL, and project domains.

### Why Migrate to Rust?
1. **Zero Runtime & Single Binary**: Eliminates .NET runtime dependencies; smaller distribution size (~15MB vs ~70MB self-contained).
2. **Deterministic Memory Footprint**: Eliminates Garbage Collection (GC) pauses; idle RAM drops from ~80MB (.NET WinForms) to < 10MB.
3. **True Native Win32 Integration**: Direct FFI with `windows-sys` for `iphlpapi.dll` (`GetExtendedTcpTable`), Registry, and Service Controller without marshalling overhead.
4. **Fearless Concurrency**: Async background port polling, process surveillance, and file watcher without threadpool starvation or race conditions.

---

## 2. Architecture & Crate Layout

```
dotnet-rs/
├── .cargo/
│   └── config.toml           # MSVC optimization, static CRT, lto, strip
├── Cargo.toml                # Root workspace configuration
├── crates/
│   ├── dotnet-windows/       # Direct Win32 FFI (IP helper, registry, services)
│   ├── dotnet-core/          # Core domain, catalog, installer, persistence, configs
│   ├── dotnet-cli/           # Headless developer CLI
│   └── dotnet-app/           # GUI & System Tray native client
└── docs/                     # Architecture, specifications, and knowledge base
```

### Module Mapping (.NET C# → Rust)

| .NET Component (`dotnet-cs`) | Rust Workspace Target (`dotnet-rs`) | Crate / Idiom |
|---|---|---|
| `PortConflictDetector.cs` (P/Invoke `iphlpapi.dll`) | `dotnet-windows::ip_helper` | `windows-sys` `GetExtendedTcpTable` |
| `EnvironmentService.cs` (`HKCU\Environment`) | `dotnet-windows::registry` | `windows-sys::Win32::System::Registry` + `SendMessageTimeoutW` |
| `WindowsServiceManager.cs` (`ServiceController`) | `dotnet-windows::service_ctrl` | `windows-service` or Win32 SCM FFI |
| `TaskSchedulerManager.cs` (`schtasks.exe`) | `dotnet-windows::task_scheduler` | `tokio::process::Command` |
| `AppPaths.cs` (`%LOCALAPPDATA%`) | `dotnet-core::persistence::app_paths` | `std::env::var("LOCALAPPDATA")` + `std::sync::OnceLock` |
| `JsonStore.cs` (atomic write) | `dotnet-core::persistence::json_store` | `tempfile` / atomic rename + `serde_json` |
| `ToolDownloader.cs` (HTTP + SHA256) | `dotnet-core::installation::downloader` | `reqwest` (stream) + `sha2::Sha256` |
| `SafeExtractor.cs` (Zip slip/bomb defense) | `dotnet-core::installation::extractor` | `zip` crate with path traversal checks |
| `IniDocument.cs` (PHP ini parser) | `dotnet-core::config::ini_doc` | Pure zero-copy line parser preserving format |
| `HostsFileManager.cs` (Managed blocks) | `dotnet-core::services::hosts_manager` | Atomic write with marker `# >>> Dotnet (managed) >>>` |
| `LocalCertificateManager.cs` (X509 CA & SAN) | `dotnet-core::services::cert_manager` | `rcgen` pure Rust CA & self-signed cert generator |
| `PhpPoolManager.cs` (FastCGI pool) | `dotnet-core::services::php_pool` | `tokio::process::Command` supervisor |
| `ProcessTracker.cs` (Surveillance & PID) | `dotnet-core::services::process_tracker` | In-memory atomic tracker + periodic persistence |
| `Form1.cs` (WinForms UI & Tray) | `dotnet-app` (Desktop UI) | Lightweight native tray + GUI framework |

---

## 3. Concurrency & Runtime Model

1. **Async Engine**: `tokio` multi-threaded runtime (`rt-multi-thread`) handles all non-blocking I/O:
   - Tool HTTP downloads with streaming SHA-256 calculation.
   - Child process execution and pipe handling (avoiding C# buffer deadlock bug F-04).
   - Async port polling and file system change notifications.
2. **Synchronization**:
   - `parking_lot::RwLock` for high-frequency in-memory reads (e.g. active service states).
   - `tokio::sync::mpsc` channels for communication between background workers and UI/CLI.
3. **Cancellation & Graceful Termination**:
   - `tokio_util::sync::CancellationToken` for propagating shutdown signals across running services.

---

## 4. Error Handling Strategy

1. **Library Error Taxonomy**:
   - `dotnet-windows` emits `WindowsError` using `thiserror`.
   - `dotnet-core` emits `CoreError` combining subsystem errors with typed domain failures.
2. **No Panics**:
   - Strict ban on `.unwrap()` and `.expect()` in non-test production code.
   - All errors must bubble via `Result<T, E>`.
3. **Application Layer**:
   - `dotnet-cli` and `dotnet-app` use `anyhow::Result` with `.context("...")` to provide clear, actionable diagnostic messages to developers.
