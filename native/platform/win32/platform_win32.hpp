#pragma once

#include "../platform.hpp"

#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#ifndef NOMINMAX
#define NOMINMAX
#endif
#include <windows.h>

#include <queue>

class PlatformWin32 final : public Platform {
public:
    bool init(const PlatformWindowDesc& desc) override;
    void shutdown() override;
    bool poll_event(PlatformEvent& out) override;
    void* native_window_handle() const override;
    void* native_module_handle() const override;
    int width() const override;
    int height() const override;
    bool focused() const override;
    double monotonic_seconds() const override;

    void push_event(PlatformEvent event);

private:
    static LRESULT CALLBACK wnd_proc(HWND hwnd, UINT msg, WPARAM wparam, LPARAM lparam);

    HINSTANCE instance_ = nullptr;
    HWND hwnd_ = nullptr;
    int width_ = 0;
    int height_ = 0;
    bool focused_ = true;
    bool should_quit_ = false;
    std::queue<PlatformEvent> events_;
    LARGE_INTEGER qpc_freq_{};
    LARGE_INTEGER qpc_start_{};
};
