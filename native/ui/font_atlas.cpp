#include "font_atlas.hpp"

#include "../assets/asset_fs.hpp"

#include <cstdio>
#include <cstring>
#include <fstream>
#include <vector>

#define STB_TRUETYPE_IMPLEMENTATION
#include "../../third_party/stb/stb_truetype.h"

bool FontAtlas::init(Renderer& renderer, const char* ttf_path, float pixel_height) {
    shutdown(renderer);
    if (!ttf_path) {
        return false;
    }
    std::vector<unsigned char> ttf;
    if (AssetFs* fs = vs_asset_fs()) {
        std::vector<uint8_t> bytes;
        if (fs->read_bytes(ttf_path, bytes)) {
            ttf.assign(bytes.begin(), bytes.end());
        }
    }
    if (ttf.empty()) {
        std::ifstream in(ttf_path, std::ios::binary | std::ios::ate);
        if (!in) {
            std::fprintf(stderr, "[vs] font: cannot open %s\n", ttf_path);
            return false;
        }
        const auto sz = static_cast<size_t>(in.tellg());
        in.seekg(0);
        ttf.resize(sz);
        in.read(reinterpret_cast<char*>(ttf.data()), static_cast<std::streamsize>(sz));
        if (!in) {
            return false;
        }
    }

    stbtt_fontinfo info{};
    if (!stbtt_InitFont(&info, ttf.data(), 0)) {
        std::fprintf(stderr, "[vs] font: stbtt_InitFont failed\n");
        return false;
    }

    const float scale = stbtt_ScaleForPixelHeight(&info, pixel_height);
    int ascent = 0, descent = 0, line_gap = 0;
    stbtt_GetFontVMetrics(&info, &ascent, &descent, &line_gap);
    ascent_ = static_cast<float>(ascent) * scale;
    line_height_ = static_cast<float>(ascent - descent + line_gap) * scale;

    constexpr int kAtlasW = 512;
    constexpr int kAtlasH = 512;
    std::vector<uint8_t> alpha(static_cast<size_t>(kAtlasW * kAtlasH), 0);

    int pen_x = 2;
    int pen_y = 2;
    int row_h = 0;

    for (int c = 32; c < 127; ++c) {
        int adv = 0, lsb = 0;
        stbtt_GetCodepointHMetrics(&info, c, &adv, &lsb);
        int x0 = 0, y0 = 0, x1 = 0, y1 = 0;
        stbtt_GetCodepointBitmapBox(&info, c, scale, scale, &x0, &y0, &x1, &y1);
        const int gw = x1 - x0;
        const int gh = y1 - y0;
        if (pen_x + gw + 2 >= kAtlasW) {
            pen_x = 2;
            pen_y += row_h + 2;
            row_h = 0;
        }
        if (pen_y + gh + 2 >= kAtlasH) {
            std::fprintf(stderr, "[vs] font: atlas overflow at '%c'\n", c);
            break;
        }
        stbtt_MakeCodepointBitmap(
            &info,
            &alpha[static_cast<size_t>(pen_y * kAtlasW + pen_x)],
            gw,
            gh,
            kAtlasW,
            scale,
            scale,
            c);

        GlyphInfo& g = glyphs_[c];
        g.x0 = static_cast<float>(x0);
        g.y0 = static_cast<float>(y0);
        g.x1 = static_cast<float>(x1);
        g.y1 = static_cast<float>(y1);
        g.advance = static_cast<float>(adv) * scale;
        g.u0 = static_cast<float>(pen_x) / static_cast<float>(kAtlasW);
        g.v0 = static_cast<float>(pen_y) / static_cast<float>(kAtlasH);
        g.u1 = static_cast<float>(pen_x + gw) / static_cast<float>(kAtlasW);
        g.v1 = static_cast<float>(pen_y + gh) / static_cast<float>(kAtlasH);

        pen_x += gw + 2;
        if (gh > row_h) {
            row_h = gh;
        }
    }

    // Expand R8 → RGBA8 (white RGB, alpha from coverage).
    std::vector<uint8_t> rgba(static_cast<size_t>(kAtlasW * kAtlasH * 4));
    for (int i = 0; i < kAtlasW * kAtlasH; ++i) {
        rgba[static_cast<size_t>(i * 4 + 0)] = 255;
        rgba[static_cast<size_t>(i * 4 + 1)] = 255;
        rgba[static_cast<size_t>(i * 4 + 2)] = 255;
        rgba[static_cast<size_t>(i * 4 + 3)] = alpha[static_cast<size_t>(i)];
    }

    texture_ = renderer.create_texture_rgba8(
        static_cast<uint32_t>(kAtlasW),
        static_cast<uint32_t>(kAtlasH),
        rgba.data());
    if (!texture_.valid()) {
        std::fprintf(stderr, "[vs] font: texture upload failed\n");
        return false;
    }
    std::fprintf(stderr, "[vs] font atlas ready (%s @ %.0fpx)\n", ttf_path, pixel_height);
    return true;
}

void FontAtlas::shutdown(Renderer& renderer) {
    if (texture_.valid()) {
        renderer.destroy_texture(texture_);
        texture_ = {};
    }
    std::memset(glyphs_, 0, sizeof(glyphs_));
}

const GlyphInfo* FontAtlas::glyph(char c) const {
    const unsigned char uc = static_cast<unsigned char>(c);
    if (uc < 32 || uc >= 127) {
        return nullptr;
    }
    return &glyphs_[uc];
}

float FontAtlas::measure(const char* text) const {
    if (!text) {
        return 0.0f;
    }
    float w = 0.0f;
    for (const char* p = text; *p; ++p) {
        const GlyphInfo* g = glyph(*p);
        if (g) {
            w += g->advance;
        }
    }
    return w;
}
