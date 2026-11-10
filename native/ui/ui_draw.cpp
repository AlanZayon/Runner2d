#include "ui_draw.hpp"

#include <cstring>

void UiDraw::begin(std::vector<SpriteInstance>& out, const FontAtlas* font, TextureHandle white) {
    out_ = &out;
    font_ = font;
    white_ = white;
}

void UiDraw::panel(float x, float y, float w, float h, UiColor color) {
    if (!out_ || !white_.valid()) {
        return;
    }
    SpriteInstance s{};
    s.x = x;
    s.y = y;
    s.w = w;
    s.h = h;
    s.u0 = 0.0f;
    s.v0 = 0.0f;
    s.u1 = 1.0f;
    s.v1 = 1.0f;
    s.r = color.r;
    s.g = color.g;
    s.b = color.b;
    s.a = color.a;
    s.texture = white_;
    out_->push_back(s);
}

void UiDraw::text(float x, float y, const char* str, UiColor color, float scale) {
    if (!out_ || !font_ || !font_->ready() || !str) {
        return;
    }
    float pen_x = x;
    const float baseline = y + font_->ascent() * scale;
    for (const char* p = str; *p; ++p) {
        if (*p == '\n') {
            pen_x = x;
            // callers handle multi-line via separate calls usually
            continue;
        }
        const GlyphInfo* g = font_->glyph(*p);
        if (!g) {
            continue;
        }
        const float gw = (g->x1 - g->x0) * scale;
        const float gh = (g->y1 - g->y0) * scale;
        if (gw > 0.0f && gh > 0.0f) {
            SpriteInstance s{};
            s.x = pen_x + g->x0 * scale;
            s.y = baseline + g->y0 * scale;
            s.w = gw;
            s.h = gh;
            s.u0 = g->u0;
            s.v0 = g->v0;
            s.u1 = g->u1;
            s.v1 = g->v1;
            s.r = color.r;
            s.g = color.g;
            s.b = color.b;
            s.a = color.a;
            s.texture = font_->texture();
            out_->push_back(s);
        }
        pen_x += g->advance * scale;
    }
}

void UiDraw::text_centered(float cx, float y, const char* str, UiColor color, float scale) {
    if (!font_ || !str) {
        return;
    }
    const float w = font_->measure(str) * scale;
    text(cx - w * 0.5f, y, str, color, scale);
}
