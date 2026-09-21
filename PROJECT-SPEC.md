# PROJECT-SPEC — Standalone Dev Manager (nama kerja: **DevManager**)

> Dokumen spesifikasi + handoff untuk melanjutkan proyek `JackBerck/dotnet`: manajer environment developer Windows (ala Laragon) yang gratis dan open source.
> Disusun dari review repo (Form1.cs, csproj, prompt awal) dan seluruh isi folder `Services/` dan `Models/`.
> Tanggal: 2026-09-21 · Versi dokumen: 1.0 · Bahasa dokumen: Indonesia; **kode, komentar, commit, README, dan string UI: English.**

---

## 0. Cara memakai dokumen ini

### 0.1 Untuk AI agent (Antigravity) dan kontributor

1. Baca dokumen ini sampai selesai sebelum mengubah kode. Kerjakan **per fase** (Bagian 16), satu task per commit/PR kecil.
2. Sebelum mulai, baca juga file yang **belum tercakup** saat dokumen ini disusun: `Program.cs`, `Form1.cs` (bagian setelah baris ~1000), `Form1.Designer.cs`, `app.manifest`, `standalone-dev-environment-reference.md`, `standalone-migration-knowledge.md`. Jika ada temuan yang bertentangan dengan dokumen ini, **catat di `docs/DECISIONS.md`**, jangan diam-diam memilih salah satu.
3. Jika dokumen ini bertentangan dengan `standalone-dev-manager-app-prompt.md`: prompt lama itu ditulis untuk mesin pemilik (path `C:\tools\...`), jadi untuk hal yang spesifik mesin, **dokumen ini yang berlaku**. Untuk aturan safety (Bagian 13), keduanya berlaku dan yang lebih ketat menang.
4. Setelah tiap task: `dotnet build` bersih tanpa warning baru, `dotnet test` hijau, lalu ringkas perubahan di deskripsi commit.

### 0.2 Aturan kerja agent (salin ke file rules/instruksi workspace Antigravity)

- Jangan menambah fitur di luar fase yang sedang dikerjakan. Jangan "overbuild".
- Jangan menjalankan perintah destruktif (hapus folder di luar sandbox uji, ubah PATH/hosts milik mesin ini, hentikan service asli) saat pengembangan. Uji operasi berisiko hanya lewat **fake/temp directory** dan unit test. Uji manual pada mesin asli hanya bila pemilik meminta.
- Jangan pernah mengarang URL download, versi, atau checksum. Nilai itu harus diambil dari sumber upstream resmi lewat skrip (Bagian 5.4) dan dicatat asalnya.
- Semua operasi yang menulis ke file sistem/konfigurasi pengguna harus: backup → tulis atomik → validasi → log. (Bagian 13)
- Kode baru harus lewat abstraksi yang bisa di-fake (`IFileSystem`, `IProcessRunner`, `IClock`, dst.) supaya bisa diuji tanpa menyentuh mesin.
- Jangan menambah dependency NuGet tanpa mencatat alasan + lisensinya di `docs/DEPENDENCIES.md`.
- Jika ragu antara dua pilihan desain, tulis opsi + rekomendasi di `docs/DECISIONS.md` lalu lanjut dengan rekomendasi.

### 0.3 Definition of Done (berlaku untuk setiap task)

- [ ] Build & test hijau di CI (`windows-latest`).
- [ ] Ada unit test untuk logika murni (parser, editor, resolver); integration test untuk yang menyentuh OS bila memungkinkan.
- [ ] Tidak ada `catch { }` kosong; error dilog dengan konteks dan, bila perlu, ditampilkan ke user.
- [ ] Tidak ada path absolut hardcoded (`C:\tools\...`) di kode produksi.
- [ ] Tidak ada operasi I/O/proses sinkron di UI thread.
- [ ] Dokumentasi (`docs/`) diperbarui bila perilaku user-facing berubah.

### 0.4 Yang belum diverifikasi saat dokumen ini ditulis

- Isi `Program.cs`, `app.manifest`, sisa `Form1.cs`, dan dua file `.md` lain di repo belum dibaca.
- URL/endpoint upstream di Bagian 5.3 adalah **titik awal**, belum diuji dari sini. Verifikasi sebelum dipakai.
- Konvensi file rules/skills/workflow milik Antigravity: cek dokumentasi resminya untuk lokasi yang tepat; aturan di 0.2 cukup disalin apa adanya.

---

## 1. Visi, cakupan, prinsip

### 1.1 Visi

Aplikasi desktop Windows kecil dan cepat yang menggantikan **kenyamanan operasional** Laragon:

1. **Install & kelola tool developer** langsung dari aplikasi: web server (nginx, apache), runtime/bahasa (php, node, python, go, bun), package manager (npm, yarn, composer), git, dan tool lain di kemudian hari. Download → verifikasi → ekstrak → daftarkan ke PATH/env → siap dipakai, tanpa installer manual.
2. **Konfigurasi dari dalam aplikasi**: `php.ini` (setting umum + ekstension), `git config`, konfigurasi npm, dst., dengan opsi edit manual.
3. **Kelola service/proses**: tahu mana yang sedang berjalan, di port berapa, siapa pemilik port, start/stop/restart, Start All/Stop All.
4. **Jalan di background**: system tray (panah kanan bawah), start saat login, minimize ke tray.
5. **Dev workflow**: daftar project, jalankan dev command, buka browser/terminal/editor, hosts entry, site nginx.
6. **Gratis dan open source**, dikembangkan bertahap.

### 1.2 Non-goals (sekarang)

- Bukan pengganti Docker/WSL; hanya koeksistensi (deteksi konflik port).
- Bukan package manager umum seperti Scoop/winget; boleh memakainya sebagai *sumber* (opsional), tidak menggantikannya.
- Tidak mengelola cloud, remote server, atau deploy.
- Tidak ada telemetri. Kalau kelak ada, harus opt-in dan terdokumentasi.
- Cross-platform belum jadi target; arsitektur menjaga logika bisnis terpisah dari UI supaya kemungkinan itu tidak tertutup.

### 1.3 Prinsip desain

1. **Aman by default**: tidak pernah mematikan proses/menghapus file/menulis sistem secara diam-diam. Semua aksi destruktif eksplisit dan bisa dibatalkan bila mungkin.
2. **Sedikit hak istimewa**: jalan sebagai user biasa sebisa mungkin; minta admin hanya untuk operasi yang memang butuh (Bagian 4.5).
3. **Deklaratif**: menambah tool baru = menulis manifest JSON, bukan kode baru (Bagian 5).
4. **Adopt, don't fight**: instalasi yang sudah ada (mis. `C:\tools\php85`, service `MySQL84`) bisa "diadopsi" tanpa dipindah atau diubah.
5. **Modular & testable**: Core library tanpa dependensi UI; semua akses OS lewat interface.
6. **Kecil dan tenang**: UI menjawab pertanyaan utama — *apa yang jalan? port berapa? siapa pakai port ini? bisa start dengan aman?* Hindari dashboard monitoring yang tidak perlu.

---

## 2. Kondisi kode saat ini

**Stack:** WinForms, `net10.0-windows`, `Nullable` aktif, satu paket NuGet (`System.ServiceProcess.ServiceController 9.0.2`). Ada `app.manifest` dan `app.ico`. 5 commit di branch `master`; belum ada README/LICENSE/.gitignore; `bin/`, `obj/`, dan `dotnet.csproj.user` ikut ter-commit.

**Inventaris:**

| File | Peran | Catatan singkat |
|---|---|---|
| `Form1.cs` (~1270 baris) | UI seluruhnya dibangun lewat kode: 5 tab (Dashboard, Projects, Hosts, PHP & Nginx Config, Diagnostics), tray, log console | God-class; path hardcoded |
| `Models/DevServiceInfo.cs` | Definisi service **dan** state runtime dalam satu class | Perlu dipisah |
| `Models/ServiceStatus.cs` | enum Stopped/Starting/Running/Stopping/Error/Unknown | `Error` tak pernah diset; belum ada NotInstalled |
| `Models/ProjectInfo.cs` | Project (nama, path, dev command, host, dst.) | Sebagian field belum dipakai |
| `Models/HostEntryInfo.cs` | Baris hosts | OK sebagai DTO |
| `Services/WindowsServiceManager.cs` | Start/stop/status Windows Service via `ServiceController` | Error tertelan jadi "Stopped" |
| `Services/ProcessServiceManager.cs` | Jalankan & lacak proses (nginx, php-cgi) | Beberapa bug serius (Bagian 3) |
| `Services/PortConflictDetector.cs` | Deteksi pemilik port via `netstat` | Bug parsing; lambat |
| `Services/DockerPortChecker.cs` | Baca port mapping compose via regex | Terlalu naif |
| `Services/HostsFileManager.cs` | Baca/tulis hosts + backup | Parser & penulisan rapuh |
| `Services/NginxManager.cs` | `nginx -t`, reload | Path hardcoded; exit code reload tak dicek |
| `Services/PhpConfigManager.cs` | Baca/tulis `php.ini` (regex) | Edit baris yang salah; sentinel string |
| `Services/ProjectManager.cs` | CRUD project (`projects.json`), launch command | Simpan di folder exe |
| `Services/ServiceSettingsManager.cs` | Setting auto-start per service (JSON) | Simpan di folder exe; tulis non-atomik |
| `Services/StartupManager.cs` | Run on boot via `HKCU\...\Run` | Bermasalah bila app minta elevasi |
| `Services/StartMenuShortcutManager.cs` | Buat shortcut Start Menu via COM `WScript.Shell` | Rapuh; lebih baik dari installer |
| `Services/AppLogger.cs` | Log ke file + event ke UI | Tidak thread-safe, tanpa rotasi |

**Yang sudah baik dan dipertahankan:** konsep manajer per domain (service/port/hosts/nginx/php/project), backup sebelum menulis, `nginx -t` sebelum reload, hosts butuh elevasi eksplisit, tray + start minimized, per-service auto-start, pemisahan Windows Service vs proses terkelola.

**Gap terbesar terhadap visi:** belum ada installer/catalog, belum ada manajemen PATH/env, semua path hardcoded untuk mesin pemilik, dan belum ada konsep *versi* tool.

---

## 3. Temuan review (backlog perbaikan)

Prioritas: **P0** = kerjakan sebelum fitur baru (bug/risiko keselamatan), **P1** = kerjakan di Fase 1–3, **P2** = rapikan bila sempat.

### 3.1 P0

