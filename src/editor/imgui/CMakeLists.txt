cmake_minimum_required(VERSION 3.19)
project(LuandaEditorImGui LANGUAGES CXX OBJC OBJCXX)

add_executable(LuandaEditor
  MACOSX_BUNDLE WIN32
  osx_main.mm
)

target_link_libraries(LuandaEditor 
PRIVATE
  ImGui 
  LuandaEngine
  "$<LINK_LIBRARY:FRAMEWORK,Foundation>"
  "$<LINK_LIBRARY:FRAMEWORK,Cocoa>"
  "$<LINK_LIBRARY:FRAMEWORK,Metal>"
  "$<LINK_LIBRARY:FRAMEWORK,MetalKit>"
  "$<LINK_LIBRARY:FRAMEWORK,QuartzCore>"
  "$<LINK_LIBRARY:FRAMEWORK,GameController>"
)
