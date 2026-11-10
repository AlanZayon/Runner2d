#include "gameplay_scene.hpp"

#include "../assets/texture_loader.hpp"
#include "log.hpp"

bool GameplayScene::init(
    Renderer& renderer,
    const char* sprite_png_path,
    const char* solid_png_path,
    uint64_t seed,
    bool seed_locked) {
    texture_ = load_texture_png(renderer, sprite_png_path);
    if (!texture_.valid()) {
        return false;
    }
    solid_texture_ = load_texture_png(renderer, solid_png_path);
    if (!solid_texture_.valid()) {
        vs_log("solid texture failed (%s) — falling back to sprite", solid_png_path ? solid_png_path : "?");
        solid_texture_ = texture_;
    }
    const VsResult created = seed_locked ? vs_world_create_seeded(seed) : vs_world_create();
    if (created != VS_OK) {
        vs_log("vs_world_create failed");
        return false;
    }
    world_ready_ = true;

    VsRunState run{};
    uint64_t active_seed = seed;
    if (vs_world_get_run_state(&run) == VS_OK) {
        active_seed = run.seed;
    }
    vs_log(
        "GameplayScene ready (sprite=%u solid=%u seed=%llu locked=%u)",
        texture_.id,
        solid_texture_.id,
        static_cast<unsigned long long>(active_seed),
        seed_locked ? 1u : 0u);
    return true;
}

void GameplayScene::shutdown() {
    if (world_ready_) {
        vs_world_destroy();
        world_ready_ = false;
    }
}

void GameplayScene::set_viewport(float w, float h) {
    if (world_ready_) {
        vs_world_set_viewport(w, h);
    }
}

void GameplayScene::tick(const VsInputState& input, float dt) {
    if (!world_ready_) {
        return;
    }
    vs_world_tick(&input, dt);
}

void GameplayScene::extract(std::vector<SpriteInstance>& out, Camera2D& camera) const {
    out.clear();
    if (!world_ready_) {
        return;
    }
    VsSpriteInstance raw[256];
    uint32_t count = 0;
    VsCamera2D cam{};
    if (vs_world_extract(raw, 256, &count, &cam) != VS_OK) {
        return;
    }
    out.reserve(count);
    for (uint32_t i = 0; i < count; ++i) {
        SpriteInstance s{};
        s.x = raw[i].x;
        s.y = raw[i].y;
        s.w = raw[i].w;
        s.h = raw[i].h;
        s.u0 = raw[i].u0;
        s.v0 = raw[i].v0;
        s.u1 = raw[i].u1;
        s.v1 = raw[i].v1;
        s.r = raw[i].r;
        s.g = raw[i].g;
        s.b = raw[i].b;
        s.a = raw[i].a;
        // Sim texture_id 1 = character art; 2 (+anything else) = solid tint quad.
        s.texture = (raw[i].texture_id == 1) ? texture_ : solid_texture_;
        out.push_back(s);
    }
    camera.position = {cam.x, cam.y};
    camera.zoom = cam.zoom > 0.0f ? cam.zoom : 1.0f;
    camera.set_viewport(cam.viewport_w, cam.viewport_h);
    camera.shake_offset = {0.0f, 0.0f};
}