| ID | Lokasi | Masalah | Perbaikan |
|---|---|---|---|
| F-01 | `PortConflictDetector.GetPidFromNetstat` | `findstr :{port}` mencocokkan substring: `:80` ikut cocok dengan `:8080`, `:8000`, juga alamat remote. Lalu mengambil PID dari baris `LISTENING` **pertama** → PID/owner bisa salah. Juga spawn `cmd.exe` tiap panggilan dan dipanggil dari UI thread. | Ganti dengan P/Invoke `GetExtendedTcpTable` (`TCP_TABLE_OWNER_PID_LISTENER`, IPv4 + IPv6) di `IPortMonitor` dengan cache + poller latar (Bagian 8.3). Hapus `netstat`. |
| F-02 | `ProcessServiceManager.StopProcess` (fallback) | Jika proses tidak dilacak, **membunuh proses apa pun yang memegang port** (`Kill(true)`). Melanggar aturan "never force-kill arbitrary processes" dan bisa mematikan aplikasi lain. | Hanya hentikan proses yang (a) diluncurkan app, atau (b) image path-nya berada di dalam direktori install tool tersebut. Selain itu: tampilkan pemilik port dan minta konfirmasi eksplisit. Coba stop graceful dulu (Bagian 8.2). |
| F-03 | `ProcessServiceManager.GetStatus` | Port dipakai proses mana pun ⇒ status `Running`. Nginx tampil Running padahal port 80 dipakai IIS/aplikasi lain. | Status dihitung dari **kecocokan pemilik port dengan tool ini**. Pemilik asing ⇒ `PortConflict` (dengan info pemilik), bukan Running. |
| F-04 | `ProcessServiceManager.StartProcess` | `RedirectStandardOutput/Error = true` tetapi **tidak pernah dibaca** ⇒ buffer pipe penuh, proses anak (nginx/php-cgi) bisa hang. | Arahkan output ke file log per service (dengan rotasi) atau baca asinkron (`BeginOutputReadLine`). |
| F-05 | `Form1.SetupServicesList`, `NginxManager`, `PhpConfigManager` | Path `C:\tools\...` hardcoded di ≥3 tempat. | Semua path berasal dari `IInstalledToolStore` / setting (Bagian 4.3, 6). |
| F-06 | `AppLogger`, `ProjectManager`, `ServiceSettingsManager`, `*.Backup*` | Semua data tulis ke `AppDomain.BaseDirectory`. Gagal jika app di `Program Files`, hilang saat update, tercampur dengan binary. | Pindah ke `%LOCALAPPDATA%\DevManager\` (mode portable opsional, Bagian 4.3). Sediakan migrasi dari lokasi lama. |
| F-07 | `PhpConfigManager` | (a) `UpdateSetting` mengedit kemunculan **pertama** termasuk baris yang di-comment; jika ada baris aktif dengan key sama lebih bawah, baris aktif itulah yang berlaku ⇒ perubahan tidak efektif. (b) `GetSettingValue` mengembalikan string sentinel `"Not Set"`/`"File not found"` yang bisa tertulis balik ke `php.ini` bila UI menyimpan. (c) menulis ulang dengan newline OS, bukan newline asli file. | Ganti dengan `IniDocument` (Bagian 9.2): prioritaskan baris aktif → uncomment baris pertama → tambah di akhir section; nilai nullable, bukan sentinel; pertahankan newline & komentar. |
| F-08 | `HostsFileManager` | (a) `RemoveEntry` menghapus **baris apa pun** yang `parts[1]` cocok tanpa memastikan `parts[0]` adalah IP — bisa menghapus komentar. (b) hanya hostname pertama per baris; `Update` membuang inline comment & hostname lain. (c) `ReadEntries` menganggap komentar bawaan Windows (`# 127.0.0.1 localhost`) sebagai entri "disabled". (d) `AppendAllText` menambah baris kosong tiap tambah. (e) tulis tidak atomik, tanpa retry saat file terkunci. (f) belum ada deteksi duplikat & edit/enable-disable di UI. | Pakai **managed block** bertanda (Bagian 10.3): app hanya mengedit blok miliknya; parser penuh; tulis atomik + retry; validasi hostname. |
| F-09 | `Form1` ctor + `StartSingleService` | Saat boot dengan `--autostart`, `Task.Run(AutoStartBootServices)` berjalan dari konstruktor: `Invoke(...)` sebelum window handle dibuat ⇒ `InvalidOperationException` (tertelan di task). Selain itu `MessageBox` konflik port dipanggil dari thread latar saat form tersembunyi ⇒ dialog tak terlihat yang memblokir. | Mulai auto-start setelah `Shown`/`HandleCreated`. Ganti MessageBox dengan notifikasi tray (balloon/toast) + entri log; dialog hanya untuk aksi interaktif. |
| F-10 | `StartupManager` + `app.manifest` *(manifest belum dibaca)* | Jika manifest memakai `requireAdministrator`, entri `HKCU\...\Run` **tidak akan dijalankan Windows saat logon** (app elevasi diblokir dari Run key). | Putuskan model elevasi (Bagian 4.5). Bila app harus elevasi: autostart via **Task Scheduler** "Run with highest privileges" (trigger at logon). Bila `asInvoker`: Run key sudah cukup. |

### 3.2 P1

| ID | Lokasi | Masalah | Perbaikan |
|---|---|---|---|
| F-11 | `Form1.StartAllGroupServices` | Loop ke **semua** service; `AutoStartWithGroup` (Nginx/PHP = false) diabaikan, padahal prompt mensyaratkan Nginx/PHP-FCGI opsional. | Grup (`Profile`) eksplisit berisi daftar service; Start All memakai grup aktif. |
| F-12 | `Form1.StartSingleService` | `isOwnProcess` memakai `procName.Contains("mysql"/"php"/...)`; rapuh. | Ownership dari image path + PID/StartTime (Bagian 8.3). |
| F-13 | `WindowsServiceManager.GetStatus` | `catch` ⇒ `Stopped` untuk **semua** exception (service tidak terpasang, akses ditolak, dst.). Start/Stop butuh admin tetapi kegagalannya hanya di log. | Bedakan `NotInstalled` (`InvalidOperationException`), `AccessDenied`, `Error(msg)`. Cek `CanStop`. Kembalikan `OperationResult` dengan pesan. |
| F-14 | `ServiceStatus`, `DevServiceInfo.StatusMessage` | `Error` & `StatusMessage` tak pernah dipakai. | Tambah `NotInstalled`, `PortConflict`, `RunningExternal`; isi pesan error. |
| F-15 | `ProcessServiceManager` | `static Dictionary` tidak thread-safe (event `Exited` jalan di threadpool); pelacakan hilang saat app restart. | `ConcurrentDictionary`; persist `{serviceId, pid, startTime}` dan re-adopt saat start (validasi PID + StartTime + image path). |
| F-16 | `NginxManager` | `ReadToEnd()` stdout lalu stderr berurutan (potensi deadlock); reload tidak cek exit code dan mengembalikan sukses meski nginx tidak jalan; tidak memberi `-p prefix`; belum ada stop graceful (`-s quit`) dan viewer error log (dipersyaratkan prompt). | Jalankan lewat `IProcessRunner` (baca stdout/stderr paralel, timeout, exit code). Tambah `Start/Stop/Reload/Validate/TailErrorLog`. |
| F-17 | `DockerPortChecker` | Regex hanya `- "3306:3306"`. Tidak menangani `127.0.0.1:3306:3306`, range, sintaks panjang (`published/target`), `${VAR:-3306}`, `/udp`, multi-file/override/profile; `ServiceName` tak diisi; regex bisa cocok baris non-port. | Pakai `docker compose config --format json` (Bagian 12). |
| F-18 | `AppLogger` | `File.AppendAllText` dari banyak thread ⇒ IOException tertelan (log hilang); tanpa rotasi; event static tanpa unsubscribe. | Pakai logging terstruktur dengan rolling file (mis. Serilog) + sink in-memory untuk UI (Bagian 4.6). |
| F-19 | `Models/DevServiceInfo` | Mencampur definisi statis dan state runtime. | Pisah `ServiceDescriptor` (dari manifest/config) dan `ServiceState` (Bagian 4.4). |
| F-20 | Semua `Services/*` | Semua `static` ⇒ tidak bisa di-test/di-fake; I/O sinkron di UI thread; `RefreshServicesStatus()` membuang & membangun ulang seluruh kartu tiap refresh. | Interface + DI; UI subscribe event perubahan state; update kartu in-place. |
| F-21 | `ProjectManager.LaunchDevCommand`/`OpenTerminal` | Perintah dirakit dengan string (`cmd /k cd /d "..." && cmd`) tanpa escape; proses anak mewarisi elevasi bila app elevated (file project bisa jadi milik admin); **PATH proses stale** setelah installer mengubah env. | Jalankan dev command **tanpa elevasi** (Bagian 4.5), bangun environment block dari registry (Bagian 7.4), prefer `wt.exe -d`, validasi path. |
| F-22 | `Form1` (php-cgi) | `php-cgi` di Windows single-process; request paralel antre. | Setting "PHP pool size" (N instance di port berurutan + `upstream` nginx) — Bagian 10.4. |

### 3.3 P2

| ID | Masalah | Perbaikan |
|---|---|---|
| F-23 | `StartMenuShortcutManager` memakai `dynamic` COM, objek COM tidak dilepas, rapuh terhadap trimming. | Shortcut dibuat oleh installer rilis (Bagian 15.4); hapus kelas ini atau ganti `IShellLink` lewat CsWin32. |
| F-24 | `Form1.cs` >1200 baris; `Form1.Designer.cs` nyaris tak terpakai. | Pecah per tab jadi `UserControl` + presenter/ViewModel (Bagian 4.6). |
| F-25 | Hygiene repo: `bin/`, `obj/`, `.csproj.user` ter-commit; tanpa `.gitignore`/README/LICENSE. | Bagian 15.1. |
| F-26 | `System.ServiceProcess.ServiceController 9.0.2` di `net10.0`. | Samakan ke 10.0.x. |
| F-27 | Nama `dotnet` bentrok dengan CLI `dotnet`, susah dicari. | Ganti nama proyek/namespace/assembly sebelum rilis publik (Bagian 18). |
| F-28 | `ProjectInfo` (PhpVersion, DatabaseType/Name, RequiresRedis) belum dipakai. | Dipakai di Fase 4. |
| F-29 | JSON settings ditulis non-atomik dan tanpa `schemaVersion`. | `AtomicFile.WriteAllText` + versi skema + migrasi. |

---
## 4. Arsitektur target

### 4.1 Layout solution

