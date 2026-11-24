# jambul

## 1. Project overview

jambul is an interactive notch for Linux: a black pill pinned to the top-center of the screen that expands into a media player and dashboard. It targets KDE Plasma on Wayland. It is a personal project, inspired by [Poni](https://github.com/liimalim/Poni) (Windows). No Poni code or assets are copied.

Features: media widget (MPRIS), notification toasts, dashboard (clock, battery, volume, brightness, Wi-Fi, Bluetooth, weather), pomodoro timer, and auto-hide while a window is fullscreen.

UI text is Indonesian. Code, comments, commit messages and docs are English.

## 2. Tech stack

- **Shell:** Tauri 2 (Rust) with the webkit2gtk-4.1 webview.
- **Overlay:** `gtk-layer-shell` on the GTK3 window (overlay layer, anchored top, exclusive zone -1, click-through via input region).
- **IPC:** `zbus` 5 (tokio) for MPRIS, notification monitoring (`BecomeMonitor`), logind brightness, and a custom D-Bus service.
- **Fullscreen detection:** a KWin script loaded through `org.kde.kwin.Scripting`.
- **System tools:** `wpctl` (volume), `nmcli` (Wi-Fi), `bluetoothctl` (Bluetooth), sysfs (battery, backlight).
- **Frontend:** a single static `dist/index.html` with vanilla JS and CSS. No build step, no framework, no bundler (`withGlobalTauri: true`).
- **External services:** Open-Meteo (weather) and ipwho.is (location by IP), both called from the frontend.
- **Packaging:** `PKGBUILD` for Arch/AUR.

## 3. Project structure

```
src/main.rs            window setup, layer-shell, set_hit_rect, command registration
src/mpris.rs           MPRIS polling and media_control command
src/notifications.rs   notification monitor, Plasma popup inhibit
src/system.rs          battery, volume, brightness, Wi-Fi, Bluetooth commands
src/fullscreen.rs      D-Bus service dev.jambul.Jambul and KWin script loader
kwin/fullscreen.js     KWin script that reports fullscreen state
dist/index.html        the whole UI
tauri.conf.json        Tauri config (one transparent, undecorated window)
capabilities/          Tauri permissions
PKGBUILD, jambul.desktop   packaging and autostart
```

Pure logic (parsers, conversions) lives in small functions next to its module with `#[cfg(test)]` unit tests. IO stays in the commands.

Run with `cargo run`. Test with `cargo test`.

## 4. Design system

The UI is a black notch; everything else is transparent and click-through.

- **Colors:** pill `#000`; primary text `#fff`; secondary `#ccc`; muted `#aaa`; surfaces and inactive buttons `#222`; tracks `#333`; accent (sliders, active toggle) `#fff` with `#000` text.
- **Shape:** the pill is flat on top and rounded at the bottom: closed `0 0 18px 18px`, toast `0 0 24px 24px`, open `0 0 28px 28px`. Buttons use a 12px radius.
- **States and sizes (px):** closed 160x32, toast 420x68, open 420x264, away (fullscreen) hidden. The hit rect sent to `set_hit_rect` must match the visible size.
- **Motion:** width and height transition 0.3s with `cubic-bezier(.3,1.4,.5,1)` (springy).
- **Type:** `13px sans-serif` base; 12px for dashboard rows; 15px bold titles; tabular numbers for timers.
- **Layout:** rows are flex with 10 to 14px gaps and 20px side padding; text truncates with an ellipsis instead of overflowing.
- **Icons:** plain unicode glyphs and inline SVG data URIs. No icon library.
- **Defaults:** missing album art falls back to the built-in music-note SVG.

## 5. What to avoid

- **Never commit or push without being told to.** Finish the work, report what changed, and wait for an explicit request. One reason per commit; split unrelated changes.
- **No redundant code.** Reuse what exists before writing new code. No duplicated logic, no dead code, no leftover unused variables or imports, no speculative options "for later".
- **No new dependencies** for a few lines of code. Prefer the standard library, Tauri, `zbus`, or tools already used.
- **No comments that restate the code.** Comment only a non-obvious why, in one line. Mark unverified assumptions with `// TODO: unverified: ...`.
- **No magic numbers or strings.** Use named constants, as the existing code does.
- **Keep IO out of pure logic.** Parsers and conversions stay pure and tested; commands only do IO.
- **Do not copy Poni code or assets.** Analysis findings are for understanding only.
- **Do not change the global git identity.** The identity is set locally in this repo (CevinWays). Never use the work email or enter personal details into outside services.
- **Do not install or download third-party code** without the user asking or approving.
- **Do not claim something works unless it was checked.** Visual behavior needs the user's eyes on the screen; say what was not verified.
- **Do not hardcode the layout size in only one place.** Pill sizes live in both the CSS and the `sizes` table in `dist/index.html`; change both.
- **Commit trailer:** end commit messages with `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`.
