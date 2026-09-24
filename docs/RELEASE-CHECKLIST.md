# Release Verification Checklist

Follow this checklist before tagging a new release (`vX.Y.Z`).

## 1. Automated Verification
- [ ] `dotnet build --configuration Release` completes with **0 Warnings** and **0 Errors**.
- [ ] `dotnet test --configuration Release` passes all unit tests (minimum 48 tests).
- [ ] GitHub Actions `build-test.yml` workflow passes on `windows-latest`.

## 2. Packaging Verification
- [ ] Run `packaging/build-portable.ps1`:
  - `Dotnet.App.exe` is built self-contained for `win-x64`.
  - `Dotnet-v1.0.0-win-x64-portable.zip` is created.
  - `SHA256SUMS.txt` is generated with valid hash.
- [ ] Compile Inno Setup installer `packaging/DotnetSetup.iss`:
  - Output installer `DotnetSetup-1.0.0.exe` generated.
  - Test install on clean directory: Start Menu shortcut and Desktop shortcut created.
  - Test uninstaller: cleanly removes app binaries without touching `%LOCALAPPDATA%\Dotnet\data`.

## 3. Functional Smoke Testing
- [ ] **Single Instance:** Double-launching `Dotnet.App.exe` restores already running window instead of creating a second process.
- [ ] **Tool Catalog:** Test installing / adopting tool (Node, PHP, Git) — verifies PATH addition and terminal invocation.
- [ ] **Services Dashboard:**
  - Start All / Stop All respects active profile (`standalone` vs `docker`).
  - Database card correctly identifies `[PORTABLE]` vs `[WIN-SERVICE]`.
- [ ] **Nginx & Local Domains:**
  - 1-click domain setup routes `.test` domain in hosts file and Nginx vhost.
  - 1-click HTTPS generates valid Root CA and leaf certificate in `%LOCALAPPDATA%\Dotnet\ssl`.
- [ ] **Diagnostics:** Port monitor and Docker Compose inspector accurately display conflict owners.
- [ ] **Exit Policy:** Closing app prompts user to stop background services or keep running in tray.

## 4. Documentation & Tagging
- [ ] Update `CHANGELOG.md` or release notes.
- [ ] Verify `Directory.Build.props` version matches target tag.
- [ ] Create signed git tag: `git tag -a v1.0.0 -m "Release v1.0.0"`.
- [ ] Push tag: `git push origin v1.0.0`.