```
DevManager.sln
├─ src/
│  ├─ DevManager.Core/            (net10.0-windows, class library, TANPA referensi UI)
│  │   ├─ Abstractions/           IFileSystem, IProcessRunner, IClock, IHttpDownloader, IRegistry
│  │   ├─ Catalog/                ToolDefinition, CatalogLoader, VersionResolver
│  │   ├─ Installation/           ToolInstaller, Extractors, PostInstall steps, InstalledToolStore
│  │   ├─ Environment/            PathEditor, EnvironmentService, EnvBlockBuilder
│  │   ├─ Services/               IManagedService, WindowsServiceHost, ProcessHost, ServiceOrchestrator
│  │   ├─ Ports/                  PortMonitor (iphlpapi), PortOwnerResolver
│  │   ├─ Config/                 IniDocument, HostsFile, ConfigSchema, editors (php, git, npm)
│  │   ├─ Projects/               ProjectStore, SiteGenerator (nginx), FrameworkDetector
│  │   ├─ Docker/                 ComposePortChecker
│  │   ├─ Persistence/            AppPaths, JsonStore (atomik, versioned), Backups
│  │   └─ Native/                 P/Invoke (iphlpapi, kernel32, user32) — CsWin32 bila memungkinkan
│  ├─ DevManager.App/             (WinForms sekarang; boleh dimigrasi, Bagian 4.6)
│  │   ├─ Views/                  UserControl per tab
│  │   ├─ Presenters/ (atau ViewModels)
│  │   ├─ Tray/  Notifications/  Theming/
│  │   └─ Program.cs              single-instance, DI host, --autostart, --elevated-op
│  └─ DevManager.Catalog/         data: manifest JSON + skema JSON (di-embed dan bisa di-update)
├─ tests/
│  ├─ DevManager.Core.Tests/      xUnit (+ FluentAssertions/Shouldly opsional)
│  └─ DevManager.Integration.Tests/  hanya jalan di CI Windows, memakai temp dir
├─ tools/                          skrip CI pembuat/pembaru manifest (Bagian 5.4)
├─ docs/                           SPEC (dokumen ini), DECISIONS.md, DEPENDENCIES.md, ARCHITECTURE.md
└─ .github/workflows/
```

**Aturan dependensi:** `App → Core`. `Core` tidak boleh mereferensikan `System.Windows.Forms`. Native/OS diakses hanya lewat interface di `Abstractions/` agar bisa di-fake.

**Stack yang direkomendasikan:** `Microsoft.Extensions.Hosting` (DI, config, logging), `Serilog` (file sink + sink in-memory untuk UI), `System.Text.Json` (source generator), `CommunityToolkit.Mvvm` bila pindah ke WPF. Tambahan opsional: `Microsoft.Windows.CsWin32` untuk P/Invoke, `SharpCompress` atau bundel `7zr.exe` untuk arsip `.7z`.

### 4.2 Alur data ringkas

```
Catalog (manifest JSON) ──► ToolInstaller ──► InstalledToolStore (state.json)
                                   │                    │
                                   ▼                    ▼
                            EnvironmentService     ServiceOrchestrator ──► WindowsServiceHost / ProcessHost
                            (PATH, env vars)              │                        │
                                                          ▼                        ▼
                                                     PortMonitor ◄─────────── status & owner
```

### 4.3 Lokasi data & mode portable

```
%LOCALAPPDATA%\DevManager\
├─ settings.json            (schemaVersion, tools root, kebijakan PATH, tema, dll.)
├─ state\installed-tools.json, services.json, projects.json
├─ tools\<toolId>\<version>\        (default TOOLS_ROOT, bisa diganti user, mis. C:\tools)
│         └─ current  →  junction ke versi aktif
├─ downloads\               (cache unduhan, dibersihkan otomatis)
├─ logs\                    (app log rolling + <service>.log)
├─ backups\                 (hosts, php.ini, dll., dengan retensi N terakhir)
└─ catalog-cache\
```

- **Mode portable:** bila ada file `portable.flag` di samping exe, semua di atas ditempatkan di `<exeDir>\data\`. PATH/registry tetap tidak disentuh kecuali user memilih.
- Semua path diakses lewat satu kelas `AppPaths` (dapat di-override di test). **Dilarang** memakai `AppDomain.BaseDirectory` untuk data tulis.
- Migrasi dari lokasi lama (`projects.json`, `service-settings.json`, `backups/`, `dev-manager.log` di folder exe) dilakukan sekali saat start pertama versi baru; file lama dibiarkan.

### 4.4 Model inti (sketsa; nama boleh disesuaikan)

```csharp
public enum ServiceStatus { NotInstalled, Stopped, Starting, Running, Stopping, PortConflict, RunningExternal, Error, Unknown }

public sealed record ServiceDescriptor(
    string Id, string DisplayName, ServiceKind Kind,          // WindowsService | Process
    string? WindowsServiceName, string? ExecutablePath, IReadOnlyList<string> Arguments,
    string? WorkingDirectory, IReadOnlyList<int> Ports, HealthCheck Health, StopStrategy Stop,
    string? OwnerToolId /* untuk verifikasi ownership */);

public sealed record ServiceState(
    string ServiceId, ServiceStatus Status, int? Pid, DateTimeOffset? StartedAt,
    PortOwner? PortOwner, string? Message);

public sealed record PortOwner(int Port, int Pid, string ProcessName, string? ImagePath, string? WindowsServiceName);

public interface IManagedService
{
    ServiceDescriptor Descriptor { get; }
    ServiceState State { get; }
    event EventHandler<ServiceState> StateChanged;
    Task<OperationResult> StartAsync(CancellationToken ct);
    Task<OperationResult> StopAsync(StopMode mode, CancellationToken ct);   // Graceful | Force (Force butuh konfirmasi di UI)
}

public interface IPortMonitor
{
    IReadOnlyList<PortBinding> Snapshot();
    PortOwner? FindOwner(int port);
    event EventHandler PortsChanged;                                        // dipicu poller, sudah di-debounce
}

public interface IToolInstaller
{
    Task<InstallResult> InstallAsync(InstallRequest request, IProgress<InstallProgress> progress, CancellationToken ct);
    Task<UninstallResult> UninstallAsync(string toolId, string version, CancellationToken ct);
    Task<AdoptResult> AdoptExistingAsync(string toolId, string directory, CancellationToken ct);
}

public interface IEnvironmentService
{
    Task<PathChangeResult> ApplyAsync(IEnumerable<PathEntry> entries, EnvScope scope, CancellationToken ct);
    Task RemoveOwnedEntriesAsync(string toolId, CancellationToken ct);       // hanya entri yang dulu ditambahkan app
    IReadOnlyDictionary<string,string> BuildProcessEnvironment();            // env block segar untuk proses anak
}

