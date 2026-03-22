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

add_executable(LuandaEditor LuandaEditor.swift)

# Add Swift compiler flags
target_compile_options(LuandaEditor PRIVATE
    $<$<COMPILE_LANGUAGE:Swift>:-parse-as-library>
)
