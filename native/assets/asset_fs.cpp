#include "asset_fs.hpp"

#include <cstdio>
#include <cstring>
#include <fstream>

#ifdef __ANDROID__
#include <android/asset_manager.h>
#endif

namespace {
AssetFs* g_fs = nullptr;
}

void vs_set_asset_fs(AssetFs* fs) {
    g_fs = fs;
}

AssetFs* vs_asset_fs() {
    return g_fs;
}

FileAssetFs::FileAssetFs(std::string root) : root_(std::move(root)) {
    while (!root_.empty() && (root_.back() == '/' || root_.back() == '\\')) {
        root_.pop_back();
    }
}

std::string FileAssetFs::resolve(const char* relative) const {
    if (!relative) {
        return root_;
    }
    // Absolute-ish Windows/Unix path
    if (relative[0] == '/' || relative[0] == '\\' ||
        (relative[0] && relative[1] == ':')) {
        return relative;
    }
    if (root_.empty()) {
        return relative;
    }
    return root_ + "/" + relative;
}

bool FileAssetFs::read_bytes(const char* relative_or_abs_path, std::vector<uint8_t>& out) const {
    out.clear();
    const std::string path = resolve(relative_or_abs_path);
    std::ifstream in(path, std::ios::binary | std::ios::ate);
    if (!in) {
        return false;
    }
    const auto sz = static_cast<size_t>(in.tellg());
    in.seekg(0);
    out.resize(sz);
    if (sz > 0) {
        in.read(reinterpret_cast<char*>(out.data()), static_cast<std::streamsize>(sz));
        if (!in) {
            out.clear();
            return false;
        }
    }
    return true;
}

#ifdef __ANDROID__
AndroidAssetFs::AndroidAssetFs(AAssetManager* mgr) : mgr_(mgr) {}

std::string AndroidAssetFs::resolve(const char* relative) const {
    return relative ? relative : "";
}

bool AndroidAssetFs::read_bytes(const char* relative_or_abs_path, std::vector<uint8_t>& out) const {
    out.clear();
    if (!mgr_ || !relative_or_abs_path) {
        return false;
    }
    // Strip leading "assets/" if present.
    const char* path = relative_or_abs_path;
    if (std::strncmp(path, "assets/", 7) == 0) {
        path += 7;
    }
    AAsset* asset = AAssetManager_open(mgr_, path, AASSET_MODE_BUFFER);
    if (!asset) {
        return false;
    }
    const off_t len = AAsset_getLength(asset);
    if (len < 0) {
        AAsset_close(asset);
        return false;
    }
    out.resize(static_cast<size_t>(len));
    const int read = AAsset_read(asset, out.data(), static_cast<size_t>(len));
    AAsset_close(asset);
    if (read != static_cast<int>(len)) {
        out.clear();
        return false;
    }
    return true;
}
#endif
