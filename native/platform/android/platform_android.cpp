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
        }
    }
    return window_ != nullptr;
}

void PlatformAndroid::shutdown() {
    window_ = nullptr;
    g_android = nullptr;
}

bool PlatformAndroid::poll_event(PlatformEvent& out) {
    int events = 0;
    android_poll_source* source = nullptr;
    while (ALooper_pollOnce(0, nullptr, &events, reinterpret_cast<void**>(&source)) >= 0) {
        if (source) {
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
        {
            PlatformEvent e{};
            e.kind = PlatformEvent::Kind::Pause;
            self->push_event(e);
        }
        break;
    case APP_CMD_GAINED_FOCUS:
        self->focused_ = true;
        {
            PlatformEvent e{};
            e.kind = PlatformEvent::Kind::FocusGained;
            self->push_event(e);
        }
        break;
    case APP_CMD_LOST_FOCUS:
        self->focused_ = false;
        {
            PlatformEvent e{};
            e.kind = PlatformEvent::Kind::FocusLost;
            self->push_event(e);
        }
        break;
    case APP_CMD_PAUSE:
        {
            PlatformEvent e{};
            e.kind = PlatformEvent::Kind::Pause;
            self->push_event(e);
        }
        break;
    case APP_CMD_RESUME:
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

    PlatformEvent e{};
    e.touch_id = id;
    e.x = x;
    e.y = y;
    switch (masked) {
    case AMOTION_EVENT_ACTION_DOWN:
    case AMOTION_EVENT_ACTION_POINTER_DOWN:
        e.kind = PlatformEvent::Kind::TouchDown;
        break;
    case AMOTION_EVENT_ACTION_MOVE:
        e.kind = PlatformEvent::Kind::TouchMove;
        e.touch_id = 0;
        e.x = AMotionEvent_getX(event, 0);
        e.y = AMotionEvent_getY(event, 0);
        break;
    case AMOTION_EVENT_ACTION_UP:
    case AMOTION_EVENT_ACTION_POINTER_UP:
        e.kind = PlatformEvent::Kind::TouchUp;
        break;
    case AMOTION_EVENT_ACTION_CANCEL:
        e.kind = PlatformEvent::Kind::TouchCancel;
        break;
    default:
        return 0;
    }
    self->push_event(e);
    return 1;
}

#endif
