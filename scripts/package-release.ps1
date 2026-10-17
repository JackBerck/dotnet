# Dotnet Rust Release Packaging Script (PowerShell)
# Compiles, verifies, and packages self-contained release artifacts.

$ErrorActionPreference = "Stop"

$RootDir = Split-Path -Parent $PSScriptRoot
Set-Location $RootDir

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "  Dotnet Rust Native - Release Packaging" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan

# 1. Compile release binaries with optimizations
Write-Host "[1/4] Compiling workspace binaries in release mode..." -ForegroundColor Yellow
cargo build --release

# Resolve target binary directories
$ReleaseDirs = @(
    (Join-Path $RootDir "target\x86_64-pc-windows-msvc\release"),
    (Join-Path $RootDir "target\release")
)

$FoundDir = $null
foreach ($d in $ReleaseDirs) {
    if (Test-Path (Join-Path $d "dotnet-cli.exe")) {
        $FoundDir = $d
        break
    }
}

if ($null -eq $FoundDir) {
    throw "Release binaries not found in target directories."
}

$CliBin = Join-Path $FoundDir "dotnet-cli.exe"
$AppBin = Join-Path $FoundDir "dotnet-app.exe"

# 2. Measure binary sizes
$CliSizeMB = [math]::Round((Get-Item $CliBin).Length / 1MB, 2)
$AppSizeMB = [math]::Round((Get-Item $AppBin).Length / 1MB, 2)

Write-Host "[2/4] Verifying binary sizes:" -ForegroundColor Green
Write-Host "      dotnet-cli.exe: $CliSizeMB MB" -ForegroundColor Gray
Write-Host "      dotnet-app.exe: $AppSizeMB MB" -ForegroundColor Gray

# 3. Create distribution bundle directory
$DistDir = Join-Path $RootDir "dist\Dotnet-Rust-win-x64"

# Gracefully signal running instances via HTTP exit endpoint and process stop
try {
    Invoke-WebRequest -Uri "http://127.0.0.1:4040/api/app/exit" -Method POST -TimeoutSec 1 -ErrorAction SilentlyContinue | Out-Null
} catch {}
Get-Process "dotnet-app" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 500

if (Test-Path $DistDir) {
    $lockedExe = Join-Path $DistDir "dotnet-app.exe"
    if (Test-Path $lockedExe) {
        try {
            Remove-Item -Force $lockedExe -ErrorAction Stop
        } catch {
            $oldExe = Join-Path $DistDir "dotnet-app.exe.old"
            if (Test-Path $oldExe) { Remove-Item -Force $oldExe -ErrorAction SilentlyContinue }
            Rename-Item -Path $lockedExe -NewName "dotnet-app.exe.old" -Force -ErrorAction SilentlyContinue
        }
    }
}
New-Item -ItemType Directory -Path $DistDir -Force | Out-Null

Write-Host "[3/4] Assembling release distribution..." -ForegroundColor Yellow
Copy-Item $CliBin -Destination (Join-Path $DistDir "dotnet.exe")
Copy-Item $CliBin -Destination (Join-Path $DistDir "dotnet-cli.exe")
Copy-Item $AppBin -Destination (Join-Path $DistDir "dotnet-app.exe")

if (Test-Path (Join-Path $RootDir "assets")) {
    Copy-Item (Join-Path $RootDir "assets") -Destination $DistDir -Recurse -Force
    if (Test-Path (Join-Path $RootDir "assets\app.ico")) {
        Copy-Item (Join-Path $RootDir "assets\app.ico") -Destination (Join-Path $DistDir "app.ico") -Force
    }
}
if (Test-Path (Join-Path $RootDir ".env.example")) {
    Copy-Item (Join-Path $RootDir ".env.example") -Destination (Join-Path $DistDir ".env.example")
}
if (Test-Path (Join-Path $RootDir "README.md")) {
    Copy-Item (Join-Path $RootDir "README.md") -Destination $DistDir
}
if (Test-Path (Join-Path $RootDir "LICENSE")) {
    Copy-Item (Join-Path $RootDir "LICENSE") -Destination $DistDir
}

# Register Start Menu shortcut for Windows Search integration
try {
    $ProgramsDir = [Environment]::GetFolderPath("Programs")
    if (-not (Test-Path $ProgramsDir)) {
        New-Item -ItemType Directory -Path $ProgramsDir -Force | Out-Null
    }
    $ShortcutPath = Join-Path $ProgramsDir "Dotnet.lnk"
    $TargetExe = Join-Path $DistDir "dotnet-app.exe"
    $IconPath = Join-Path $DistDir "app.ico"

    $WshShell = New-Object -ComObject WScript.Shell
    $Shortcut = $WshShell.CreateShortcut($ShortcutPath)
    $Shortcut.TargetPath = $TargetExe
    $Shortcut.WorkingDirectory = $DistDir
    $Shortcut.Description = "Dotnet"
    if (Test-Path $IconPath) {
        $Shortcut.IconLocation = "$IconPath,0"
    } else {
        $Shortcut.IconLocation = "$TargetExe,0"
    }
    $Shortcut.Save()
    Write-Host "      Start Menu shortcut registered: $ShortcutPath" -ForegroundColor Cyan
} catch {
    Write-Warning "Could not register Start Menu shortcut: $_"
}

# Register Windows Startup autostart (Background on logon)
try {
    $StartupDir = [Environment]::GetFolderPath("Startup")
    if (-not (Test-Path $StartupDir)) {
        New-Item -ItemType Directory -Path $StartupDir -Force | Out-Null
    }
    $StartupShortcut = Join-Path $StartupDir "Dotnet.lnk"
    $StartupItem = $WshShell.CreateShortcut($StartupShortcut)
    $StartupItem.TargetPath = $TargetExe
    $StartupItem.Arguments = "--minimized"
    $StartupItem.WorkingDirectory = $DistDir
    $StartupItem.Description = "Dotnet"
    if (Test-Path $IconPath) {
        $StartupItem.IconLocation = "$IconPath,0"
    } else {
        $StartupItem.IconLocation = "$TargetExe,0"
    }
    $StartupItem.Save()

    Set-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name "Dotnet" -Value "`"$TargetExe`" --minimized" -Force

    Write-Host "      Windows Startup autostart registered (Background): $StartupShortcut" -ForegroundColor Cyan
} catch {
    Write-Warning "Could not register Startup autostart: $_"
}

# 4. Generate zip archive
$ZipFile = Join-Path $RootDir "dist\Dotnet-Rust-v0.1.0-win-x64.zip"
if (Test-Path $ZipFile) {
    Remove-Item -Force $ZipFile
}

Write-Host "[4/4] Creating ZIP distribution archive: $ZipFile" -ForegroundColor Yellow
$ItemsToZip = Get-ChildItem -Path $DistDir | Where-Object { $_.Name -notlike "*.WebView2" -and $_.Name -notlike "*.old" } | Select-Object -ExpandProperty FullName
Compress-Archive -Path $ItemsToZip -DestinationPath $ZipFile -CompressionLevel Optimal

$ZipSizeMB = [math]::Round((Get-Item $ZipFile).Length / 1MB, 2)
Write-Host ""
Write-Host "==========================================" -ForegroundColor Green
Write-Host "  Release Package Ready: $ZipFile ($ZipSizeMB MB)" -ForegroundColor Green
Write-Host "  Total Self-Contained Footprint: $($CliSizeMB + $AppSizeMB) MB (vs C# 85MB+)" -ForegroundColor Green
Write-Host "==========================================" -ForegroundColor Green
