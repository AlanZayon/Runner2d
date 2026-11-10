#include "texture_loader.hpp"

#include "asset_fs.hpp"
#include "../app/log.hpp"

#include <vector>

#define STB_IMAGE_IMPLEMENTATION
#include "../../third_party/stb/stb_image.h"

TextureHandle load_texture_png(Renderer& renderer, const char* path) {
    int w = 0, h = 0, comp = 0;
    stbi_uc* pixels = nullptr;

    if (AssetFs* fs = vs_asset_fs()) {
        std::vector<uint8_t> bytes;
        if (fs->read_bytes(path, bytes) && !bytes.empty()) {
            pixels = stbi_load_from_memory(
                bytes.data(),
                static_cast<int>(bytes.size()),
                &w,
                &h,
                &comp,
                4);
        }
    }
    if (!pixels) {
        pixels = stbi_load(path, &w, &h, &comp, 4);
    }
    if (!pixels) {
        vs_log("Failed to load PNG: %s (%s)", path, stbi_failure_reason());
        return {};
    }
    TextureHandle handle = renderer.create_texture_rgba8(
        static_cast<uint32_t>(w), static_cast<uint32_t>(h), pixels);
    stbi_image_free(pixels);
    if (!handle.valid()) {
        vs_log("GPU upload failed for PNG: %s", path);
    } else {
        vs_log("Loaded texture %s (%dx%d) handle=%u", path, w, h, handle.id);
    }
    return handle;
}
