param(
    [string]$Version = "1.0.0",
    [string]$Configuration = "Release"
)

$ErrorActionPreference = "Stop"

Write-Host "=========================================" -ForegroundColor Cyan
Write-Host " Building Dotnet DevManager v$Version ($Configuration)" -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan

$rootDir = Split-Path -Parent $PSScriptRoot
$publishDir = Join-Path $rootDir "bin\publish"
$artifactsDir = Join-Path $rootDir "artifacts"
$projectPath = Join-Path $rootDir "src\Dotnet.App\Dotnet.App.csproj"

if (Test-Path $publishDir) {
    Remove-Item -Recurse -Force $publishDir
}
if (-not (Test-Path $artifactsDir)) {
    New-Item -ItemType Directory -Force -Path $artifactsDir | Out-Null
}

Write-Host "[1/3] Publishing self-contained win-x64 single file..." -ForegroundColor Yellow
dotnet publish $projectPath `
    --configuration $Configuration `
    --runtime win-x64 `
    --self-contained true `
    -p:PublishSingleFile=true `
    -p:IncludeNativeLibrariesForSelfExtract=true `
    --output $publishDir

if ($LASTEXITCODE -ne 0) {
    Write-Error "dotnet publish failed!"
    exit 1
}

$zipFileName = "Dotnet-v$Version-win-x64-portable.zip"
$zipPath = Join-Path $artifactsDir $zipFileName

if (Test-Path $zipPath) {
    Remove-Item -Force $zipPath
}

Write-Host "[2/3] Creating portable zip archive: $zipFileName..." -ForegroundColor Yellow
Compress-Archive -Path "$publishDir\*" -DestinationPath $zipPath -CompressionLevel Optimal

Write-Host "[3/3] Generating SHA256 checksum..." -ForegroundColor Yellow
$hash = (Get-FileHash -Path $zipPath -Algorithm SHA256).Hash
$checksumFile = Join-Path $artifactsDir "SHA256SUMS.txt"
"$hash  $zipFileName" | Out-File -FilePath $checksumFile -Encoding utf8

Write-Host "`nPortable package generated successfully!" -ForegroundColor Green
Write-Host "Archive:  $zipPath"
Write-Host "SHA256:   $hash"
