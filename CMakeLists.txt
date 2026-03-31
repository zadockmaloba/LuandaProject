cmake_minimum_required(VERSION 3.19)

project(LuandaProject LANGUAGES CXX)

list(APPEND CMAKE_MODULE_PATH "${CMAKE_SOURCE_DIR}/cmake")

set(SUPPORTED_GUI_LIBS Qt SwiftUI ImGUI)
set(LUANDA_FRONTEND_LIB ImGUI CACHE STRING "GUI library to use for the frontend")
set_property(CACHE LUANDA_FRONTEND_LIB PROPERTY STRINGS ${SUPPORTED_GUI_LIBS})

add_subdirectory("src/engine")
add_subdirectory("src/editor")
add_subdirectory("thirdparty")