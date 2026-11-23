import QtQuick
import QtQuick.Controls
import QtQuick.Window
import QtMultimedia

Window {
    id: overlay
    property bool menuVisible: false
    property point cursorPosition: Qt.point(0, 0)
    property int selectedIndex: -1
    property int lastAudibleSelection: -1
    property bool discardSelectedAction: false
    signal cancelRequested()

    SoundEffect {
        id: selectionSound
        source: "qrc:/sounds/selection-tick.wav"
        volume: 0.48
    }

    onSelectedIndexChanged: {
        if (menuVisible && selectedIndex >= 0) {
            if (lastAudibleSelection >= 0 && selectedIndex !== lastAudibleSelection)
                selectionSound.play()
            lastAudibleSelection = selectedIndex
        } else if (!menuVisible) {
            lastAudibleSelection = -1
        }
    }

    Connections {
        target: backend
        function onStateChanged() { canvas.requestPaint() }
    }

    // Keep the overlay screen-sized without marking it as fullscreen. Plasma
    // hides auto-hide panels when a fullscreen surface is mapped.
    visibility: menuVisible ? Window.Windowed : Window.Hidden
    color: "transparent"
    // A splash surface is omitted from task switchers and taskbars by KDE,
    // while retaining normal pointer and keyboard input for the radial menu.
    flags: Qt.SplashScreen | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint
    transientParent: null
    screen: Application.screens.length > 0
            ? Application.screens[Math.max(0, Math.min(backend.cursorScreenIndex, Application.screens.length - 1))]
            : null
    x: Screen.virtualX
    y: Screen.virtualY
    width: Screen.width
    height: Screen.height
    title: qsTr("Radial Keys menu")

    readonly property real outerRadius: 112
    readonly property real deadZone: 28
    readonly property real segmentRadius: (deadZone + outerRadius) / 2
    readonly property real centerX: cursorPosition.x - Screen.virtualX
    readonly property real centerY: cursorPosition.y - Screen.virtualY

    function actionDescription(slot) {
        if (!slot)
            return ""
        const preset = slot.preset
        if (preset) {
            const presetIndex = backend.presetIndex(preset.id)
            const definition = presetIndex >= 0 ? backend.builtinMacros[presetIndex] : null
            let description = definition ? definition.description : qsTr("Run this built-in action.")
            const parameters = preset.parameters || []
            if (parameters.length > 0)
                description += "\n" + parameters.join(" · ")
            return slot.label + "\n" + description
        }
        if (slot.action && slot.action.length > 0)
            return slot.label + "\n" + qsTr("Run this Python macro in a separate process.")
        return slot.label + "\n" + qsTr("No action is assigned.")
    }

    onMenuVisibleChanged: {
        if (menuVisible) {
            discardSelectedAction = false
        } else {
            if (selectedIndex >= 0 && !discardSelectedAction && !backend.fullscreenActive)
                backend.activateSlot(selectedIndex)
            selectedIndex = -1
            discardSelectedAction = false
        }
        if (menuVisible) {
            requestActivate()
            escapeHandler.forceActiveFocus()
        }
        canvas.requestPaint()
    }

    Item {
        id: escapeHandler
        anchors.fill: parent
        focus: true

        Keys.onEscapePressed: function(event) {
            event.accepted = true
            overlay.discardSelectedAction = true
            overlay.cancelRequested()
        }
    }

    MouseArea {
        id: pointerArea
        anchors.fill: parent
        hoverEnabled: true
        onPositionChanged: function(mouse) {
            overlay.selectedIndex = overlay.slotAt(mouse.x, mouse.y)
            canvas.requestPaint()
        }
        onClicked: function(mouse) {
            const index = overlay.slotAt(mouse.x, mouse.y)
            overlay.discardSelectedAction = true
            if (index < 0) {
                overlay.cancelRequested()
            } else {
                backend.activateSlot(index)
            }
        }
    }

    Repeater {
        model: backend.menuSlots

        delegate: Image {
            required property int index
            required property var modelData
            z: 1
            width: 34
            height: 34
            x: overlay.centerX + Math.cos(-Math.PI / 2 + index * Math.PI / 4) * overlay.segmentRadius - width / 2
            y: overlay.centerY + Math.sin(-Math.PI / 2 + index * Math.PI / 4) * overlay.segmentRadius - height / 2
            source: modelData.preset && modelData.preset.id === "microphone_toggle"
                ? backend.fileUrl(backend.microphoneStateKnown && backend.microphoneMuted
                    ? "qrc:/icons/microphone-muted.svg" : "qrc:/icons/microphone-unmuted.svg")
                : (modelData.icon && modelData.icon.length > 0 ? backend.fileUrl(modelData.icon) : "")
            sourceSize.width: 56
            sourceSize.height: 56
            fillMode: Image.PreserveAspectFit
            smooth: true
            asynchronous: true
            visible: modelData.icon && modelData.icon.length > 0

            HoverHandler {
                id: iconHover
                acceptedDevices: PointerDevice.Mouse
            }
            ToolTip.visible: iconHover.hovered
            ToolTip.delay: 700
            ToolTip.text: overlay.actionDescription(modelData)
        }
    }

    function slotAt(x, y) {
        const dx = x - centerX
        const dy = y - centerY
        const distance = Math.sqrt(dx * dx + dy * dy)
        if (distance < deadZone || distance > outerRadius)
            return -1
        const angle = Math.atan2(dy, dx)
        const clockwiseFromNorth = (angle + Math.PI / 2 + Math.PI * 2) % (Math.PI * 2)
        return Math.floor((clockwiseFromNorth + Math.PI / 8) / (Math.PI / 4)) % 8
    }

    Canvas {
        id: canvas
        anchors.fill: parent
        renderStrategy: Canvas.Immediate

        onPaint: {
            const context = getContext("2d")
            context.clearRect(0, 0, width, height)
            const segment = Math.PI / 4
            for (let index = 0; index < 8; ++index) {
                const start = -Math.PI / 2 - segment / 2 + index * segment
                const end = start + segment
                context.beginPath()
                context.moveTo(overlay.centerX, overlay.centerY)
                context.arc(overlay.centerX, overlay.centerY, overlay.outerRadius, start, end)
                context.closePath()
                context.fillStyle = index === overlay.selectedIndex ? "#3584e4" : "#30343b"
                context.fill()
                context.lineWidth = 2
                context.strokeStyle = "#f6f5f4"
                context.stroke()

                const labelAngle = -Math.PI / 2 + index * segment
                const labelX = overlay.centerX + Math.cos(labelAngle) * 76
                const labelY = overlay.centerY + Math.sin(labelAngle) * 76
                context.font = (index === overlay.selectedIndex ? "bold " : "") + "14px sans-serif"
                context.textAlign = "center"
                context.textBaseline = "middle"
                context.fillStyle = "#ffffff"
                const slots = backend.menuSlots
                const label = slots.length > index ? slots[index].label : qsTr("Slot %1").arg(index + 1)
                const slotIcon = slots.length > index ? slots[index].icon : ""
                if (!slotIcon || slotIcon.length === 0)
                    context.fillText(label, labelX, labelY, 68)
            }

            context.beginPath()
            context.arc(overlay.centerX, overlay.centerY, overlay.deadZone, 0, Math.PI * 2)
            context.fillStyle = "#1d2024"
            context.fill()
            context.lineWidth = 2
            context.strokeStyle = "#f6f5f4"
            context.stroke()
        }
    }
}
