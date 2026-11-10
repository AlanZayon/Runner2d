#include "vertical_shift_abi.h"

#include <cstdio>
#include <cstring>

int main() {
    if (vs_runtime_shutdown() != VS_OK) {
        std::printf("shutdown before init should be OK\n");
        return 1;
    }
    if (vs_runtime_init() != VS_OK) {
        std::printf("init failed\n");
        return 1;
    }
    if (vs_runtime_init() != VS_ERR_ALREADY_INITIALIZED) {
        std::printf("double init should fail\n");
        return 1;
    }
    const char* version = vs_runtime_version();
    if (!version || std::strlen(version) == 0) {
        std::printf("empty version\n");
        return 1;
    }
    if (vs_abi_version_major() < 1) {
        std::printf("bad major\n");
        return 1;
    }
    if (vs_runtime_tick() != 1 || vs_runtime_tick() != 2) {
        std::printf("tick sequence failed\n");
        return 1;
    }
    if (vs_runtime_shutdown() != VS_OK) {
        std::printf("shutdown failed\n");
        return 1;
    }
    if (vs_runtime_tick() != 0) {
        std::printf("tick after shutdown should be 0\n");
        return 1;
    }
    std::printf("abi_smoke OK version=%s\n", version);
    return 0;
}
