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

The look follows Poni's notch (studied from its frontend; reimplemented, nothing copied). Everything outside the notch is transparent and click-through.

- **Tokens (CSS variables on `:root`):** `--bg #000`, `--text #f5f5f7`, `--dim #8e8e93`, `--faint rgba(255,255,255,.08)`, `--card rgba(255,255,255,.055)`, status colors `--green #30d158`, `--red #ff453a`, `--yellow #ffd60a`. Use tokens, not new hex values.
- **Shape:** flat on top, rounded at the bottom. Concave "ears" (`--ear: 8px`) curve the notch into the screen edge. Cards use 14px radius, buttons and tabs 7 to 10px.
- **States and sizes (px):** the `SIZES` table in `dist/index.html` is the single source: closed 200x32, notify 380x64, open 560x220, away hidden. JS writes `--w`, `--h`, `--r` and the hit rect from it. The window itself is 640x290 (`tauri.conf.json` and `set_size_request` in `main.rs`) so the open shadow fits.
- **Motion:** `--spring cubic-bezier(.32,1.28,.54,1)` for size changes and pops, `--smooth cubic-bezier(.32,.72,0,1)` for fades, `--dur .48s`. The open content scales in from 0.94 and fades; pages slide in 6px.
- **Layout:** closed shows the clock centered, mini cover art left while playing, pomodoro time right while running. Open has a header (tab pill with an expanding active label, clock, battery) over a page area: Beranda (music card plus weather card), Kalender (month grid with Gregorian and Hijri date), Shalat (prayer times), Notifikasi (history), Kontrol (volume and brightness sliders, Wi-Fi and Bluetooth tiles), Fokus (pomodoro).
- **Type:** Inter/system sans 13px base; 14.5px/650 track title; 12.5px dim secondary; tabular numbers for any time.
- **Controls:** 34px round buttons, 42px for play, white-on-black active tiles, 5px progress bar.
- **Tanbih brand:** the prayer feature shares the brand of the author's Flutter app Tanbih (github.com/CevinWays/tanbih): teal to mint gradient (`--brand-gradient`, `#2db1b3` to `#7ac793`), the bell mark, and Outfit for the wordmark and prayer names. Prayer notifications use the app name `Tanbih`. Calculation follows Kemenag RI (Fajr 20 deg, Isha 18 deg, per-prayer margins calibrated like Tanbih), checked against the published timetable by a golden test in `src/prayer.rs`. Outfit is bundled under OFL (`dist/fonts/OFL-Outfit.txt`).
- **Icons:** inline SVG with `fill: currentColor`. No icon library or font. Missing cover art falls back to the built-in music-note SVG.

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
- **Do not hardcode pill sizes in CSS.** They come from the `SIZES` table in `dist/index.html`.
- **Commit trailer:** end commit messages with `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`.
