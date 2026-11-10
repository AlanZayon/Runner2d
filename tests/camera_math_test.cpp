#include "../native/render/camera2d.hpp"

#include <cmath>
#include <cstdio>

namespace {

bool near(float a, float b, float eps = 0.001f) {
    return std::fabs(a - b) <= eps;
}

bool near_v(Vec2 a, Vec2 b, float eps = 0.001f) {
    return near(a.x, b.x, eps) && near(a.y, b.y, eps);
}

} // namespace

int main() {
    Camera2D cam;
    cam.set_viewport(400.0f, 800.0f);
    cam.position = {0.0f, 0.0f};
    cam.zoom = 1.0f;

    // World origin maps to screen center.
    if (!near_v(cam.world_to_screen({0, 0}), {200, 400})) {
        std::printf("origin screen mapping failed\n");
        return 1;
    }

    // Round-trip.
    const Vec2 world{37.5f, -12.0f};
    const Vec2 screen = cam.world_to_screen(world);
    if (!near_v(cam.screen_to_world(screen), world)) {
        std::printf("round-trip failed\n");
        return 1;
    }

    cam.position = {10.0f, 20.0f};
    cam.zoom = 2.0f;
    cam.shake_offset = {1.0f, -1.0f};
    const Vec2 w2 = cam.screen_to_world(cam.world_to_screen({5, 5}));
    if (!near_v(w2, {5, 5})) {
        std::printf("zoom/shake round-trip failed\n");
        return 1;
    }

    // Ortho VP should map camera center to clip ~ (0,0).
    const Mat3 vp = cam.view_projection();
    const Vec2 clip = vp.transform_point(cam.position + cam.shake_offset);
    if (!near(clip.x, 0.0f) || !near(clip.y, 0.0f)) {
        std::printf("VP center not zero: %f %f\n", clip.x, clip.y);
        return 1;
    }

    std::printf("camera_math_test OK\n");
    return 0;
}
