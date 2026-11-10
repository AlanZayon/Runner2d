cmake_minimum_required(VERSION 3.24)
project(vertical_shift VERSION 0.1.0 LANGUAGES CXX C)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

option(VS_BUILD_TESTS "Build native smoke tests" ON)

include(FetchContent)

# Vulkan headers (no full SDK required to compile). Runtime uses system vulkan-1.dll via volk.
set(VULKAN_HEADERS_ENABLE_MODULE OFF CACHE BOOL "" FORCE)

FetchContent_Declare(
    VulkanHeaders
    GIT_REPOSITORY https://github.com/KhronosGroup/Vulkan-Headers.git
    GIT_TAG v1.3.296
    GIT_SHALLOW TRUE
)
FetchContent_MakeAvailable(VulkanHeaders)

FetchContent_Declare(
    volk
    GIT_REPOSITORY https://github.com/zeux/volk.git
    GIT_TAG 1.3.295
    GIT_SHALLOW TRUE
)
set(VOLK_PULL_IN_VULKAN OFF CACHE BOOL "" FORCE)
FetchContent_MakeAvailable(volk)
# volk was configured without find_package(Vulkan); attach Khronos headers manually.
target_link_libraries(volk PUBLIC Vulkan::Headers)
if(ANDROID)
    target_compile_definitions(volk PUBLIC VK_USE_PLATFORM_ANDROID_KHR)
else()
    target_compile_definitions(volk PUBLIC VK_USE_PLATFORM_WIN32_KHR)
endif()

add_subdirectory(native)

if(VS_BUILD_TESTS AND NOT ANDROID)
    enable_testing()
    add_subdirectory(tests)
endif()
