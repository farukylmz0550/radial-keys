cmake_minimum_required(VERSION 3.21)
project(radial-keys-frontend VERSION 0.1.0 LANGUAGES CXX)

find_package(Qt6 6.5 REQUIRED COMPONENTS Core Gui Qml Quick QuickControls2 DBus)

qt_standard_project_setup(REQUIRES 6.5)

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

target_link_libraries(radial-keys-frontend PRIVATE
    Qt6::Core
    Qt6::Gui
    Qt6::Qml
    Qt6::Quick
    Qt6::QuickControls2
    Qt6::DBus
)

install(TARGETS radial-keys-frontend RUNTIME DESTINATION bin)
