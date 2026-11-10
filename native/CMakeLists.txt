# Build Rust runtime-bridge staticlib, then the C++ host.

if(ANDROID)
    include(${CMAKE_CURRENT_LIST_DIR}/CMakeLists.android.inc.cmake)
    return()
endif()

set(VS_RUST_TARGET_DIR "${CMAKE_BINARY_DIR}/rust-target")
set(VS_CARGO_MANIFEST "${CMAKE_SOURCE_DIR}/crates/runtime-bridge/Cargo.toml")

if(CMAKE_BUILD_TYPE STREQUAL "Debug" OR NOT CMAKE_BUILD_TYPE)
    set(VS_CARGO_PROFILE "dev")
    set(VS_CARGO_PROFILE_DIR "debug")
    set(VS_CARGO_FLAGS "")
else()
    set(VS_CARGO_PROFILE "release")
    set(VS_CARGO_PROFILE_DIR "release")
    set(VS_CARGO_FLAGS "--release")
endif()

set(VS_RUST_LIB "${VS_RUST_TARGET_DIR}/${VS_CARGO_PROFILE_DIR}/runtime_bridge.lib")

# Cargo only rebuilds when .rs inputs change; CMake must list them or the host
# keeps linking a stale runtime_bridge.lib after gameplay edits.
file(GLOB_RECURSE VS_RUST_SOURCES CONFIGURE_DEPENDS
    "${CMAKE_SOURCE_DIR}/crates/runtime-bridge/src/*.rs"
    "${CMAKE_SOURCE_DIR}/crates/runtime-core/src/*.rs"
)

add_custom_command(
    OUTPUT "${VS_RUST_LIB}"
    COMMAND ${CMAKE_COMMAND} -E env
            CARGO_TARGET_DIR=${VS_RUST_TARGET_DIR}
            cargo build --manifest-path "${VS_CARGO_MANIFEST}" ${VS_CARGO_FLAGS}
    WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}"
    DEPENDS
        ${VS_RUST_SOURCES}
        "${CMAKE_SOURCE_DIR}/crates/runtime-bridge/Cargo.toml"
        "${CMAKE_SOURCE_DIR}/crates/runtime-core/Cargo.toml"
        "${CMAKE_SOURCE_DIR}/Cargo.toml"
    COMMENT "Building Rust runtime-bridge (${VS_CARGO_PROFILE})"
    VERBATIM
)

add_custom_target(runtime_bridge_rust DEPENDS "${VS_RUST_LIB}")

add_library(runtime_bridge_imported STATIC IMPORTED GLOBAL)
set_target_properties(runtime_bridge_imported PROPERTIES
    IMPORTED_LOCATION "${VS_RUST_LIB}"
)
add_dependencies(runtime_bridge_imported runtime_bridge_rust)

# --- Shaders (glslc) ---
if(NOT DEFINED ENV{VULKAN_SDK} OR "$ENV{VULKAN_SDK}" STREQUAL "")
    set(_VS_VULKAN_SDK "C:/VulkanSDK/1.4.363.0")
else()
    set(_VS_VULKAN_SDK "$ENV{VULKAN_SDK}")
endif()

find_program(GLSLC glslc
    HINTS
        "${_VS_VULKAN_SDK}/Bin"
        "$ENV{VULKAN_SDK}/Bin"
)
if(NOT GLSLC)
    message(FATAL_ERROR "glslc not found. Install Vulkan SDK and ensure VULKAN_SDK is set.")
endif()
message(STATUS "Using glslc: ${GLSLC}")

set(VS_SHADER_SRC_DIR "${CMAKE_SOURCE_DIR}/shaders")
set(VS_SHADER_OUT_DIR "${CMAKE_BINARY_DIR}/shaders")
file(MAKE_DIRECTORY "${VS_SHADER_OUT_DIR}")

set(VS_SHADER_SPV
    "${VS_SHADER_OUT_DIR}/sprite.vert.spv"
    "${VS_SHADER_OUT_DIR}/sprite.frag.spv"
)

add_custom_command(
    OUTPUT "${VS_SHADER_OUT_DIR}/sprite.vert.spv"
    COMMAND "${GLSLC}" -fshader-stage=vert "${VS_SHADER_SRC_DIR}/sprite.vert" -o "${VS_SHADER_OUT_DIR}/sprite.vert.spv"
    DEPENDS "${VS_SHADER_SRC_DIR}/sprite.vert"
    COMMENT "Compiling sprite.vert"
    VERBATIM
)
add_custom_command(
    OUTPUT "${VS_SHADER_OUT_DIR}/sprite.frag.spv"
    COMMAND "${GLSLC}" -fshader-stage=frag "${VS_SHADER_SRC_DIR}/sprite.frag" -o "${VS_SHADER_OUT_DIR}/sprite.frag.spv"
    DEPENDS "${VS_SHADER_SRC_DIR}/sprite.frag"
    COMMENT "Compiling sprite.frag"
    VERBATIM
)
add_custom_target(vs_shaders DEPENDS ${VS_SHADER_SPV})