public sealed record OperationResult(bool Success, string? Message = null, Exception? Error = null);
```

### 4.5 Model elevasi (keputusan penting)

Yang **benar-benar butuh admin**: edit `hosts`, start/stop **Windows Service** (MySQL84, postgresql, Memurai), edit PATH **Machine**, menulis ke `Program Files`. Yang **tidak** butuh admin: install tool portable ke `%LOCALAPPDATA%`, edit PATH **User**, jalankan proses anak (nginx/php-cgi/mysqld sebagai proses), port ≥ 1 (Windows tidak melarang non-admin bind port 80 pada umumnya).

| Opsi | Cara | Kelebihan | Kekurangan |
|---|---|---|---|
| **A. Selalu admin** (`requireAdministrator`) | Seperti Laragon | Sederhana, semua fitur jalan | UAC tiap start; **Run key tidak jalan saat logon** (pakai Task Scheduler highest privileges); proses anak (composer, npm, node) ikut elevated ⇒ file project dimiliki admin; permukaan serangan lebih besar |
| **B. `asInvoker` + elevasi on-demand** (rekomendasi) | UI non-elevated. Operasi privileged dikirim ke instance elevated (`DevManager.exe --elevated-op <json>` via `runas`, hasil lewat named pipe/exit code + file hasil) | Least privilege; autostart via Run key biasa; proses dev tidak elevated | UAC prompt per operasi (mitigasi: batch operasi; setting "ingat selama sesi" lewat helper yang tetap hidup dan divalidasi ketat) |

**Rekomendasi:** implementasikan **B** dengan antarmuka `IPrivilegedOperations` (hosts, service control, machine PATH). Sediakan **A sebagai opsi** ("Run as administrator" di Settings) yang membuat Scheduled Task (`schtasks`/`Microsoft.Win32.TaskScheduler`) untuk autostart terelevasi. Pilih database sebagai **proses terkelola** (bukan Windows Service) untuk instalasi baru agar tidak butuh admin. Instalasi lama yang berupa Windows Service tetap didukung (butuh admin untuk kontrol).

Bila app berjalan elevated, proses anak untuk project harus **di-de-elevasi** (mis. luncurkan via `explorer.exe`/token shell, atau `CreateProcess` dengan token dari proses shell). Catat sebagai known limitation bila belum diselesaikan.

### 4.6 UI, threading, logging

- **Threading:** semua I/O, proses, dan pemanggilan OS di `Task`/background; UI hanya menerima event dan meng-update view. Tidak ada `Invoke` di konstruktor. Satu `SynchronizationContext` dipakai `Presenter` untuk marshal.
- **UI framework — keputusan pemilik (Bagian 18):**
  - *Default:* tetap **WinForms** selama Fase 0–2, tetapi pecah `Form1` jadi `UserControl` per tab + presenter. Jangan investasi polish visual dulu.
  - *Rekomendasi jangka menengah:* **WPF + MVVM (CommunityToolkit.Mvvm) + library Fluent (mis. WPF-UI)** untuk tema gelap/terang modern, data binding, dan DPI yang baik. Migrasi murah bila Core sudah bersih. Alternatif **Avalonia** bila cross-platform jadi tujuan.
- **Logging:** `ILogger<T>` di seluruh Core; Serilog rolling file (mis. 10 MB × 5) + sink in-memory yang mengisi log console UI. **Redact** nilai yang tampak seperti secret (password, token, `--password=...`). Tidak pernah log isi `.env`.
- **Single instance:** `Mutex` bernama + named pipe agar instance kedua mengaktifkan jendela pertama (dan meneruskan argumen).
- **Error UX:** error operasional → notifikasi tray + entri log + tombol "lihat detail"; dialog modal hanya untuk keputusan (mis. konflik port: Stop & Continue / Change Port / Cancel).

---

## 5. Tool Catalog (manifest deklaratif)

### 5.1 Prinsip

- Satu file JSON per tool di `DevManager.Catalog/tools/<id>.json`, divalidasi terhadap `catalog.schema.json` (dites di CI).
- Manifest mendeskripsikan **apa** yang di-install dan dikonfigurasi; **bagaimana** dieksekusi oleh `ToolInstaller` lewat sejumlah `strategy`/`step` bertipe. Menambah tool umum tidak boleh butuh kode baru.
- Setiap versi wajib punya `sha256` (dihitung oleh skrip CI dari file yang diunduh dari upstream, bukan disalin dari halaman web tanpa verifikasi).
- Hanya host yang ada di `allowedHosts` yang boleh diunduh; HTTPS wajib.

### 5.2 Skema (ringkas) dan contoh

Nilai `version`, `url`, `sha256` di bawah adalah **placeholder** — jangan dipakai apa adanya.

```json
{
  "schemaVersion": 1,
  "id": "php",
  "displayName": "PHP",
  "category": "language",
  "homepage": "https://www.php.net/",
  "license": "PHP-3.01",
  "platforms": ["win-x64"],
  "prerequisites": ["vcredist-2015-2022-x64"],
  "allowedHosts": ["windows.php.net"],
  "channels": { "stable": ["8.4", "8.3"], "preview": [] },
  "versions": [
    {
      "version": "8.x.y",
      "variant": "nts",
      "source": {
        "type": "zip",
        "url": "https://windows.php.net/downloads/releases/php-8.x.y-nts-Win32-vs17-x64.zip",
        "sha256": "<computed-by-ci>",
        "stripRoot": false
      }
    }
  ],
  "layout": { "binDirs": ["."], "envVars": {} },
  "postInstall": [
    { "op": "copyIfMissing", "from": "php.ini-development", "to": "php.ini" },
    { "op": "iniSet", "file": "php.ini", "key": "extension_dir", "value": "\"ext\"" },
    { "op": "iniEnableExtensions", "file": "php.ini", "names": ["curl", "fileinfo", "mbstring", "openssl", "pdo_mysql", "pdo_pgsql", "pdo_sqlite", "sqlite3", "zip", "intl", "gd", "exif", "sodium"] },
    { "op": "iniSet", "file": "php.ini", "key": "date.timezone", "value": "${settings.timezone}" },
    { "op": "ensureSharedFile", "id": "cacert", "to": "extras/cacert.pem" },
    { "op": "iniSet", "file": "php.ini", "key": "curl.cainfo", "value": "\"${toolDir}\\extras\\cacert.pem\"" },
    { "op": "iniSet", "file": "php.ini", "key": "openssl.cafile", "value": "\"${toolDir}\\extras\\cacert.pem\"" }
  ],
  "verify": { "command": "php.exe", "args": ["-v"], "expectRegex": "^PHP (\\d+\\.\\d+\\.\\d+)" },
  "services": [
    {
      "id": "php-cgi",
      "kind": "process",
      "exe": "php-cgi.exe",
      "args": ["-b", "127.0.0.1:{port}"],
      "defaultPorts": [9000],
      "health": { "type": "tcp" },
      "stop": { "type": "kill-tree" },
      "poolSizeSupported": true
    }
  ],
  "configSchema": "php.ini"
}
```

Contoh tool CLI sederhana (satu berkas):

```json
{
  "schemaVersion": 1,
  "id": "composer",
  "displayName": "Composer",
  "category": "package-manager",
  "requires": [{ "tool": "php", "min": "8.1" }],
  "versions": [
    { "version": "2.x.y", "source": { "type": "single-file", "url": "https://getcomposer.org/download/2.x.y/composer.phar", "sha256": "<computed-by-ci>", "fileName": "composer.phar" } }
  ],
  "layout": { "binDirs": ["."] },
  "postInstall": [
    { "op": "writeShim", "name": "composer.bat", "content": "@php \"%~dp0composer.phar\" %*" }
  ],
  "verify": { "command": "composer.bat", "args": ["--version"], "expectRegex": "Composer version" }
}
```

**Tipe `source.type` yang harus didukung:** `zip`, `single-file`, `sevenzip` (butuh `7zr`/SharpCompress), `silent-installer` (mis. Python; jalankan dengan flag silent + verifikasi Authenticode), `manual-import` (user mengunduh sendiri lalu mengimpor arsip lokal — untuk sumber yang membatasi hotlink), `adopt` (direktori yang sudah ada).

**Tipe `postInstall.op` minimal:** `copyIfMissing`, `iniSet`, `iniEnableExtensions`, `writeShim`, `writeFileFromTemplate`, `ensureSharedFile`, `runCommand` (allowlist argumen), `setEnvVar`. Pastikan tiap op **idempoten** dan **bisa di-rollback**.

### 5.3 Peta tool → strategi (titik awal; **verifikasi tiap URL sebelum dipakai**)

| Tool | Sumber upstream (pola) | Strategi | Catatan |
|---|---|---|---|
| **PHP** | `windows.php.net/downloads/releases/` (arsip lama di `/archives/`); ada `releases.json` | zip **NTS** untuk FastCGI | Butuh VC++ runtime; ada `php.ini-development`/`-production`, tidak ada `php.ini` default; butuh `cacert.pem` (curl.se) agar Composer/HTTPS jalan; ekstensi Xdebug dari xdebug.org (DLL per versi/TS/arch) opsional |
| **Node.js** | `nodejs.org/dist/vX/node-vX-win-x64.zip`, `SHASUMS256.txt`, `dist/index.json` | zip | Sudah termasuk npm. PATH tambahan `%APPDATA%\npm`. Ketersediaan `corepack` bergantung versi Node — cek; fallback Yarn via `npm i -g yarn` |
| **npm / yarn** | ikut Node / npm registry | dependent-tool | Jangan ubah prefix global tanpa persetujuan; tampilkan `npm config` di UI |
| **Composer** | `getcomposer.org/download/<ver>/composer.phar` | single-file + shim | Verifikasi sesuai dokumentasi upstream |
| **Git** | `github.com/git-for-windows/git/releases` (**MinGit** zip untuk embedding; **PortableGit** `.7z.exe` bila butuh Git Bash) | zip / sevenzip | Konfigurasi via `git config --global` (jangan parse `.gitconfig` manual) |
| **Nginx** | `nginx.org/download/nginx-X.zip` | zip | Nginx hanya menerbitkan tanda tangan PGP, bukan SHA256 — hash di-pin oleh maintainer catalog via CI |
| **Apache httpd** | Apache Lounge / Apache Haus (build Windows pihak ketiga) | `manual-import` atau zip | Periksa kebijakan hotlink/mirror sebelum mengunduh otomatis |
| **Bun** | `github.com/oven-sh/bun/releases` → `bun-windows-x64.zip` (juga varian `baseline`) | zip | Folder root di dalam zip: gunakan `stripRoot` |
| **Go** | `go.dev/dl/` (JSON: `?mode=json&include=all` memuat sha256) | zip | PATH: `<go>\bin` + `%USERPROFILE%\go\bin` |
| **Python** | `python.org/ftp/python/X.Y.Z/` | `silent-installer` (mis. `InstallAllUsers=0 TargetDir=... PrependPath=0`), **bukan** embeddable zip | Embeddable zip tidak memiliki pip/venv penuh; verifikasi Authenticode installer |
| **MySQL/MariaDB** (Fase 5) | dev.mysql.com / mariadb.org zip | zip + init (`mysqld --initialize-insecure`) | Jalankan sebagai proses terkelola; password root: jangan simpan plaintext |
| **PostgreSQL** (Fase 5) | EDB binaries zip | zip + `initdb` + `pg_ctl` | Butuh VC++ runtime |
| **Redis/alternatif** (Fase 5) | Redis resmi tidak punya build Windows native; Memurai berlisensi komersial | keputusan pemilik (Bagian 18) | Minimal: *adopt* instalasi yang ada |

**Prerequisite checker:** deteksi VC++ 2015–2022 x64 Redistributable (registry) sebelum install PHP/MySQL/PostgreSQL; bila hilang, tampilkan tautan resmi, jangan pasang diam-diam.

### 5.4 Distribusi & pembaruan catalog

1. **Sumber kebenaran:** manifest di repo. Skrip `tools/update-catalog.*` (jalan terjadwal di GitHub Actions) mengkueri upstream (mis. `nodejs.org/dist/index.json`, GitHub Releases API, `go.dev/dl/?mode=json`, `windows.php.net .../releases.json`), mengunduh artefak, **menghitung SHA256**, memvalidasi terhadap skema, lalu membuka PR. Jangan ada hash yang diketik manual.
2. **Di aplikasi:** catalog bawaan di-embed; app dapat mengambil pembaruan catalog dari GitHub Releases repo ini (cache di `catalog-cache/`). Tahap awal: HTTPS + hash di dalam catalog. Tahap lanjut: tanda tangan catalog (mis. minisign/ed25519) dengan public key tertanam di app.
3. **Versi:** channel `stable`/`preview`; tampilkan hanya versi yang masih didukung upstream secara default.

---
## 6. Pipeline instalasi

Urutan (tiap tahap melapor progres via `IProgress<InstallProgress>`, bisa dibatalkan `CancellationToken`):

1. **Resolve** — pilih versi/varian dari catalog; periksa `platforms`, `requires`, `prerequisites`.
2. **Preflight** — cek ruang disk, izin tulis ke `TOOLS_ROOT`, konflik (tool sama versi sama sudah terpasang?), dan bayangan PATH (Bagian 7.3).
3. **Download** — `HttpClient` tunggal, HTTPS saja, host di `allowedHosts`, resume (`Range`) bila didukung, timeout & retry dengan backoff, simpan ke `downloads/<sha256>.part` lalu rename.
4. **Verify** — hitung SHA256 dan bandingkan; **gagal ⇒ hapus file dan hentikan**. Untuk `silent-installer`: verifikasi tanda tangan Authenticode (`WinVerifyTrust`) dan publisher yang diharapkan.
5. **Extract ke staging** — `TOOLS_ROOT\.staging\<guid>\`. Ekstraktor wajib: tolak path traversal (zip-slip), tolak symlink keluar direktori, batasi total ukuran ekstrak (anti zip-bomb), tangani `stripRoot`.
6. **Post-install** — jalankan langkah manifest secara berurutan (Bagian 5.2). Setiap langkah mencatat undo-nya.
7. **Commit atomik** — pindahkan staging ke `TOOLS_ROOT\<id>\<version>\` (rename dalam volume yang sama); perbarui junction `current` bila versi ini dijadikan aktif.
8. **Register** — tulis `installed-tools.json` (id, versi, path, sha256, waktu, **daftar PATH/env yang ditambahkan**), terapkan PATH/env (Bagian 7).
9. **Verify runtime** — jalankan `verify.command` dengan environment baru; simpan versi terdeteksi.
10. **Rollback** — bila tahap 5–9 gagal: batalkan langkah yang sudah dilakukan (hapus staging/direktori baru, kembalikan PATH). **Tidak pernah** menyentuh instalasi lain.

**Uninstall:** hapus direktori versi, hapus hanya entri PATH/env yang tercatat sebagai milik app, sarankan (jangan paksa) menghapus data user (mis. `data\` MySQL). Konfirmasi eksplisit untuk semua yang destruktif.

**Adopt existing:** pindai lokasi umum (`C:\tools\*`, `PATH`, `where <exe>`, uninstall keys registry, daftar Windows Service) untuk mendeteksi instalasi yang ada, tampilkan sebagai "Detected", dan biarkan user *mengadopsi* (hanya mencatat path; tanpa memindah/mengubah). Ini juga cara mengakomodasi lingkungan pemilik proyek (Lampiran B) tanpa install ulang.

**Multi-versi:** beberapa versi berdampingan; satu versi "aktif" per tool lewat **junction `current`** (dibuat dengan `mklink /J` atau P/Invoke; tidak butuh admin). PATH menunjuk ke `...\<tool>\current\...` sehingga ganti versi cukup memutar junction. Catatan: junction tidak bisa diputar bila ada proses yang mengunci direktori — deteksi dan beri pesan yang jelas.

---

## 7. PATH dan environment

### 7.1 Aturan

- **Scope default: User** (`HKCU\Environment`). Scope Machine hanya via opsi eksplisit yang butuh admin.
- Baca `Path` dengan `RegistryValueOptions.DoNotExpandEnvironmentNames` dan tulis kembali sebagai `REG_EXPAND_SZ` agar `%VAR%` yang ada tidak hilang. **Jangan pakai `setx`** (menulis `REG_SZ`, kehilangan referensi `%VAR%`, dan berisiko memotong nilai panjang).
- Idempoten: jangan tambah entri duplikat (bandingkan setelah normalisasi: trailing slash, huruf besar/kecil, ekspansi variabel).
- **Catat kepemilikan**: `installed-tools.json` menyimpan persis entri mana yang ditambahkan app. Uninstall hanya menghapus entri itu.
- **Backup** nilai PATH sebelum tiap perubahan (ke `backups/`), simpan N terakhir.
- **Tidak pernah** membersihkan/merapikan PATH otomatis, dan tidak menghapus entri yang bukan miliknya.
- Setelah menulis: broadcast `WM_SETTINGCHANGE` dengan `lParam = "Environment"` via `SendMessageTimeout(HWND_BROADCAST, ..., SMTO_ABORTIFHUNG, 5000)` agar Explorer dan terminal baru memuat env baru.

### 7.2 Urutan PATH (jebakan penting)

PATH efektif suatu proses = **PATH Machine + PATH User** (Machine di depan). Menaruh entri di User PATH **tidak** meng-override `php.exe`/`node.exe`/`git.exe` yang sudah ada di Machine PATH. Karena itu:

- Sebelum apply, jalankan **analisis bayangan**: untuk tiap exe utama, cari semua kemunculannya di PATH efektif dan tunjukkan mana yang menang.
- UI menampilkan peringatan jelas: "PHP dari `C:\Program Files\...` akan lebih dulu dipakai daripada versi terkelola" + opsi (a) biarkan, (b) ubah PATH Machine (admin, konfirmasi eksplisit), (c) pakai *shim*.
- Pengaturan `PathPolicy`: `Prepend` (default untuk User scope) atau `Append`.

### 7.3 Env var per tool

Dideklarasikan di manifest (`layout.envVars`), mis. `GOPATH`, `COMPOSER_HOME`, `PHPRC` (opsional, arahkan ke direktori `php.ini` aktif). Terapkan dengan aturan yang sama (catat kepemilikan, backup nilai lama, jangan menimpa nilai yang diset user tanpa konfirmasi).

### 7.4 Environment untuk proses anak

Proses yang sudah berjalan (termasuk app ini) **tidak** melihat perubahan PATH. Karena itu:

- `EnvironmentService.BuildProcessEnvironment()` membangun env block dari registry (Machine + User, ekspansi `%VAR%`), digabung dengan variabel proses saat ini yang tidak dikelola.
- Semua peluncuran proses anak (dev command, terminal, `verify`, service) memakai env block ini.
- Terminal yang sudah terbuka perlu dibuka ulang — beri tahu user di UI.

---

## 8. Service runtime dan port monitor

### 8.1 Jenis host

| Host | Untuk | Catatan |
|---|---|---|
| `WindowsServiceHost` | MySQL84, postgresql-x64-17, Memurai yang sudah ada | Pakai `ServiceController`; status via `QueryServiceStatusEx` bila perlu PID. Butuh admin untuk start/stop (lewat `IPrivilegedOperations`). Bedakan `NotInstalled` / `AccessDenied` / `Error` |
| `ProcessHost` | nginx, php-cgi, mysqld, postgres, redis (portable) | Diluncurkan app, tanpa jendela; output ke file log per service |

### 8.2 `ProcessHost` — persyaratan

- **Start:** validasi exe ada, port bebas (lihat konflik, Bagian 8.4), `CreateNoWindow`, working dir, env block segar. Output stdout/stderr → `logs\<service>.log` (rotasi) atau baca async; **tidak boleh** redirect tanpa membaca.
- **Job Object** (Win32): pilihan per service "stop saat app keluar" memakai `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`. Default: **tanya saat Exit dari tray**; simpan pilihan.
- **Stop bertingkat:** (1) graceful jika didukung manifest (`nginx -s quit`, `mysqladmin shutdown`, `pg_ctl stop -m fast`), (2) tunggu N detik, (3) `Kill(entireProcessTree)` **hanya** untuk proses milik service ini, dan untuk mode Force butuh konfirmasi UI.
- **Crash detection:** event `Exited` ⇒ status `Error` dengan exit code; kebijakan restart opsional (`never`/`on-failure` dengan batas + backoff).
- **Persist & re-adopt:** simpan `{serviceId, pid, processStartTime, imagePath}`; saat app start ulang, validasi (PID hidup, StartTime sama, image path cocok) lalu kelola kembali, bukan menganggapnya "orang asing".
- **Thread-safety:** `ConcurrentDictionary`; event dari threadpool di-marshal lewat presenter.
- **PHP pool:** untuk `php-cgi` (single-process di Windows) dukung N instance di port `base..base+N-1` (setting `poolSize`, default 1) + blok `upstream` di nginx (Bagian 10.4).

### 8.3 `PortMonitor` — persyaratan

- P/Invoke `GetExtendedTcpTable` dengan `TCP_TABLE_OWNER_PID_LISTENER` untuk `AF_INET` dan `AF_INET6`; parsing struktur yang benar (byte order port). Tambahkan UDP (`GetExtendedUdpTable`) hanya bila dibutuhkan.
- Poller latar (mis. 1–2 dtk saat jendela terlihat, lebih jarang saat di tray) yang menghasilkan snapshot + diff; UI dan orchestrator berlangganan `PortsChanged`. **Tidak ada spawn proses** untuk cek port.
- `PortOwnerResolver`: PID → nama proses + **image path penuh** via `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION)` + `QueryFullProcessImageName` (lebih andal daripada `Process.MainModule`), dan, bila PID adalah proses Windows Service, nama service-nya. Tangani akses ditolak dengan tampilan "Unknown (access denied)".
- **Ownership:** sebuah port "milik" service bila image path proses berada di dalam direktori install tool (atau sama dengan `ExecutablePath`), atau PID = PID terlacak. Nama proses saja **tidak cukup**.
- Selain port service, sediakan kueri umum "siapa memakai port X?" untuk tab Diagnostics dan cek Docker.

### 8.4 Alur konflik port

Sebelum start (service atau Docker):

1. Ambil pemilik port. Bila pemilik = service itu sendiri ⇒ status `Running`, tidak ada aksi.
2. Bila pemilik lain: status `PortConflict`; tampilkan dialog/kartu **Port 3306 — dipakai `MySQL84` (PID 1234)** dengan opsi *Stop `MySQL84` & lanjut* / *Ubah port* / *Batal*. Tidak ada aksi otomatis.
3. "Stop & lanjut" memakai jalur stop bertingkat (8.2) dan hanya untuk service yang dikenal app; untuk proses asing tampilkan konfirmasi terpisah yang menyebut nama proses, PID, dan path.

### 8.5 Profile / grup

Ganti flag `AutoStartWithGroup` dengan **profile** eksplisit:

```json
{ "profiles": [
  { "id": "standalone", "services": ["mysql", "postgres", "redis"], "optional": ["nginx", "php-cgi"] },
  { "id": "docker", "services": [], "preStart": ["docker-port-check"] }
]}
```

Start All/Stop All beroperasi pada profile aktif; service `optional` punya toggle sendiri (sesuai alur `composer run dev` yang tidak bergantung pada nginx).

---

## 9. Manajemen konfigurasi

### 9.1 Pendekatan berbasis skema

Satu **form generator** yang dikendalikan `configSchema` per tool — menambah tool tidak berarti menulis UI baru.

```json
{
  "id": "php.ini",
  "target": { "type": "ini", "file": "${toolDir}/php.ini" },
  "groups": [
    { "title": "Limits", "fields": [
      { "key": "memory_limit", "label": "Memory limit", "type": "size", "default": "512M", "validate": { "pattern": "^-?\\d+[KMG]?$" } },
      { "key": "upload_max_filesize", "type": "size" },
      { "key": "post_max_size", "type": "size" },
      { "key": "max_execution_time", "type": "int", "min": 0 }
    ]},
    { "title": "Locale", "fields": [ { "key": "date.timezone", "type": "timezone" } ] },
    { "title": "Extensions", "fields": [ { "type": "extensionList", "dir": "${toolDir}/ext" } ] }
  ],
  "validators": [ { "type": "command", "exe": "php.exe", "args": ["-c", "{file}", "-m"], "failOnStderrMatch": "PHP Warning|Startup" } ],
  "presets": ["development", "production"]
}
```

Tipe target: `ini` (PHP, MySQL `my.ini` bila perlu), `cli-config` (Git: `git config --global`; npm: `npm config`), `json`, `template-file` (nginx site). Selalu sediakan tombol **Open in editor** (edit manual) dan **Show diff / Revert**.

### 9.2 `IniDocument` (menggantikan regex di `PhpConfigManager`)

Parser berbasis baris yang menyimpan urutan, komentar, section, dan jenis newline asli. Perilaku `Set(key, value)`:

1. Ada baris **aktif** dengan key itu ⇒ ubah baris tersebut (bila ada duplikat aktif, ubah yang **terakhir** karena itulah yang berlaku) dan pertahankan komentar di akhir baris bila ada.
2. Tidak ada baris aktif tetapi ada baris ter-comment (`;key = ...`) ⇒ hilangkan `;` pada kemunculan pertama di section yang benar dan set nilainya.
3. Tidak ada sama sekali ⇒ tambahkan di akhir section yang sesuai (atau di akhir file dengan blok `; Added by DevManager`).

Tambahkan `Get(key): string?` (null bila tidak ada, **bukan** string sentinel), `EnableExtension(name)`, `DisableExtension(name)`, `ListExtensions()` (`extension=` dan `zend_extension=`), dan `ToDiff(other)`. Tulis: backup → tulis ke temp → validasi (Bagian 9.1 `validators`) → ganti atomik; bila validasi gagal, kembalikan backup.

### 9.3 Preset & ekstensi PHP

- Preset `development` / `production` menyalin dari `php.ini-development` / `php.ini-production` **hanya bila user meminta** (dengan diff), tidak menimpa `php.ini` yang sudah disesuaikan.
- Daftar ekstensi berasal dari isi folder `ext/` (bukan daftar hardcoded), dengan penanda aktif/nonaktif dan peringatan bila ekstensi membutuhkan DLL pendukung yang tidak ada.
- Opsi lanjutan: instal Xdebug (unduh DLL yang cocok dengan versi PHP/TS/arch, pasang `zend_extension`).

### 9.4 Git, npm, dan lainnya

- **Git:** baca/tulis lewat `git config --global` (`user.name`, `user.email`, `init.defaultBranch`, `core.autocrlf`, `credential.helper`, `pull.rebase`). **Jangan** menyimpan kredensial; gunakan credential manager bawaan Git.
- **npm/yarn/bun:** `npm config get/set` (registry, prefix, cache) dengan penjelasan efek. Jangan mengganti prefix global tanpa persetujuan.
- **Nginx:** `nginx.conf` + `conf/sites/*.conf` (Bagian 10.4); validasi `nginx -t` sebelum reload.
- Semua perubahan konfigurasi: backup → diff → tulis atomik → validasi → log; gagal ⇒ rollback otomatis.

---
## 10. Project manager, hosts, dan site nginx

### 10.1 Project

`ProjectInfo` diperluas dan benar-benar dipakai:

```
Id, Name, Path, Framework (laravel | nextjs | node | php | static | custom),
PhpVersion (referensi ke tool terinstal), NodeVersion?, Database {Type, Name}, RequiresRedis,
DevCommand, DevPort?, Host (mis. myapp.test), NginxSite (auto/manual), Notes
```

- **Framework detection** (`FrameworkDetector`, read-only): `composer.json` memuat `laravel/framework` ⇒ Laravel; `package.json` dengan `next` ⇒ Next.js; dst. Hanya untuk *saran default*, user bisa ganti.
- **Aksi:** Open folder, Open terminal (`wt.exe -d <path>` bila ada, fallback PowerShell), Open in editor (`code .` bila tersedia), Run dev command, Open browser, Open `.env` (read-only view atau editor eksternal), Open nginx site.
- **Tidak mengubah file project** tanpa aksi eksplisit user (aturan lama dipertahankan). Membuat site nginx/hosts entry adalah aksi eksplisit "Set up local domain".
- Dev command dijalankan **tanpa elevasi**, dengan env block segar (Bagian 7.4), working directory = path project. Jangan merakit string `cmd /k` dari input mentah: gunakan `ProcessStartInfo` dengan `ArgumentList`, dan tampilkan command yang akan dijalankan sebelum eksekusi pertama kali untuk project baru.

### 10.2 Penyimpanan

`projects.json` di `state\` (Bagian 4.3), ditulis atomik, dengan `schemaVersion`. Path project boleh di mana saja; validasi keberadaan saat dimuat dan tandai "missing" tanpa menghapus entri.

### 10.3 Hosts file

- **Managed block:** app hanya membaca/menulis di antara penanda:

  ```
  # >>> DevManager (managed) >>>
  127.0.0.1	myapp.test
  # 127.0.0.1	old.test        <- entri disabled ditandai khusus, bukan komentar biasa
  # <<< DevManager (managed) <<<
  ```

  Baris di luar blok **tidak pernah diubah**, hanya ditampilkan read-only (dan dideteksi bila hostname bentrok).
- **Parser penuh:** IP + banyak hostname per baris, inline comment, IPv6, CRLF/LF.
- **Validasi hostname** sebelum tulis: regex hostname RFC 1123; TLD yang aman: `.test`, `.localhost` (RFC 6761). Peringatkan untuk `.local` (bentrok mDNS) dan TLD nyata seperti `.dev`/`.app` (HSTS preload memaksa HTTPS). **Blokir/peringatkan keras** domain publik (mis. `google.com`) — mencegah salah ketik atau penyalahgunaan.
- **Tulis:** cek elevasi (`IPrivilegedOperations`), backup dengan retensi, tulis ke temp lalu ganti, retry singkat bila file terkunci (antivirus/DNS Client), lalu opsional `ipconfig /flushdns`. Fitur UI lengkap: add, edit, remove, enable/disable, deteksi duplikat.
- Alternatif tanpa admin: sarankan domain `*.localhost` (di-resolve ke loopback oleh browser modern) sebagai opsi.

### 10.4 Site nginx

- Generator dari template (`writeFileFromTemplate`) ke `conf/sites/<project>.conf`; **escape/validasi** semua nilai yang disisipkan (hostname, path). Nilai user tidak boleh disisipkan mentah ke config.
- Template minimal: **Laravel/PHP** (`root <project>\public`, `try_files`, `fastcgi_pass` ke upstream php-cgi, `fastcgi_param SCRIPT_FILENAME`), **reverse proxy** (untuk Next.js/Vite dev di `127.0.0.1:3000`, dukungan WebSocket upgrade), **static**.
- `upstream php_pool { server 127.0.0.1:9000; server 127.0.0.1:9001; ... }` bila `poolSize > 1`.
- Alur: generate → tampilkan diff → tulis → `nginx -t` → jika lulus baru reload; jika gagal, kembalikan file lama dan tampilkan error.
- Viewer **error log** (tail N baris, refresh) di tab Nginx.
- Include `conf/sites/*.conf` di `nginx.conf` hanya bila belum ada, lewat perubahan yang ditampilkan (jangan menulis ulang `nginx.conf` user).

### 10.5 HTTPS lokal (Fase 5)

Opsional: buat CA lokal + sertifikat per host (mirip mkcert); impor CA ke trust store **hanya dengan persetujuan eksplisit**; simpan kunci privat di direktori user dengan ACL terbatas.

---

## 11. Tray, startup, dan lifecycle

- **Tray (`NotifyIcon`)**: menu Open, Start All, Stop All, daftar service dengan toggle + status (ikon/teks), Open Projects, Settings, Exit. Klik ganda membuka jendela. Ikon berubah saat ada error/konflik.
- **Close-to-tray:** tombol X menyembunyikan jendela (setting, default aktif); **Exit** dari menu tray benar-benar keluar dan menjalankan kebijakan "stop managed services on exit" (Bagian 8.2).
- **Notifikasi:** balloon/toast untuk konflik port saat auto-start, service crash, install selesai, pembaruan tersedia. Hindari spam (deduplikasi, batas frekuensi).
- **Autostart:** Model B ⇒ `HKCU\...\Run` dengan `--autostart` (start minimized). Model A ⇒ Scheduled Task highest privileges (Bagian 4.5). Path exe di entri harus diperbarui saat app dipindah/diupdate.
- **Auto-start layanan saat boot:** dijalankan setelah UI/handle siap, berurutan sesuai dependensi (mis. DB sebelum web), dengan timeout per service dan hasil di log; kegagalan tidak memblokir service lain.
- **Single instance** (Bagian 4.6) dan **crash handling**: `Application.ThreadException`/`AppDomain.UnhandledException` ⇒ log + notifikasi, jangan hilang diam-diam.
- **Pembaruan aplikasi** (Fase 6): pertimbangkan Velopack atau MSIX/winget; jangan membuat mekanisme update sendiri tanpa verifikasi tanda tangan.

---

## 12. Cek konflik port Docker

Ganti regex dengan **`docker compose config --format json`** (menyelesaikan interpolasi env, multi-file `-f`, override, profile):

1. Deteksi `docker` di PATH; bila tidak ada, beri pesan ramah (fitur tidak aktif, bukan error).
2. Jalankan `docker compose -f <file> [-f ...] [--profile x] config --format json` di working directory compose; parse `services.*.ports[]` → `{ host_ip, published, target, protocol }` (termasuk range `published: "8000-8010"`).
3. Untuk tiap port publish yang relevan: cek `IPortMonitor` (TCP/UDP sesuai protocol; perhatikan `host_ip` — `127.0.0.1` vs `0.0.0.0`).
4. **Pengecualian:** jika port dipegang oleh container dari project compose yang sama (sudah `up`), itu bukan konflik — cek via `docker compose ps --format json`.
5. Hasil `DockerPortConflict { Service, HostPort, ContainerPort, Protocol, Owner }` → alur konflik (Bagian 8.4). Sediakan tombol "Ubah port" yang hanya **menyarankan** override (jangan mengedit compose file user secara diam-diam).
6. Fallback bila `docker compose config` gagal: parser YAML sungguhan (mis. YamlDotNet) dengan dukungan sintaks pendek & panjang — bukan regex.

---

## 13. Persyaratan keamanan dan safety (checklist wajib)

**Proses & file**
- [ ] Tidak pernah `Kill` proses yang bukan milik service tersebut tanpa konfirmasi eksplisit yang menyebut nama+PID+path.
- [ ] Tidak pernah menghapus file/direktori di luar `TOOLS_ROOT`/`AppPaths` yang dikelola app; hapus rekursif hanya setelah memverifikasi path berada di bawah root yang diharapkan (cek path ter-normalisasi, tolak `..` dan reparse point yang keluar root).
- [ ] Backup sebelum ubah `hosts`, `php.ini`, `nginx.conf`, PATH; retensi N terakhir.
- [ ] Tulis atomik (temp + replace) untuk semua file konfigurasi/state.
- [ ] Tidak menghapus Laragon secara otomatis, tidak membersihkan PATH otomatis, tidak mengubah firewall diam-diam.

**Unduhan**
- [ ] HTTPS wajib; host di allowlist manifest; **SHA256 wajib cocok**; installer `.exe`/`.msi` hanya dijalankan bila tanda tangan Authenticode valid dan publisher sesuai.
- [ ] Ekstraksi aman dari zip-slip/zip-bomb; jangan mengeksekusi apa pun dari staging sebelum verifikasi.
- [ ] Catalog remote: integritas terjamin (hash tertanam; lanjut ke tanda tangan). Manifest tidak boleh memuat perintah shell bebas — hanya `op` bertipe dengan argumen tervalidasi.

**Input & injeksi**
- [ ] Hostname, path, dan nama service divalidasi sebelum disisipkan ke `hosts`, config nginx, atau argumen proses.
- [ ] Tidak ada perakitan string `cmd /c` dari input user; gunakan `ArgumentList`.
- [ ] Dev command project adalah perintah milik user: tampilkan sebelum dijalankan pertama kali; jangan menjalankannya otomatis dari data yang diimpor/di-sync.

**Secret**
- [ ] Tidak menyimpan password database plaintext. Bila perlu menyimpan (mis. password root hasil `initdb`), gunakan DPAPI (`ProtectedData`, scope CurrentUser) atau tampilkan sekali dan jangan simpan.
- [ ] Log di-redact; jangan log isi `.env`, header auth, atau argumen yang mengandung `password`/`token`/`secret`.

**Hak akses**
- [ ] Operasi privileged hanya lewat `IPrivilegedOperations` dengan daftar operasi terbatas (bukan "jalankan perintah admin apa saja"); helper elevated memvalidasi input dan tidak menerima path/perintah sembarang.
- [ ] Proses anak project berjalan non-elevated.

---

## 14. Strategi pengujian

| Lapisan | Apa | Cara |
|---|---|---|
| Unit | `IniDocument`, hosts parser/writer, `PathEditor` (string murni), `VersionResolver`, template nginx, parser compose JSON, validator hostname, ekstraktor (zip-slip), skema catalog | xUnit; data uji di `tests/TestData`; property-based test untuk `PathEditor` (idempoten, tidak menghapus entri asing) |
| Contract | Semua manifest catalog valid terhadap skema; URL berasal dari `allowedHosts`; hash berformat benar | Test yang memuat seluruh `catalog/tools/*.json` |
| Integration (Windows CI) | `PortMonitor` (buka `TcpListener` di port acak, cek owner = PID test), `ProcessHost` (jalankan proses dummy: start/stop/graceful/crash), registry PATH pada **hive uji atau key sementara**, ekstraksi & commit atomik ke temp dir | `windows-latest`; jangan menyentuh `HKCU\Environment` asli — abstraksikan lewat `IRegistry` dan uji dengan implementasi in-memory; satu-dua tes E2E opsional dengan key sementara |
| Fake | `IFileSystem`, `IProcessRunner`, `IHttpDownloader`, `IClock`, `IRegistry` | Dipakai di test installer (simulasi gagal di tengah ⇒ rollback bersih) |
| Manual/smoke | Checklist rilis: install/uninstall tiap tool, ganti versi, PATH terlihat di terminal baru, tray, autostart, konflik port | Dokumentasikan di `docs/RELEASE-CHECKLIST.md` |

Target awal: ≥80% cakupan untuk logika murni di `Core`. Test tidak boleh bergantung pada jaringan (gunakan server HTTP lokal atau file fixture).

---

## 15. Repo hygiene, CI/CD, rilis, lisensi

### 15.1 Bersihkan repo (Fase 0)

1. Tambah `.gitignore` (template Visual Studio/.NET) lalu `git rm -r --cached bin obj dotnet.csproj.user` dan commit. (Riwayat lama biarkan kecuali ada data sensitif — periksa dulu.)
2. Tambah `README.md` (deskripsi, screenshot, status pengembangan, cara build, roadmap singkat), `LICENSE`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md` (cara lapor kerentanan), template issue/PR.
3. Pindahkan tiga file `.md` di root ke `docs/` (`docs/reference/…`) dan tautkan dari README.
4. Isi *About*/topics repo (windows, developer-tools, laragon-alternative, dotnet, winforms).
5. Rename proyek (Bagian 18) sebelum ada pengguna eksternal — mengubah nama nanti jauh lebih mahal.
6. `Directory.Build.props`: `Nullable`, `TreatWarningsAsErrors` (bertahap), analyzers, `LangVersion`, versi terpusat; `global.json` untuk mengunci SDK; `.editorconfig`.

