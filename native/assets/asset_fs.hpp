#pragma once

#include <cstdint>
#include <string>
#include <vector>

/** Portable asset open (filesystem or Android AAssetManager). */
class AssetFs {
public:
    virtual ~AssetFs() = default;
    virtual bool read_bytes(const char* relative_or_abs_path, std::vector<uint8_t>& out) const = 0;
    virtual std::string resolve(const char* relative) const = 0;
};

class FileAssetFs final : public AssetFs {
public:
    explicit FileAssetFs(std::string root);
    bool read_bytes(const char* relative_or_abs_path, std::vector<uint8_t>& out) const override;
    std::string resolve(const char* relative) const override;

private:
    std::string root_;
};

#ifdef __ANDROID__
struct AAssetManager;

class AndroidAssetFs final : public AssetFs {
public:
    explicit AndroidAssetFs(AAssetManager* mgr);
    bool read_bytes(const char* relative_or_abs_path, std::vector<uint8_t>& out) const override;
    std::string resolve(const char* relative) const override;

private:
    AAssetManager* mgr_ = nullptr;
};
#endif

void vs_set_asset_fs(AssetFs* fs);
AssetFs* vs_asset_fs();
