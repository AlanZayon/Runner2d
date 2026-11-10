#pragma once

#include "vec2.hpp"

struct Rect {
    float x = 0.0f;
    float y = 0.0f;
    float w = 0.0f;
    float h = 0.0f;

    Vec2 min() const { return {x, y}; }
    Vec2 max() const { return {x + w, y + h}; }
    Vec2 center() const { return {x + w * 0.5f, y + h * 0.5f}; }

    bool contains(Vec2 p) const {
        return p.x >= x && p.y >= y && p.x <= x + w && p.y <= y + h;
    }
};
