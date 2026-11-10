#include "hud.hpp"

#include <cstdio>

namespace {
UiColor kWhite{1.0f, 1.0f, 1.0f, 1.0f};
UiColor kDim{0.05f, 0.07f, 0.10f, 0.55f};
UiColor kAccent{0.95f, 0.82f, 0.25f, 1.0f};
UiColor kWarn{1.0f, 0.45f, 0.25f, 1.0f};
} // namespace

void hud_draw(UiDraw& ui, float viewport_w, float viewport_h, const VsRunState& run) {
    (void)viewport_h;
    ui.panel(8.0f, 8.0f, 280.0f, 96.0f, kDim);

    char line[96];
    std::snprintf(line, sizeof(line), "SCORE  %u", run.score);
    ui.text(18.0f, 14.0f, line, kAccent, 0.85f);

    std::snprintf(line, sizeof(line), "COINS  %u", run.coins);
    ui.text(18.0f, 40.0f, line, kWhite, 0.75f);

    std::snprintf(line, sizeof(line), "DIST   %.0f", run.distance);
    ui.text(18.0f, 64.0f, line, kWhite, 0.75f);

    // Ability bits strip
    if (run.ability_mask != 0) {
        ui.panel(viewport_w - 168.0f, 8.0f, 160.0f, 36.0f, kDim);
        std::snprintf(line, sizeof(line), "ABIL 0x%X", run.ability_mask);
        ui.text(viewport_w - 156.0f, 14.0f, line, kWhite, 0.7f);
    }

    if (run.falling) {
        ui.panel(viewport_w * 0.5f - 140.0f, 50.0f, 280.0f, 40.0f, UiColor{0.05f, 0.15f, 0.25f, 0.7f});
        ui.text_centered(viewport_w * 0.5f, 56.0f, "FALLING — A/D strafe", kWhite, 0.8f);
    }

    if (run.qte_active) {
        ui.panel(viewport_w * 0.5f - 160.0f, viewport_h * 0.35f, 320.0f, 72.0f, UiColor{0.2f, 0.05f, 0.05f, 0.8f});
        ui.text_centered(viewport_w * 0.5f, viewport_h * 0.35f + 10.0f, "QTE — press K / E", kWarn, 1.0f);
        std::snprintf(line, sizeof(line), "prompt %u", run.qte_prompt);
        ui.text_centered(viewport_w * 0.5f, viewport_h * 0.35f + 42.0f, line, kWhite, 0.75f);
    }
}
