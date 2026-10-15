# MIGRATION PROGRESS: Dotnet (C# to Rust)

> Real-time progress synchronization with `MIGRATION-SPEC.md`.

---

## Executive Phase Overview

| Phase | Description | Status | Progress |
|---|---|---|---|
| **Phase 0** | Workspace Scaffolding, Toolchain, Git & Docs | 🟢 **Done** | 100% |
| **Phase 1** | Native Windows Core (IP helper, Registry, SCM, Task Scheduler, Hosts, Nginx, SSL) | 🟢 **Done** | 100% |
| **Phase 2** | Catalog, Safe Downloader, Zip-Slip Extractor & PATH Manager | 🟢 **Done** | 100% |
| **Phase 3** | Service Orchestrator, Process Tracker, Profile Switcher & Path Shadow Analyzer | 🟢 **Done** | 100% |
| **Phase 4** | Web Hosting, Project Management & Virtual Hosts | 🟢 **Done** | 100% |
| **Phase 5** | Database Portable Initializer, Docker Compose & CLI Config | 🟢 **Done** | 100% |
| **Phase 6** | Desktop GUI, System Tray & Standalone Release Packaging | 🟢 **Done** | 100% |

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

### Phase 2 — Catalog & Safe Installation (Done)
- [x] **P2-1:** Tool manifest deserialization models and embedded catalog loader (`CatalogLoader`).
- [x] **P2-2:** Async streaming download with on-the-fly SHA-256 verification and allowed hosts whitelisting (`ToolDownloader`).
- [x] **P2-3:** Zip-Slip and Zip-Bomb protected extraction engine with `stripRoot` support (`SafeExtractor`).
- [x] **P2-4:** Windows NTFS Directory Junction management (`JunctionManager`).
- [x] **P2-5:** Post-installation operations runner (`iniSet`, `iniEnableExtensions`, `copyIfMissing`, `writeShim`) (`PostInstallRunner`).
- [x] **P2-6:** Persistent installed tools store with atomic JSON storage (`InstalledToolStore`).
- [x] **P2-7:** Complete tool installer and uninstaller lifecycle orchestrator (`ToolInstaller`).

### Phase 3 — Service Orchestration & Process Surveillance (Done)
- [x] **P3-1:** Process surveillance & PID tracking with startup re-adoption in `running-processes.json` (`ProcessTracker`).
- [x] **P3-2:** Service profile switcher (`standalone` vs `docker`) and persistent store (`ProfileStore`).
- [x] **P3-3:** Dependency-ordered startup (Databases first -> Web servers) & reverse shutdown (`ServiceOrchestrator`).
- [x] **P3-4:** PATH shadow analyzer detecting System PATH vs User PATH binary collisions (`PathShadowAnalyzer`).
- [x] **P3-5:** Backward-compatible envelope and UTF-8 BOM deserializer (`JsonStore`).

### Phase 4 — Web Hosting, Project Management & Virtual Hosts (Done)
- [x] **P4-1:** `FrameworkDetector`: Auto-detects 7 framework profiles (Laravel, PHP, Next.js, Vite, Node, Static, Custom) and computes RFC 1123 safe hostnames.
- [x] **P4-2:** `ProjectInfo` & `ProjectManager`: Full JSON schema parity with C#, legacy migration support, CRUD persistence, and Windows developer launchers (cmd dev server, explorer, WT/powershell, vscode, notepad .env).
- [x] **P4-3:** `NginxSiteGenerator`: VHost generator supporting HTTP, HTTPS (301 redirect + TLS 1.2/1.3 + SAN SSL), FastCGI PHP routing, WebSocket Upgrade reverse proxying, restricted domains defense, and atomic rollback on syntax error.
- [x] **P4-4:** `PhpPoolManager`: Multi-worker sequential FastCGI pool controller (ports 9000..900N), `upstream php_pool` generator, and process supervisor.
- [x] **P4-5:** CLI integration: `dotnet projects` (list, add, remove, run, explore, terminal, code, domain) and `dotnet phppool` (status, start, stop).

### Phase 5 — Database Portable Initializer, Docker Compose & CLI Config (Done)
- [x] **P5-1:** Portable MySQL non-elevated initialization (`--initialize-insecure --datadir=... --console`) with state detection.
- [x] **P5-2:** Portable PostgreSQL initialization (`initdb -D ... -U postgres -A trust --encoding=UTF8`) with state detection.
- [x] **P5-3:** Docker Compose v2 port collision inspector with native YAML fallback parser, port range expansion, and sequential conflict override suggestion.
- [x] **P5-4:** `PhpConfigManager`: Structured `php.ini` management, extension inspection/toggling, and `development`/`production` preset applier with timestamped backups.
- [x] **P5-5:** `CliConfigManager`: Global settings manager for Git, npm, and Composer.
- [x] **P5-6:** CLI commands: `dotnet db init/status`, `dotnet compose check`, `dotnet php-ext list/enable/disable`, `dotnet php-preset`.

### Phase 6 — Desktop GUI, Tray & Packaging (Done)
- [x] **P6-1:** System tray integration (native Win32 `Shell_NotifyIconW`, popup context menu, balloon alert notifications, `SingleInstanceGuard` named mutex).
- [x] **P6-2:** Desktop application window (Embedded HTTP micro-server, Single Page Dashboard UI with 7 tabs: Services, Projects, Tools Catalog, Hosts Manager, PHP & Config, Diagnostics & Ports, Event Logs).
- [x] **P6-3:** Release packaging scripts (`scripts/package-release.ps1` & `scripts/package-release.sh`) for standalone self-contained release distribution.
