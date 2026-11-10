#pragma once

#include "../renderer/renderer.hpp"

#include <string>

/** Decode a PNG from disk (stb_image) and upload via renderer. */
TextureHandle load_texture_png(Renderer& renderer, const char* path);
