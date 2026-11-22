function updateCursorPosition() {
    const position = workspace.cursorPos;
    callDBus(
        "io.github.radialkeys.Service",
        "/io/github/radialkeys/Service",
        "io.github.radialkeys.Service1",
        "SetCursorPosition",
        position.x,
        position.y,
        function () {}
    );
}

workspace.cursorPosChanged.connect(updateCursorPosition);
updateCursorPosition();
