#pragma once

#include "../render/camera2d.hpp"

#include <cstdint>

struct RendererCreateInfo {
    void* window_handle = nullptr;  // HWND or ANativeWindow*
    void* module_handle = nullptr;  // HINSTANCE (Win32) / unused on Android
    int width = 0;
    int height = 0;
    bool enable_validation = false;
    const char* shader_dir = nullptr; // directory containing sprite.vert.spv / sprite.frag.spv
    const char* asset_dir = nullptr;  // optional hint for callers
};

struct TextureHandle {
    uint32_t id = 0; // 0 = invalid
    bool valid() const { return id != 0; }
};

struct SpriteInstance {
    float x = 0.0f;
    float y = 0.0f;
    float w = 0.0f;
    float h = 0.0f;
    float u0 = 0.0f;
    float v0 = 0.0f;
    float u1 = 1.0f;
    float v1 = 1.0f;
    float r = 1.0f;
    float g = 1.0f;
    float b = 1.0f;
    float a = 1.0f;
    TextureHandle texture{};
};

class Renderer {
public:
    virtual ~Renderer() = default;
    virtual bool init(const RendererCreateInfo& info) = 0;
    virtual void shutdown() = 0;
    virtual bool resize(int width, int height) = 0;
    /** Destroy/recreate VkSurface + swapchain (Android window lifecycle). */
    virtual bool recreate_window_surface(void* window_handle, int width, int height) {
        (void)window_handle;
        (void)width;
        (void)height;
        return true;
    }
    virtual void lose_window_surface() {}
    virtual bool has_surface() const { return true; }

    /** Upload RGBA8 pixels; returns handle (id starts at 1). */
    virtual TextureHandle create_texture_rgba8(uint32_t width, uint32_t height, const uint8_t* rgba) = 0;
    virtual void destroy_texture(TextureHandle handle) = 0;

    virtual bool begin_frame() = 0;
    virtual void set_camera(const Camera2D& camera) = 0;
    virtual void draw_sprites(const SpriteInstance* sprites, uint32_t count) = 0;
    /** Clear background, flush batches, present. */
    virtual bool end_frame() = 0;

    virtual uint64_t frames_presented() const = 0;
    virtual uint32_t last_frame_draw_calls() const = 0;
    virtual uint32_t last_frame_sprites() const = 0;
    /** GPU frame time in ms when timestamp queries enabled; otherwise -1. */
    virtual double last_gpu_frame_ms() const { return -1.0; }
};

Renderer* create_vulkan_renderer();
