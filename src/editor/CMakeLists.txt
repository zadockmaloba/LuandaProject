cmake_minimum_required(VERSION 3.19)
project(LuandaEditor LANGUAGES CXX)

find_package(Qt6 6.5 REQUIRED COMPONENTS Core Widgets OpenGLWidgets)

qt_standard_project_setup()

qt_add_executable(LuandaEditor
    WIN32 MACOSX_BUNDLE
    main.cpp
    luandaeditor.cpp
    luandaeditor.h
    explorerwidget.h explorerwidget.cpp
    viewportwidget.h viewportwidget.cpp
)

target_link_libraries(LuandaEditor
    PRIVATE
        Qt::Core
        Qt::Widgets
        Qt::OpenGLWidgets
)

install(TARGETS LuandaEditor
    BUNDLE  DESTINATION .
    RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR}
    LIBRARY DESTINATION ${CMAKE_INSTALL_LIBDIR}
)

qt_generate_deploy_app_script(
    TARGET LuandaEditor
    OUTPUT_SCRIPT deploy_script
    NO_UNSUPPORTED_PLATFORM_ERROR
)
install(SCRIPT ${deploy_script})
