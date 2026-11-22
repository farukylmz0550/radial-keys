import QtQuick
import QtQuick.Window

Window {
    id: overlay
    property bool menuVisible: false
    property point cursorPosition: Qt.point(0, 0)
    property int selectedIndex: -1
    property bool discardSelectedAction: false
    signal cancelRequested()

    Connections {
        target: backend
        function onStateChanged() { canvas.requestPaint() }
    }

    visible: menuVisible
    visibility: menuVisible ? Window.FullScreen : Window.Hidden
    color: "transparent"
    flags: Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint
    transientParent: null
    screen: Application.screens.length > 0
            ? Application.screens[Math.max(0, Math.min(backend.cursorScreenIndex, Application.screens.length - 1))]
            : null
    title: qsTr("Radial Keys menu")

    readonly property real outerRadius: 112
    readonly property real deadZone: 28
    readonly property real centerX: cursorPosition.x - Screen.virtualX
    readonly property real centerY: cursorPosition.y - Screen.virtualY

    onMenuVisibleChanged: {
        if (menuVisible) {
            discardSelectedAction = false
        } else {
            if (selectedIndex >= 0 && !discardSelectedAction)
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
            width: 28
            height: 28
            x: overlay.centerX + Math.cos(-Math.PI / 2 + index * Math.PI / 4) * 48 - width / 2
            y: overlay.centerY + Math.sin(-Math.PI / 2 + index * Math.PI / 4) * 48 - height / 2
            source: modelData.icon && modelData.icon.length > 0 ? backend.fileUrl(modelData.icon) : ""
            sourceSize.width: 56
            sourceSize.height: 56
            fillMode: Image.PreserveAspectFit
            smooth: true
            asynchronous: true
            visible: modelData.icon && modelData.icon.length > 0
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
