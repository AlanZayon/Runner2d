# ADR 0004 — Phase 2 Sprite Pipeline

## Context

Phase 1 only cleared the swapchain. Phase 2 requires a stable textured sprite with camera and batching on Vulkan.

## Problem

How to add shaders, PNG textures, orthographic camera, and CPU-side sprite batching without adopting an engine or expanding the Rust ABI prematurely?

## Alternatives

1. Immediate mode per-sprite draw calls — simplest; poor mobile batching story.
2. Full AssetManager + atlas packer + async IO now — premature before gameplay exists.
3. Minimal pipeline: GLSL→SPIR-V via `glslc`, `stb_image` PNG decode, `Camera2D`, dynamic VB batch, C++ `DemoScene` until Phase 3.

## Choice

Alternative 3.

| Piece | Choice |
|-------|--------|
| Shaders | `shaders/sprite.vert` / `.frag` compiled by CMake/`glslc` |
| Textures | Vendored `stb_image` + `TextureHandle` upload |
| Camera | C++ `Camera2D` with `world_to_screen` / `screen_to_world` |
| Batching | Same-texture quads → one `vkCmdDrawIndexed` |
| Content | `DemoScene` extracts sprites (placeholder for Phase 3 gameplay) |

## Consequences

- Vulkan SDK / `glslc` required to configure the native build.
- Gameplay still does not emit GPU commands; Phase 3 will feed the same `SpriteInstance` batch from Rust via ABI buffers.
- Atlas packing, animation, and quality profiles remain future work.

## Validation

- Textured sprites visible; `--validation` clean for ≥3 s.
- 100 and 1000 sprites, same texture → `draw_calls=1`.
- `camera_math_test` CTest passes.
- BM-01 recorded in `docs/performance/baselines.md`.
