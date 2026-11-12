# Included from native/CMakeLists.txt when ANDROID is set.

set(VS_ANDROID_GLUE_DIR "${ANDROID_NDK}/sources/android/native_app_glue")
add_library(android_native_app_glue STATIC
    "${VS_ANDROID_GLUE_DIR}/android_native_app_glue.c"
)
target_include_directories(android_native_app_glue PUBLIC "${VS_ANDROID_GLUE_DIR}")
target_link_libraries(android_native_app_glue PUBLIC android log)

# Prebuilt Rust staticlib produced by scripts/build_android.ps1 / cargo-ndk.
set(VS_RUST_ANDROID_LIB "${CMAKE_SOURCE_DIR}/build/android-rust/aarch64-linux-android/release/libruntime_bridge.a")
if(NOT EXISTS "${VS_RUST_ANDROID_LIB}")
    message(WARNING "Missing ${VS_RUST_ANDROID_LIB} — run scripts/build_android.ps1 rust step first")
endif()

add_library(runtime_bridge_imported STATIC IMPORTED GLOBAL)
set_target_properties(runtime_bridge_imported PROPERTIES
    IMPORTED_LOCATION "${VS_RUST_ANDROID_LIB}"
)

add_library(vs_math STATIC
    render/camera2d.cpp
    render/camera2d.hpp
    math/vec2.hpp
    math/mat3.hpp
    math/rect.hpp
)
target_include_directories(vs_math PUBLIC ${CMAKE_CURRENT_SOURCE_DIR})

add_library(vs_platform STATIC
    platform/android/platform_android.cpp
    platform/android/platform_android.hpp
    platform/platform.hpp
)
target_include_directories(vs_platform PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}
    ${VS_ANDROID_GLUE_DIR}
)
target_link_libraries(vs_platform PUBLIC android_native_app_glue android log)

add_library(vs_renderer STATIC
    renderer/vulkan/renderer_vulkan.cpp
    renderer/vulkan/renderer_vulkan.hpp
    renderer/renderer.hpp
    assets/texture_loader.cpp
    assets/texture_loader.hpp
    assets/asset_fs.cpp
    assets/asset_fs.hpp
)
target_include_directories(vs_renderer PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}
    ${CMAKE_SOURCE_DIR}
)
target_link_libraries(vs_renderer PUBLIC
    volk
    Vulkan::Headers
    vs_math
    android
    log
)
target_compile_definitions(vs_renderer PUBLIC VK_USE_PLATFORM_ANDROID_KHR)

add_library(vs_audio STATIC
    audio/audio_device.cpp
    audio/audio_device.hpp
    audio/audio_mixer.cpp
    audio/audio_mixer.hpp
    audio/log_audio.hpp
)
target_include_directories(vs_audio PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}
    ${CMAKE_SOURCE_DIR}/third_party/miniaudio
)
target_link_libraries(vs_audio PUBLIC android log OpenSLES)

add_library(vs_ui STATIC
    ui/font_atlas.cpp
    ui/font_atlas.hpp
    ui/ui_draw.cpp
    ui/ui_draw.hpp
    ui/hud.cpp
    ui/hud.hpp
    ui/screens.cpp
    ui/screens.hpp
    ui/debug_overlay.cpp
    ui/debug_overlay.hpp
)
target_include_directories(vs_ui PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}
    ${CMAKE_CURRENT_SOURCE_DIR}/ffi
    ${CMAKE_SOURCE_DIR}
)
target_link_libraries(vs_ui PUBLIC vs_renderer)

add_library(vertical_shift SHARED
    app/android_main.cpp
    app/app_bootstrap.cpp
    app/app_bootstrap.hpp
    app/game_loop.cpp
    app/game_loop.hpp
    app/gameplay_scene.cpp
    app/gameplay_scene.hpp
    app/input_replay.cpp
    app/input_replay.hpp
    app/input_state.hpp
    app/log.hpp
    app/save_store.cpp
    app/save_store.hpp
    input/gesture_recognizer.cpp
    input/gesture_recognizer.hpp
)
target_include_directories(vertical_shift PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}
    ${CMAKE_CURRENT_SOURCE_DIR}/ffi
    ${VS_ANDROID_GLUE_DIR}
)
if(NOT DEFINED VS_APP_VERSION)
    set(VS_APP_VERSION "dev")
endif()
target_compile_definitions(vertical_shift PRIVATE
    "VS_SHADER_DIR=\"shaders\""
    "VS_ASSET_DIR=\"\""
    "VS_APP_VERSION=\"${VS_APP_VERSION}\""
)
target_link_libraries(vertical_shift PRIVATE
    vs_platform
    vs_renderer
    vs_math
    vs_audio
    vs_ui
    runtime_bridge_imported
    android_native_app_glue
    android
    log
    vulkan
)
# Export ANativeActivity_onCreate from glue.
set_property(TARGET vertical_shift APPEND_STRING PROPERTY LINK_FLAGS " -u ANativeActivity_onCreate")
