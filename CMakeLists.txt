cmake_minimum_required(VERSION 3.19)

if(WIN32)
    set(Qt6_DIR_HINT "C:/Qt/6.10.2" FORCE)
endif()

project(LuandaProject LANGUAGES CXX)

list(APPEND CMAKE_MODULE_PATH "${CMAKE_SOURCE_DIR}/cmake")

add_subdirectory("src/editor")