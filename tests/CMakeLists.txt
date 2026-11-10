add_executable(abi_smoke
    abi_smoke.cpp
)
target_include_directories(abi_smoke PRIVATE
    ${CMAKE_SOURCE_DIR}/native/ffi
)
target_link_libraries(abi_smoke PRIVATE
    runtime_bridge_imported
    ws2_32
    userenv
    ntdll
    bcrypt
    advapi32
)
add_dependencies(abi_smoke runtime_bridge_rust)
add_test(NAME abi_smoke COMMAND abi_smoke)

add_executable(camera_math_test
    camera_math_test.cpp
)
target_link_libraries(camera_math_test PRIVATE vs_math)
add_test(NAME camera_math_test COMMAND camera_math_test)
