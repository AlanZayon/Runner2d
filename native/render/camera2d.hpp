#pragma once

#include "../math/mat3.hpp"
#include "../math/vec2.hpp"

struct Camera2D {
    Vec2 position{0.0f, 0.0f}; // world-space center of view
    float zoom = 1.0f;
    float viewport_w = 1.0f;
    float viewport_h = 1.0f;
    Vec2 shake_offset{0.0f, 0.0f}; // visual-only; does not affect gameplay camera

    void set_viewport(float width, float height) {
        viewport_w = width > 0.0f ? width : 1.0f;
        viewport_h = height > 0.0f ? height : 1.0f;
    }

    /** View * ortho matrix (world → clip). Y grows downward in world space. */
    Mat3 view_projection() const;

    Vec2 world_to_screen(Vec2 world) const;
    Vec2 screen_to_world(Vec2 screen) const;
};
