#pragma once

#include "../render/camera2d.hpp"
#include "../renderer/renderer.hpp"

#include "vertical_shift_abi.h"

#include <cstdint>
#include <vector>

class GameplayScene {
public:
    /// Pass `seed_locked=true` with an explicit seed (`--seed`). Otherwise a random course is created.
    bool init(
        Renderer& renderer,
        const char* sprite_png_path,
        const char* solid_png_path,
        uint64_t seed,
        bool seed_locked);
    void shutdown();
    void set_viewport(float w, float h);
    void tick(const VsInputState& input, float dt);
    void extract(std::vector<SpriteInstance>& out, Camera2D& camera) const;

private:
    TextureHandle texture_{};       // id 1 — player art
    TextureHandle solid_texture_{}; // id 2 — tinted solids
    bool world_ready_ = false;
};
