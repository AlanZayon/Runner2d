#include "game_loop.hpp"

#include "input_replay.hpp"
#include "log.hpp"
#include "save_store.hpp"

#include "../assets/asset_fs.hpp"
#include "../audio/audio_device.hpp"
#include "../audio/audio_mixer.hpp"
#include "../input/gesture_recognizer.hpp"
#include "../ui/debug_overlay.hpp"
#include "../ui/font_atlas.hpp"
#include "../ui/hud.hpp"
#include "../ui/screens.hpp"
#include "../ui/ui_draw.hpp"

#include "vertical_shift_abi.h"

#ifndef __ANDROID__
#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#ifndef NOMINMAX
#define NOMINMAX
#endif
#include <windows.h>
#else
// Mirror common Win32 VK codes used by InputState / volume keys.
#ifndef VK_ESCAPE
#define VK_ESCAPE 0x1B
#define VK_RETURN 0x0D
#define VK_SPACE 0x20
#define VK_OEM_4 0xDB
#define VK_OEM_6 0xDD
#define VK_OEM_MINUS 0xBD
#define VK_OEM_PLUS 0xBB
#define VK_OEM_PERIOD 0xBE
#define VK_SUBTRACT 0x6D
#define VK_ADD 0x6B
#define VK_F3 0x72
#endif
#endif

#ifndef VK_F3
#define VK_F3 0x72
#endif

#include <algorithm>
#include <chrono>
#include <cmath>
#include <cstring>
#include <string>
#include <vector>

#ifndef VS_ASSET_DIR
#define VS_ASSET_DIR "assets"
#endif

namespace {

void record_frame(GameLoopStats& stats, double frame_ms) {
    stats.sum_frame_ms += frame_ms;
    stats.max_frame_ms = std::max(stats.max_frame_ms, frame_ms);
    stats.recent_ms[stats.recent_index] = frame_ms;
    stats.recent_index = (stats.recent_index + 1) % GameLoopStats::kSamples;
    if (stats.recent_count < GameLoopStats::kSamples) {
        ++stats.recent_count;
    }
}

double approx_p95(const GameLoopStats& stats) {
    if (stats.recent_count == 0) {
        return 0.0;
    }
    double tmp[GameLoopStats::kSamples];
    std::memcpy(tmp, stats.recent_ms, sizeof(double) * static_cast<size_t>(stats.recent_count));
    std::sort(tmp, tmp + stats.recent_count);
    const int idx = static_cast<int>(std::floor(0.95 * (stats.recent_count - 1)));
    return tmp[idx];
}

TextureHandle make_white_texture(Renderer& renderer) {
    const uint8_t px[4] = {255, 255, 255, 255};
    return renderer.create_texture_rgba8(1, 1, px);
}

Camera2D make_ui_camera(float vw, float vh) {
    Camera2D cam{};
    cam.position = {vw * 0.5f, vh * 0.5f};
    cam.zoom = 1.0f;
    cam.set_viewport(vw, vh);
    cam.shake_offset = {0.0f, 0.0f};
    return cam;
}

void apply_shake_impulse(float& mag, float add) {
    mag = std::max(mag, add);
}

uint32_t quality_hysteresis(uint32_t current, double avg_frame_ms) {
    // Drop quality if sustained >22ms; raise if <14ms (hysteresis gap).
    if (avg_frame_ms > 22.0 && current > 0) {
        return current - 1;
    }
    if (avg_frame_ms < 14.0 && current < 3) {
        return current + 1;
    }
    return current;
}

} // namespace

