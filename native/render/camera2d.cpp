#include "camera2d.hpp"

Mat3 Camera2D::view_projection() const {
    const float half_w = viewport_w * 0.5f / zoom;
    const float half_h = viewport_h * 0.5f / zoom;
    const Vec2 center = position + shake_offset;
    const Mat3 view = Mat3::translation(Vec2{-center.x, -center.y});
    const Mat3 proj = Mat3::ortho(-half_w, half_w, -half_h, half_h);
    return proj * view;
}

Vec2 Camera2D::world_to_screen(Vec2 world) const {
    const Vec2 center = position + shake_offset;
    const float sx = (world.x - center.x) * zoom + viewport_w * 0.5f;
    const float sy = (world.y - center.y) * zoom + viewport_h * 0.5f;
    return {sx, sy};
}

Vec2 Camera2D::screen_to_world(Vec2 screen) const {
    const Vec2 center = position + shake_offset;
    const float wx = (screen.x - viewport_w * 0.5f) / zoom + center.x;
    const float wy = (screen.y - viewport_h * 0.5f) / zoom + center.y;
    return {wx, wy};
}
