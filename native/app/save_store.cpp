#include "save_store.hpp"

#include "log.hpp"

#ifndef __ANDROID__
#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#ifndef NOMINMAX
#define NOMINMAX
#endif
#include <windows.h>
#include <shlobj.h>
#endif

#include <cstdlib>
#include <fstream>
#include <sstream>

namespace {

std::string g_process_override_dir;

bool read_u32_field(const std::string& json, const char* key, uint32_t& out) {
    const std::string needle = std::string("\"") + key + "\"";
    const auto pos = json.find(needle);
    if (pos == std::string::npos) {
        return false;
    }
    const auto colon = json.find(':', pos + needle.size());
    if (colon == std::string::npos) {
        return false;
    }
    out = static_cast<uint32_t>(std::strtoul(json.c_str() + colon + 1, nullptr, 10));
    return true;
}

bool read_f32_field(const std::string& json, const char* key, float& out) {
    const std::string needle = std::string("\"") + key + "\"";
    const auto pos = json.find(needle);
    if (pos == std::string::npos) {
        return false;
    }
    const auto colon = json.find(':', pos + needle.size());
    if (colon == std::string::npos) {
        return false;
    }
    out = static_cast<float>(std::strtod(json.c_str() + colon + 1, nullptr));
    return true;
}

} // namespace

void SaveStore::set_process_override_dir(const std::string& dir) {
    g_process_override_dir = dir;
    while (!g_process_override_dir.empty() &&
           (g_process_override_dir.back() == '/' || g_process_override_dir.back() == '\\')) {
        g_process_override_dir.pop_back();
    }
}

const std::string& SaveStore::process_override_dir() {
    return g_process_override_dir;
}

void SaveStore::set_override_dir(const std::string& dir) {
    override_dir_ = dir;
    while (!override_dir_.empty() &&
           (override_dir_.back() == '/' || override_dir_.back() == '\\')) {
        override_dir_.pop_back();
    }
}

std::string SaveStore::resolve_path() const {
    const std::string& ov =
        !override_dir_.empty() ? override_dir_ : g_process_override_dir;
    if (!ov.empty()) {
        return ov + "/vertical_shift_save.json";
    }
#ifndef __ANDROID__
    char appdata[MAX_PATH]{};
    if (SUCCEEDED(SHGetFolderPathA(nullptr, CSIDL_APPDATA, nullptr, 0, appdata))) {
        std::string dir = std::string(appdata) + "\\VerticalShift";
        CreateDirectoryA(dir.c_str(), nullptr);
        return dir + "\\vertical_shift_save.json";
    }
#endif
    return "vertical_shift_save.json";
}

void SaveStore::migrate_from_v1() {
    data_.version = 2;
    data_.quality_profile = 1; // Balanced
    vs_log("save: migrated v1 → v2 (quality_profile=1)");
}

bool SaveStore::load() {
    path_ = resolve_path();
    data_ = SaveData{};
    std::ifstream in(path_);
    if (!in) {
        vs_log("save: no file yet (%s) — defaults", path_.c_str());
        return false;
    }
    std::ostringstream ss;
    ss << in.rdbuf();
    const std::string json = ss.str();

    uint32_t version = 0;
    if (!read_u32_field(json, "version", version) || (version != 1 && version != 2)) {
        vs_log("save: invalid/unsupported version — defaults (corruption soft-fail)");
        data_ = SaveData{};
        return false;
    }
    data_.version = version;
    read_u32_field(json, "best_score", data_.best_score);
    read_f32_field(json, "best_distance", data_.best_distance);
    read_f32_field(json, "vol_master", data_.vol_master);
    read_f32_field(json, "vol_sfx", data_.vol_sfx);
    if (version >= 2) {
        read_u32_field(json, "quality_profile", data_.quality_profile);
    } else {
        migrate_from_v1();
    }
    if (data_.vol_master < 0.0f || data_.vol_master > 1.0f) {
        data_.vol_master = 1.0f;
    }
    if (data_.vol_sfx < 0.0f || data_.vol_sfx > 1.0f) {
        data_.vol_sfx = 1.0f;
    }
    if (data_.quality_profile > 3) {
        data_.quality_profile = 1;
    }
    vs_log(
        "save: loaded best=%u dist=%.0f vol=%.2f/%.2f quality=%u (%s)",
        data_.best_score,
        data_.best_distance,
        data_.vol_master,
        data_.vol_sfx,
        data_.quality_profile,
        path_.c_str());
    return true;
}

bool SaveStore::save() const {
    std::ofstream out(path_);
    if (!out) {
        vs_log("save: write failed (%s)", path_.c_str());
        return false;
    }
    out << "{\n"
        << "  \"version\": " << data_.version << ",\n"
        << "  \"best_score\": " << data_.best_score << ",\n"
        << "  \"best_distance\": " << data_.best_distance << ",\n"
        << "  \"vol_master\": " << data_.vol_master << ",\n"
        << "  \"vol_sfx\": " << data_.vol_sfx << ",\n"
        << "  \"quality_profile\": " << data_.quality_profile << "\n"
        << "}\n";
    return true;
}

void SaveStore::consider_run(uint32_t score, float distance) {
    bool changed = false;
    if (score > data_.best_score) {
        data_.best_score = score;
        changed = true;
    }
    if (distance > data_.best_distance) {
        data_.best_distance = distance;
        changed = true;
    }
    if (changed) {
        save();
    }
}