### 15.2 Lisensi

- Lisensi kode: **MIT atau Apache-2.0** (Apache-2.0 memberi klausul paten eksplisit) — keputusan pemilik.
- Aplikasi **mengunduh** tool dari upstream, bukan mendistribusikannya ulang; tiap manifest memuat field `license` dan UI menampilkannya saat install. Periksa syarat masing-masing (mis. Memurai berlisensi komersial, Apache Lounge/Haus punya ketentuan sendiri). Jangan memakai nama/logo/aset Laragon. *(Bukan nasihat hukum; verifikasi ke sumber lisensi.)*
- `docs/DEPENDENCIES.md`: daftar dependensi NuGet + lisensinya; hasilkan otomatis bila bisa.

### 15.3 CI (GitHub Actions, `windows-latest`)

- `build-test.yml`: restore, build Release, test, upload hasil; cache NuGet.
- `catalog-update.yml`: terjadwal, jalankan skrip update catalog → buka PR (Bagian 5.4).
- `release.yml`: pada tag `v*` — publish **self-contained single-file win-x64** (`-p:PublishSingleFile=true -p:SelfContained=true`; **jangan** aktifkan trimming untuk WinForms/WPF), buat installer, hitung SHA256, buat GitHub Release + catatan rilis.
- Versi: SemVer dari tag (mis. MinVer). Pra-rilis `-preview.N` untuk build awal.

