#pragma once

#ifdef __ANDROID__

#include "../platform.hpp"

#include <android/native_window.h>
#include <android/sensor.h>
#include <android_native_app_glue.h>

#include <queue>

class PlatformAndroid final : public Platform {
public:
    explicit PlatformAndroid(android_app* app);

    bool init(const PlatformWindowDesc& desc) override;
    void shutdown() override;
    bool poll_event(PlatformEvent& out) override;
    void* native_window_handle() const override;
    void* native_module_handle() const override;
    int width() const override;
    int height() const override;
    bool focused() const override;
    double monotonic_seconds() const override;
    float tilt_x() const override;

    android_app* app() const { return app_; }
    AAssetManager* asset_manager() const;
    void push_event(PlatformEvent e);

    static void handle_cmd(android_app* app, int32_t cmd);
    static int32_t handle_input(android_app* app, AInputEvent* event);

private:
    void update_size_from_window();
    void init_sensors();
    void shutdown_sensors();
    void set_sensors_enabled(bool enabled);
    void drain_sensors();

    android_app* app_ = nullptr;
    ANativeWindow* window_ = nullptr;
    int width_ = 0;
    int height_ = 0;
    bool focused_ = false;
    bool running_ = true;
    std::queue<PlatformEvent> events_;
    int64_t start_ns_ = 0;

    ASensorManager* sensor_manager_ = nullptr;
    const ASensor* gravity_sensor_ = nullptr;
    ASensorEventQueue* sensor_queue_ = nullptr;
    bool sensors_enabled_ = false;
    float tilt_filtered_ = 0.0f;
    float tilt_x_ = 0.0f;
};

#endif
