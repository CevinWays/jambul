mod fullscreen;
mod mpris;
mod notifications;
mod system;

use gtk::cairo::{RectangleInt, Region};
use gtk::prelude::*;
use gtk_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use tauri::{Manager, WebviewWindow};

#[tauri::command]
fn set_hit_rect(window: WebviewWindow, x: i32, y: i32, width: i32, height: i32) -> Result<(), String> {
    let gtk_window = window.gtk_window().map_err(|error| error.to_string())?;
    let gdk_window = gtk_window.window().ok_or("window not realized")?;
    let region = Region::create_rectangle(&RectangleInt::new(x, y, width, height));
    gdk_window.input_shape_combine_region(&region, 0, 0);
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            set_hit_rect,
            mpris::media_control,
            system::battery_status,
            system::volume_get,
            system::volume_set
        ])
        .setup(|app| {
            let connection = tauri::async_runtime::block_on(zbus::Connection::session())?;
            app.manage(connection.clone());
            fullscreen::start(app.handle().clone(), connection.clone());
            mpris::start_polling(app.handle().clone(), connection);
            notifications::start_monitoring(app.handle().clone());
            let window = app.get_webview_window("main").ok_or("main window missing")?;
            let gtk_window = window.gtk_window()?;
            // webview in a layer-shell window never paints without an explicit size request
            gtk_window.set_size_request(600, 240);
            gtk_window.init_layer_shell();
            gtk_window.set_layer(Layer::Overlay);
            gtk_window.set_anchor(Edge::Top, true);
            gtk_window.set_exclusive_zone(0);
            gtk_window.set_keyboard_mode(KeyboardMode::OnDemand);
            window.show()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to run jambul");
}
