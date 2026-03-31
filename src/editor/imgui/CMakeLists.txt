cmake_minimum_required(VERSION 3.19)
project(LuandaEditorImGui LANGUAGES CXX OBJC OBJCXX)

if (WIN32)
  message(FATAL_ERROR "WIN32 editor not supported yet.")
elseif(APPLE)
  set(PLATFORM_SPECIFIC_SRC osx_main.mm)
  set(PLATFORM_SPECIFIC_LIBS
    "$<LINK_LIBRARY:FRAMEWORK,Foundation>"
    "$<LINK_LIBRARY:FRAMEWORK,Cocoa>"
    "$<LINK_LIBRARY:FRAMEWORK,Metal>"
    "$<LINK_LIBRARY:FRAMEWORK,MetalKit>"
    "$<LINK_LIBRARY:FRAMEWORK,QuartzCore>"
    "$<LINK_LIBRARY:FRAMEWORK,GameController>"
  )
else()
  message(FATAL_ERROR "Platform not supported yet.")
endif()

add_executable(LuandaEditor
  MACOSX_BUNDLE WIN32
  ${PLATFORM_SPECIFIC_SRC}
)

if (APPLE)
set(CMAKE_OSX_DEPLOYMENT_TARGET 15.0)
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
  LuandaEngine
  ${PLATFORM_SPECIFIC_LIBS}
)
