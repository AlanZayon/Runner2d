#include "screens.hpp"

#include <cstdio>

namespace {
UiColor kWhite{1.0f, 1.0f, 1.0f, 1.0f};
UiColor kMuted{0.75f, 0.78f, 0.82f, 1.0f};
UiColor kAccent{0.95f, 0.82f, 0.25f, 1.0f};
UiColor kPanel{0.04f, 0.05f, 0.08f, 0.82f};
} // namespace

void screens_draw(UiDraw& ui, AppScreen screen, const ScreenDrawParams& p) {
    const float cx = p.viewport_w * 0.5f;
    const float cy = p.viewport_h * 0.5f;
    char line[128];

    switch (screen) {
    case AppScreen::Title:
        ui.panel(0.0f, 0.0f, p.viewport_w, p.viewport_h, UiColor{0.03f, 0.04f, 0.07f, 0.92f});
        ui.text_centered(cx, cy - 90.0f, "VERTICAL SHIFT", kAccent, 1.6f);
        ui.text_centered(
            cx,
            cy - 20.0f,
            p.touch_ui ? "Tap — Play" : "Enter / Space — Play",
            kWhite,
            0.9f);
        ui.text_centered(
            cx,
            cy + 20.0f,
            p.touch_ui ? "Swipe up jump / down roll / right dash" : "Esc — Pause   R — Restart",
            kMuted,
            0.75f);
        std::snprintf(line, sizeof(line), "Best score %u", p.best_score);
        ui.text_centered(cx, cy + 70.0f, line, kMuted, 0.7f);
        if (p.version && p.version[0]) {
            std::snprintf(line, sizeof(line), "v%s", p.version);
            ui.text_centered(cx, p.viewport_h - 48.0f, line, kMuted, 0.65f);
        }
        break;

    case AppScreen::Paused:
        ui.panel(cx - 220.0f, cy - 120.0f, 440.0f, 240.0f, kPanel);
        ui.text_centered(cx, cy - 90.0f, "PAUSED", kAccent, 1.3f);
        ui.text_centered(
            cx,
            cy - 30.0f,
            p.touch_ui ? "Tap — Resume   Back — Title" : "Esc — Resume",
            kWhite,
            0.85f);
        ui.text_centered(cx, cy + 10.0f, "[ / ] master volume", kMuted, 0.7f);
        ui.text_centered(cx, cy + 40.0f, "- / =  sfx volume", kMuted, 0.7f);
        std::snprintf(line, sizeof(line), "vol M=%.2f  SFX=%.2f", p.vol_master, p.vol_sfx);
        ui.text_centered(cx, cy + 80.0f, line, kMuted, 0.7f);
        break;

    case AppScreen::Countdown: {
        // Compact badge at the top — world stays fully readable underneath.
        const int n = p.countdown_value;
        std::snprintf(line, sizeof(line), "%d", n);
        ui.text_centered(cx, 56.0f, line, kAccent, 1.8f);
        break;
    }

    case AppScreen::GameOver: {
        ui.panel(cx - 240.0f, cy - 140.0f, 480.0f, 280.0f, kPanel);
        ui.text_centered(cx, cy - 110.0f, "GAME OVER", kAccent, 1.4f);
        const uint32_t score = p.run ? p.run->score : 0;
        const float dist = p.run ? p.run->distance : 0.0f;
        std::snprintf(line, sizeof(line), "Score %u", score);
        ui.text_centered(cx, cy - 50.0f, line, kWhite, 1.0f);
        std::snprintf(line, sizeof(line), "Distance %.0f", dist);
        ui.text_centered(cx, cy - 10.0f, line, kMuted, 0.85f);
        std::snprintf(line, sizeof(line), "Best %u / %.0f", p.best_score, p.best_distance);
        ui.text_centered(cx, cy + 30.0f, line, kMuted, 0.75f);
        ui.text_centered(
            cx,
            cy + 80.0f,
            p.touch_ui ? "Tap — Restart" : "R / Enter — Restart",
            kWhite,
            0.85f);
        break;
    }

    case AppScreen::Playing:
    default:
        break;
    }
}
