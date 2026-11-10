#include "platform_win32.hpp"

#include <windowsx.h>

#include <string>

namespace {
PlatformWin32* g_platform = nullptr;

std::wstring utf8_to_wide(const char* utf8) {
    if (!utf8 || !utf8[0]) {
        return L"VERTICAL SHIFT";
    }
    const int n = MultiByteToWideChar(CP_UTF8, 0, utf8, -1, nullptr, 0);
    if (n <= 0) {
        return L"VERTICAL SHIFT";
    }
    std::wstring out(static_cast<size_t>(n - 1), L'\0');
    MultiByteToWideChar(CP_UTF8, 0, utf8, -1, out.data(), n);
    return out;
}
} // namespace

#ifndef __ANDROID__
Platform* create_platform() {
    return new PlatformWin32();
}
#endif

bool PlatformWin32::init(const PlatformWindowDesc& desc) {
    instance_ = GetModuleHandleW(nullptr);
    g_platform = this;

    QueryPerformanceFrequency(&qpc_freq_);
    QueryPerformanceCounter(&qpc_start_);

    WNDCLASSEXW wc{};
    wc.cbSize = sizeof(wc);
    wc.style = CS_HREDRAW | CS_VREDRAW;
    wc.lpfnWndProc = &PlatformWin32::wnd_proc;
    wc.hInstance = instance_;
    wc.hCursor = LoadCursorW(nullptr, IDC_ARROW);
    wc.lpszClassName = L"VerticalShiftWindow";
    if (!RegisterClassExW(&wc)) {
        // May already be registered on restart.
        if (GetLastError() != ERROR_CLASS_ALREADY_EXISTS) {
            return false;
        }
    }

    RECT rect{0, 0, desc.width, desc.height};
    AdjustWindowRect(&rect, WS_OVERLAPPEDWINDOW, FALSE);
    width_ = desc.width;
    height_ = desc.height;

    const std::wstring title = utf8_to_wide(desc.title_utf8);

    hwnd_ = CreateWindowExW(
        0,
        wc.lpszClassName,
        title.c_str(),
        WS_OVERLAPPEDWINDOW,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        rect.right - rect.left,
        rect.bottom - rect.top,
        nullptr,
        nullptr,
        instance_,
        this);

    if (!hwnd_) {
        return false;
    }

    ShowWindow(hwnd_, SW_SHOW);
    UpdateWindow(hwnd_);
    return true;
}

void PlatformWin32::shutdown() {
    if (hwnd_) {
        DestroyWindow(hwnd_);
    }
    hwnd_ = nullptr;
    g_platform = nullptr;
}

bool PlatformWin32::poll_event(PlatformEvent& out) {
    MSG msg{};
    while (PeekMessageW(&msg, nullptr, 0, 0, PM_REMOVE)) {
        if (msg.message == WM_QUIT) {
            should_quit_ = true;
            PlatformEvent quit{};
            quit.kind = PlatformEvent::Kind::Quit;
            events_.push(quit);
        } else {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
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

void* PlatformWin32::native_window_handle() const {
    return hwnd_;
}

void* PlatformWin32::native_module_handle() const {
    return instance_;
}

int PlatformWin32::width() const {
    return width_;
}

int PlatformWin32::height() const {
    return height_;
}

bool PlatformWin32::focused() const {
    return focused_;
}

double PlatformWin32::monotonic_seconds() const {
    LARGE_INTEGER now{};
    QueryPerformanceCounter(&now);
    return static_cast<double>(now.QuadPart - qpc_start_.QuadPart) /
           static_cast<double>(qpc_freq_.QuadPart);
}

void PlatformWin32::push_event(PlatformEvent event) {
    events_.push(event);
}

LRESULT CALLBACK PlatformWin32::wnd_proc(HWND hwnd, UINT msg, WPARAM wparam, LPARAM lparam) {
    PlatformWin32* self = g_platform;
    if (msg == WM_NCCREATE) {
        auto* cs = reinterpret_cast<CREATESTRUCTW*>(lparam);
        self = static_cast<PlatformWin32*>(cs->lpCreateParams);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, reinterpret_cast<LONG_PTR>(self));
        g_platform = self;
    } else {
        self = reinterpret_cast<PlatformWin32*>(GetWindowLongPtrW(hwnd, GWLP_USERDATA));
    }

    if (!self) {
        return DefWindowProcW(hwnd, msg, wparam, lparam);
    }

    switch (msg) {
    case WM_SIZE: {
        self->width_ = LOWORD(lparam);
        self->height_ = HIWORD(lparam);
        PlatformEvent e{};
        e.kind = PlatformEvent::Kind::Resize;
        e.width = self->width_;
        e.height = self->height_;
        self->push_event(e);
        return 0;
    }
    case WM_SETFOCUS: {
        self->focused_ = true;
        PlatformEvent e{};
        e.kind = PlatformEvent::Kind::FocusGained;
        self->push_event(e);
        return 0;
    }
    case WM_KILLFOCUS: {
        self->focused_ = false;
        PlatformEvent e{};
        e.kind = PlatformEvent::Kind::FocusLost;
        self->push_event(e);
        return 0;
    }
    case WM_KEYDOWN: {
        PlatformEvent e{};
        e.kind = PlatformEvent::Kind::KeyDown;
        e.key = static_cast<int>(wparam);
        self->push_event(e);
        return 0;
    }
    case WM_KEYUP: {
        PlatformEvent e{};
        e.kind = PlatformEvent::Kind::KeyUp;
        e.key = static_cast<int>(wparam);
        self->push_event(e);
        return 0;
    }
    case WM_LBUTTONDOWN: {
        SetCapture(hwnd);
        PlatformEvent e{};
        e.kind = PlatformEvent::Kind::TouchDown;
        e.touch_id = 0;
        e.x = static_cast<float>(GET_X_LPARAM(lparam));
        e.y = static_cast<float>(GET_Y_LPARAM(lparam));
        self->push_event(e);
        return 0;
    }
    case WM_MOUSEMOVE: {
        if (wparam & MK_LBUTTON) {
            PlatformEvent e{};
            e.kind = PlatformEvent::Kind::TouchMove;
            e.touch_id = 0;
            e.x = static_cast<float>(GET_X_LPARAM(lparam));
            e.y = static_cast<float>(GET_Y_LPARAM(lparam));
            self->push_event(e);
        }
        return 0;
    }
    case WM_LBUTTONUP: {
        ReleaseCapture();
        PlatformEvent e{};
        e.kind = PlatformEvent::Kind::TouchUp;
        e.touch_id = 0;
        e.x = static_cast<float>(GET_X_LPARAM(lparam));
        e.y = static_cast<float>(GET_Y_LPARAM(lparam));
        self->push_event(e);
        return 0;
    }
    case WM_CLOSE:
        PostQuitMessage(0);
        return 0;
    case WM_DESTROY:
        return 0;
    default:
        break;
    }

    return DefWindowProcW(hwnd, msg, wparam, lparam);
}
