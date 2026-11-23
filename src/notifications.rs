use std::collections::HashMap;

use futures_util::StreamExt;
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use zbus::message::Type;
use zbus::zvariant::OwnedValue;
use zbus::{Connection, MessageStream};

const NOTIFICATIONS_INTERFACE: &str = "org.freedesktop.Notifications";
const NOTIFY_MEMBER: &str = "Notify";
const MONITOR_RULE: &str =
    "type='method_call',interface='org.freedesktop.Notifications',member='Notify'";
pub const NOTIFICATION_EVENT: &str = "notification";

type NotifyArguments = (
    String,
    u32,
    String,
    String,
    String,
    Vec<String>,
    HashMap<String, OwnedValue>,
    i32,
);

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Notification {
    pub app_name: String,
    pub summary: String,
    pub body: String,
}

pub fn to_notification(arguments: NotifyArguments) -> Option<Notification> {
    let (app_name, _, _, summary, body, ..) = arguments;
    if summary.is_empty() && body.is_empty() {
        return None;
    }
    Some(Notification { app_name, summary, body })
}

async fn monitor(app: AppHandle) -> zbus::Result<()> {
    let connection = Connection::session().await?;
    connection
        .call_method(
            Some("org.freedesktop.DBus"),
            "/org/freedesktop/DBus",
            Some("org.freedesktop.DBus.Monitoring"),
            "BecomeMonitor",
            &(vec![MONITOR_RULE], 0u32),
        )
        .await?;

    let mut stream = MessageStream::from(&connection);
    while let Some(message) = stream.next().await {
        let Ok(message) = message else { continue };
        let header = message.header();
        let is_notify = message.message_type() == Type::MethodCall
            && header.interface().map(|name| name.as_str()) == Some(NOTIFICATIONS_INTERFACE)
            && header.member().map(|name| name.as_str()) == Some(NOTIFY_MEMBER);
        if !is_notify {
            continue;
        }
        let Ok(arguments) = message.body().deserialize::<NotifyArguments>() else {
            continue;
        };
        if let Some(notification) = to_notification(arguments) {
            let _ = app.emit(NOTIFICATION_EVENT, notification);
        }
    }
    Ok(())
}

pub fn start_monitoring(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        if let Err(error) = monitor(app).await {
            eprintln!("notification monitor stopped: {error}");
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arguments(summary: &str, body: &str) -> NotifyArguments {
        (
            "app".into(),
            0,
            String::new(),
            summary.into(),
            body.into(),
            vec![],
            HashMap::new(),
            -1,
        )
    }

    #[test]
    fn keeps_notification_with_summary() {
        let notification = to_notification(arguments("Halo", "")).unwrap();
        assert_eq!(notification.summary, "Halo");
        assert_eq!(notification.app_name, "app");
    }

    #[test]
    fn drops_empty_notification() {
        assert_eq!(to_notification(arguments("", "")), None);
    }
}
