function reportFullscreen() {
  const active = workspace.activeWindow;
  const isFullscreen = active !== null && active.fullScreen;
  callDBus("dev.jambul.Jambul", "/dev/jambul", "dev.jambul.Jambul", "SetFullscreen", isFullscreen);
}

function watchWindow(window) {
  window.fullScreenChanged.connect(reportFullscreen);
}

workspace.windowActivated.connect(reportFullscreen);
workspace.windowAdded.connect(watchWindow);
workspace.windowRemoved.connect(reportFullscreen);
workspace.windowList().forEach(watchWindow);
reportFullscreen();
