# PROGRESS TRACKING — Dotnet (Standalone Dev Manager)

> Sinkronisasi otomatis dengan [`PROJECT-SPEC.md`](PROJECT-SPEC.md).
> Terakhir diperbarui: **2026-09-23** (Pasca Update 5 / Fase 4 MVP).

---

## Ringkasan Eksekutif

| Fase | Deskripsi | Status | Progress |
|---|---|---|---|
| **Fase 0** | Fondasi Repo & Arsitektur Solusi | 🟡 Sedang Berjalan | 85% |
| **Fase 1** | Stabilkan Inti (P0 & Fondasi Keamanan) | 🟢 **Selesai** | 90% |
| **Fase 2** | Catalog & Installer Tool (Node, PHP, MinGit, Nginx, Composer, Bun, Go) | 🟢 **MVP Selesai** | 90% |
| **Fase 3** | Runtime Terintegrasi, Tray & Profile | 🟢 **MVP Selesai** | 95% |
| **Fase 4** | Konfigurasi & Project Manager v2 | 🟢 **MVP Selesai** | 95% |
| **Fase 5** | Database Portable & Perluasan (Docker v2) | ⚪ Belum Dimulai | 0% |
| **Fase 6** | Rilis, Installer & Distribusi | ⚪ Belum Dimulai | 0% |

---

## Detail Pelacakan Backlog Masalah (Section 3 SPEC)

### Prioritas P0 (Critical Bug / Safety)

| ID | Masalah di Spec | Status | Implementasi |
|---|---|---|---|
| **F-01** | `netstat` substring match lambat & rawan salah PID | ✅ **SELESAI** | P/Invoke `GetExtendedTcpTable` (IPv4/IPv6) di `PortConflictDetector.cs` (Update 1). |
| **F-02** | `ProcessServiceManager.StopProcess` bunuh proses asing di port | ✅ **SELESAI** | Verifikasi kepemilikan proses sebelum stop; cegah blind force-kill (Update 1). |
| **F-03** | Port terpakai proses lain dianggap `Running` | ✅ **SELESAI** | Deteksi port conflict spesifik proses eksternal (Update 1). |
| **F-04** | Redirect stdout/stderr pipe buffer deadlock | ✅ **SELESAI** | Redirect ke file log & pembacaan stream async (Update 1 & 2). |
| **F-05** | Path hardcoded `C:\tools\...` | ✅ **SELESAI** | `InstalledToolStore` & `ToolInstaller` kelola instalasi dinamis; `AdoptExistingScanner` adopsi instalasi lokal (Update 3). |
| **F-06** | Tulis data ke folder exe (`BaseDirectory`) | ✅ **SELESAI** | Pindah ke `%LOCALAPPDATA%\Dotnet\` via `AppPaths.cs` (Update 1). |
| **F-07** | `PhpConfigManager` regex rapuh & sentinel string `"Not Set"` | ✅ **SELESAI** | `IniDocument` parser murni, preservasi komentar/section, tanpa sentinel string (Update 2). |
| **F-08** | Edit `hosts` tanpa managed block & non-atomik | ✅ **SELESAI** | Managed Block `# >>> Dotnet (managed) >>>`, isolasi baris sistem, tulis atomik (Update 2). |
| **F-09** | Crash autostart di konstruktor Form | ✅ **SELESAI** | Pindah eksekusi autostart ke event `Shown` (Update 1). |
| **F-10** | `StartupManager` vs UAC Run key logon | ✅ **SELESAI** | `TaskSchedulerManager` via `schtasks.exe /Create /RL HIGHEST /SC ONLOGON` (Update 4). |

### Prioritas P1 & P2 (Relevan)

