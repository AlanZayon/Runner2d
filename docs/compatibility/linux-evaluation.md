# Linux x64 — evaluation checklist (not implemented)

Phase 11 documents acceptance criteria only. Do **not** mark Linux supported until every item passes on a real distro/WSL build.

## Prerequisites

- Vulkan ICD (`vulkaninfo` OK)
- `glslc`, CMake ≥ 3.24, Ninja, Clang or GCC, Rust stable `x86_64-unknown-linux-gnu`
- X11 or Wayland windowing decision recorded in a new ADR

## Acceptance

1. CMake preset `linux-clang-release` (or equivalent) configures and links.
2. `PlatformLinux` + Vulkan WSI (`VK_KHR_xcb_surface` or Wayland) presents sprites.
3. Keyboard + optional mouse-gesture sim match Win32 actions.
4. `--cpu-bench 1000 --seed 42` completes; numbers logged in baselines.
5. Asset/save paths use `$XDG_DATA_HOME` or cwd fallback.
6. Matrix row updated: Build/Run = Yes with distro + GPU named.

## Out of scope until then

- Shipping `.deb`/Flatpak, multi-distro CI, OpenGL fallback.
