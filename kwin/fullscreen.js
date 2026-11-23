function reportFullscreen() {
  const active = workspace.activeWindow;
  const isFullscreen = active !== null && active.fullScreen;
  callDBus("dev.notch.Notch", "/dev/notch", "dev.notch.Notch", "SetFullscreen", isFullscreen);
}

function watchWindow(window) {
  window.fullScreenChanged.connect(reportFullscreen);
}

workspace.windowActivated.connect(reportFullscreen);
workspace.windowAdded.connect(watchWindow);
workspace.windowRemoved.connect(reportFullscreen);
workspace.windowList().forEach(watchWindow);
reportFullscreen();