| ID | Deskripsi | Status | Implementasi |
|---|---|---|---|
| **F-13** | `WindowsServiceManager` catch semua exception jadi `Stopped` | ✅ **SELESAI** | Deteksi `NotInstalled` vs `Stopped`, return `OperationResult` (Update 2). |
| **F-14** | `ServiceStatus` tidak menampilkan status NotInstalled/Conflict | ✅ **SELESAI** | Badge `⛔ NOT INSTALLED` & disable tombol di Form1 (Update 2). |
| **F-15** | Adopsi proses eksisting tanpa restart service | ✅ **SELESAI** | `ProcessTracker` re-adopsi PID + start time + path saat startup & crash detection (Update 4). |
| **F-16** | `NginxManager` deadlock stdout/stderr & tanpa stop graceful | ✅ **SELESAI** | Async stream timeout, exit code check, `-s quit`, tail error log (Update 2). |
| **F-25** | Git hygiene: untrack `bin/`, `obj/`, `.user` | ✅ **SELESAI** | `.gitignore` dikonfigurasi & repo dibersihkan (Update 1). |
| **F-29** | JSON settings ditulis non-atomik & tanpa versi skema | ✅ **SELESAI** | `JsonStore.cs` atomik (`.tmp` -> replace) + `schemaVersion` (Update 2). |
| **STYLE**| Penyesuaian tema visual legacy Windows 7/XP | ✅ **SELESAI** | Seluruh tab & dialog (ExitPolicy, Diagnostics) mengikuti `STYLE-SPEC.md`. |

---

## Rincian Task Per Fase

### Fase 0 — Fondasi Repo
- [x] **T0-1:** `.gitignore`, untrack `bin/`, `obj/`, `*.user` (F-25).
- [x] **T0-2:** `README.md` terstruktur.
- [x] **T0-3:** Ganti nama proyek/identitas menjadi `Dotnet`.
- [x] **T0-4:** Pecah solution: `Dotnet.Core` / `Dotnet.App` / `Dotnet.Core.Tests`.
- [ ] **T0-5:** CI GitHub Actions `build-test.yml`.
- [x] **T0-6:** `AppPaths` (%LOCALAPPDATA%), `JsonStore` atomik + `schemaVersion` (F-06, F-29).

### Fase 1 — Stabilkan Inti
- [ ] **T1-1:** Abstraksi (`IFileSystem`, `IProcessRunner`), DI host, Serilog (F-18, F-20).
- [x] **T1-2:** P/Invoke `GetExtendedTcpTable` (iphlpapi); hapus `netstat` (F-01, F-03).
- [x] **T1-3:** Safe process stop & log output (F-02, F-04).
- [x] **T1-4:** `WindowsServiceManager` status akurat & `OperationResult` (F-13, F-14).
- [x] **T1-5:** `IniDocument` parser murni + PHP config refactor (F-07).
- [x] **T1-6:** Hosts: managed block, parser aman, tulis atomik (F-08).
- [x] **T1-7:** `NginxManager` async runner, stop graceful, tail log (F-16).
- [x] **T1-8:** Lifecycle UI autostart pasca `Shown` & classic WinForms style (F-09, STYLE-SPEC).
- [x] **T1-9:** Penentuan model elevasi & Task Scheduler autostart (F-10).

### Fase 2 — Catalog & Tool Installer (MVP Selesai)
- [x] **T2-1:** Skema manifest tool (JSON) + model deklaratif `ToolDefinition`.
- [x] **T2-2:** `ToolDownloader` (HTTPS, SHA256 stream) + `SafeExtractor` (anti zip-slip/bomb, `stripRoot`).
- [x] **T2-3:** `InstalledToolStore` (`installed-tools.json`) + `AdoptExistingScanner` (`C:\tools\*`, NVM, PATH).
- [x] **T2-4:** `PathEditor` (murni, teruji) + `EnvironmentService` (HKCU `REG_EXPAND_SZ`, backup, broadcast `WM_SETTINGCHANGE`).
- [x] **T2-5:** `PostInstallRunner` (`copyIfMissing`, `iniSet`, `iniEnableExtensions`, `writeShim`) + `JunctionManager` (`current` junction).
- [x] **T2-6:** Tab UI **Tools & Packages** di Form1 (Catalog list, Install, Uninstall, Adopt, Version switch, progress).
- [x] **T2-7:** Manifest awal: Node.js, PHP, Composer, Git (MinGit), Nginx, Bun, Go ter-embed di assembly.
- [ ] **T2-8:** Skrip sinkronisasi catalog dari upstream resmi (GitHub Actions / scheduled CI).

