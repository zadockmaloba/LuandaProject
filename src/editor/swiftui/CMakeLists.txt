cmake_minimum_required(VERSION 3.26...4.2)
project(LuandaEditorSwift LANGUAGES CXX Swift)

# Verify that we have a new enough compiler
if("${CMAKE_Swift_COMPILER_VERSION}" VERSION_LESS 5.9)
  message(FATAL_ERROR "Bidirectional C++ Interop requires Swift 5.9 or greater. Have ${CMAKE_Swift_COMPILER_VERSION}")
endif()

if(NOT "${CMAKE_CXX_COMPILER_ID}" STREQUAL "Clang" AND
   NOT "${CMAKE_CXX_COMPILER_ID}" STREQUAL "AppleClang")
  message(FATAL_ERROR "Project requires building with Clang.
  Have ${CMAKE_CXX_COMPILER_ID}")
endif()

list(APPEND CMAKE_MODULE_PATH "${CMAKE_SOURCE_DIR}/cmake/modules")

set(CMAKE_OSX_DEPLOYMENT_TARGET 14.0)
set(BUNDLE_IDENTIFIER "com.luandaproject.luanda-editor")
set(BUNDLE_NAME "Luanda")
# set(ICON_FILE "icon.icns")

# Set up swiftrt.o and runtime library search paths
include(InitializeSwift)
# cmake/modules/AddSwift.cmake provides the function for creating the Swift to
# C++ bridging header
include(AddSwift)

add_executable(LuandaEditor MACOSX_BUNDLE
    LuandaEditor.swift
    MetalView.swift
)

include_directories(
    ${CMAKE_CURRENT_SOURCE_DIR}/include
)

# Expose the Objective-C bridging header so Swift sources can call C functions
set_property(TARGET LuandaEditor PROPERTY SWIFT_OBJC_BRIDGING_HEADER
    "${CMAKE_CURRENT_SOURCE_DIR}/include/LuandaBridge.h")

set_target_properties(LuandaEditor PROPERTIES
    MACOSX_BUNDLE TRUE
    MACOSX_BUNDLE_BUNDLE_NAME "${BUNDLE_NAME}"
    MACOSX_BUNDLE_BUNDLE_VERSION "1.0.0"
    MACOSX_BUNDLE_SHORT_VERSION_STRING "1.0"
    MACOSX_BUNDLE_GUI_IDENTIFIER "${BUNDLE_IDENTIFIER}"
    MACOSX_BUNDLE_INFO_PLIST "${CMAKE_SOURCE_DIR}/cmake/Info.plist.in"
)

# Add Swift compiler flags
target_compile_options(LuandaEditor PRIVATE
    $<$<COMPILE_LANGUAGE:Swift>:-parse-as-library>
    $<$<COMPILE_LANGUAGE:Swift>:-Xlinker>
    $<$<COMPILE_LANGUAGE:Swift>:-debug_dylib>
)

# Link the static library
target_link_libraries(LuandaEditor LuandaEngine
)

add_dependencies(LuandaEditor LuandaEngine)