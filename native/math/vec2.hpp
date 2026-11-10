#pragma once

#include <cmath>

struct Vec2 {
    float x = 0.0f;
    float y = 0.0f;

    Vec2() = default;
    Vec2(float x_, float y_) : x(x_), y(y_) {}

    Vec2 operator+(Vec2 o) const { return {x + o.x, y + o.y}; }
    Vec2 operator-(Vec2 o) const { return {x - o.x, y - o.y}; }
    Vec2 operator*(float s) const { return {x * s, y * s}; }
    Vec2 operator/(float s) const { return {x / s, y / s}; }

    Vec2& operator+=(Vec2 o) {
        x += o.x;
        y += o.y;
        return *this;
    }

    float length_sq() const { return x * x + y * y; }
    float length() const { return std::sqrt(length_sq()); }

    static float dot(Vec2 a, Vec2 b) { return a.x * b.x + a.y * b.y; }

    static Vec2 lerp(Vec2 a, Vec2 b, float t) {
        return a + (b - a) * t;
    }
};

inline Vec2 operator*(float s, Vec2 v) {
    return v * s;
}
