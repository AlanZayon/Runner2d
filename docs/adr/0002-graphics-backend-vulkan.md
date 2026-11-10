# ADR 0002 — Graphics Backend: Vulkan

## Context

First platform is Windows desktop; Android is a priority later. Intel UHD with Vulkan loader 1.3.x is present on the reference machine.

## Problem

Which low-level graphics API should back the custom 2D renderer?

## Alternatives

1. **Vulkan** — shared Windows/Android path; explicit control; higher initial complexity.
2. **Direct3D 11/12** — excellent Windows story; weak Android continuity.
3. **OpenGL / OpenGL ES** — simpler; weaker validation and long-term mobile/desktop control.
4. **Multiple backends immediately** — maximum portability cost before a single sprite works.

## Choice

**Vulkan** with a Win32 surface (`VK_KHR_win32_surface`) as the sole Phase 1 backend.

Window creation uses raw Win32 in our platform layer (not SDL-as-framework).

## Consequences

- Preferred: full LunarG Vulkan SDK for validation layers and `glslc`.
- Phase 1 fallback (used on the reference host after SDK UAC cancel): FetchContent **Vulkan-Headers** + **volk**, runtime load of system `vulkan-1.dll`. Builds without the SDK; `--validation` only works if layers are installed.
- Renderer abstraction stays small and 2D-oriented; no engine-like multi-backend until needed.
- Android later reuses Vulkan concepts with a different surface/lifecycle.

## Validation

- Phase 1 clears a swapchain image to a known color and presents stably.
- BM-00 records frame times with validation layers on/off documented.