int run_game_loop(
    Platform& platform,
    Renderer& renderer,
    GameplayScene& scene,
    InputState& input,
    const GameLoopConfig& config,
    GameLoopStats& stats) {
    double accumulator = 0.0;
    double prev = platform.monotonic_seconds();
    bool focus_paused = !platform.focused();
    bool running = true;
    const double start = prev;
    std::vector<SpriteInstance> world_sprites;
    std::vector<SpriteInstance> ui_sprites;
    Camera2D camera{};
    float shake_mag = 0.0f;

    AppScreen screen = config.benchmark_seconds > 0.0 ? AppScreen::Playing : AppScreen::Title;
    SaveStore save;
    save.load();
    GestureRecognizer gestures;
    auto apply_gesture_thresholds = [&](int w, int h) {
        const float shortest = static_cast<float>(w < h ? w : h);
        const float min_dist = std::max(24.0f, shortest * 0.030f);
        gestures.set_thresholds(min_dist, 0.65f);
    };
    apply_gesture_thresholds(platform.width(), platform.height());
    InputReplay replay;
    bool debug_overlay = config.debug_overlay;
    bool f3_edge = false;
    uint32_t quality = config.quality_profile;
    if (save.data().quality_profile <= 3) {
        quality = save.data().quality_profile;
    }
    vs_world_set_quality(quality);
    if (config.record_replay_path) {
        replay.begin_record(config.record_replay_path);
    }
    if (config.replay_path) {
        replay.begin_playback(config.replay_path);
    }
    double quality_check_accum = 0.0;

    AudioDevice audio_device;
    AudioMixer mixer;
    const bool audio_ok = audio_device.init();
    // Relative to AssetFs root (portable pack or build assets dir).
    const std::string audio_dir = "audio";
    std::string font_path = "fonts/ui.ttf";
    if (AssetFs* fs = vs_asset_fs()) {
        font_path = fs->resolve("fonts/ui.ttf");
    }
#ifndef __ANDROID__
    else {
        font_path = std::string(VS_ASSET_DIR) + "/fonts/ui.ttf";
    }
#endif
    if (audio_ok) {
        mixer.init(audio_device, audio_dir.c_str());
        mixer.set_master_volume(save.data().vol_master);
        mixer.set_sfx_volume(save.data().vol_sfx);
    }

    FontAtlas font;
    const bool font_ok = font.init(renderer, font_path.c_str(), 28.0f);
    TextureHandle white = make_white_texture(renderer);
    if (!white.valid()) {
        vs_log("UI white texture failed");
    }

    scene.set_viewport(static_cast<float>(platform.width()), static_cast<float>(platform.height()));

    auto restart_run = [&]() {
        vs_world_reset();
        scene.set_viewport(static_cast<float>(platform.width()), static_cast<float>(platform.height()));
        shake_mag = 0.0f;
        screen = AppScreen::Playing;
    };

    while (running) {
        PlatformEvent event{};
        while (platform.poll_event(event)) {
            switch (event.kind) {
            case PlatformEvent::Kind::Quit:
                running = false;
                break;
            case PlatformEvent::Kind::FocusLost:
            case PlatformEvent::Kind::Pause:
                // System background: freeze sim only. Don't enter the Pause menu — returning
                // should continue the run. (Back/Esc still opens Paused intentionally.)
                focus_paused = true;
                if (!platform.native_window_handle()) {
                    renderer.lose_window_surface();
                }
                vs_log("Paused (focus/background)");
                break;
            case PlatformEvent::Kind::FocusGained:
            case PlatformEvent::Kind::Resume:
                focus_paused = false;
                prev = platform.monotonic_seconds();
                accumulator = 0.0;
                if (platform.native_window_handle()) {
                    const int w = platform.width();
                    const int h = platform.height();
                    if (w > 0 && h > 0) {
                        if (!renderer.has_surface()) {
                            if (!renderer.recreate_window_surface(
                                    platform.native_window_handle(), w, h)) {
                                vs_log("Resume: surface recreate deferred");
                            }
                        } else {
                            renderer.resize(w, h);
                        }
                        scene.set_viewport(static_cast<float>(w), static_cast<float>(h));
                        apply_gesture_thresholds(w, h);
                    }
                }
                vs_log("Resumed");
                break;
            case PlatformEvent::Kind::Resize:
                if (event.width > 0 && event.height > 0) {
                    if (platform.native_window_handle() && !renderer.has_surface()) {
                        renderer.recreate_window_surface(
                            platform.native_window_handle(), event.width, event.height);
                    } else if (renderer.has_surface()) {
                        renderer.resize(event.width, event.height);
                    }
                    scene.set_viewport(static_cast<float>(event.width), static_cast<float>(event.height));
                    apply_gesture_thresholds(event.width, event.height);
                }
                break;
            case PlatformEvent::Kind::Back:
                gestures.on_event(event, platform.monotonic_seconds());
                if (screen == AppScreen::Playing) {
                    screen = AppScreen::Paused;
                } else if (screen == AppScreen::Paused) {
                    screen = AppScreen::Playing;
                    prev = platform.monotonic_seconds();
                    accumulator = 0.0;
                } else if (screen == AppScreen::Title) {
                    running = false;
                }
                break;
            case PlatformEvent::Kind::TouchDown:
            case PlatformEvent::Kind::TouchMove:
            case PlatformEvent::Kind::TouchUp:
            case PlatformEvent::Kind::TouchCancel:
                gestures.on_event(event, platform.monotonic_seconds());
                break;
            case PlatformEvent::Kind::KeyDown: {
                const int vk = event.key;
                // Screen / volume keys handled here; gameplay keys go to InputState.
                if (vk == VK_ESCAPE) {
                    if (screen == AppScreen::Playing) {
                        screen = AppScreen::Paused;
                    } else if (screen == AppScreen::Paused) {
                        screen = AppScreen::Playing;
                        prev = platform.monotonic_seconds();
                        accumulator = 0.0;
                    } else if (screen == AppScreen::Title) {
                        running = false;
                    }
                    break;
                }
                if (vk == VK_RETURN || vk == VK_SPACE) {
                    if (screen == AppScreen::Title) {
                        restart_run();
                        break;
                    }
                    if (screen == AppScreen::GameOver) {
                        restart_run();
                        break;
                    }
                }
                if (vk == 'R') {
                    if (screen == AppScreen::GameOver || screen == AppScreen::Playing ||
                        screen == AppScreen::Paused) {
                        restart_run();
                        break;
                    }
                }
                if (vk == VK_F3) {
                    f3_edge = true;
                    break;
                }
                // 1..4 = Low..Ultra quality
                if (vk >= '1' && vk <= '4') {
                    quality = static_cast<uint32_t>(vk - '1');
                    vs_world_set_quality(quality);
                    save.data().quality_profile = quality;
                    save.data().version = 2;
                    save.save();
                    vs_log("quality profile → %u", quality);
                    break;
                }
                if (screen == AppScreen::Paused) {
                    auto& d = save.data();
                    if (vk == VK_OEM_4 /* [ */) {
                        d.vol_master = std::max(0.0f, d.vol_master - 0.05f);
                        mixer.set_master_volume(d.vol_master);
                        save.save();
                    } else if (vk == VK_OEM_6 /* ] */) {
                        d.vol_master = std::min(1.0f, d.vol_master + 0.05f);
                        mixer.set_master_volume(d.vol_master);
                        save.save();
                    } else if (vk == VK_OEM_MINUS || vk == VK_SUBTRACT) {
                        d.vol_sfx = std::max(0.0f, d.vol_sfx - 0.05f);
                        mixer.set_sfx_volume(d.vol_sfx);
                        save.save();
                    } else if (vk == VK_OEM_PLUS || vk == VK_ADD || vk == VK_OEM_PERIOD) {
                        // '=' is OEM_PLUS on US keyboards
                        d.vol_sfx = std::min(1.0f, d.vol_sfx + 0.05f);
                        mixer.set_sfx_volume(d.vol_sfx);
                        save.save();
                    }
                }
                input.on_key(vk, true);
                break;
            }
            case PlatformEvent::Kind::KeyUp:
                input.on_key(event.key, false);
                break;
            default:
                break;
            }
        }

        if (!running) {
            break;
        }

        if (f3_edge) {
            debug_overlay = !debug_overlay;
            f3_edge = false;
            vs_log("debug overlay %s", debug_overlay ? "ON" : "OFF");
        }

        if (gestures.ui_confirm_edge()) {
            if (screen == AppScreen::Title || screen == AppScreen::GameOver) {
                restart_run();
            } else if (screen == AppScreen::Paused) {
                screen = AppScreen::Playing;
                prev = platform.monotonic_seconds();
                accumulator = 0.0;
            }
        }
        gestures.clear_ui_edges();

        const double now = platform.monotonic_seconds();
        double frame_dt = now - prev;
        prev = now;
        if (frame_dt < 0.0) {
            frame_dt = 0.0;
        }
        if (frame_dt > 0.25) {
            frame_dt = 0.25;
        }

        const bool sim_active = (screen == AppScreen::Playing) && !focus_paused;

        if (sim_active) {
            accumulator += frame_dt;
            int updates = 0;
            while (accumulator >= config.fixed_dt && updates < config.max_updates_per_frame) {
                VsInputState vs_input{};
                if (replay.playing()) {
                    if (!replay.next_tick(vs_input)) {
                        // Exhausted replay — idle input.
                        std::memset(&vs_input, 0, sizeof(vs_input));
                    }
                } else {
                    input.consume_for_tick(&vs_input);
                    gestures.consume_into(vs_input, platform.monotonic_seconds());
                    // Phone tilt drives Falling Mode strafe; keyboard A/D wins if held.
                    {
                        const float tilt = platform.tilt_x();
                        if (std::fabs(vs_input.move_x) < 0.05f && std::fabs(tilt) > 0.001f) {
                            vs_input.move_x = tilt;
                        }
                    }
                    // In-run taps must not fire QTE confirms (Title/GameOver only).
                    if (screen == AppScreen::Playing) {
                        vs_input.qte_pressed = 0;
                    }
                }
                // Host owns restart via screens; don't let dead auto-reset steal GameOver.
                vs_input.restart_pressed = 0;
                if (replay.recording()) {
                    replay.record_tick(vs_input);
                }
                const auto t_sim0 = std::chrono::steady_clock::now();
                scene.tick(vs_input, static_cast<float>(config.fixed_dt));
                (void)vs_runtime_tick();

                VsGameEvent events[32];
                uint32_t ev_count = 0;
                if (vs_world_drain_events(events, 32, &ev_count) == VS_OK) {
                    for (uint32_t i = 0; i < ev_count; ++i) {
                        mixer.play_event(events[i].kind);
                        switch (events[i].kind) {
                        case VS_EVENT_DEATH:
                            apply_shake_impulse(shake_mag, 14.0f);
                            break;
                        case VS_EVENT_DASH:
                            apply_shake_impulse(shake_mag, 5.0f);
                            break;
                        case VS_EVENT_DESTROY:
                            apply_shake_impulse(shake_mag, 8.0f);
                            break;
                        default:
                            break;
                        }
                    }
                }
                stats.sum_sim_ms += std::chrono::duration<double, std::milli>(
                                        std::chrono::steady_clock::now() - t_sim0)
                                        .count();

                ++stats.sim_ticks;
                accumulator -= config.fixed_dt;
                ++updates;
            }
            if (updates == config.max_updates_per_frame) {
                accumulator = 0.0;
            }
        } else {
            accumulator = 0.0;
            // Still clear edge flags so they don't pile up.
            VsInputState discard{};
            input.consume_for_tick(&discard);
            gestures.consume_into(discard, platform.monotonic_seconds());
        }

        // Decay camera shake
        if (shake_mag > 0.0f) {
            shake_mag = std::max(0.0f, shake_mag - static_cast<float>(frame_dt) * 40.0f);
        }

        VsRunState run{};
        (void)vs_world_get_run_state(&run);

        if (screen == AppScreen::Playing && !run.alive) {
            save.consider_run(run.score, run.distance);
            screen = AppScreen::GameOver;
            vs_log("GAME OVER score=%u coins=%u dist=%.0f", run.score, run.coins, run.distance);
        }

        // After Android TERM/INIT, swapchain may be missing or 0×0 for a few frames — keep trying.
        if (platform.native_window_handle() && !renderer.has_surface()) {
            const int w = platform.width();
            const int h = platform.height();
            if (w > 0 && h > 0) {
                if (renderer.recreate_window_surface(platform.native_window_handle(), w, h)) {
                    scene.set_viewport(static_cast<float>(w), static_cast<float>(h));
                    apply_gesture_thresholds(w, h);
                }
            }
        }

        if (renderer.has_surface() && renderer.begin_frame()) {
            const float vw = static_cast<float>(platform.width());
            const float vh = static_cast<float>(platform.height());
            const auto t_ex0 = std::chrono::steady_clock::now();

            if (screen == AppScreen::Playing || screen == AppScreen::Paused ||
                screen == AppScreen::GameOver) {
                scene.extract(world_sprites, camera);
                // Visual-only shake
                if (shake_mag > 0.1f) {
                    const float t = static_cast<float>(now);
                    camera.shake_offset = {
                        std::sin(t * 73.1f) * shake_mag,
                        std::cos(t * 91.7f) * shake_mag * 0.85f};
                } else {
                    camera.shake_offset = {0.0f, 0.0f};
                }
            }
            stats.sum_extract_ms += std::chrono::duration<double, std::milli>(
                                        std::chrono::steady_clock::now() - t_ex0)
                                        .count();

            const auto t_dr0 = std::chrono::steady_clock::now();
            if (screen == AppScreen::Playing || screen == AppScreen::Paused ||
                screen == AppScreen::GameOver) {
                if (debug_overlay) {
                    VsDebugAabb aabbs[128];
                    uint32_t aabb_n = 0;
                    if (vs_world_extract_debug_aabbs(aabbs, 128, &aabb_n) == VS_OK && aabb_n > 0) {
                        debug_overlay_append_aabbs(world_sprites, aabbs, aabb_n, white);
                    }
                }
                renderer.set_camera(camera);
                if (!world_sprites.empty()) {
                    renderer.draw_sprites(world_sprites.data(), static_cast<uint32_t>(world_sprites.size()));
                }
            }

            ui_sprites.clear();
            if (font_ok && white.valid()) {
                UiDraw ui;
                ui.begin(ui_sprites, &font, white);
                if (screen == AppScreen::Playing || screen == AppScreen::Paused) {
                    hud_draw(ui, vw, vh, run);
                }
                ScreenDrawParams sp{};
                sp.viewport_w = vw;
                sp.viewport_h = vh;
                sp.run = &run;
                sp.best_score = save.data().best_score;
                sp.best_distance = save.data().best_distance;
                sp.vol_master = save.data().vol_master;
                sp.vol_sfx = save.data().vol_sfx;
#ifdef __ANDROID__
                sp.touch_ui = true;
#else
                sp.touch_ui = false;
#endif
                screens_draw(ui, screen, sp);
                if (debug_overlay) {
                    DebugOverlayStats dos{};
                    dos.frame_ms = frame_dt * 1000.0;
                    dos.fps = frame_dt > 0.0 ? 1.0 / frame_dt : 0.0;
                    dos.sim_ms = stats.sim_ticks > 0
                                     ? stats.sum_sim_ms / static_cast<double>(stats.sim_ticks)
                                     : 0.0;
                    dos.extract_ms = stats.frames > 0
                                         ? stats.sum_extract_ms / static_cast<double>(stats.frames)
                                         : 0.0;
                    dos.draw_ms = stats.frames > 0
                                      ? stats.sum_draw_ms / static_cast<double>(stats.frames)
                                      : 0.0;
                    dos.gpu_ms = renderer.last_gpu_frame_ms();
                    dos.sprites = renderer.last_frame_sprites();
                    dos.draw_calls = renderer.last_frame_draw_calls();
                    dos.seed = run.seed;
                    dos.quality = quality;
                    debug_overlay_draw(ui, dos, run);
                }

                if (!ui_sprites.empty()) {
                    renderer.set_camera(make_ui_camera(vw, vh));
                    renderer.draw_sprites(ui_sprites.data(), static_cast<uint32_t>(ui_sprites.size()));
                }
            }

            if (!renderer.end_frame()) {
                vs_log("Present failed");
                running = false;
                break;
            }
            stats.sum_draw_ms += std::chrono::duration<double, std::milli>(
                                     std::chrono::steady_clock::now() - t_dr0)
                                     .count();
            stats.last_draw_calls = renderer.last_frame_draw_calls();
            stats.last_sprites = renderer.last_frame_sprites();
        }

        record_frame(stats, frame_dt * 1000.0);
        ++stats.frames;
        stats.elapsed_seconds = now - start;

        quality_check_accum += frame_dt;
        if (quality_check_accum >= 2.0 && stats.recent_count > 30) {
            quality_check_accum = 0.0;
            const double avg =
                stats.frames > 0 ? stats.sum_frame_ms / static_cast<double>(stats.frames) : 16.0;
            const uint32_t next_q = quality_hysteresis(quality, avg);
            if (next_q != quality) {
                quality = next_q;
                vs_world_set_quality(quality);
                save.data().quality_profile = quality;
                save.data().version = 2;
                vs_log("quality auto → %u (avg_frame=%.2fms)", quality, avg);
            }
        }

        if (config.benchmark_seconds > 0.0 && stats.elapsed_seconds >= config.benchmark_seconds) {
            running = false;
        }
    }

    replay.end_record();
    replay.end_playback();
    save.data().vol_master = mixer.master_volume();
    save.data().vol_sfx = mixer.sfx_volume();
    save.data().quality_profile = quality;
    save.data().version = 2;
    save.save();

    mixer.shutdown();
    audio_device.shutdown();
    if (white.valid()) {
        renderer.destroy_texture(white);
    }
    font.shutdown(renderer);

    stats.elapsed_seconds = platform.monotonic_seconds() - start;
    const double avg = stats.frames > 0 ? stats.sum_frame_ms / static_cast<double>(stats.frames) : 0.0;
    const double avg_sim =
        stats.sim_ticks > 0 ? stats.sum_sim_ms / static_cast<double>(stats.sim_ticks) : 0.0;
    const double avg_ex =
        stats.frames > 0 ? stats.sum_extract_ms / static_cast<double>(stats.frames) : 0.0;
    const double avg_dr =
        stats.frames > 0 ? stats.sum_draw_ms / static_cast<double>(stats.frames) : 0.0;
    vs_log(
        "Loop done: frames=%llu sim=%llu elapsed=%.3fs avg=%.3fms p95=%.3fms draws=%u sprites=%u presented=%llu",
        static_cast<unsigned long long>(stats.frames),
        static_cast<unsigned long long>(stats.sim_ticks),
        stats.elapsed_seconds,
        avg,
        approx_p95(stats),
        stats.last_draw_calls,
        stats.last_sprites,
        static_cast<unsigned long long>(renderer.frames_presented()));
    vs_log(
        "timers: avg_sim=%.4fms avg_extract=%.4fms avg_draw+present=%.4fms",
        avg_sim,
        avg_ex,
        avg_dr);
    return 0;
}

