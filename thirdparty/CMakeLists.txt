cmake_minimum_required(VERSION 3.19)
project(ThirdPartyLibs LANGUAGES CXX OBJC OBJCXX)

if (${LUANDA_FRONTEND_LIB} STREQUAL "ImGUI")
  if (WIN32)
    set(BACKEND_SOURCES
      imgui/backends/imgui_impl_dx12.cpp
      imgui/backends/imgui_impl_win32.cpp
    )
    set(BACKEND_HEADERS
      imgui/backends/imgui_impl_dx12.h
      imgui/backends/imgui_impl_win32.h
    )
  elseif (APPLE)
    set(BACKEND_SOURCES
      ${CMAKE_CURRENT_SOURCE_DIR}/imgui/backends/imgui_impl_metal.mm
      ${CMAKE_CURRENT_SOURCE_DIR}/imgui/backends/imgui_impl_osx.mm
    )
    set(BACKEND_HEADERS
      ${CMAKE_CURRENT_SOURCE_DIR}/imgui/backends/imgui_impl_metal.h
      ${CMAKE_CURRENT_SOURCE_DIR}/imgui/backends/imgui_impl_osx.h
    )
  else()
    message(FATAL_ERROR "Platform not yet supported")
  endif()

  add_library(
    ImGui STATIC
    ${CMAKE_CURRENT_SOURCE_DIR}/imgui/imgui.cpp
    ${CMAKE_CURRENT_SOURCE_DIR}/imgui/imgui_demo.cpp
    ${CMAKE_CURRENT_SOURCE_DIR}/imgui/imgui_draw.cpp
    ${CMAKE_CURRENT_SOURCE_DIR}/imgui/imgui_tables.cpp
    ${CMAKE_CURRENT_SOURCE_DIR}/imgui/imgui_widgets.cpp
    ${BACKEND_SOURCES}
  )
  target_include_directories(ImGui
    PUBLIC
      ${CMAKE_CURRENT_SOURCE_DIR}/imgui/
      ${CMAKE_CURRENT_SOURCE_DIR}/imgui/backends/
  )

  add_library(
    ImGuiTextEditor STATIC
    ${CMAKE_CURRENT_SOURCE_DIR}/ImGuiColorTextEdit/TextEditor.cpp
  )
  target_include_directories(ImGuiTextEditor
    PUBLIC
      ${CMAKE_CURRENT_SOURCE_DIR}/ImGuiColorTextEdit/
  )
  target_link_libraries(ImGuiTextEditor
    PRIVATE
      ImGui
  )
endif()
