# MIGRATION PROGRESS: Dotnet (C# to Rust)

> Real-time progress synchronization with `MIGRATION-SPEC.md`.

---

## Executive Phase Overview

| Phase | Description | Status | Progress |
|---|---|---|---|
| **Phase 0** | Workspace Scaffolding, Toolchain, Git & Docs | 🟢 **Done** | 100% |
| **Phase 1** | Native Windows Core (IP helper, Registry, SCM, Task Scheduler, Hosts, Nginx, SSL) | 🟢 **Done** | 100% |
| **Phase 2** | Catalog, Safe Downloader, Zip-Slip Extractor & PATH Manager | ⚪ **Planned** | 0% |
| **Phase 3** | Service Orchestrator, Process Tracker & Dynamic Tray | ⚪ **Planned** | 0% |
| **Phase 4** | Config Engine (PHP INI, CLI presets) & Project Manager v2 | ⚪ **Planned** | 20% |
| **Phase 5** | Database Portable Initializer & Docker Compose Inspector | ⚪ **Planned** | 0% |
| **Phase 6** | Desktop GUI, System Tray & Standalone Release Packaging | ⚪ **Planned** | 0% |

---

## Detailed Task Breakdown

### Phase 0 — Workspace Scaffolding & Knowledge Base (Done)
- [x] **P0-1:** Setup Cargo workspace (`dotnet-core`, `dotnet-windows`, `dotnet-cli`, `dotnet-app`).
- [x] **P0-2:** Configure `.cargo/config.toml` (MSVC static CRT, LTO, optimization).
- [x] **P0-3:** Setup `.env.example` and `.env` with developer defaults.
- [x] **P0-4:** Setup comprehensive `.gitignore`.
- [x] **P0-5:** Embed official upstream tool manifests (`php.json`, `node.json`, `nginx.json`, etc.).
- [x] **P0-6:** Create base knowledge documentation (`MIGRATION-SPEC.md`, `RUST-BEST-PRACTICES.md`, `DOMAIN-KNOWLEDGE.md`).
- [x] **P0-7:** Initialize Git repository and make initial commit.

### Phase 1 — Native Windows Core & Core System Services (Done)
- [x] **P1-1:** Direct Win32 `GetExtendedTcpTable` wrapper without `netstat` (IPv4).
- [x] **P1-2:** Add IPv6 support to `GetExtendedTcpTable` in `dotnet-windows::ip_helper`.
- [x] **P1-3:** `HKCU\Environment\Path` atomic read & write (`REG_EXPAND_SZ`).
- [x] **P1-4:** `WM_SETTINGCHANGE` broadcast with timeout guard.
- [x] **P1-5:** Windows Service Controller query & start/stop integration (`dotnet-windows::service_ctrl`).
- [x] **P1-6:** Task Scheduler elevated autostart wrapper (`dotnet-windows::task_scheduler`).
- [x] **P1-7:** Hosts file manager with managed block `# >>> Dotnet (managed) >>>` and backup (`dotnet-core::services::hosts_manager`).
- [x] **P1-8:** Nginx async runner with syntax test (`-t`), graceful reload (`-s reload`), quit (`-s quit`) and error log viewer (`dotnet-core::services::nginx_manager`).
- [x] **P1-9:** Pure Rust Local Root CA & SAN SSL certificate generator via `rcgen` (`dotnet-core::services::cert_manager`).
- [x] **P1-10:** Process supervisor with non-blocking pipe log redirection and safe termination (`dotnet-core::services::process_service`).

### Phase 2 — Catalog & Safe Installation
- [x] **P2-1:** Tool manifest deserialization models.
- [ ] **P2-2:** Async streaming download with on-the-fly SHA-256 verification.
- [ ] **P2-3:** Zip-Slip and Zip-Bomb protected extraction engine.
- [x] **P2-4:** Windows User PATH editor (pure, tested, duplicate-free).
- [ ] **P2-5:** Post-installation operations runner (`iniSet`, `iniEnableExtensions`, `copyIfMissing`).

### Phase 3 — Service Orchestration & Process Surveillance
- [ ] **P3-1:** Process surveillance & PID tracking (`running-processes.json`).
- [ ] **P3-2:** Startup process re-adoption with validation of image path and start time.
- [ ] **P3-3:** Dependency-ordered startup (Databases first -> Web servers) & reverse shutdown.
- [ ] **P3-4:** Profile switcher (`standalone` vs `docker`).

### Phase 4 — Configuration & Projects
- [x] **P4-1:** `IniDocument` format-preserving parser and editor.
- [ ] **P4-2:** Nginx virtual host generator with automatic SSL routing.
- [ ] **P4-3:** Atomic `nginx -t` validation with rollback on error.
- [ ] **P4-4:** Hosts file managed block (`# >>> Dotnet (managed) >>>`) editor.
- [ ] **P4-5:** Local Root CA and SAN SSL certificate generator (`rcgen`).

### Phase 5 — Database & Docker
- [ ] **P5-1:** Portable MySQL non-elevated initialization (`--initialize-insecure`).
- [ ] **P5-2:** Portable PostgreSQL initialization (`initdb -U root -E UTF8`).
- [ ] **P5-3:** Docker Compose v2 port collision inspector.

### Phase 6 — Desktop GUI, Tray & Packaging
- [ ] **P6-1:** System tray integration (real-time service badges, quick actions).
- [ ] **P6-2:** Desktop application window (Tabs: Dashboard, Tools, Projects, Config, Diagnostics).
- [ ] **P6-3:** Release packaging script for single self-contained executable.
