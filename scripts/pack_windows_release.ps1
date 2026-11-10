# Pack a runnable Windows Release distribution zip (Phase 11).
# Prerequisites: cmake --preset windows-msvc-release && cmake --build --preset windows-msvc-release
param(
    [switch]$SkipBuild,
    [switch]$SkipSmoke
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

$BuildDir = Join-Path $Root "build\windows-msvc-release"
$Exe = Join-Path $BuildDir "native\vertical_shift.exe"
$Assets = Join-Path $BuildDir "assets"
$Shaders = Join-Path $BuildDir "shaders"
$OutDir = Join-Path $Root "dist\windows-x64-release"
$ZipPath = Join-Path $Root "dist\vertical_shift-windows-x64-release.zip"

if (-not $SkipBuild) {
    $vs = "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat"
    if (-not (Test-Path $vs)) {
        throw "VsDevCmd.bat not found; build Release manually then re-run with -SkipBuild"
    }
    cmd /c "`"$vs`" -arch=x64 && cmake --preset windows-msvc-release && cmake --build --preset windows-msvc-release"
}

if (-not (Test-Path $Exe)) { throw "Missing $Exe — build Release first" }
if (-not (Test-Path $Assets)) { throw "Missing $Assets" }
if (-not (Test-Path $Shaders)) { throw "Missing $Shaders" }

if (Test-Path $OutDir) { Remove-Item -Recurse -Force $OutDir }
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
Copy-Item $Exe (Join-Path $OutDir "vertical_shift.exe")
Copy-Item -Recurse $Assets (Join-Path $OutDir "assets")
Copy-Item -Recurse $Shaders (Join-Path $OutDir "shaders")

$RuntimeReadme = @"
VERTICAL SHIFT — Windows x64 Release package
===========================================

Requirements: Windows 10/11 x64, Vulkan-capable GPU + driver.

Run:
  vertical_shift.exe
  vertical_shift.exe --seed 42
  vertical_shift.exe --cpu-bench 1000 --seed 42
  vertical_shift.exe --bench --bench-seconds 5 --seed 42

Controls: Enter play, Esc pause, Space jump, Shift dash, S roll.
Touch/mouse swipe: up jump, down roll, right dash.

This package is the Phase 11 distribution artifact for the reference platform.
See docs/compatibility/matrix.md in the source tree for support claims.
"@
Set-Content -Path (Join-Path $OutDir "README.txt") -Value $RuntimeReadme -Encoding UTF8

New-Item -ItemType Directory -Force -Path (Join-Path $Root "dist") | Out-Null
if (Test-Path $ZipPath) { Remove-Item -Force $ZipPath }
Compress-Archive -Path (Join-Path $OutDir "*") -DestinationPath $ZipPath -Force
Write-Host "Packed: $ZipPath"

if (-not $SkipSmoke) {
    $smokeDir = Join-Path $Root "dist\_smoke_unpack"
    if (Test-Path $smokeDir) { Remove-Item -Recurse -Force $smokeDir }
    Expand-Archive -Path $ZipPath -DestinationPath $smokeDir -Force
    $smokeExe = Join-Path $smokeDir "vertical_shift.exe"
    Push-Location $smokeDir
    try {
        & $smokeExe --cpu-bench 100 --seed 42
        if ($LASTEXITCODE -ne 0) { throw "Smoke failed with exit $LASTEXITCODE" }
    } finally {
        Pop-Location
    }
    Write-Host "Smoke OK (unpacked zip --cpu-bench 100 --seed 42)"
}
