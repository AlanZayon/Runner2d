#pragma once

#include <cstdarg>
#include <cstdio>

inline void vs_log(const char* fmt, ...) {
    std::fputs("[vs] ", stdout);
    va_list args;
    va_start(args, fmt);
    std::vfprintf(stdout, fmt, args);
    va_end(args);
    std::fputc('\n', stdout);
    std::fflush(stdout);
}