# Copy demo assets next to the executable output root.
set(VS_ASSET_OUT_DIR "${CMAKE_BINARY_DIR}/assets")
file(MAKE_DIRECTORY "${VS_ASSET_OUT_DIR}/demo")
file(MAKE_DIRECTORY "${VS_ASSET_OUT_DIR}/audio")
file(MAKE_DIRECTORY "${VS_ASSET_OUT_DIR}/fonts")
configure_file(
    "${CMAKE_SOURCE_DIR}/assets/demo/sprite.png"
    "${VS_ASSET_OUT_DIR}/demo/sprite.png"
    COPYONLY
)
configure_file(
    "${CMAKE_SOURCE_DIR}/assets/demo/white.png"
    "${VS_ASSET_OUT_DIR}/demo/white.png"
    COPYONLY
)
file(GLOB VS_AUDIO_WAVS "${CMAKE_SOURCE_DIR}/assets/audio/*.wav")
foreach(_wav IN LISTS VS_AUDIO_WAVS)
    get_filename_component(_wav_name "${_wav}" NAME)
    configure_file("${_wav}" "${VS_ASSET_OUT_DIR}/audio/${_wav_name}" COPYONLY)
endforeach()
configure_file(
    "${CMAKE_SOURCE_DIR}/assets/fonts/ui.ttf"
    "${VS_ASSET_OUT_DIR}/fonts/ui.ttf"
    COPYONLY
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
    platform/win32/platform_win32.cpp
    platform/win32/platform_win32.hpp
    platform/platform.hpp
)
target_include_directories(vs_platform PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}
)
target_compile_definitions(vs_platform PUBLIC WIN32_LEAN_AND_MEAN NOMINMAX UNICODE _UNICODE)

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
)
target_compile_definitions(vs_renderer PUBLIC
    VK_USE_PLATFORM_WIN32_KHR
    WIN32_LEAN_AND_MEAN
    NOMINMAX
    UNICODE
    _UNICODE
)
if(VS_GPU_TIMESTAMPS)
    target_compile_definitions(vs_renderer PUBLIC VS_GPU_TIMESTAMPS)
endif()
option(VS_ENABLE_ASAN "Enable MSVC AddressSanitizer" OFF)
if(VS_ENABLE_ASAN AND MSVC)
    add_compile_options(/fsanitize=address)
    add_link_options(/fsanitize=address)
    message(STATUS "AddressSanitizer enabled (MSVC)")
endif()
add_dependencies(vs_renderer vs_shaders)

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
target_compile_definitions(vs_audio PUBLIC
    WIN32_LEAN_AND_MEAN
    NOMINMAX
)
target_link_libraries(vs_audio PUBLIC winmm ole32)

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

add_executable(vertical_shift
    app/main.cpp
    app/app_bootstrap.cpp
    app/app_bootstrap.hpp
    app/game_loop.cpp
    app/game_loop.hpp
    app/gameplay_scene.cpp
    app/gameplay_scene.hpp
    app/input_state.hpp
    app/log.hpp
    app/save_store.cpp
    app/save_store.hpp
    app/input_replay.cpp
    app/input_replay.hpp
    input/gesture_recognizer.cpp
    input/gesture_recognizer.hpp
)
target_include_directories(vertical_shift PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}
    ${CMAKE_CURRENT_SOURCE_DIR}/ffi
)
# Quoted string macros for runtime shader/asset discovery.
target_compile_definitions(vertical_shift PRIVATE
    "VS_SHADER_DIR=\"${VS_SHADER_OUT_DIR}\""
    "VS_ASSET_DIR=\"${VS_ASSET_OUT_DIR}\""
)
target_link_libraries(vertical_shift PRIVATE
    vs_platform
    vs_renderer
    vs_math
    vs_audio
    vs_ui
    runtime_bridge_imported
    user32
    gdi32
    winmm
    shell32
    ole32
    ws2_32
    userenv
    ntdll
    bcrypt
    advapi32
    psapi
)
add_dependencies(vertical_shift runtime_bridge_rust vs_shaders)
