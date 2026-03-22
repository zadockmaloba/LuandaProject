cmake_minimum_required(VERSION 3.19)

project(LuandaProject LANGUAGES CXX)

list(APPEND CMAKE_MODULE_PATH "${CMAKE_SOURCE_DIR}/cmake")

add_subdirectory("src/engine")
add_subdirectory("src/editor")