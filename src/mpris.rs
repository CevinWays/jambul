use std::collections::HashMap;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};
use zbus::zvariant::OwnedValue;
use zbus::{proxy, Connection};

const MPRIS_PREFIX: &str = "org.mpris.MediaPlayer2.";
const STATUS_PLAYING: &str = "Playing";
const POLL_INTERVAL: Duration = Duration::from_secs(1);
pub const MEDIA_EVENT: &str = "media";

#[proxy(
    interface = "org.mpris.MediaPlayer2.Player",
    default_path = "/org/mpris/MediaPlayer2"
)]
trait Player {
    fn play_pause(&self) -> zbus::Result<()>;
    fn next(&self) -> zbus::Result<()>;
    fn previous(&self) -> zbus::Result<()>;
    #[zbus(property)]
    fn playback_status(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn metadata(&self) -> zbus::Result<HashMap<String, OwnedValue>>;
    #[zbus(property)]
    fn position(&self) -> zbus::Result<i64>;
}

#[derive(Serialize, Clone, Default, PartialEq, Debug)]
pub struct MediaState {
    pub title: String,
    pub artist: String,
    pub art_url: Option<String>,
    pub playing: bool,
    pub position_us: i64,
    pub length_us: i64,
}

pub fn pick_player(players: &[(String, String)]) -> Option<&str> {
    players
        .iter()
        .find(|(_, status)| status == STATUS_PLAYING)
        .or_else(|| players.first())
        .map(|(name, _)| name.as_str())
}

fn field<T: TryFrom<OwnedValue>>(metadata: &HashMap<String, OwnedValue>, key: &str) -> Option<T> {
    metadata
        .get(key)?
        .try_clone()
        .ok()
        .and_then(|value| T::try_from(value).ok())
}

pub fn build_state(
    status: &str,
    metadata: &HashMap<String, OwnedValue>,
    position_us: i64,
) -> MediaState {
    let artists = field::<Vec<String>>(metadata, "xesam:artist").unwrap_or_default();
    let length_us = field::<i64>(metadata, "mpris:length")
        .or_else(|| field::<u64>(metadata, "mpris:length").map(|length| length as i64))
        .unwrap_or(0);
    MediaState {
        title: field::<String>(metadata, "xesam:title").unwrap_or_default(),
        artist: artists.join(", "),
        art_url: field::<String>(metadata, "mpris:artUrl"),
        playing: status == STATUS_PLAYING,
        position_us,
        length_us,
    }
}

async fn list_players(connection: &Connection) -> zbus::Result<Vec<(String, String)>> {
    let bus = zbus::fdo::DBusProxy::new(connection).await?;
    let mut players = Vec::new();
    for name in bus.list_names().await? {
        if !name.starts_with(MPRIS_PREFIX) {
            continue;
        }
        let status = match player_proxy(connection, name.as_str()).await {
            Ok(proxy) => proxy.playback_status().await.unwrap_or_default(),
            Err(_) => continue,
        };
        players.push((name.to_string(), status));
    }
    Ok(players)
}

async fn player_proxy<'a>(connection: &'a Connection, name: &str) -> zbus::Result<PlayerProxy<'a>> {
    PlayerProxy::builder(connection)
        .destination(name.to_owned())?
        .build()
        .await
}

async fn read_state(connection: &Connection) -> zbus::Result<Option<MediaState>> {
    let players = list_players(connection).await?;
    let Some(name) = pick_player(&players) else {
        return Ok(None);
    };
    let proxy = player_proxy(connection, name).await?;
    let status = proxy.playback_status().await?;
    let metadata = proxy.metadata().await.unwrap_or_default();
    let position_us = proxy.position().await.unwrap_or(0);
    Ok(Some(build_state(&status, &metadata, position_us)))
}

pub fn start_polling(app: AppHandle, connection: Connection) {
    tauri::async_runtime::spawn(async move {
        let mut last_emitted: Option<Option<MediaState>> = None;
        loop {
            // shortcut: polling every second, switch to PropertiesChanged signals when latency or CPU matters
            let state = read_state(&connection).await.ok().flatten();
            if last_emitted.as_ref() != Some(&state) {
                let _ = app.emit(MEDIA_EVENT, state.clone());
                last_emitted = Some(state);
            }
            tokio::time::sleep(POLL_INTERVAL).await;
        }
    });
}

#[tauri::command]
pub async fn media_control(connection: State<'_, Connection>, action: String) -> Result<(), String> {
    let players = list_players(&connection).await.map_err(|error| error.to_string())?;
    let name = pick_player(&players).ok_or("no media player")?;
    let proxy = player_proxy(&connection, name).await.map_err(|error| error.to_string())?;
    let result = match action.as_str() {
        "play_pause" => proxy.play_pause().await,
        "next" => proxy.next().await,
        "previous" => proxy.previous().await,
        other => return Err(format!("unknown action: {other}")),
    };
    result.map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn player(name: &str, status: &str) -> (String, String) {
        (name.to_string(), status.to_string())
    }

    #[test]
    fn prefers_playing_player() {
        let players = vec![player("a", "Paused"), player("b", "Playing")];
        assert_eq!(pick_player(&players), Some("b"));
    }

    #[test]
    fn falls_back_to_first_player() {
        let players = vec![player("a", "Paused"), player("b", "Stopped")];
        assert_eq!(pick_player(&players), Some("a"));
    }

    #[test]
    fn no_players_gives_none() {
        assert_eq!(pick_player(&[]), None);
    }

    #[test]
    fn empty_metadata_gives_blank_state() {
        let state = build_state("Paused", &HashMap::new(), 5);
        assert_eq!(state.title, "");
        assert!(!state.playing);
        assert_eq!(state.position_us, 5);
    }
}
