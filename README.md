# jambul

An interactive notch for Linux: a pill pinned to the top-center of the screen that expands into a media player and dashboard. Built with Tauri 2 and `gtk-layer-shell`, targeting KDE Plasma on Wayland.

Inspired by [Poni](https://github.com/liimalim/Poni) for Windows. Not affiliated with it, and no code or assets are copied.

## Features

- Pill overlay that expands on click, with click-through everywhere outside it
- Media widget for any MPRIS player (title, artist, art, progress, controls)
- Notification toasts, shown by monitoring `org.freedesktop.Notifications` (Plasma's own popups still appear)
- Dashboard: clock, battery, volume slider (`wpctl`) and weather (Open-Meteo, no API key)
- Hides itself while a window is fullscreen (KDE only, through a KWin script)

## Requirements

- KDE Plasma on Wayland (other layer-shell compositors may work, but the fullscreen hiding is KDE-only)
- `rustup` (stable), `webkit2gtk-4.1`, `gtk3`, `gtk-layer-shell`
- PipeWire with `wpctl` for the volume slider

On Arch:

```bash
sudo pacman -S --needed rustup webkit2gtk-4.1 gtk3 gtk-layer-shell
rustup default stable
```

## Run

```bash
cargo run
```

Click the pill to open it. Send a test notification with `notify-send "Hello" "World"`.

## How it works

- `src/main.rs` creates the Tauri window and turns it into a layer-shell surface (overlay layer, anchored to the top, no exclusive zone). The webview does not paint in a layer-shell window unless `set_size_request` is called before `init_layer_shell`.
- `src/mpris.rs` polls the active MPRIS player over D-Bus (`zbus`) once a second.
- `src/notifications.rs` uses `BecomeMonitor` to read `Notify` calls without replacing the notification server.
- `src/system.rs` reads the battery from sysfs and drives the volume through `wpctl`.
- `src/fullscreen.rs` serves `dev.jambul.Jambul` on the session bus and loads `kwin/fullscreen.js` into KWin, which reports fullscreen changes back.
- `dist/index.html` is the whole UI, with no build step.

## Known limitations

- The weather location is hard-coded to Jakarta in `dist/index.html`.
- MPRIS is polled, not driven by `PropertiesChanged` signals.
- While fullscreen, the window is hidden with CSS rather than unmapped.
- No brightness, Wi-Fi or Bluetooth toggles yet.
- The app icon is a placeholder.
