# PROGRESS TRACKING — Dotnet (Standalone Dev Manager)

> Sinkronisasi otomatis dengan [`PROJECT-SPEC.md`](PROJECT-SPEC.md).
> Terakhir diperbarui: **2026-09-22** (Pasca Update 2).

---

## Ringkasan Eksekutif

| Fase | Deskripsi | Status | Progress |
|---|---|---|---|
| **Fase 0** | Fondasi Repo & Arsitektur Solusi | 🟡 Sedang Berjalan | 85% |
| **Fase 1** | Stabilkan Inti (P0 & Fondasi Keamanan) | 🟡 Sedang Berjalan | 75% |
| **Fase 2** | Catalog & Installer Tool (Node, PHP, MinGit, Nginx, Composer) | ⚪ Belum Dimulai | 0% |
| **Fase 3** | Runtime Terintegrasi, Tray & Profile | ⚪ Belum Dimulai | 0% |
| **Fase 4** | Konfigurasi & Project Manager v2 | ⚪ Belum Dimulai | 0% |
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
| **F-05** | Path hardcoded `C:\tools\...` | 🟡 **PARSIAL** | Configurable path di `PhpConfigManager` & `NginxManager`; katalog dinamis di Fase 2. |
| **F-06** | Tulis data ke folder exe (`BaseDirectory`) | ✅ **SELESAI** | Pindah ke `%LOCALAPPDATA%\Dotnet\` via `AppPaths.cs` (Update 1). |
| **F-07** | `PhpConfigManager` regex rapuh & sentinel string `"Not Set"` | ✅ **SELESAI** | `IniDocument` parser murni, preservasi komentar/section, tanpa sentinel string (Update 2). |
| **F-08** | Edit `hosts` tanpa managed block & non-atomik | ✅ **SELESAI** | Managed Block `# >>> Dotnet (managed) >>>`, isolasi baris sistem, tulis atomik (Update 2). |
| **F-09** | Crash autostart di konstruktor Form | ✅ **SELESAI** | Pindah eksekusi autostart ke event `Shown` (Update 1). |
| **F-10** | `StartupManager` vs UAC Run key logon | ⚪ **PENDING** | Keputusan model elevasi di Fase 1 akhir / Fase 3. |

### Prioritas P1 & P2 (Relevan Update 1 & 2)

| ID | Deskripsi | Status | Implementasi |
|---|---|---|---|
| **F-13** | `WindowsServiceManager` catch semua exception jadi `Stopped` | ✅ **SELESAI** | Deteksi `NotInstalled` vs `Stopped`, return `OperationResult` (Update 2). |
| **F-14** | `ServiceStatus` tidak menampilkan status NotInstalled/Conflict | ✅ **SELESAI** | Badge `⛔ NOT INSTALLED` & disable tombol di Form1 (Update 2). |
| **F-16** | `NginxManager` deadlock stdout/stderr & tanpa stop graceful | ✅ **SELESAI** | Async stream timeout, exit code check, `-s quit`, tail error log (Update 2). |
| **F-25** | Git hygiene: untrack `bin/`, `obj/`, `.user` | ✅ **SELESAI** | `.gitignore` dikonfigurasi & repo dibersihkan (Update 1). |
| **F-29** | JSON settings ditulis non-atomik & tanpa versi skema | ✅ **SELESAI** | `JsonStore.cs` atomik (`.tmp` -> replace) + `schemaVersion` (Update 2). |
| **STYLE**| Penyesuaian tema visual legacy Windows 7/XP | ✅ **SELESAI** | UI direstyle ke `SystemColors.Control` & `STYLE-SPEC.md` (Update 2). |

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
- [ ] **T1-9:** Penentuan model elevasi & Task Scheduler autostart (F-10).

### Fase 2 — Catalog & Tool Installer (Target Berikutnya)
- [ ] **T2-1:** Skema manifest tool (JSON) + validator.
- [ ] **T2-2:** Downloader (resume/retry), SHA256 checksum verifier, safe zip extractor.
- [ ] **T2-3:** `InstalledToolStore` + scanner instalasi lokal (`C:\tools\*`, nvm-windows).
- [ ] **T2-4:** `PathEditor` (User PATH, broadcast WM_SETTINGCHANGE).
- [ ] **T2-5:** Post-install steps (`copyIfMissing`, `iniSet`, `writeShim`).
- [ ] **T2-6:** Tab UI **Tools** (Catalog list, Install, Uninstall, Version switch).
- [ ] **T2-7:** Manifest awal: Node, PHP, Composer, Git (MinGit), Nginx, Bun, Go.
- [ ] **T2-8:** Skrip sinkronisasi catalog dari upstream resmi.

---

## Log Riwayat Update

- **Update 1 (`90ac4fa`):** Split solution (`Core` & `App`), P/Invoke port checking, process output logging, storage migration `%LOCALAPPDATA%`, autostart crash fix.
- **Update 2 (Current):** `IniDocument` parser murni, Hosts Managed Block marker, status service akurat (`NotInstalled`), `JsonStore` atomik + `schemaVersion`, Safe async Nginx, restyle UI classic Win7/XP (`STYLE-SPEC.md`), setup project `Dotnet.Core.Tests` (12 passing tests).
