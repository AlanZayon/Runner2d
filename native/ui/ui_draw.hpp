#pragma once

#include "font_atlas.hpp"

#include "../renderer/renderer.hpp"

#include <cstdint>
#include <vector>

struct UiColor {
    float r, g, b, a;
};

/** Emits screen-space quads into the sprite batch (camera identity / overlay). */
class UiDraw {
public:
    void begin(std::vector<SpriteInstance>& out, const FontAtlas* font, TextureHandle white);
    void panel(float x, float y, float w, float h, UiColor color);
    void text(float x, float y, const char* str, UiColor color, float scale = 1.0f);
    void text_centered(float cx, float y, const char* str, UiColor color, float scale = 1.0f);

private:
    std::vector<SpriteInstance>* out_ = nullptr;
    const FontAtlas* font_ = nullptr;
    TextureHandle white_{};
};
