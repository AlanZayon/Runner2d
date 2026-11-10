# Phase 9 Android bring-up helper.
# Prerequisites: Android SDK + NDK, rustup target aarch64-linux-android, glslc (Vulkan SDK).
param(
    [switch]$RustOnly,
    [switch]$AssetsOnly,
    [switch]$SkipGradle
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

function Ensure-Dir($p) {
    if (-not (Test-Path $p)) { New-Item -ItemType Directory -Force -Path $p | Out-Null }
}

# --- SPIR-V for APK assets ---
$ShaderOut = Join-Path $Root "build\android-assets\shaders"
Ensure-Dir $ShaderOut
$Glslc = if ($env:VULKAN_SDK) { Join-Path $env:VULKAN_SDK "Bin\glslc.exe" } else { "C:\VulkanSDK\1.4.363.0\Bin\glslc.exe" }
if (-not (Test-Path $Glslc)) { throw "glslc not found at $Glslc" }
& $Glslc -fshader-stage=vert (Join-Path $Root "shaders\sprite.vert") -o (Join-Path $ShaderOut "sprite.vert.spv")
& $Glslc -fshader-stage=frag (Join-Path $Root "shaders\sprite.frag") -o (Join-Path $ShaderOut "sprite.frag.spv")
Write-Host "Shaders -> $ShaderOut"

if ($AssetsOnly) { return }

# --- Rust aarch64 staticlib ---
$RustOut = Join-Path $Root "build\android-rust"
Ensure-Dir $RustOut
$env:CARGO_TARGET_DIR = $RustOut

$Ndk = $env:ANDROID_NDK_HOME
if (-not $Ndk) { $Ndk = $env:ANDROID_NDK_ROOT }
if (-not $Ndk) {
    $sdk = $env:ANDROID_HOME
    if (-not $sdk) { $sdk = $env:ANDROID_SDK_ROOT }
    if ($sdk) {
        $ndkRoot = Join-Path $sdk "ndk"
        if (Test-Path $ndkRoot) {
            $Ndk = Get-ChildItem $ndkRoot | Sort-Object Name -Descending | Select-Object -First 1 -ExpandProperty FullName
        }
    }
}
if (-not $Ndk -or -not (Test-Path $Ndk)) {
    Write-Warning "ANDROID_NDK_HOME not set — Rust link may fail. Install NDK and set ANDROID_NDK_HOME."
} else {
    Write-Host "Using NDK: $Ndk"
    $toolchain = Join-Path $Ndk "toolchains\llvm\prebuilt\windows-x86_64\bin"
    $env:PATH = "$toolchain;$env:PATH"
    $env:CC_aarch64_linux_android = Join-Path $toolchain "aarch64-linux-android26-clang.cmd"
    $env:AR_aarch64_linux_android = Join-Path $toolchain "llvm-ar.exe"
    $env:CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER = Join-Path $toolchain "aarch64-linux-android26-clang.cmd"
}

rustup target add aarch64-linux-android 2>$null
cargo build --manifest-path (Join-Path $Root "crates\runtime-bridge\Cargo.toml") --release --target aarch64-linux-android
Write-Host "Rust lib: $RustOut\aarch64-linux-android\release\libruntime_bridge.a"

if ($RustOnly) { return }
if ($SkipGradle) { return }

# --- Gradle assemble ---
$AndroidDir = Join-Path $Root "android"
Push-Location $AndroidDir
if (-not (Test-Path ".\gradlew.bat")) {
    Write-Warning "gradlew.bat missing — generate wrapper with: gradle wrapper"
    Write-Host "Then: .\gradlew.bat :app:assembleDebug"
} else {
    .\gradlew.bat :app:assembleDebug
}
Pop-Location
