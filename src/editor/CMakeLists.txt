cmake_minimum_required(VERSION 3.15)
project(LuandaEditor)

if (UNIX)
    set(CMAKE_CXX_STANDARD 17)
    set(CMAKE_CXX_STANDARD_REQUIRED ON)
    add_subdirectory("qt")
endif()