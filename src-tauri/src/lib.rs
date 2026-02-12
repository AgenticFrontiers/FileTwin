mod sync;

use tauri::Manager;

#[tauri::command]
fn get_host_name() -> String {
    hostname::get()
        .map(|h| h.to_string_lossy().into_owned())
        .unwrap_or_else(|_| "Unknown".to_string())
}

#[tauri::command]
async fn start_host(app: tauri::AppHandle) -> Result<(), String> {
    sync::start_host(app).await
}

#[tauri::command]
async fn stop_host(app: tauri::AppHandle) -> Result<(), String> {
    sync::stop_host(app).await
}

#[tauri::command]
async fn start_browse(app: tauri::AppHandle) -> Result<(), String> {
    sync::start_browse(app).await
}

#[tauri::command]
async fn stop_browse(app: tauri::AppHandle) -> Result<(), String> {
    sync::stop_browse(app).await
}

#[tauri::command]
async fn connect_to(
    host: String,
    port: u16,
    app: tauri::AppHandle,
) -> Result<(), String> {
    sync::connect_to(host, port, app).await
}

#[tauri::command]
async fn disconnect(app: tauri::AppHandle) -> Result<(), String> {
    sync::disconnect(app).await
}

#[tauri::command]
async fn send_clipboard(text: String, app: tauri::AppHandle) -> Result<(), String> {
    sync::send_clipboard(text, app).await
}

#[tauri::command]
async fn send_bring_to_front(app: tauri::AppHandle) -> Result<(), String> {
    sync::send_bring_to_front(app).await
}

#[tauri::command]
async fn pick_and_send_file(app: tauri::AppHandle) -> Result<(), String> {
    sync::pick_and_send_file(app).await
}

#[tauri::command]
async fn capture_screenshot_and_send(app: tauri::AppHandle) -> Result<(), String> {
    sync::capture_screenshot_and_send(app).await
}

#[tauri::command]
async fn save_received_file(name: String, data: String, app: tauri::AppHandle) -> Result<String, String> {
    sync::save_received_file(name, data, app).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(sync::SyncState::default())
        .invoke_handler(tauri::generate_handler![
            get_host_name,
            start_host,
            stop_host,
            start_browse,
            stop_browse,
            connect_to,
            disconnect,
            send_clipboard,
            send_bring_to_front,
            pick_and_send_file,
            capture_screenshot_and_send,
            save_received_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
