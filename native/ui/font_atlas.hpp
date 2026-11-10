#pragma once

#include "../renderer/renderer.hpp"

#include <cstdint>
#include <string>
#include <vector>

struct GlyphInfo {
    float u0 = 0, v0 = 0, u1 = 0, v1 = 0;
    float x0 = 0, y0 = 0, x1 = 0, y1 = 0; // pixel offsets from baseline
    float advance = 0;
};

/** Rasterized ASCII glyph atlas uploaded as RGBA8 texture. */
class FontAtlas {
public:
    bool init(Renderer& renderer, const char* ttf_path, float pixel_height = 28.0f);
    void shutdown(Renderer& renderer);

    bool ready() const { return texture_.valid(); }
    TextureHandle texture() const { return texture_; }
    float line_height() const { return line_height_; }
    float ascent() const { return ascent_; }

    const GlyphInfo* glyph(char c) const;
    float measure(const char* text) const;

private:
    TextureHandle texture_{};
    GlyphInfo glyphs_[128]{};
    float line_height_ = 28.0f;
    float ascent_ = 22.0f;
};