### 15.4 Distribusi

- **Installer:** Inno Setup (sederhana) atau MSIX/WiX; membuat shortcut Start Menu, opsi autostart, opsi mode portable (zip). Ini menggantikan `StartMenuShortcutManager`.
- **winget:** kirim manifest setelah rilis stabil pertama.
- **Code signing:** unsigned ⇒ SmartScreen memperingatkan. Untuk OSS, pertimbangkan program penandatanganan gratis (mis. SignPath Foundation) atau sertifikat sendiri; dokumentasikan hash rilis di halaman Release.
- **Runtime:** self-contained agar user tidak perlu memasang .NET 10; ukuran lebih besar adalah trade-off yang wajar.

---
## 16. Roadmap fase dan acceptance criteria

Kerjakan berurutan. Jangan mulai fase berikutnya sebelum acceptance criteria fase sebelumnya terpenuhi.

### Fase 0 — Fondasi repo

| ID | Task |
|---|---|
| T0-1 | `.gitignore`, untrack `bin/ obj/ *.user` (F-25) |
| T0-2 | README, LICENSE, CONTRIBUTING, SECURITY, template issue/PR; pindah `.md` lama ke `docs/` |
| T0-3 | Rename proyek/namespace/assembly (Bagian 18) |
| T0-4 | Pecah solution: `Core` / `App` / `Tests`; **pindahkan kode apa adanya** (tanpa ubah perilaku) — hanya agar build lolos |
| T0-5 | CI `build-test.yml` |
| T0-6 | `AppPaths`, `JsonStore` atomik + `schemaVersion`, migrasi data dari folder exe (F-06, F-29) |

