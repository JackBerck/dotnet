# Dotnet — Standalone Local Development Manager

[![Build and Test](https://github.com/JackBerck/dotnet/actions/workflows/build-test.yml/badge.svg)](https://github.com/JackBerck/dotnet/actions/workflows/build-test.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows%2010%2F11%20x64-lightgrey.svg)](https://microsoft.com/windows)

A fast, lightweight, and native Windows developer environment manager (open-source Laragon alternative). Built with .NET 10 and classic Windows Forms aesthetics.

---

## Key Features

- **Standalone Tool Catalog & Safe Installer:**
  - One-click installer for **PHP (NTS), Node.js, MinGit, Nginx, Composer, Bun, Go, MySQL, and PostgreSQL**.
  - Non-elevated user-space installations (`%LOCALAPPDATA%\Dotnet\tools`).
  - SHA-256 verification and zip-slip/zip-bomb protection.
  - User PATH management with atomic registry writes (`REG_EXPAND_SZ`) and `WM_SETTINGCHANGE` broadcast.
  - Adopt existing local tools (`C:\tools\*`, NVM-Windows, PATH).

- **Dual-Mode Database & Services:**
  - Dual-mode service execution: run databases as **Portable Non-Elevated Processes** (`mysqld --initialize-insecure`, `initdb`) or control existing **Windows Services** (`MySQL84`, `postgresql-x64-17`, `Memurai`).
  - Native Windows IP helper API (`GetExtendedTcpTable`) for lightning-fast port inspection without spawning `netstat`.
  - Process persistence & crash detection via `ProcessTracker`.

- **Service Profiles & Orchestration:**
  - Switch between `standalone` (local databases + web servers) and `docker` profiles.
  - Dependency-ordered startup (Databases first -> Web servers) and reverse shutdown.

- **Project Manager v2 & 1-Click Local Domains:**
  - Automatic framework detection for **Laravel, Next.js, Vite, Node.js, PHP, and Static HTML**.
  - One-click `.test` domain setup: automatically maps to Windows `hosts` file via managed blocks and creates Nginx virtual host configs in `%LOCALAPPDATA%\Dotnet\nginx\sites\`.
  - Atomic `nginx -t` validation with automatic rollback upon configuration syntax errors.
  - Quick action toolbar: Terminal (`wt.exe` / `cmd.exe`), VS Code, Browser launcher.

- **Local HTTPS & Certificate Authority:**
  - Built-in Local Root Certificate Authority generated using pure .NET `X509Certificate2` cryptography.
  - Generates per-domain SAN SSL certificates (`domain.test`, `*.domain.test`) with automatic Nginx SSL routing (port 443 + HTTP redirect).

- **Docker Compose Port Checker v2:**
  - Asynchronously inspects `docker-compose.yml` via `docker compose config --format json`.
  - Resolves port ranges (`8000-8010`), protocols, and detects port conflicts before starting containers.
  - Automatically excludes ports already held by containers in the same compose project.

- **Classic Aesthetic & Safety:**
  - Nostalgic Windows 7/XP classic visual theme per `STYLE-SPEC.md`.
  - Exit policy dialog to prevent orphaned background processes.
  - Task Scheduler elevated autostart bypassing Windows UAC logon restrictions.

---

## System Requirements

- **Operating System:** Windows 10 or Windows 11 (64-bit).
- **Runtime:** None required when using the self-contained Release distribution.
- **For Development:** [.NET 10.0 SDK](https://dotnet.microsoft.com/en-us/download/dotnet/10.0).

---

## Installation & Distribution

### 1. Portable Version
1. Download `Dotnet-v1.0.0-win-x64-portable.zip` from the [Releases](https://github.com/JackBerck/dotnet/releases) page.
2. Extract to any directory.
3. Run `Dotnet.App.exe`.

### 2. Windows Installer
1. Download `DotnetSetup-1.0.0.exe`.
2. Run the installer (installs into user-space `%LOCALAPPDATA%\Programs\Dotnet` without requiring administrator privileges).

---

## Building from Source

```powershell
# 1. Clone repository
git clone https://github.com/JackBerck/dotnet.git
cd dotnet

# 2. Restore and build
dotnet restore Dotnet.slnx
dotnet build Dotnet.slnx --configuration Release

# 3. Run unit tests
dotnet test Dotnet.slnx --configuration Release

# 4. Launch Application
dotnet run --project src/Dotnet.App/Dotnet.App.csproj
```

### Packaging Script
To produce a self-contained single-file win-x64 release package with SHA256 checksums:
```powershell
powershell -ExecutionPolicy Bypass -File packaging/build-portable.ps1 -Version "1.0.0"
```

---

## Architecture

```
Dotnet.slnx
├── src/
│   ├── Dotnet.Core/       # Pure logic, catalog engine, installers, process runners, zero UI
│   │   ├── Catalog/       # Declarative JSON manifests & tool definitions
│   │   ├── Config/        # IniDocument parser & Hosts file managed block manager
│   │   ├── Installation/  # Safe extractor, downloader, environment editor, adopter
│   │   ├── Models/        # Service info, profiles, schemas, projects
│   │   ├── Persistence/   # AppPaths (%LOCALAPPDATA%), atomic JsonStore
│   │   └── Services/      # Port monitor, Nginx site generator, CA manager, DB initializer
│   └── Dotnet.App/        # Windows Forms UI matching classic Win7/XP aesthetics
├── tests/
│   └── Dotnet.Core.Tests/ # xUnit test suite (48 tests covering parser, hosts, certificates, etc.)
├── packaging/             # Inno Setup installer & portable build script
└── docs/                  # Architectural specs, checklist, dependencies, and reference notes
```

---

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) before submitting pull requests.

---

## License

This project is open-source software licensed under the [MIT License](LICENSE).
