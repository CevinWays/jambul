use std::fs;
use std::process::Command;

use serde::Serialize;

const POWER_SUPPLY_DIR: &str = "/sys/class/power_supply";
const BATTERY_PREFIX: &str = "BAT";
const DEFAULT_SINK: &str = "@DEFAULT_AUDIO_SINK@";
const MAX_VOLUME_PERCENT: u32 = 100;

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
}
