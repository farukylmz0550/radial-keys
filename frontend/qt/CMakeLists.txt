cmake_minimum_required(VERSION 3.21)
project(radial-keys-frontend VERSION 2.0.0 LANGUAGES CXX)

find_package(Qt6 6.5 REQUIRED COMPONENTS Core Gui Qml Quick QuickControls2 DBus Multimedia)

qt_standard_project_setup(REQUIRES 6.5)
if(QT_KNOWN_POLICY_QTP0004)
    qt_policy(SET QTP0004 NEW)
endif()

qt_add_executable(radial-keys-frontend
    src/main.cpp
    src/BackendClient.cpp
    src/BackendClient.h
)

qt_add_qml_module(radial-keys-frontend
    URI RadialKeys.Frontend
    VERSION 1.0
    QML_FILES
        qml/Main.qml
        qml/menu/RadialMenu.qml
)

qt_add_resources(radial-keys-frontend radial_keys_icons
    PREFIX "/icons"
    BASE "${CMAKE_CURRENT_SOURCE_DIR}/../../assets/icons"
    FILES
        ../../assets/icons/open-website.svg
        ../../assets/icons/launch-application.svg
        ../../assets/icons/run-command.svg
        ../../assets/icons/open-folder.svg
        ../../assets/icons/screenshot-full.svg
        ../../assets/icons/screenshot-region.svg
        ../../assets/icons/media-play-pause.svg
        ../../assets/icons/media-next.svg
        ../../assets/icons/media-previous.svg
        ../../assets/icons/media-stop.svg
        ../../assets/icons/volume-toggle.svg
        ../../assets/icons/volume-up.svg
        ../../assets/icons/volume-down.svg
        ../../assets/icons/microphone-muted.svg
        ../../assets/icons/microphone-unmuted.svg
        ../../assets/icons/lock-screen.svg
        ../../assets/icons/show-desktop.svg
)

qt_add_resources(radial-keys-frontend radial_keys_branding
    PREFIX "/branding"
    BASE "${CMAKE_CURRENT_SOURCE_DIR}/../../assets"
    FILES ../../assets/radial-keys-logo.svg
)

qt_add_resources(radial-keys-frontend radial_keys_sounds
    PREFIX "/sounds"
    BASE "${CMAKE_CURRENT_SOURCE_DIR}/../../assets/sounds"
    FILES ../../assets/sounds/selection-tick.wav
)

target_link_libraries(radial-keys-frontend PRIVATE
    Qt6::Core
    Qt6::Gui
    Qt6::Qml
    Qt6::Quick
    Qt6::QuickControls2
    Qt6::DBus
    Qt6::Multimedia
)

install(TARGETS radial-keys-frontend RUNTIME DESTINATION bin)
