# jambul

An interactive notch for Linux: a pill pinned to the top-center of the screen that expands into a media player and dashboard. Built with Tauri 2 and `gtk-layer-shell`, targeting KDE Plasma on Wayland.

Inspired by [Poni](https://github.com/liimalim/Poni) for Windows. Not affiliated with it, and no code or assets are copied.

## Features

- Pill overlay that expands on click, with click-through everywhere outside it
- Media widget for any MPRIS player (title, artist, art, progress, controls)
- Notification toasts, shown by monitoring `org.freedesktop.Notifications` (jambul asks Plasma to hold back its own popups; unverified)
- Dashboard: clock, battery, volume (`wpctl`), brightness (logind), Wi-Fi (`nmcli`) and Bluetooth (`bluetoothctl`) toggles, weather (Open-Meteo, located by IP via ipwho.is)
- Calendar with Hijri date, notification history, and prayer times (Kemenag RI method, Tanbih branding)
- Pomodoro timer (25 min focus, 5 min break), shown in the closed pill while running
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

## Install

```bash
sudo pacman -S --needed networkmanager bluez-utils
cargo build --release --features tauri/custom-protocol
install -Dm755 target/release/jambul ~/.local/bin/jambul
```

`networkmanager` (`nmcli`) and `bluez-utils` (`bluetoothctl`) are only needed for the Wi-Fi and Bluetooth toggles. Brightness goes through logind and needs no extra package.

To try it without installing, use `cargo run`.

## Usage

Start and stop:

```bash
cd ~/Developer/jambul && cargo run
jambul &        # start
pkill -x jambul # stop
```

Only run one instance at a time; starting it twice puts two pills on top of each other.

Start automatically at login:

```bash
mkdir -p ~/.config/autostart
sed "s|^Exec=.*|Exec=$HOME/.local/bin/jambul|" jambul.desktop > ~/.config/autostart/jambul.desktop
```

Turn autostart off again:

```bash
rm ~/.config/autostart/jambul.desktop
```

Using the pill:

- Click the pill to open or close it.
- Media controls work with any MPRIS player (Spotify, Firefox, mpv, ...).
- The dashboard has volume and brightness sliders and Wi-Fi and Bluetooth toggles.
- Pomodoro: press play to start a 25 minute focus; the remaining time shows in the closed pill. A toast announces the end of each phase, and the next phase waits for you to press play. The reset button returns to a fresh focus.
- Notifications appear as a toast for 5 seconds. Test with `notify-send "Hello" "World"`.
- The pill hides while a window is fullscreen and comes back afterwards.

On Arch you can also build a package with the included `PKGBUILD` (`makepkg -si`, needs a `v0.1.0` tag on GitHub); it installs the autostart entry system-wide. Remove it with `sudo pacman -R jambul`.

## How it works

- `src/main.rs` creates the Tauri window and turns it into a layer-shell surface (overlay layer, anchored to the top, no exclusive zone). The webview does not paint in a layer-shell window unless `set_size_request` is called before `init_layer_shell`.
- `src/mpris.rs` polls the active MPRIS player over D-Bus (`zbus`) once a second.
- `src/notifications.rs` uses `BecomeMonitor` to read `Notify` calls without replacing the notification server.
- `src/system.rs` reads the battery from sysfs and drives the volume through `wpctl`.
- `src/fullscreen.rs` serves `dev.jambul.Jambul` on the session bus and loads `kwin/fullscreen.js` into KWin, which reports fullscreen changes back.
- `dist/index.html` is the whole UI, with no build step.

## Known limitations

- MPRIS is polled, not driven by `PropertiesChanged` signals.
- While fullscreen, the window is hidden with CSS rather than unmapped.
- The app icon is a placeholder.
- The pomodoro state is lost when jambul quits.
- The weather location comes from the IP address, so it is city-level at best.

## License

MIT, see [LICENSE](LICENSE).

## Credits

- Prayer times follow the Kemenag RI method, branded after the author's Tanbih app.
- Outfit font by The Outfit Project Authors, SIL Open Font License 1.1 (`dist/fonts/OFL-Outfit.txt`).