**Acceptance:** CI hijau; aplikasi berperilaku sama seperti sebelumnya; data tulis berada di `%LOCALAPPDATA%`.

### Fase 1 — Stabilkan inti (semua P0)

| ID | Task | Menutup |
|---|---|---|
| T1-1 | Abstraksi (`IFileSystem`, `IProcessRunner`, `IClock`, `IRegistry`), DI host, Serilog | F-18, F-20 |
| T1-2 | `PortMonitor` (iphlpapi) + `PortOwnerResolver`; hapus `netstat` | F-01, F-03, F-12 |
| T1-3 | `ProcessHost` baru: log ke file, stop bertingkat, ownership, persist/re-adopt, thread-safe | F-02, F-04, F-15 |
| T1-4 | `WindowsServiceHost`: `NotInstalled/AccessDenied/Error`, `OperationResult` | F-13, F-14 |
| T1-5 | `IniDocument` + PHP config memakainya; hapus sentinel string | F-07 |
| T1-6 | Hosts: managed block, parser penuh, tulis atomik, validasi | F-08 |
| T1-7 | `NginxManager` via `IProcessRunner`: start/stop/reload/validate/tail log | F-16 |
| T1-8 | Lifecycle UI: auto-start setelah `Shown`, notifikasi tray, profile, update kartu in-place | F-09, F-11, F-20 |
| T1-9 | Putuskan model elevasi; perbaiki `StartupManager` sesuai keputusan | F-10 |

**Acceptance:** semua unit/integration test baru hijau; tidak ada `netstat`/`cmd /c` di kode produksi; tidak ada jalur yang mem-`Kill` proses asing tanpa konfirmasi; dashboard tidak freeze saat refresh; auto-start saat boot tidak menampilkan dialog tersembunyi.

### Fase 2 — Catalog & installer (MVP visi utama)

| ID | Task |
|---|---|
| T2-1 | Skema manifest + validator + test kontrak (Bagian 5) |
| T2-2 | Downloader (resume, retry), verifikasi SHA256, ekstraktor `zip` & `single-file` aman (zip-slip/zip-bomb) |
| T2-3 | `InstalledToolStore` + **adopt-existing scanner** (termasuk deteksi nvm-windows dan path `C:\tools\*`) |
| T2-4 | `PathEditor` (murni, teruji) + `EnvironmentService` (HKCU, `REG_EXPAND_SZ`, broadcast, backup, kepemilikan) + analisis bayangan + `BuildProcessEnvironment` |
| T2-5 | Post-install ops: `copyIfMissing`, `iniSet`, `iniEnableExtensions`, `writeShim`, `ensureSharedFile` |
| T2-6 | Tab **Tools**: daftar catalog, install/uninstall, versi aktif (junction `current`), progres & batal |
| T2-7 | Manifest awal: **node, php, composer, git (MinGit), nginx, bun, go** (python `silent-installer` menyusul) |
| T2-8 | Skrip `update-catalog` + workflow terjadwal (hash dihitung otomatis) |

