cmake_minimum_required(VERSION 3.15)
project(ms_toollib VERSION 1.5.10 LANGUAGES C)

set(CMAKE_C_STANDARD 11)

find_program(CARGO cargo REQUIRED)

set(RUST_LIB_DIR "${CMAKE_CURRENT_SOURCE_DIR}/target/release")
if(WIN32)
    set(RUST_LIB "${RUST_LIB_DIR}/ms_toollib.lib")
else()
    set(RUST_LIB "${RUST_LIB_DIR}/libms_toollib.a")
endif()

add_custom_target(rust_lib ALL
    COMMAND ${CARGO} build --release
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
    COMMENT "Building Rust static library (ms_toollib)..."
    BYPRODUCTS ${RUST_LIB}
)

add_executable(test_board
    tests/test_board.c
)
add_dependencies(test_board rust_lib)
target_include_directories(test_board PRIVATE include)
target_link_libraries(test_board PRIVATE ${RUST_LIB})
if(WIN32)
    target_link_libraries(test_board PRIVATE
        ws2_32 Advapi32 Iphlpapi Psapi user32 userenv bcrypt ntdll)
endif()

install(DIRECTORY include/ms_toollib DESTINATION include)
