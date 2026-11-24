use std::path::PathBuf;

use tauri::{AppHandle, Emitter};
use zbus::{interface, Connection};

const SERVICE_NAME: &str = "dev.jambul.Jambul";
const OBJECT_PATH: &str = "/dev/jambul";
const KWIN_SERVICE: &str = "org.kde.KWin";
const SCRIPT_NAME: &str = "jambul-fullscreen";
const SCRIPT_SOURCE: &str = include_str!("../kwin/fullscreen.js");
pub const FULLSCREEN_EVENT: &str = "fullscreen";

struct JambulService {
    app: AppHandle,
}

#[interface(name = "dev.jambul.Jambul")]
impl JambulService {
    fn set_fullscreen(&self, fullscreen: bool) {
        let _ = self.app.emit(FULLSCREEN_EVENT, fullscreen);
    }
}

fn script_path() -> PathBuf {
    let directory = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    directory.join(format!("{SCRIPT_NAME}.js"))
}

async fn load_kwin_script(connection: &Connection) -> zbus::Result<()> {
    let path = script_path();
    std::fs::write(&path, SCRIPT_SOURCE)?;
    // the previous run's script has the same plugin name and would make loadScript fail
    connection
        .call_method(
            Some(KWIN_SERVICE),
            "/Scripting",
            Some("org.kde.kwin.Scripting"),
            "unloadScript",
            &(SCRIPT_NAME,),
        )
        .await?;
    let reply = connection
        .call_method(
            Some(KWIN_SERVICE),
            "/Scripting",
            Some("org.kde.kwin.Scripting"),
            "loadScript",
            &(path.to_string_lossy().as_ref(), SCRIPT_NAME),
        )
        .await?;
    let script_id: i32 = reply.body().deserialize()?;
    connection
        .call_method(
            Some(KWIN_SERVICE),
            format!("/Scripting/Script{script_id}"),
            Some("org.kde.kwin.Script"),
            "run",
            &(),
        )
        .await?;
    Ok(())
}

pub fn start(app: AppHandle, connection: Connection) {
    tauri::async_runtime::spawn(async move {
        let served = async {
            connection
                .object_server()
                .at(OBJECT_PATH, JambulService { app })
                .await?;
            connection.request_name(SERVICE_NAME).await
        }
        .await;
        if let Err(error) = served {
            eprintln!("fullscreen service not started: {error}");
            return;
        }
        // best-effort: outside KDE there is no KWin to load the script into
        if let Err(error) = load_kwin_script(&connection).await {
            eprintln!("kwin fullscreen script not loaded: {error}");
        }
    });
}
