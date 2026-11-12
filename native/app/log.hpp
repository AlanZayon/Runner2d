#pragma once

#include <cstdarg>
#include <cstdio>

#ifdef __ANDROID__
#include <android/log.h>
#endif

#ifndef VS_APP_VERSION
#define VS_APP_VERSION "dev"
#endif

inline void vs_log(const char* fmt, ...) {
#ifdef __ANDROID__
    va_list args;
    va_start(args, fmt);
    __android_log_vprint(ANDROID_LOG_INFO, "VerticalShift", fmt, args);
    va_end(args);
#else
    std::fputs("[vs] ", stdout);
    va_list args;
    va_start(args, fmt);
    std::vfprintf(stdout, fmt, args);
    va_end(args);
    std::fputc('\n', stdout);
    std::fflush(stdout);
#endif
}
