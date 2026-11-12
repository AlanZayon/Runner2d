#ifdef __ANDROID__

#include "platform_android.hpp"

#include <android/input.h>
#include <android/log.h>
#include <time.h>

namespace {
PlatformAndroid* g_android = nullptr;
}

Platform* create_platform() {
    // Created explicitly from android_main with the app pointer.
    return nullptr;
}

PlatformAndroid::PlatformAndroid(android_app* app) : app_(app) {
    g_android = this;
    app_->userData = this;
    app_->onAppCmd = &PlatformAndroid::handle_cmd;
    app_->onInputEvent = &PlatformAndroid::handle_input;
    struct timespec ts {};
    clock_gettime(CLOCK_MONOTONIC, &ts);
    start_ns_ = static_cast<int64_t>(ts.tv_sec) * 1000000000LL + ts.tv_nsec;
    init_sensors();
}

bool PlatformAndroid::init(const PlatformWindowDesc&) {
    // Wait until we have a window from the activity.
    while (running_ && !window_) {
        PlatformEvent e{};
        if (!poll_event(e)) {
            // Block briefly for lifecycle.
            int events = 0;
            android_poll_source* source = nullptr;
            if (ALooper_pollOnce(100, nullptr, &events, reinterpret_cast<void**>(&source)) >= 0) {
                if (source) {
                    source->process(app_, source);
                }
            }
            if (app_->destroyRequested) {
                running_ = false;
                break;
            }
        } else {
            // Lifecycle events already queued; keep draining later.
            push_event(e);
        }
        if (app_->window) {
            window_ = app_->window;
            update_size_from_window();
            focused_ = true;
            set_sensors_enabled(true);
        }
    }
    return window_ != nullptr;
}

void PlatformAndroid::shutdown() {
    set_sensors_enabled(false);
    shutdown_sensors();
    window_ = nullptr;
    g_android = nullptr;
}

bool PlatformAndroid::poll_event(PlatformEvent& out) {
    int events = 0;
    android_poll_source* source = nullptr;
    int ident = 0;
    while ((ident = ALooper_pollOnce(0, nullptr, &events, reinterpret_cast<void**>(&source))) >= 0) {
        if (ident == LOOPER_ID_USER) {
            drain_sensors();
        } else if (source) {
            source->process(app_, source);
        }
        if (app_->destroyRequested) {
            PlatformEvent quit{};
            quit.kind = PlatformEvent::Kind::Quit;
            events_.push(quit);
            running_ = false;
            break;
        }
    }
    // Keep tilt fresh even when the looper had no pending fds this frame.
    drain_sensors();

    if (events_.empty()) {
        out = {};
        return false;
    }
    out = events_.front();
    events_.pop();
    return true;
}

void* PlatformAndroid::native_window_handle() const {
    return window_;
}

void* PlatformAndroid::native_module_handle() const {
    return nullptr;
}

int PlatformAndroid::width() const {
    return width_;
}

int PlatformAndroid::height() const {
    return height_;
}

bool PlatformAndroid::focused() const {
    return focused_;
}

double PlatformAndroid::monotonic_seconds() const {
    struct timespec ts {};
    clock_gettime(CLOCK_MONOTONIC, &ts);
    const int64_t now =
        static_cast<int64_t>(ts.tv_sec) * 1000000000LL + ts.tv_nsec;
    return static_cast<double>(now - start_ns_) / 1e9;
}

float PlatformAndroid::tilt_x() const {
    return tilt_x_;
}

void PlatformAndroid::init_sensors() {
    if (!app_ || !app_->looper) {
        return;
    }
    sensor_manager_ = ASensorManager_getInstanceForPackage("com.verticalshift.game");
    if (!sensor_manager_) {
        return;
    }
    // Gravity is steadier than raw accel for Subway-style bank control.
    gravity_sensor_ = ASensorManager_getDefaultSensor(sensor_manager_, ASENSOR_TYPE_GRAVITY);
    if (!gravity_sensor_) {
        gravity_sensor_ = ASensorManager_getDefaultSensor(sensor_manager_, ASENSOR_TYPE_ACCELEROMETER);
    }
    if (!gravity_sensor_) {
        return;
    }
    sensor_queue_ = ASensorManager_createEventQueue(
        sensor_manager_, app_->looper, LOOPER_ID_USER, nullptr, nullptr);
}

