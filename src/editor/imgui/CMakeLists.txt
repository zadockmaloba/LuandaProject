cmake_minimum_required(VERSION 3.19)

if (__APPLE__)
set(ADDITIONAL_TARGET_LANGUAGES OBJC OBJCXX)
endif()

project(LuandaEditorImGui LANGUAGES CXX ${ADDITIONAL_TARGET_LANGUAGES})

if (WIN32)
  set(PLATFORM_SPECIFIC_SRC win32_main.cpp)
elseif(APPLE)
  set(PLATFORM_SPECIFIC_SRC osx_main.mm)
  set(PLATFORM_SPECIFIC_LIBS
    "$<LINK_LIBRARY:FRAMEWORK,Foundation>"
    "$<LINK_LIBRARY:FRAMEWORK,Cocoa>"
    "$<LINK_LIBRARY:FRAMEWORK,AppKit>"
    "$<LINK_LIBRARY:FRAMEWORK,Metal>"
    "$<LINK_LIBRARY:FRAMEWORK,MetalKit>"
    "$<LINK_LIBRARY:FRAMEWORK,QuartzCore>"
    "$<LINK_LIBRARY:FRAMEWORK,GameController>"
  )
elseif(UNIX)
  set(PLATFORM_SPECIFIC_SRC unix_main.cpp)
  set(PLATFORM_SPECIFIC_LIBS
    SDL3
    vulkan
    pthread
    xcb
  )
else()
  message(FATAL_ERROR "Platform not supported yet.")
endif()

add_executable(LuandaEditor
  MACOSX_BUNDLE WIN32
  ${PLATFORM_SPECIFIC_SRC}
  viewcontroller.hpp viewcontroller.cpp
  viewportwidget.hpp viewportwidget.cpp
)

if (APPLE)
set(CMAKE_OSX_DEPLOYMENT_TARGET 15.7)
set(BUNDLE_IDENTIFIER "com.luandaproject.luanda-editor")
set(BUNDLE_NAME "LuandaEditor")

set_target_properties(LuandaEditor PROPERTIES
    MACOSX_BUNDLE TRUE
    MACOSX_BUNDLE_BUNDLE_NAME "${BUNDLE_NAME}"
    MACOSX_BUNDLE_BUNDLE_VERSION "1.0.0"
    MACOSX_BUNDLE_SHORT_VERSION_STRING "1.0"
    MACOSX_BUNDLE_GUI_IDENTIFIER "${BUNDLE_IDENTIFIER}"
    MACOSX_BUNDLE_INFO_PLIST "${CMAKE_SOURCE_DIR}/cmake/Info.plist.in"
)
endif()

target_link_libraries(LuandaEditor
PRIVATE
  ImGui
  ImGuiTextEditor
  LuandaEngine
  ${PLATFORM_SPECIFIC_LIBS}
)

if(WIN32)
 # Copy discovered runtime DLL dependencies next to the executable.
  # add_custom_command(TARGET LuandaEditor POST_BUILD
  #   COMMAND ${CMAKE_COMMAND} -E copy_if_different
  #     $<TARGET_RUNTIME_DLLS:LuandaEditor>
  #     $<TARGET_FILE_DIR:LuandaEditor>
  #   COMMAND_EXPAND_LISTS
  # )
  # dxcompiler/dxil may be linked through import libs but still not deployed.
  set(DXCOMPILER_SEARCH_HINTS
    "$ENV{VCToolsRedistDir}/x64/Microsoft.D3D.DXC"
    "$ENV{WindowsSdkVerBinPath}/x64"
    "$ENV{WindowsSdkBinPath}/x64"
  )
  file(GLOB WINDOWS_SDK_BIN_X64 "C:/Program Files (x86)/Windows Kits/10/bin/*/x64")
  list(APPEND DXCOMPILER_SEARCH_HINTS ${WINDOWS_SDK_BIN_X64})

  find_file(DXCOMPILER_DLL NAMES dxcompiler.dll HINTS ${DXCOMPILER_SEARCH_HINTS})
  if(DXCOMPILER_DLL)
    add_custom_command(TARGET LuandaEditor POST_BUILD
      COMMAND ${CMAKE_COMMAND} -E copy_if_different
        "${DXCOMPILER_DLL}"
        $<TARGET_FILE_DIR:LuandaEditor>
    )
  else()
    message(WARNING "dxcompiler.dll was not found in known SDK/VS paths; LuandaEditor may fail to start.")
  endif()

  find_file(DXIL_DLL NAMES dxil.dll HINTS ${DXCOMPILER_SEARCH_HINTS})
  if(DXIL_DLL)
    add_custom_command(TARGET LuandaEditor POST_BUILD
      COMMAND ${CMAKE_COMMAND} -E copy_if_different
        "${DXIL_DLL}"
        $<TARGET_FILE_DIR:LuandaEditor>
    )
  else()
    message(WARNING "dxil.dll was not found in known SDK/VS paths; DXC shader compilation may fail.")
  endif()
endif()