**Acceptance:** pada mesin Windows bersih **tanpa hak admin**: install node+php+composer+git dari UI → buka terminal baru → `php -v`, `composer --version`, `node -v`, `git --version` bekerja; uninstall menghapus hanya entri PATH milik app; install yang gagal di tengah tidak meninggalkan sisa (rollback); hash yang salah membatalkan instalasi.

### Fase 3 — Runtime terintegrasi & tray

| ID | Task |
|---|---|
| T3-1 | `ServiceDescriptor` dari manifest (nginx, php-cgi) + `ServiceOrchestrator` + profile |
| T3-2 | Tray lengkap, close-to-tray, single instance, kebijakan Exit, notifikasi |
| T3-3 | Job Object opsional, restart policy, deteksi crash |
| T3-4 | `IPrivilegedOperations` + helper elevated (hosts, Windows Service, PATH Machine) **atau** Scheduled Task mode admin |
| T3-5 | Tab Diagnostics: tabel port + pemilik, analisis bayangan PATH, log |

**Acceptance:** start/stop nginx & php-cgi terkelola dari UI/tray; status akurat dan konflik port dijelaskan (pemilik, PID, path); app tetap benar setelah restart (re-adopt); autostart berfungsi sesuai model elevasi terpilih.

### Fase 4 — Konfigurasi & project

| ID | Task |
|---|---|
| T4-1 | Form generator berbasis `configSchema`: php.ini (limit, timezone, ekstensi), git, npm |
| T4-2 | Project manager v2: framework detection, aksi lengkap, dev command non-elevated |
| T4-3 | Generator site nginx + integrasi hosts + viewer error log |
| T4-4 | PHP pool size + upstream |

**Acceptance:** ubah setting PHP dari UI menghasilkan diff, backup, dan perubahan efektif (`php -i` mencerminkannya); membuat domain lokal untuk project Laravel dalam ≤3 klik; validasi nginx mencegah reload config rusak.

### Fase 5 — Perluasan

Database portable (MySQL/MariaDB, PostgreSQL) sebagai proses terkelola; keputusan Redis; Docker check v2 (`compose config`); HTTPS lokal; Python (`silent-installer`); Apache (`manual-import`/zip); tool tambahan lewat manifest saja.

### Fase 6 — Rilis

Installer + mode portable, code signing, winget, auto-update (Velopack/MSIX), i18n (mulai dari resource string EN/ID), dokumentasi pengguna, checklist rilis.

---

## 17. Prompt siap pakai untuk Antigravity

> Simpan dokumen ini di repo sebagai `docs/PROJECT-SPEC.md`, lalu jalankan prompt berikut satu per satu. Tinjau hasil tiap fase sebelum lanjut.

**Prompt 0 — Orientasi (jalankan pertama)**
```
Read docs/PROJECT-SPEC.md fully. Then read every file listed in section 0.1 point 2 that
you have not read yet (Program.cs, the rest of Form1.cs, Form1.Designer.cs, app.manifest,
and the two other .md files in the repo root). Do NOT change any code yet.
Produce docs/DECISIONS.md with: (1) anything in those files that contradicts the spec,
(2) whether app.manifest requests administrator (relevant to finding F-10),
(3) a proposed task order for Phase 0, including risks. Stop and wait for my review.
```

**Prompt 1 — Fase 0**
```
Implement Phase 0 of docs/PROJECT-SPEC.md (tasks T0-1..T0-6), one commit per task.
Constraints: do not change runtime behavior; only restructure. Keep the app building
(dotnet build) after every commit. For T0-3 (rename) use the name I choose: <NAME>.
Add a minimal CI workflow. Summarize what changed and anything you were unsure about.
```

**Prompt 2 — Fase 1**
```
Implement Phase 1 (T1-1..T1-9) following sections 3.1, 4, 8, 9.2 and 10.3 of the spec.
For each task: write the failing tests first for the pure logic (IniDocument, hosts parser,
port owner resolution with a real TcpListener, ProcessHost with a dummy child process), then
implement. Never spawn netstat/cmd.exe. Never kill a process we do not own without the
confirmation flow described in 8.4. Do not touch the real hosts file, real registry PATH or
real services in tests; use fakes/temp dirs. T1-9: implement the elevation model I selected
in DECISIONS.md (default: option B in section 4.5). Report residual risks at the end.
```

**Prompt 3 — Fase 2**
```
Implement Phase 2 (T2-1..T2-8). Start with the manifest schema and tests (T2-1), then the
installer pipeline with fakes for network and registry. Catalog entries must have their
sha256 produced by the tools/update-catalog script from upstream artifacts — never invent
URLs, versions or hashes. If an upstream URL cannot be verified, leave the entry out and list
it in DECISIONS.md. Acceptance criteria are in section 16, Phase 2.
```

**Prompt 4 — Review berkala (setelah tiap fase)**
```
Review the diff of this phase against docs/PROJECT-SPEC.md sections 13 (security checklist)
and 0.3 (Definition of Done). List violations with file/line, fix them, and re-run build+tests.
```

---

## 18. Keputusan terbuka (pemilik proyek)

| # | Keputusan | Rekomendasi | Alasan |
|---|---|---|---|
| D1 | **Nama proyek** | Ganti sebelum rilis; pilih nama unik yang tidak mengandung "dotnet"/"laragon" | Mudah dicari, tidak bentrok dengan CLI `dotnet`, hindari masalah merek |
| D2 | **Lisensi** | MIT (simpel) atau Apache-2.0 (klausul paten) | Ramah kontributor dan pengguna |
| D3 | **Model elevasi** | B (asInvoker + elevasi on-demand), A sebagai opsi | Least privilege; autostart sederhana; proses dev tidak elevated |
| D4 | **UI framework** | WinForms sampai akhir Fase 2, lalu evaluasi WPF + Fluent | Jangan blokir fitur inti oleh polish UI |
| D5 | **`TOOLS_ROOT` default** | `%LOCALAPPDATA%\<App>\tools` (tanpa admin); izinkan ganti ke mis. `C:\tools` | Tidak butuh admin; pengguna lama bisa tetap di `C:\tools` |
| D6 | **Kebijakan PATH** | User scope, `Prepend`, dengan analisis bayangan; Machine scope hanya opt-in | Aman; jujur soal urutan PATH efektif |
| D7 | **Proses terkelola saat app keluar** | Tanya saat Exit, ingat pilihan | Mencegah proses yatim atau service mati tak terduga |
| D8 | **Database di catalog** | Fase 5, sebagai proses terkelola; adopt Windows Service yang ada | Menghindari kebutuhan admin |
| D9 | **Redis di Windows** | Adopt instalasi yang ada (Memurai) di awal; putuskan alternatif kemudian | Redis resmi tidak punya build Windows native |
| D10 | **Hosting catalog** | Di repo + GitHub Releases, hash tertanam; tanda tangan catalog di Fase 6 | Sederhana dulu, aman bertahap |
| D11 | **Distribusi** | Inno Setup + zip portable; winget setelah stabil | Cukup untuk OSS kecil |
| D12 | **Bahasa** | Kode/README/UI English; dokumen internal boleh Indonesia; i18n di Fase 6 | Jangkauan kontributor lebih luas |

---

## Lampiran A — Pemetaan kode lama → modul baru

| Lama | Baru | Catatan |
|---|---|---|
| `DevServiceInfo` | `ServiceDescriptor` + `ServiceState` | Pisah definisi vs runtime |
| `ServiceStatus` | `ServiceStatus` (diperluas) | + NotInstalled, PortConflict, RunningExternal |
| `WindowsServiceManager` | `WindowsServiceHost : IManagedService` | + hasil operasi terstruktur |
| `ProcessServiceManager` | `ProcessHost : IManagedService` | Tulis ulang (F-02..F-04, F-15) |
| `PortConflictDetector` | `PortMonitor` + `PortOwnerResolver` | Tanpa netstat |
| `DockerPortChecker` | `ComposePortChecker` | `docker compose config` |
| `HostsFileManager` | `HostsFile` (parser/writer murni) + `IPrivilegedOperations.ApplyHosts` | Managed block |
| `NginxManager` | `NginxController` + `SiteGenerator` | Via `IProcessRunner` |
| `PhpConfigManager` | `IniDocument` + editor berbasis `configSchema` | Tanpa regex per-key |
| `ProjectManager` / `ProjectInfo` | `ProjectStore` + `ProjectLauncher` + `FrameworkDetector` | Dev command non-elevated |
| `ServiceSettingsManager` | `SettingsStore`/`ServicesState` (`JsonStore`) | Atomik + versi skema |
| `StartupManager` | `AutostartService` (Run key / Scheduled Task) | Sesuai model elevasi |
| `StartMenuShortcutManager` | Installer rilis | Hapus kelas |
| `AppLogger` | `ILogger<T>` + Serilog + sink UI | Rolling, thread-safe, redact |
| `Form1` | `MainForm` + `UserControl` per tab + presenter | Tanpa path hardcoded |

## Lampiran B — Lingkungan referensi pemilik (untuk dogfooding)

Sumber: `standalone-dev-manager-app-prompt.md` di repo. Gunakan sebagai skenario uji **adopt-existing** dan regresi:

| Komponen | Detail |
|---|---|
| PHP | 8.5.x di `C:\tools\php85`, `php.ini` di folder yang sama, FastCGI `php-cgi.exe -b 127.0.0.1:9000` |
| Nginx | `C:\tools\nginx`, `conf\nginx.conf`, `conf\sites\`, port 80 |
| MySQL | 8.4.x, Windows Service `MySQL84`, port 3306 |
| PostgreSQL | 17.x, Windows Service `postgresql-x64-17`, port 5432 |
| Redis | Memurai, port 6379, `memurai-cli` |
| Lainnya | Node via NVM, npm, dan Bun sudah terpasang (adopt harus mengenali nvm-windows dan **tidak** melawannya) |
| Workflow | Laravel dev memakai `composer run dev`; nginx opsional, tidak menggantikan workflow ini |
| Domain lokal | contoh `*.test` untuk beberapa project |

**Skenario regresi wajib lulus:** adopt seluruh komponen di atas tanpa mengubah/memindahkan file; status dashboard akurat (termasuk saat Docker memakai 3306); Start All/Stop All hanya menyentuh service dalam profile; nginx bisa di-toggle terpisah; edit `php.ini` dan hosts menghasilkan backup + diff.

---

*Akhir dokumen. Perubahan pada dokumen ini dicatat di `docs/DECISIONS.md` (keputusan) dan riwayat git (revisi).*