void PlatformAndroid::shutdown_sensors() {
    if (sensor_queue_ && sensor_manager_) {
        ASensorManager_destroyEventQueue(sensor_manager_, sensor_queue_);
    }
    sensor_queue_ = nullptr;
    gravity_sensor_ = nullptr;
    sensor_manager_ = nullptr;
    sensors_enabled_ = false;
    tilt_filtered_ = 0.0f;
    tilt_x_ = 0.0f;
}

void PlatformAndroid::set_sensors_enabled(bool enabled) {
    if (!sensor_queue_ || !gravity_sensor_ || sensors_enabled_ == enabled) {
        return;
    }
    if (enabled) {
        ASensorEventQueue_enableSensor(sensor_queue_, gravity_sensor_);
        // ~60 Hz — smooth enough for banked turns without burning battery.
        ASensorEventQueue_setEventRate(sensor_queue_, gravity_sensor_, 16000);
    } else {
        ASensorEventQueue_disableSensor(sensor_queue_, gravity_sensor_);
        tilt_filtered_ = 0.0f;
        tilt_x_ = 0.0f;
    }
    sensors_enabled_ = enabled;
}

void PlatformAndroid::drain_sensors() {
    if (!sensor_queue_ || !sensors_enabled_) {
        return;
    }
    ASensorEvent event{};
    while (ASensorEventQueue_getEvents(sensor_queue_, &event, 1) > 0) {
        // Portrait: bank left/right → gravity.x. Right-side-down ⇒ +x ⇒ move right.
        const float g = ASENSOR_STANDARD_GRAVITY > 0.0f ? ASENSOR_STANDARD_GRAVITY : 9.80665f;
        float raw = event.vector.x / g;
        // Mild low-pass so micro-jitters don't yank the runner.
        constexpr float kFilter = 0.22f;
        tilt_filtered_ += (raw - tilt_filtered_) * kFilter;

        constexpr float kDeadzone = 0.06f;
        constexpr float kFullTilt = 0.38f; // ~22° for full deflection
        float v = tilt_filtered_;
        if (v > -kDeadzone && v < kDeadzone) {
            v = 0.0f;
        } else if (v > 0.0f) {
            v = (v - kDeadzone) / (kFullTilt - kDeadzone);
        } else {
            v = (v + kDeadzone) / (kFullTilt - kDeadzone);
        }
        if (v > 1.0f) {
            v = 1.0f;
        } else if (v < -1.0f) {
            v = -1.0f;
        }
        tilt_x_ = v;
    }
}

AAssetManager* PlatformAndroid::asset_manager() const {
    return app_ && app_->activity ? app_->activity->assetManager : nullptr;
}

void PlatformAndroid::push_event(PlatformEvent e) {
    events_.push(e);
}

void PlatformAndroid::update_size_from_window() {
    if (!window_) {
        return;
    }
    width_ = ANativeWindow_getWidth(window_);
    height_ = ANativeWindow_getHeight(window_);
}

