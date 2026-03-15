cmake_minimum_required(VERSION 3.19)
project(LuandaProject LANGUAGES CXX)

find_package(Qt6 6.5 REQUIRED COMPONENTS Core Widgets)

qt_standard_project_setup()

add_subdirectory("src/editor")