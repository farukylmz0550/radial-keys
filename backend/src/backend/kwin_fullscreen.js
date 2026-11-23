var watchedWindow = null;
var blocked = null;

function updateFullscreenState() {
    var current = workspace.activeWindow;
    var nextBlocked = !!(current && current.fullScreen);
    if (blocked === nextBlocked)
        return;
    blocked = nextBlocked;
    callDBus("io.github.radialkeys.Service",
             "/io/github/radialkeys/Service",
             "io.github.radialkeys.Service1",
             "SetFullscreenActive",
             nextBlocked);
}

function watchActiveWindow(window) {
    if (watchedWindow) {
        try {
            watchedWindow.fullScreenChanged.disconnect(updateFullscreenState);
        } catch (error) {
        }
    }
    watchedWindow = window;
    if (watchedWindow)
        watchedWindow.fullScreenChanged.connect(updateFullscreenState);
    updateFullscreenState();
}

workspace.windowActivated.connect(watchActiveWindow);
workspace.windowAdded.connect(function(window) {
    window.fullScreenChanged.connect(updateFullscreenState);
});
watchActiveWindow(workspace.activeWindow);