### Fase 3 — Runtime Terintegrasi, Tray & Profile (MVP Selesai)
- [x] **T3-1:** `ServiceProfile` & `ProfileStore` (`profiles.json`) untuk switch profil (`standalone` vs `docker`).
- [x] **T3-2:** `ServiceOrchestrator` untuk dependency-aware start (database -> web server) & stop order.
- [x] **T3-3:** `ProcessTracker` (persistensi `running-processes.json`, re-adopsi proses startup F-15, crash detection).
- [x] **T3-4:** `TaskSchedulerManager` untuk autostart elevated saat logon tanpa hambatan UAC (F-10).
- [x] **T3-5:** Port Monitor snapshot (`GetAllActiveTcpListeners`) & PATH shadow analysis di tab diagnostics.
- [x] **T3-6:** Tray menu dinamis (ganti profil, status layanan realtime, exit policy dialog pencegah proses orphan per D7).

### Fase 4 — Konfigurasi & Project Manager v2 (MVP Selesai)
- [x] **T4-1:** Skema-driven config generator (`ConfigSchema.cs`, `ConfigSchemaStore.cs`, `PhpConfigManager.cs` dynamic `ext/*.dll` scanner + preset diff, `CliConfigManager.cs` untuk Git & npm).
- [x] **T4-2:** Project Manager v2 (`FrameworkDetector.cs` untuk Laravel, Next.js, Vite, PHP, Node, Static; non-elevated command runner; Windows Terminal `wt.exe` / `cmd.exe` & VS Code launcher).
- [x] **T4-3:** Nginx Site Generator (`NginxSiteGenerator.cs` vhost di `%LOCALAPPDATA%\Dotnet\nginx\sites\`, 1-click domain `.test`, auto include injection di `nginx.conf`, rollback saat `nginx -t` gagal, auto hosts file mapping, error log viewer 30-line tail).
- [x] **T4-4:** PHP FastCGI Pool Manager (`PhpPoolManager.cs` multi-worker sequential ports 9000..900N, auto Nginx `upstream php_pool` generator).
- [x] **T4-5:** Tab UI **Projects** v2 (split-view grid, toolbar Add/Auto-detect/Run/Domain/Terminal/VSCode/Browser/Delete, 1-click dialog domain) & Tab **Config** v2 (PHP limits, presets, dynamic extensions checklist, fastcgi pool size, nginx test & reload).

---

## Log Riwayat Update

- **Update 1 (`90ac4fa`):** Split solution (`Core` & `App`), P/Invoke port checking, process output logging, storage migration `%LOCALAPPDATA%`, autostart crash fix.
- **Update 2 (`055af5c`):** `IniDocument` parser murni, Hosts Managed Block marker, status service akurat (`NotInstalled`), `JsonStore` atomik + `schemaVersion`, Safe async Nginx, restyle UI classic Win7/XP (`STYLE-SPEC.md`), setup project `Dotnet.Core.Tests` (12 passing tests).
- **Update 3 (`632e1d7`):** Tool Catalog & Installer MVP. Manifest deklaratif ter-embed (PHP, Node, Composer, Git, Nginx, Bun, Go), `SafeExtractor` (anti zip-slip/bomb), `ToolDownloader` (SHA-256), `EnvironmentService` (User PATH `REG_EXPAND_SZ` + `WM_SETTINGCHANGE`), `InstalledToolStore`, `AdoptExistingScanner`, `JunctionManager`, tab WinForms "Tools & Packages" classic style, 20 unit tests lolos.
- **Update 4 (Fase 3):** Runtime terintegrasi & profile system (`standalone` vs `docker`), `ServiceOrchestrator` start/stop ordered, `ProcessTracker` re-adopsi proses eksisting (F-15) + crash detection, `TaskSchedulerManager` elevated logon autostart (F-10), Port Monitor snapshot & PATH shadow analyzer di Tab Diagnostics, Exit Policy dialog (D7), 27 unit tests lolos.
- **Update 5 (Fase 4):** Konfigurasi & Project Manager v2. Schema config generator (`ConfigSchema`), dynamic PHP extensions scanner & diff preset, async `git config`/`npm config`, `FrameworkDetector` (Laravel, Next.js, Vite, Node, Static), terminal & VS Code runner, `NginxSiteGenerator` dengan vhost di `%LOCALAPPDATA%\Dotnet\nginx\sites\` + auto include + hosts file mapping + atomic rollback, `PhpPoolManager` multi-worker FastCGI upstream, UI modern legacy Win7/XP untuk Tab Projects v2 dan Tab Config v2, 36 unit tests lolos (0 warnings, 0 errors).

