#pragma once

#include <cstdint>

struct PlatformWindowDesc {
    const char* title_utf8 = "VERTICAL SHIFT";
    int width = 432;   // portrait-friendly window
    int height = 768;
};

struct PlatformEvent {
    enum class Kind {
        None,
        Quit,
        Resize,
        FocusGained,
        FocusLost,
        Pause,
        Resume,
        KeyDown,
        KeyUp,
        TouchDown,
        TouchMove,
        TouchUp,
        TouchCancel,
        Back, // Android back / synthetic
    } kind = Kind::None;
    int width = 0;
    int height = 0;
    int key = 0;
    int touch_id = 0;
    float x = 0.0f;
    float y = 0.0f;
};

class Platform {
public:
    virtual ~Platform() = default;

    virtual bool init(const PlatformWindowDesc& desc) = 0;
    virtual void shutdown() = 0;
    virtual bool poll_event(PlatformEvent& out) = 0;
    virtual void* native_window_handle() const = 0;
    virtual void* native_module_handle() const = 0;
    virtual int width() const = 0;
    virtual int height() const = 0;
    virtual bool focused() const = 0;

    /** Monotonic seconds since an arbitrary epoch. */
    virtual double monotonic_seconds() const = 0;

    /** Device tilt for Falling Mode strafe, in [-1, 1]. 0 on platforms without sensors. */
    virtual float tilt_x() const { return 0.0f; }
};

Platform* create_platform();