int run_cpu_bench(
    GameplayScene& scene,
    uint32_t ticks,
    uint64_t /*seed*/,
    bool /*seed_locked*/,
    GameLoopStats& stats) {
    using clock = std::chrono::steady_clock;
    std::vector<SpriteInstance> sprites;
    Camera2D camera{};
    VsInputState input{};
    const auto wall0 = clock::now();
    for (uint32_t i = 0; i < ticks; ++i) {
        const auto t0 = clock::now();
        scene.tick(input, 1.0f / 60.0f);
        stats.sum_sim_ms +=
            std::chrono::duration<double, std::milli>(clock::now() - t0).count();
        const auto t1 = clock::now();
        scene.extract(sprites, camera);
        stats.sum_extract_ms +=
            std::chrono::duration<double, std::milli>(clock::now() - t1).count();
        ++stats.sim_ticks;
    }
    stats.elapsed_seconds =
        std::chrono::duration<double>(clock::now() - wall0).count();
    stats.frames = ticks;
    const double avg_sim =
        stats.sim_ticks > 0 ? stats.sum_sim_ms / static_cast<double>(stats.sim_ticks) : 0.0;
    const double avg_ex =
        stats.sim_ticks > 0 ? stats.sum_extract_ms / static_cast<double>(stats.sim_ticks) : 0.0;
    vs_log(
        "CPU-BENCH ticks=%u wall=%.3fs avg_sim=%.4fms avg_extract=%.4fms sprites_last=%u",
        ticks,
        stats.elapsed_seconds,
        avg_sim,
        avg_ex,
        static_cast<unsigned>(sprites.size()));
    return 0;
}

double vs_stats_approx_p95(const GameLoopStats& stats) {
    return approx_p95(stats);
}