void PlatformAndroid::handle_cmd(android_app* app, int32_t cmd) {
    auto* self = static_cast<PlatformAndroid*>(app->userData);
    if (!self) {
        return;
    }
    switch (cmd) {
    case APP_CMD_INIT_WINDOW:
        self->window_ = app->window;
        self->update_size_from_window();
        {
            PlatformEvent e{};
            e.kind = PlatformEvent::Kind::Resume;
            e.width = self->width_;
            e.height = self->height_;
            self->push_event(e);
            PlatformEvent r{};
            r.kind = PlatformEvent::Kind::Resize;
            r.width = self->width_;
            r.height = self->height_;
            self->push_event(r);
        }
        break;
    case APP_CMD_TERM_WINDOW:
        self->window_ = nullptr;
        self->width_ = 0;
        self->height_ = 0;
        {
            // Explicit surface teardown so Resume always recreates against the new window.
            PlatformEvent e{};
            e.kind = PlatformEvent::Kind::Pause;
            self->push_event(e);
        }
        break;
    case APP_CMD_GAINED_FOCUS:
        self->focused_ = true;
        self->set_sensors_enabled(true);
        {
            PlatformEvent e{};
            e.kind = PlatformEvent::Kind::FocusGained;
            self->push_event(e);
        }
        break;
    case APP_CMD_LOST_FOCUS:
        self->focused_ = false;
        self->set_sensors_enabled(false);
        {
            PlatformEvent e{};
            e.kind = PlatformEvent::Kind::FocusLost;
            self->push_event(e);
        }
        break;
    case APP_CMD_PAUSE:
        self->set_sensors_enabled(false);
        {
            PlatformEvent e{};
            e.kind = PlatformEvent::Kind::Pause;
            self->push_event(e);
        }
        break;
    case APP_CMD_RESUME:
        if (self->focused_) {
            self->set_sensors_enabled(true);
        }
        {
            PlatformEvent e{};
            e.kind = PlatformEvent::Kind::Resume;
            self->push_event(e);
        }
        break;
    default:
        break;
    }
}

int32_t PlatformAndroid::handle_input(android_app* app, AInputEvent* event) {
    auto* self = static_cast<PlatformAndroid*>(app->userData);
    if (!self) {
        return 0;
    }
    const int32_t type = AInputEvent_getType(event);
    if (type == AINPUT_EVENT_TYPE_KEY) {
        const int32_t key = AKeyEvent_getKeyCode(event);
        const int32_t action = AKeyEvent_getAction(event);
        if (key == AKEYCODE_BACK && action == AKEY_EVENT_ACTION_UP) {
            PlatformEvent e{};
            e.kind = PlatformEvent::Kind::Back;
            self->push_event(e);
            return 1;
        }
        return 0;
    }
    if (type != AINPUT_EVENT_TYPE_MOTION) {
        return 0;
    }
    const int32_t action = AMotionEvent_getAction(event);
    const int32_t masked = action & AMOTION_EVENT_ACTION_MASK;
    const int32_t index = (action & AMOTION_EVENT_ACTION_POINTER_INDEX_MASK) >>
                          AMOTION_EVENT_ACTION_POINTER_INDEX_SHIFT;
    const float x = AMotionEvent_getX(event, static_cast<size_t>(index));
    const float y = AMotionEvent_getY(event, static_cast<size_t>(index));
    const int id = AMotionEvent_getPointerId(event, static_cast<size_t>(index));

    switch (masked) {
    case AMOTION_EVENT_ACTION_DOWN:
    case AMOTION_EVENT_ACTION_POINTER_DOWN: {
        PlatformEvent e{};
        e.kind = PlatformEvent::Kind::TouchDown;
        e.touch_id = id;
        e.x = x;
        e.y = y;
        self->push_event(e);
        break;
    }
    case AMOTION_EVENT_ACTION_MOVE: {
        // Emit one Move per active pointer so the recognizer can track its primary id.
        const size_t count = AMotionEvent_getPointerCount(event);
        for (size_t i = 0; i < count; ++i) {
            PlatformEvent e{};
            e.kind = PlatformEvent::Kind::TouchMove;
            e.touch_id = AMotionEvent_getPointerId(event, i);
            e.x = AMotionEvent_getX(event, i);
            e.y = AMotionEvent_getY(event, i);
            self->push_event(e);
        }
        break;
    }
    case AMOTION_EVENT_ACTION_UP:
    case AMOTION_EVENT_ACTION_POINTER_UP: {
        PlatformEvent e{};
        e.kind = PlatformEvent::Kind::TouchUp;
        e.touch_id = id;
        e.x = x;
        e.y = y;
        self->push_event(e);
        break;
    }
    case AMOTION_EVENT_ACTION_CANCEL: {
        PlatformEvent e{};
        e.kind = PlatformEvent::Kind::TouchCancel;
        e.touch_id = id;
        e.x = x;
        e.y = y;
        self->push_event(e);
        break;
    }
    default:
        return 0;
    }
    return 1;
}

#endif
