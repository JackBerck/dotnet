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
if (Test-Path $DistDir) {
    Remove-Item -Recurse -Force $DistDir
}
New-Item -ItemType Directory -Path $DistDir -Force | Out-Null

Write-Host "[3/4] Assembling release distribution..." -ForegroundColor Yellow
Copy-Item $CliBin -Destination (Join-Path $DistDir "dotnet.exe")
Copy-Item $CliBin -Destination (Join-Path $DistDir "dotnet-cli.exe")
Copy-Item $AppBin -Destination (Join-Path $DistDir "dotnet-app.exe")

if (Test-Path (Join-Path $RootDir ".env.example")) {
    Copy-Item (Join-Path $RootDir ".env.example") -Destination (Join-Path $DistDir ".env.example")
}
if (Test-Path (Join-Path $RootDir "README.md")) {
    Copy-Item (Join-Path $RootDir "README.md") -Destination $DistDir
}
if (Test-Path (Join-Path $RootDir "LICENSE")) {
    Copy-Item (Join-Path $RootDir "LICENSE") -Destination $DistDir
}

# 4. Generate zip archive
$ZipFile = Join-Path $RootDir "dist\Dotnet-Rust-v0.1.0-win-x64.zip"
if (Test-Path $ZipFile) {
    Remove-Item -Force $ZipFile
}

Write-Host "[4/4] Creating ZIP distribution archive: $ZipFile" -ForegroundColor Yellow
Compress-Archive -Path "$DistDir\*" -DestinationPath $ZipFile -CompressionLevel Optimal

$ZipSizeMB = [math]::Round((Get-Item $ZipFile).Length / 1MB, 2)
Write-Host ""
Write-Host "==========================================" -ForegroundColor Green
Write-Host "  Release Package Ready: $ZipFile ($ZipSizeMB MB)" -ForegroundColor Green
Write-Host "  Total Self-Contained Footprint: $($CliSizeMB + $AppSizeMB) MB (vs C# 85MB+)" -ForegroundColor Green
Write-Host "==========================================" -ForegroundColor Green
