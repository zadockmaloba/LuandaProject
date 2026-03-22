cmake_minimum_required(VERSION 3.19)
project(LuandaEditorQt LANGUAGES CXX)

if(WIN32)
    set(Qt6_DIR_HINT "C:/Qt/6.10.2" FORCE)
endif()

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
        LuandaEngine
)

if(WIN32)
    find_program(WINDEPLOYQT_EXECUTABLE
        NAMES windeployqt windeployqt.exe
        HINTS "${QT_HOST_PATH}/bin"
    )

    # Copy all runtime DLL dependencies next to the executable after each build.
    add_custom_command(TARGET LuandaEditor POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy_if_different
            $<TARGET_RUNTIME_DLLS:LuandaEditor>
            $<TARGET_FILE_DIR:LuandaEditor>
        COMMAND_EXPAND_LISTS
    )

    if(WINDEPLOYQT_EXECUTABLE)
        add_custom_command(TARGET LuandaEditor POST_BUILD
            COMMAND ${WINDEPLOYQT_EXECUTABLE}
                --no-translations
                --no-compiler-runtime
                $<TARGET_FILE:LuandaEditor>
        )
    else()
        message(WARNING "windeployqt was not found; post-build Qt deployment is disabled.")
    endif()

    install(FILES
        $<TARGET_RUNTIME_DLLS:LuandaEditor>
        DESTINATION ${CMAKE_INSTALL_BINDIR}
    )
endif()

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
