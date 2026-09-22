# Dotnet - Standalone Local Dev Environment Manager

Aplikasi manajemen lingkungan *development* (Native Laragon Replacement) berbasis Windows Forms.

## Prerequisites (Syarat Sistem)
- **OS:** Windows 10/11 (Menggunakan *native* P/Invoke Win32 API).
- **SDK:** [.NET 10.0 SDK](https://dotnet.microsoft.com/en-us/download/dotnet/10.0)
- **IDE:** Visual Studio 2022 atau VS Code (dengan *extension* C# Dev Kit).

## Cara Clone & Build (Untuk Developer)

1. **Clone repository:**
   ```bash
   git clone <URL_REPO_ANDA>
   cd dotnet
   ```

2. **Restore dependensi & build:**
   ```bash
   dotnet build
   ```

3. **Jalankan aplikasi (Debug):**
   ```bash
   dotnet run --project src\Dotnet.App\Dotnet.App.csproj
   ```

## Struktur Project
- `src/Dotnet.Core`: Logika inti, interaksi *process*, manajemen *service*, penyimpanan data (`%LOCALAPPDATA%\Dotnet`).
- `src/Dotnet.App`: *User Interface* berbasis Windows Forms.
- `PROJECT-SPEC.md`: Spesifikasi dan arsitektur detail (*Master document*).
- `STYLE-SPEC.md`: Panduan UI *legacy style*.
