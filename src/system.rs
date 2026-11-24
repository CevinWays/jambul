use std::fs;
use std::process::Command;

use serde::Serialize;

const POWER_SUPPLY_DIR: &str = "/sys/class/power_supply";
const BATTERY_PREFIX: &str = "BAT";
const DEFAULT_SINK: &str = "@DEFAULT_AUDIO_SINK@";
const MAX_VOLUME_PERCENT: u32 = 100;
const BACKLIGHT_DIR: &str = "/sys/class/backlight";
const MIN_BRIGHTNESS_PERCENT: u32 = 5;

#[derive(Serialize, Debug, PartialEq)]
pub struct BatteryState {
    pub percent: u32,
    pub charging: bool,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct VolumeState {
    pub percent: u32,
    pub muted: bool,
}

pub fn parse_battery(capacity: &str, status: &str) -> Option<BatteryState> {
    Some(BatteryState {
        percent: capacity.trim().parse().ok()?,
        charging: matches!(status.trim(), "Charging" | "Full"),
    })
}

pub fn parse_volume(output: &str) -> Option<VolumeState> {
    let mut words = output.split_whitespace();
    let ratio: f32 = words.nth(1)?.parse().ok()?;
    Some(VolumeState {
        percent: (ratio * 100.0).round() as u32,
        muted: output.contains("[MUTED]"),
    })
}

#[derive(Serialize, Debug, PartialEq)]
pub struct RadioState {
    pub wifi: bool,
    pub bluetooth: bool,
}

pub fn parse_wifi_enabled(output: &str) -> bool {
    output.trim() == "enabled"
}

pub fn parse_bluetooth_powered(output: &str) -> bool {
    output.lines().any(|line| line.trim() == "Powered: yes")
}

pub fn brightness_percent(current: u32, max: u32) -> Option<u32> {
    (max > 0).then(|| (current * 100 + max / 2) / max)
}

pub fn brightness_raw(percent: u32, max: u32) -> u32 {
    percent.clamp(MIN_BRIGHTNESS_PERCENT, 100) * max / 100
}

fn run(program: &str, arguments: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(arguments)
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn backlight() -> Option<(String, u32, u32)> {
    let entry = fs::read_dir(BACKLIGHT_DIR).ok()?.flatten().next()?;
    let read = |file: &str| fs::read_to_string(entry.path().join(file)).ok()?.trim().parse().ok();
    Some((entry.file_name().to_string_lossy().into_owned(), read("brightness")?, read("max_brightness")?))
}

fn wpctl(arguments: &[&str]) -> Result<String, String> {
    let output = Command::new("wpctl")
        .args(arguments)
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

#[tauri::command]
pub async fn battery_status() -> Option<BatteryState> {
    let entries = fs::read_dir(POWER_SUPPLY_DIR).ok()?;
    entries
        .flatten()
        .filter(|entry| entry.file_name().to_string_lossy().starts_with(BATTERY_PREFIX))
        .find_map(|entry| {
            let capacity = fs::read_to_string(entry.path().join("capacity")).ok()?;
            let status = fs::read_to_string(entry.path().join("status")).unwrap_or_default();
            parse_battery(&capacity, &status)
        })
}

#[tauri::command]
pub async fn volume_get() -> Result<VolumeState, String> {
    let output = wpctl(&["get-volume", DEFAULT_SINK])?;
    parse_volume(&output).ok_or_else(|| format!("unexpected wpctl output: {output}"))
}

#[tauri::command]
pub async fn volume_set(percent: u32) -> Result<(), String> {
    let ratio = percent.min(MAX_VOLUME_PERCENT) as f32 / 100.0;
    wpctl(&["set-volume", DEFAULT_SINK, &format!("{ratio:.2}")]).map(|_| ())
}

#[tauri::command]
pub async fn brightness_get() -> Option<u32> {
    let (_, current, max) = backlight()?;
    brightness_percent(current, max)
}

#[tauri::command]
pub async fn brightness_set(percent: u32) -> Result<(), String> {
    let (name, _, max) = backlight().ok_or("no backlight device")?;
    let connection = zbus::Connection::system().await.map_err(|error| error.to_string())?;
    connection
        .call_method(
            Some("org.freedesktop.login1"),
            "/org/freedesktop/login1/session/auto",
            Some("org.freedesktop.login1.Session"),
            "SetBrightness",
            &("backlight", name.as_str(), brightness_raw(percent, max)),
        )
        .await
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn radio_status() -> RadioState {
    RadioState {
        wifi: run("nmcli", &["radio", "wifi"]).map(|output| parse_wifi_enabled(&output)).unwrap_or(false),
        bluetooth: run("bluetoothctl", &["show"]).map(|output| parse_bluetooth_powered(&output)).unwrap_or(false),
    }
}

#[tauri::command]
pub async fn wifi_set(enabled: bool) -> Result<(), String> {
    run("nmcli", &["radio", "wifi", if enabled { "on" } else { "off" }]).map(|_| ())
}

#[tauri::command]
pub async fn bluetooth_set(enabled: bool) -> Result<(), String> {
    run("bluetoothctl", &["power", if enabled { "on" } else { "off" }]).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_discharging_battery() {
        assert_eq!(
            parse_battery("42\n", "Discharging\n"),
            Some(BatteryState { percent: 42, charging: false })
        );
    }

    #[test]
    fn full_battery_counts_as_charging() {
        assert!(parse_battery("100", "Full").unwrap().charging);
    }

    #[test]
    fn rejects_garbage_capacity() {
        assert_eq!(parse_battery("n/a", "Full"), None);
    }

    #[test]
    fn parses_volume() {
        assert_eq!(parse_volume("Volume: 0.30\n"), Some(VolumeState { percent: 30, muted: false }));
    }

    #[test]
    fn parses_muted_volume() {
        let state = parse_volume("Volume: 0.47 [MUTED]").unwrap();
        assert_eq!(state.percent, 47);
        assert!(state.muted);
    }

    #[test]
    fn rejects_unexpected_volume_output() {
        assert_eq!(parse_volume(""), None);
    }

    #[test]
    fn parses_radio_states() {
        assert!(parse_wifi_enabled("enabled\n"));
        assert!(!parse_wifi_enabled("disabled"));
        assert!(parse_bluetooth_powered("Controller X\n\tPowered: yes\n"));
        assert!(!parse_bluetooth_powered("\tPowered: no"));
    }

    #[test]
    fn converts_brightness() {
        assert_eq!(brightness_percent(50, 200), Some(25));
        assert_eq!(brightness_percent(1, 0), None);
        assert_eq!(brightness_raw(0, 200), 10);
        assert_eq!(brightness_raw(100, 200), 200);
    }
}
