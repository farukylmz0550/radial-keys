import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Dialogs
import QtQuick.Window
import "menu"

ApplicationWindow {
    id: settings
    property bool settingsVisible: true
    width: 760
    height: 760
    minimumWidth: 620
    minimumHeight: 620
    visible: settingsVisible && !backend.menuOpen
    onClosing: function(close) {
        settingsVisible = false
        close.accepted = false
    }

    Connections {
        target: backend
        function onMenuOpenChanged() {
            if (backend.menuOpen)
                settings.settingsVisible = false
        }
    }

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

    readonly property var actionProfiles: [
        { name: qsTr("Media"), slots: [
            ["Play / pause", "media_play_pause", "media-play-pause.svg"], ["Previous track", "media_previous", "media-previous.svg"], ["Next track", "media_next", "media-next.svg"], ["Stop", "media_stop", "media-stop.svg"], ["Volume up", "volume_up", "volume-up.svg"], ["Volume down", "volume_down", "volume-down.svg"], ["Spotify", "open_website", "open-website.svg", ["https://open.spotify.com"]], ["YouTube", "open_website", "open-website.svg", ["https://www.youtube.com"]]
        ]},
        { name: qsTr("Productivity"), slots: [
            ["Open terminal", "launch_application", "launch-application.svg", ["konsole"]], ["Downloads", "open_folder", "open-folder.svg", ["~/Downloads"]], ["Full screenshot", "screenshot_full", "screenshot-full.svg"], ["Area screenshot", "screenshot_region", "screenshot-region.svg"], ["Show desktop", "show_desktop", "show-desktop.svg"], ["Lock screen", "lock_screen", "lock-screen.svg"], ["Netflix", "open_website", "open-website.svg", ["https://www.netflix.com"]], ["Play / pause", "media_play_pause", "media-play-pause.svg"]
        ]},
        { name: qsTr("System"), slots: [
            ["Mute", "volume_toggle", "volume-toggle.svg"], ["Volume up", "volume_up", "volume-up.svg"], ["Volume down", "volume_down", "volume-down.svg"], ["Microphone", "microphone_toggle", "microphone-unmuted.svg"], ["Screenshot", "screenshot_full", "screenshot-full.svg"], ["Show desktop", "show_desktop", "show-desktop.svg"], ["Lock screen", "lock_screen", "lock-screen.svg"], ["Netflix", "open_website", "open-website.svg", ["https://www.netflix.com"]]
        ]},
        { name: qsTr("Gaming"), slots: [
            ["Play / pause", "media_play_pause", "media-play-pause.svg"], ["Volume up", "volume_up", "volume-up.svg"], ["Volume down", "volume_down", "volume-down.svg"], ["Mute", "volume_toggle", "volume-toggle.svg"], ["Screenshot", "screenshot_full", "screenshot-full.svg"], ["Open Steam", "launch_application", "launch-application.svg", ["steam"]], ["Spotify", "open_website", "open-website.svg", ["https://open.spotify.com"]], ["YouTube", "open_website", "open-website.svg", ["https://www.youtube.com"]]
        ]}
    ]

    function applyProfile(profileIndex) {
        const slots = actionProfiles[profileIndex].slots
        for (let i = 0; i < slots.length; ++i) {
            const slot = slots[i]
            backend.setSlotPreset(i, slot[0], slot[1], slot[3] || [], "qrc:/icons/" + slot[2])
        }
        backend.setActiveProfileName(actionProfiles[profileIndex].name)
    }

    function selectProfile(profileIndex) {
        if (profileIndex < actionProfiles.length)
            applyProfile(profileIndex)
        else
            backend.activateSavedProfile(profileIndex - actionProfiles.length)
    }

    function profileIndexByName(name) {
        if (!name)
            return 0
        const names = actionProfiles.map(profile => profile.name)
        for (let i = 0; i < backend.savedProfiles.length; ++i)
            names.push(backend.savedProfiles[i].name)
        const index = names.indexOf(name)
        return index >= 0 ? index : 0
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
                            text: qsTr("Mouse activation")
                            font.pixelSize: 20
                            font.weight: Font.DemiBold
                        }
                        RowLayout {
                            Layout.fillWidth: true
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
                            wrapMode: Text.WordWrap
                            color: "#ffffff"
                            text: qsTr("When enabled, Radial Keys reserves this button on the first detected mouse. Other mouse movement, buttons, and scrolling continue to work.")
                        }
                        Label {
                            Layout.fillWidth: true
                            visible: backend.fullscreenActive
                            wrapMode: Text.WordWrap
                            color: "#f9c74f"
                            text: qsTr("Radial Keys is temporarily disabled while a full-screen window is active.")
                        }
                        RowLayout {
                            Layout.fillWidth: true
                            Label {
                                Layout.fillWidth: true
                                wrapMode: Text.WordWrap
                                text: backend.mouseHelperStatus
                                color: "#ffffff"
                            }
                            Button {
                                text: backend.activationType !== "mouse"
                                      ? qsTr("Use mouse activation")
                                      : backend.activationEnabled ? qsTr("Disable") : qsTr("Enable mouse button")
                                enabled: backend.serviceAvailable
                                onClicked: {
                                    if (backend.activationType !== "mouse")
                                        backend.setActivationDevice("mouse", backend.mouseButton.length > 0 ? backend.mouseButton : "back")
                                    else if (backend.activationEnabled)
                                        backend.disableGlobalActivation()
                                    else
                                        backend.enableGlobalActivation()
                                }
                            }
                        }
                        Label {
                            Layout.fillWidth: true
                            visible: !backend.activationEnabled
                            wrapMode: Text.WordWrap
                            color: "#ffffff"
                            text: qsTr("The selected button is reserved on the first detected mouse. Other mouse movement, buttons, and scrolling continue to work.")
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

                RowLayout {
                    Layout.fillWidth: true
                    Label { text: qsTr("Profile") }
                    ComboBox {
                        id: profilePicker
                        model: settings.actionProfiles.map(profile => profile.name)
                            .concat(backend.savedProfiles.map(profile => profile.name))
                        currentIndex: settings.profileIndexByName(backend.activeProfileName)
                        onActivated: settings.selectProfile(currentIndex)
                        Accessible.name: qsTr("Radial menu profile")
                    }
                    Button {
                        text: qsTr("Save profile")
                        enabled: backend.menuSlots.length === 8
                        onClicked: {
                            profileNameField.text = profilePicker.currentIndex >= settings.actionProfiles.length
                                ? backend.activeProfileName : qsTr("My profile")
                            profileDialog.open()
                        }
                    }
                    Label {
                        Layout.fillWidth: true
                        wrapMode: Text.WordWrap
                        color: settings.palette.mid
                        text: qsTr("Choose a ready-made layout or save the current eight actions as your own profile.")
                    }
                }

                Label {
                    text: qsTr("Radial actions")
                    font.pixelSize: 20
                    font.weight: Font.DemiBold
                }
                Label {
                    Layout.fillWidth: true
                    wrapMode: Text.WordWrap
                    color: palette.mid
                    text: qsTr("Choose a built-in macro or edit a custom Python macro here. Macros run with your user permissions.")
                }

                RowLayout {
                    Layout.fillWidth: true
                    CheckBox {
                        id: singleInstanceCheck
                        text: qsTr("Allow only one running copy of each macro")
                        checked: backend.macroSingleInstance
                        onToggled: backend.setMacroOptions(checked, timeoutCheck.checked ? timeoutSpin.value : -1)
                    }
                    Item { Layout.fillWidth: true }
                    CheckBox {
                        id: timeoutCheck
                        text: qsTr("Execution timeout")
                        checked: backend.macroTimeoutSeconds > 0
                        onToggled: backend.setMacroOptions(singleInstanceCheck.checked, checked ? timeoutSpin.value : -1)
                    }
                    SpinBox {
                        id: timeoutSpin
                        from: 1
                        to: 86400
                        editable: true
                        value: backend.macroTimeoutSeconds > 0 ? backend.macroTimeoutSeconds : 300
                        enabled: timeoutCheck.checked
                        Accessible.name: qsTr("Macro timeout in seconds")
                        onValueModified: backend.setMacroOptions(singleInstanceCheck.checked, value)
                    }
                    Label { text: qsTr("seconds") }
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
                                property string presetId: modelData.preset ? modelData.preset.id : ""
                                property var parameterValues: modelData.preset ? modelData.preset.parameters : []
                                Layout.fillWidth: true
                                spacing: 8

                                function currentPreset() {
                                    const presetIndex = backend.presetIndex(presetId)
                                    return presetIndex >= 0 ? backend.builtinMacros[presetIndex] : null
                                }

                                function actionTooltip() {
                                    if (presetId.length > 0) {
                                        const preset = currentPreset()
                                        let description = preset ? preset.description : qsTr("Run this built-in action.")
                                        if (parameterValues.length > 0)
                                            description += "\n" + parameterValues.join(" · ")
                                        return labelText + "\n" + description
                                    }
                                    return scriptText.length > 0
                                        ? labelText + "\n" + qsTr("Run this Python macro in a separate process.")
                                        : labelText + "\n" + qsTr("No action is assigned.")
                                }

                                function saveAction() {
                                    if (presetId.length > 0)
                                        backend.setSlotPreset(index, labelText, presetId, parameterValues, iconText)
                                    else
                                        backend.setSlot(index, labelText, scriptText, iconText)
                                }

                                function runningForSlot() {
                                    for (let i = 0; i < backend.runningMacros.length; ++i) {
                                        if (backend.runningMacros[i].slot === index)
                                            return backend.runningMacros[i]
                                    }
                                    return null
                                }

                                function latestFailure() {
                                    for (let i = 0; i < backend.macroResults.length; ++i) {
                                        const result = backend.macroResults[i]
                                        if (result.slot === index)
                                            return result.outcome === "completed" ? null : result
                                    }
                                    return null
                                }

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
                                        source: actionRow.presetId === "microphone_toggle"
                                            ? backend.fileUrl(backend.microphoneStateKnown && backend.microphoneMuted
                                                ? "qrc:/icons/microphone-muted.svg" : "qrc:/icons/microphone-unmuted.svg")
                                            : (actionRow.iconText.length > 0 ? backend.fileUrl(actionRow.iconText) : "")
                                        sourceSize.width: 48
                                        sourceSize.height: 48
                                        fillMode: Image.PreserveAspectFit
                                        visible: actionRow.iconText.length > 0
                                        Accessible.name: qsTr("Icon for slot %1").arg(actionRow.index + 1)
                                        HoverHandler {
                                            id: editorIconHover
                                            acceptedDevices: PointerDevice.Mouse
                                        }
                                        ToolTip.visible: editorIconHover.hovered
                                        ToolTip.delay: 700
                                        ToolTip.text: actionRow.actionTooltip()
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
                                        onClicked: actionRow.saveAction()
                                    }
                                }

                                RowLayout {
                                    Layout.fillWidth: true
                                    ComboBox {
                                        Layout.fillWidth: true
                                        model: backend.builtinMacros
                                        textRole: "label"
                                        currentIndex: backend.presetIndex(actionRow.presetId)
                                        Accessible.name: qsTr("Built-in macro for slot %1").arg(actionRow.index + 1)
                                        onActivated: function(selectedIndex) {
                                            const selected = backend.builtinMacros[selectedIndex]
                                            actionRow.presetId = selected.id
                                            actionRow.parameterValues = selected.defaults.slice(0)
                                            actionRow.scriptText = ""
                                            actionRow.iconText = "qrc:/icons/" + selected.icon
                                            actionRow.labelText = selected.label
                                        }
                                    }
                                    Button {
                                        text: actionRow.scriptText.length > 0 ? qsTr("Edit Python…") : qsTr("Custom Python…")
                                        onClicked: {
                                            pythonDialog.targetRow = actionRow
                                            pythonEditor.text = actionRow.scriptText.length > 0
                                                ? backend.readPythonSource(actionRow.scriptText) : ""
                                            pythonDialog.open()
                                        }
                                    }
                                    Button {
                                        text: qsTr("Import .py…")
                                        onClicked: {
                                            macroPicker.targetRow = actionRow
                                            macroPicker.open()
                                        }
                                    }
                                    ToolButton {
                                        text: qsTr("Clear")
                                        enabled: actionRow.scriptText.length > 0 || actionRow.presetId.length > 0
                                        onClicked: {
                                            actionRow.scriptText = ""
                                            actionRow.presetId = ""
                                            actionRow.parameterValues = []
                                            actionRow.iconText = ""
                                            backend.setSlot(actionRow.index, actionRow.labelText, "", "")
                                        }
                                    }
                                }

                                Label {
                                    Layout.fillWidth: true
                                    visible: actionRow.currentPreset() !== null
                                    wrapMode: Text.WordWrap
                                    color: settings.palette.mid
                                    text: actionRow.currentPreset() ? actionRow.currentPreset().description : ""
                                }

                                Repeater {
                                    model: actionRow.currentPreset() ? actionRow.currentPreset().parameter_labels.length : 0
                                    delegate: TextField {
                                        required property int index
                                        Layout.fillWidth: true
                                        text: actionRow.parameterValues.length > index ? actionRow.parameterValues[index] : ""
                                        placeholderText: actionRow.currentPreset().parameter_labels[index]
                                        color: settings.palette.text
                                        placeholderTextColor: settings.palette.placeholderText
                                        Accessible.name: placeholderText
                                        onTextEdited: {
                                            const updated = actionRow.parameterValues.slice(0)
                                            updated[index] = text
                                            actionRow.parameterValues = updated
                                        }
                                    }
                                }

                                RowLayout {
                                    Layout.fillWidth: true
                                    visible: actionRow.scriptText.length > 0
                                    Label {
                                        Layout.fillWidth: true
                                        text: {
                                            const state = actionRow.modelData.script_integrity || "untracked"
                                            if (state === "changed") return qsTr("Python script changed since it was accepted.")
                                            if (state === "missing") return qsTr("Python script file is missing.")
                                            if (state === "untracked") return qsTr("Python script has no integrity baseline yet.")
                                            return qsTr("Python script integrity verified.")
                                        }
                                        color: actionRow.modelData.script_integrity === "changed" || actionRow.modelData.script_integrity === "missing"
                                            ? "#ffb4ab" : settings.palette.mid
                                        wrapMode: Text.WordWrap
                                    }
                                    Button {
                                        text: qsTr("Accept current script")
                                        visible: actionRow.modelData.script_integrity === "changed" || actionRow.modelData.script_integrity === "untracked"
                                        onClicked: backend.acceptScriptUpdate(actionRow.index)
                                    }
                                }

                                RowLayout {
                                    Layout.fillWidth: true
                                    visible: actionRow.runningForSlot() !== null
                                    Label {
                                        Layout.fillWidth: true
                                        text: actionRow.runningForSlot()
                                            ? qsTr("Running: %1").arg(actionRow.runningForSlot().label) : ""
                                    }
                                    Button {
                                        text: qsTr("Stop")
                                        onClicked: backend.terminateSlotMacros(actionRow.index)
                                    }
                                }

                                Label {
                                    Layout.fillWidth: true
                                    visible: actionRow.latestFailure() !== null
                                    wrapMode: Text.WordWrap
                                    color: "#ffb4ab"
                                    text: actionRow.latestFailure()
                                        ? qsTr("Last run: %1 (exit %2) %3")
                                            .arg(actionRow.latestFailure().outcome)
                                            .arg(actionRow.latestFailure().exit_code === null ? "—" : actionRow.latestFailure().exit_code)
                                            .arg(actionRow.latestFailure().error_output) : ""
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
                                            actionRow.saveAction()
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

    Dialog {
        id: profileDialog
        modal: true
        palette: settings.palette
        title: qsTr("Save profile")
        standardButtons: Dialog.Save | Dialog.Cancel
        width: Math.min(settings.width - 48, 420)
        x: (settings.width - width) / 2
        y: (settings.height - height) / 2
        onAccepted: backend.saveCurrentProfile(profileNameField.text)

        contentItem: ColumnLayout {
            spacing: 10
            Label { text: qsTr("Profile name") }
            TextField {
                id: profileNameField
                Layout.fillWidth: true
                maximumLength: 32
                color: settings.palette.text
                placeholderText: qsTr("For example, Work or Streaming")
                onAccepted: profileDialog.accept()
            }
            Label {
                Layout.fillWidth: true
                visible: backend.errorMessage.length > 0
                wrapMode: Text.WordWrap
                color: "#ffb4ab"
                text: backend.errorMessage
            }
        }
    }

    Dialog {
        id: pythonDialog
        property var targetRow
        modal: true
        palette: settings.palette
        title: qsTr("Custom Python macro")
        standardButtons: Dialog.Ok | Dialog.Cancel
        width: Math.min(settings.width - 48, 820)
        height: Math.min(settings.height - 48, 620)
        x: (settings.width - width) / 2
        y: (settings.height - height) / 2
        onAccepted: {
            if (targetRow) {
                targetRow.presetId = ""
                targetRow.parameterValues = []
                targetRow.scriptText = ""
                backend.setSlotPython(targetRow.index, targetRow.labelText, pythonEditor.text, targetRow.iconText)
            }
        }

        contentItem: ColumnLayout {
            spacing: 8
            Label {
                Layout.fillWidth: true
                wrapMode: Text.WordWrap
                color: settings.palette.mid
                text: qsTr("Python runs as your user account with access to your installed libraries and environment. Review code before saving it.")
            }
            TextArea {
                id: pythonEditor
                Layout.fillWidth: true
                Layout.fillHeight: true
                font.family: "monospace"
                font.pixelSize: 13
                wrapMode: TextEdit.NoWrap
                selectByMouse: true
                color: settings.palette.text
                placeholderText: qsTr("Write Python code for this macro…")
                background: Rectangle {
                    color: settings.palette.base
                    border.color: settings.palette.midlight
                    radius: 6
                }
            }
        }
    }

    FileDialog {
        id: macroPicker
        property var targetRow
        title: qsTr("Import Python code")
        fileMode: FileDialog.OpenFile
        nameFilters: [qsTr("Python scripts (*.py)")]
        onAccepted: {
            if (targetRow) {
                pythonDialog.targetRow = targetRow
                pythonEditor.text = backend.readPythonSource(backend.localPath(selectedFile.toString()))
                pythonDialog.open()
            }
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
