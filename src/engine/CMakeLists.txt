# CMake configuration for Rust engine library
cmake_minimum_required(VERSION 3.15)

# Find Cargo (Rust build tool)
find_program(CARGO_EXECUTABLE cargo REQUIRED)

if(NOT CARGO_EXECUTABLE)
    message(FATAL_ERROR "Cargo not found. Please install Rust toolchain from https://rustup.rs/")
endif()

# Determine build profile based on CMake build type
if(CMAKE_BUILD_TYPE STREQUAL "Debug")
    set(CARGO_BUILD_TYPE "debug")
    set(CARGO_BUILD_FLAG "")
else()
    set(CARGO_BUILD_TYPE "release")
    set(CARGO_BUILD_FLAG "--release")
endif()

# Set Rust library output paths
set(RUST_TARGET_DIR "${CMAKE_CURRENT_SOURCE_DIR}/target")
set(RUST_LIB_DIR "${RUST_TARGET_DIR}/${CARGO_BUILD_TYPE}")

# Determine library name based on platform
if(APPLE)
    set(RUST_LIB_NAME "libluanda_engine.a")
elseif(WIN32)
    set(RUST_LIB_NAME "luanda_engine.lib")
else()
    set(RUST_LIB_NAME "libluanda_engine.a")
endif()

set(RUST_LIB_PATH "${RUST_LIB_DIR}/${RUST_LIB_NAME}")

# Create custom target to build Rust library
add_custom_target(
    LuandaEngine_build
    COMMAND ${CARGO_EXECUTABLE} build ${CARGO_BUILD_FLAG}
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
    COMMENT "Building Rust engine library (${CARGO_BUILD_TYPE})"
    BYPRODUCTS ${RUST_LIB_PATH}
)

# Create custom command to ensure library is built
add_custom_command(
    OUTPUT ${RUST_LIB_PATH}
    COMMAND ${CARGO_EXECUTABLE} build ${CARGO_BUILD_FLAG}
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
    COMMENT "Compiling Rust library"
)

# Create imported library target
add_library(LuandaEngine STATIC IMPORTED GLOBAL)
set_target_properties(LuandaEngine PROPERTIES
    IMPORTED_LOCATION ${RUST_LIB_PATH}
)
target_include_directories(LuandaEngine
    INTERFACE ${CMAKE_CURRENT_SOURCE_DIR}/include
)

# Add dependency to ensure Rust library is built
add_dependencies(LuandaEngine LuandaEngine_build)

# Set include directory (if you have a C header for FFI)
# target_include_directories(LuandaEngine INTERFACE ${CMAKE_CURRENT_SOURCE_DIR}/include)

# Add cargo clean target
add_custom_target(
    LuandaEngine_clean
    COMMAND ${CARGO_EXECUTABLE} clean
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
    COMMENT "Cleaning Rust build artifacts"
)

message(STATUS "Rust engine configuration:")
message(STATUS "  Cargo: ${CARGO_EXECUTABLE}")
message(STATUS "  Build type: ${CARGO_BUILD_TYPE}")
message(STATUS "  Library path: ${RUST_LIB_PATH}")
