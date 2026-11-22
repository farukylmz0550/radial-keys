import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Dialogs
import QtQuick.Window
import "menu"

ApplicationWindow {
    id: settings
    width: 760
    height: 760
    minimumWidth: 620
    minimumHeight: 620
    visible: true
    title: qsTr("Radial Keys")
    color: "#202124"
    palette.window: "#202124"
    palette.windowText: "#ffffff"
    palette.base: "#25262b"
    palette.alternateBase: "#2c2d33"
    palette.text: "#ffffff"
    palette.button: "#32343a"
    palette.buttonText: "#ffffff"
    palette.brightText: "#ffffff"
    palette.highlight: "#005a9e"
    palette.highlightedText: "#ffffff"
    palette.placeholderText: "#ffffff"
    palette.mid: "#ffffff"
    palette.midlight: "#474950"
    palette.dark: "#18191d"

    palette.inactive.window: "#202124"
    palette.inactive.base: "#25262b"
    palette.inactive.button: "#32343a"
    palette.inactive.text: "#ffffff"
    palette.inactive.windowText: "#ffffff"
    palette.inactive.buttonText: "#ffffff"
    palette.inactive.placeholderText: "#ffffff"
    palette.inactive.highlightedText: "#ffffff"

    palette.disabled.window: "#202124"
    palette.disabled.base: "#25262b"
    palette.disabled.button: "#32343a"
    palette.disabled.text: "#ffffff"
    palette.disabled.windowText: "#ffffff"
    palette.disabled.buttonText: "#ffffff"
    palette.disabled.placeholderText: "#ffffff"
    palette.disabled.highlightedText: "#ffffff"

    function directionName(id) {
        switch (id) {
        case "north": return qsTr("North")
        case "north-east": return qsTr("North-east")
        case "east": return qsTr("East")
        case "south-east": return qsTr("South-east")
        case "south": return qsTr("South")
        case "south-west": return qsTr("South-west")
        case "west": return qsTr("West")
        case "north-west": return qsTr("North-west")
        default: return qsTr("Direction")
        }
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        ColumnLayout {
            Layout.fillWidth: true
            Layout.margins: 24
            spacing: 6

            Label {
                text: qsTr("Radial Keys")
                font.pixelSize: 26
                font.weight: Font.DemiBold
            }
            Label {
                Layout.fillWidth: true
                wrapMode: Text.WordWrap
                color: "#ffffff"
                text: qsTr("Open a radial menu and run your own Python macros.")
            }
        }

        TabBar {
            id: tabs
            Layout.fillWidth: true
            Layout.leftMargin: 24
            Layout.rightMargin: 24

            TabButton { text: qsTr("General") }
            TabButton { text: qsTr("Actions") }
        }

        StackLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            currentIndex: tabs.currentIndex

            ScrollView {
                clip: true
                contentWidth: availableWidth

                ColumnLayout {
                    width: parent.width
                    anchors.margins: 24
                    spacing: 24

                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 12

                        Label {
                            text: qsTr("Activation input")
                            font.pixelSize: 20
                            font.weight: Font.DemiBold
                        }
                        ComboBox {
                            Layout.fillWidth: true
                            Layout.maximumWidth: 360
                            model: [qsTr("Keyboard shortcut"), qsTr("Mouse button")]
                            currentIndex: backend.activationType === "mouse" ? 1 : 0
                            onActivated: backend.setActivationDevice(currentIndex === 0 ? "keyboard" : "mouse", backend.mouseButton.length > 0 ? backend.mouseButton : "back")
                        }
                        RowLayout {
                            Layout.fillWidth: true
                            visible: backend.activationType === "mouse"
                            Label {
                                Layout.fillWidth: true
                                text: qsTr("Activation button")
                            }
                            ComboBox {
                                id: mouseButtonPicker
                                Layout.preferredWidth: 240
                                model: [qsTr("Back / side button"), qsTr("Forward / extra button"), qsTr("Middle button"), qsTr("Left button"), qsTr("Right button")]
                                currentIndex: Math.max(0, ["back", "forward", "middle", "left", "right"].indexOf(backend.mouseButton))
                                onActivated: backend.setActivationDevice("mouse", ["back", "forward", "middle", "left", "right"][currentIndex])
                            }
                        }
                        Label {
                            Layout.fillWidth: true
                            visible: backend.activationType === "mouse"
                            wrapMode: Text.WordWrap
                            color: "#ffffff"
                            text: qsTr("When enabled, Radial Keys reserves this button on the first detected mouse. Other mouse movement, buttons, and scrolling continue to work.")
                        }
                        RowLayout {
                            Layout.fillWidth: true
                            visible: backend.activationType === "mouse"
                            Label {
                                Layout.fillWidth: true
                                wrapMode: Text.WordWrap
                                text: backend.mouseHelperStatus
                                color: "#ffffff"
                            }
                            Button {
                                text: backend.activationEnabled ? qsTr("Disable") : qsTr("Enable mouse button")
                                enabled: backend.serviceAvailable
                                onClicked: backend.activationEnabled ? backend.disableGlobalActivation() : backend.enableGlobalActivation()
                            }
                        }
                        Label {
                            Layout.fillWidth: true
                            visible: backend.activationType === "mouse" && !backend.activationEnabled
                            wrapMode: Text.WordWrap
                            color: "#ffffff"
                            text: qsTr("The package grants session access to mouse devices and the virtual mouse relay. No keyboard input is read.")
                        }
                    }

                    Rectangle {
                        Layout.fillWidth: true
                        height: 1
                        color: "#474950"
                        opacity: 0.35
                    }

                    ColumnLayout {
                        Layout.fillWidth: true
                        visible: backend.activationType === "keyboard"
                        spacing: 12

                        Label {
                            text: qsTr("Keyboard shortcut")
                            font.pixelSize: 20
                            font.weight: Font.DemiBold
                        }
                        Label {
                            Layout.fillWidth: true
                            wrapMode: Text.WordWrap
                            text: qsTr("Press %1 once to open the menu and again to close it.").arg(backend.shortcut)
                        }
                        RowLayout {
                            Layout.fillWidth: true
                            Label {
                                Layout.fillWidth: true
                                text: backend.shortcutRegistered
                                      ? qsTr("Shortcut is enabled")
                                      : qsTr("Shortcut is disabled")
                                color: "#ffffff"
                            }
                            Button {
                                text: backend.activationEnabled ? qsTr("Disable") : qsTr("Enable %1").arg(backend.shortcut)
                                enabled: backend.serviceAvailable
                                onClicked: backend.activationEnabled ? backend.disableGlobalActivation() : warningDialog.open()
                            }
                        }
                        Label {
                            Layout.fillWidth: true
                            visible: !backend.activationEnabled
                            wrapMode: Text.WordWrap
                            color: "#ffffff"
                            text: qsTr("Enabling the shortcut asks your desktop to register it. You can disable it here at any time.")
                        }
                    }

                    Rectangle {
                        Layout.fillWidth: true
                        height: 1
                        color: "#474950"
                        opacity: 0.35
                    }

                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 12

                        Label {
                            text: qsTr("Menu preview")
                            font.pixelSize: 20
                            font.weight: Font.DemiBold
                        }
                        RowLayout {
                            Layout.fillWidth: true
                            Label {
                                Layout.fillWidth: true
                                text: backend.menuOpen ? qsTr("Menu is open") : qsTr("Menu is closed")
                                color: "#ffffff"
                            }
                            Button {
                                text: qsTr("Show menu")
                                enabled: backend.serviceAvailable && backend.rendererAvailable && !backend.menuOpen
                                onClicked: backend.showMenu()
                            }
                        }
                    }

                    Rectangle {
                        Layout.fillWidth: true
                        height: 1
                        color: "#474950"
                        opacity: 0.35
                    }

                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 6
                        Label {
                            text: qsTr("Service status")
                            font.pixelSize: 20
                            font.weight: Font.DemiBold
                        }
                        Label {
                            Layout.fillWidth: true
                            wrapMode: Text.WordWrap
                            text: !backend.serviceAvailable
                                  ? qsTr("The backend is not running.")
                                  : backend.rendererAvailable
                                    ? qsTr("Backend and menu renderer are connected.")
                                    : qsTr("Backend is running; the menu renderer is unavailable.")
                            color: backend.serviceAvailable && backend.rendererAvailable ? palette.text : palette.mid
                        }
                        Label {
                            Layout.fillWidth: true
                            visible: backend.errorMessage.length > 0
                            wrapMode: Text.WordWrap
                            color: settings.palette.windowText
                            font.weight: Font.Medium
                            text: backend.errorMessage
                        }
                    }
                }
            }

            ColumnLayout {
                Layout.fillWidth: true
                Layout.fillHeight: true
                Layout.leftMargin: 24
                Layout.rightMargin: 24
                Layout.topMargin: 18
                Layout.bottomMargin: 18
                spacing: 12

                Label {
                    text: qsTr("Radial actions")
                    font.pixelSize: 20
                    font.weight: Font.DemiBold
                }
                Label {
                    Layout.fillWidth: true
                    wrapMode: Text.WordWrap
                    color: palette.mid
                    text: qsTr("Choose a label and Python file for each direction. Select a segment to run its macro. Scripts run as your user account.")
                }

                ScrollView {
                    id: slotScroll
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    clip: true
                    contentWidth: availableWidth

                    ColumnLayout {
                        width: slotScroll.availableWidth
                        spacing: 0

                        Repeater {
                            id: actionRepeater
                            model: backend.menuSlots

                            delegate: ColumnLayout {
                                id: actionRow
                                required property int index
                                required property var modelData
                                property string labelText: modelData.label
                                property string scriptText: modelData.action
                                property string iconText: modelData.icon || ""
                                Layout.fillWidth: true
                                spacing: 8

                                RowLayout {
                                    Layout.fillWidth: true
                                    Label {
                                        Layout.preferredWidth: 150
                                        text: qsTr("%1 · Slot %2").arg(settings.directionName(actionRow.modelData.id)).arg(actionRow.index + 1)
                                        font.weight: Font.DemiBold
                                    }
                                    Image {
                                        Layout.preferredWidth: 24
                                        Layout.preferredHeight: 24
                                        source: actionRow.iconText.length > 0 ? backend.fileUrl(actionRow.iconText) : ""
                                        sourceSize.width: 48
                                        sourceSize.height: 48
                                        fillMode: Image.PreserveAspectFit
                                        visible: actionRow.iconText.length > 0
                                        Accessible.name: qsTr("Icon for slot %1").arg(actionRow.index + 1)
                                    }
                                    TextField {
                                        id: labelField
                                        Layout.fillWidth: true
                                        text: actionRow.labelText
                                        placeholderText: qsTr("Button label")
                                        color: settings.palette.text
                                        placeholderTextColor: settings.palette.placeholderText
                                        maximumLength: 32
                                        Accessible.name: qsTr("Label for slot %1").arg(actionRow.index + 1)
                                        onTextEdited: actionRow.labelText = text
                                    }
                                    Button {
                                        text: qsTr("Save")
                                        enabled: backend.serviceAvailable && actionRow.labelText.trim().length > 0
                                        onClicked: backend.setSlot(actionRow.index, actionRow.labelText, actionRow.scriptText, actionRow.iconText)
                                    }
                                }

                                RowLayout {
                                    Layout.fillWidth: true
                                    TextField {
                                        id: macroPath
                                        Layout.fillWidth: true
                                        text: actionRow.scriptText
                                        placeholderText: qsTr("Python file (.py)")
                                        color: settings.palette.text
                                        placeholderTextColor: settings.palette.placeholderText
                                        Accessible.name: qsTr("Python macro file for slot %1").arg(actionRow.index + 1)
                                        onTextEdited: actionRow.scriptText = text
                                    }
                                    Button {
                                        text: qsTr("Browse…")
                                        onClicked: {
                                            macroPicker.targetRow = actionRow
                                            macroPicker.open()
                                        }
                                    }
                                    ToolButton {
                                        text: qsTr("Clear")
                                        enabled: actionRow.scriptText.length > 0
                                        onClicked: {
                                            actionRow.scriptText = ""
                                            backend.setSlot(actionRow.index, actionRow.labelText, "", actionRow.iconText)
                                        }
                                    }
                                }

                                RowLayout {
                                    Layout.fillWidth: true
                                    TextField {
                                        id: iconPath
                                        Layout.fillWidth: true
                                        text: actionRow.iconText
                                        placeholderText: qsTr("Icon file (PNG, JPEG, or SVG)")
                                        color: settings.palette.text
                                        placeholderTextColor: settings.palette.placeholderText
                                        Accessible.name: qsTr("Icon file for slot %1").arg(actionRow.index + 1)
                                        onTextEdited: actionRow.iconText = text
                                    }
                                    Button {
                                        text: qsTr("Choose icon…")
                                        onClicked: {
                                            iconPicker.targetRow = actionRow
                                            iconPicker.open()
                                        }
                                    }
                                    ToolButton {
                                        text: qsTr("Clear icon")
                                        enabled: actionRow.iconText.length > 0
                                        onClicked: {
                                            actionRow.iconText = ""
                                            backend.setSlot(actionRow.index, actionRow.labelText, actionRow.scriptText, "")
                                        }
                                    }
                                }

                                Rectangle {
                                    Layout.fillWidth: true
                                    Layout.topMargin: 12
                                    Layout.bottomMargin: 12
                                    height: 1
                                    color: "#474950"
                                    opacity: 0.3
                                }
                            }
                        }
                    }
                }

                Label {
                    Layout.fillWidth: true
                    visible: backend.errorMessage.length > 0
                    wrapMode: Text.WordWrap
                    color: settings.palette.windowText
                    font.weight: Font.Medium
                    text: backend.errorMessage
                }
            }
        }
    }

    Dialog {
        id: warningDialog
        modal: true
        palette: settings.palette
        title: qsTr("Enable global shortcut?")
        standardButtons: Dialog.Ok | Dialog.Cancel
        width: Math.min(settings.width - 32, 400)
        height: 180
        x: (settings.width - width) / 2
        y: (settings.height - height) / 2
        onAccepted: backend.enableGlobalActivation()

        contentItem: Label {
            wrapMode: Text.WordWrap
            color: settings.palette.windowText
            text: qsTr("The desktop will register %1 for Radial Keys. You can disable the shortcut here later.").arg(backend.shortcut)
        }
    }

    FileDialog {
        id: macroPicker
        property var targetRow
        title: qsTr("Choose a Python macro")
        fileMode: FileDialog.OpenFile
        nameFilters: [qsTr("Python scripts (*.py)")]
        onAccepted: {
            if (targetRow)
                targetRow.scriptText = backend.localPath(selectedFile.toString())
        }
    }

    FileDialog {
        id: iconPicker
        property var targetRow
        title: qsTr("Choose an icon")
        fileMode: FileDialog.OpenFile
        nameFilters: [qsTr("Images (*.png *.jpg *.jpeg *.svg)")]
        onAccepted: {
            if (targetRow)
                targetRow.iconText = backend.localPath(selectedFile.toString())
        }
    }

    RadialMenu {
        id: radialMenu
        menuVisible: backend.menuOpen
        cursorPosition: backend.cursorPosition
        onCancelRequested: backend.cancelMenu()
    }
}
