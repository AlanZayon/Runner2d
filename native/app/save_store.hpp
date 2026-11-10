#pragma once

#include <cstdint>
#include <string>

struct SaveData {
    uint32_t version = 2;
    uint32_t best_score = 0;
    float best_distance = 0.0f;
    float vol_master = 1.0f;
    float vol_sfx = 1.0f;
    /** VsQualityProfile 0..3 */
    uint32_t quality_profile = 1;
};

class SaveStore {
public:
    /** Prefer override dir, else %APPDATA%/VerticalShift/, else cwd. */
    bool load();
    bool save() const;

    SaveData& data() { return data_; }
    const SaveData& data() const { return data_; }

    void consider_run(uint32_t score, float distance);

    /** Inject Android filesDir (or tests). Empty clears override. */
    void set_override_dir(const std::string& dir);

    const std::string& path() const { return path_; }

    /** Process-wide default applied in SaveStore::load if set. */
    static void set_process_override_dir(const std::string& dir);
    static const std::string& process_override_dir();

private:
    SaveData data_{};
    std::string path_;
    std::string override_dir_;

    std::string resolve_path() const;
    void migrate_from_v1();
};
