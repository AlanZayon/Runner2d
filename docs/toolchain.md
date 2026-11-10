# Toolchain setup (Windows x64)

## Required

1. **VS 2022 Build Tools** with MSVC v143 + Windows 10/11 SDK  
2. **Rust** stable (`rustup default stable-x86_64-pc-windows-msvc`)  
3. **CMake** ≥ 3.24 (`winget install Kitware.CMake`)  
4. **Ninja** (`winget install Ninja-build.Ninja`)  
5. GPU driver providing `C:\Windows\System32\vulkan-1.dll`

## Open an MSVC developer shell

```bat
"C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat" -arch=x64
```

Verify:

```bat
where cl
where cmake
where ninja
where cargo
```

## Vulkan SDK

**Required for Phase 2+** (CMake fails configure without `glslc`).

Installed on the reference host:

- Path: `C:\VulkanSDK\1.4.363.0` (`VULKAN_SDK` machine env)
- `glslc` under `%VULKAN_SDK%\Bin` (wired into `native/CMakeLists.txt`)
- `VK_LAYER_KHRONOS_validation` (`--validation`)

Runtime still loads the system Vulkan loader via **volk**; headers come from FetchContent **Vulkan-Headers**.

## Configure / build / test

```bat
cmake --preset windows-msvc-debug
cmake --build --preset windows-msvc-debug
ctest --preset windows-msvc-debug --output-on-failure

cmake --preset windows-msvc-release
cmake --build --preset windows-msvc-release

cmake --preset windows-msvc-relwithdebinfo
cmake --build --preset windows-msvc-relwithdebinfo
```

### AddressSanitizer (optional)

```bat
cmake --preset windows-msvc-asan
cmake --build --preset windows-msvc-asan
```

Requires a recent MSVC that accepts `/fsanitize=address`. If configure/link fails on Build Tools-only installs, skip ASAN and use Debug reviews instead ([ADR 0014](adr/0014-phase12-spec-compliance.md)).
