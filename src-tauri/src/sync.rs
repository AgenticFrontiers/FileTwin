use base64::Engine;
use chrono::Local;
use futures_util::{SinkExt, StreamExt};
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use tauri_plugin_dialog::{DialogExt, FilePath};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio_tungstenite::{accept_async, connect_async, tungstenite::Message};

const SERVICE_TYPE: &str = "_remotesync._tcp.local.";
const WS_PORT: u16 = 18765;
const CONNECT_TIMEOUT_SECS: u64 = 15;
const CONNECT_MAX_ATTEMPTS: u32 = 3;

static HOSTING: AtomicBool = AtomicBool::new(false);
static BROWSING: AtomicBool = AtomicBool::new(false);

#[derive(Default)]
pub struct SyncState {
    pub host_tx: Mutex<Option<mpsc::Sender<String>>>,
    pub client_tx: Mutex<Option<mpsc::Sender<String>>>,
    pub peer_name: Mutex<Option<String>>,
    pub browse_receiver: Mutex<Option<mdns_sd::Receiver<ServiceEvent>>>,
    pub daemon: Mutex<Option<ServiceDaemon>>,
    pub service_info: Mutex<Option<ServiceInfo>>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum WsMessage {
    Hello { name: String },
    Clipboard { text: String },
    File { name: String, data: String },
    BringToFront,
}

fn emit_connected(app: &AppHandle, name: &str) {
    let _ = app.emit("connected", serde_json::json!({ "name": name }));
}

fn emit_disconnected(app: &AppHandle) {
    let _ = app.emit("disconnected", ());
}

fn emit_peers(app: &AppHandle, peers: Vec<Peer>) {
    let _ = app.emit("peers", peers);
}

fn emit_remote_clipboard(app: &AppHandle, text: &str) {
    let _ = app.emit("remote_clipboard", serde_json::json!({ "text": text }));
}

fn emit_remote_file(app: &AppHandle, name: &str, data: &str) {
    let _ = app.emit(
        "remote_file",
        serde_json::json!({ "name": name, "data": data }),
    );
}

fn emit_bring_to_front(app: &AppHandle) {
    let _ = app.emit("bring_to_front", ());
}

#[derive(Clone, Serialize)]
pub struct Peer {
    pub name: String,
    pub host: String,
    pub port: u16,
}

fn send_via_tx(tx: &Mutex<Option<mpsc::Sender<String>>>, msg: &WsMessage) -> Result<(), String> {
    let json = serde_json::to_string(msg).map_err(|e| e.to_string())?;
    let guard = tx.lock().map_err(|_| "lock failed")?;
    if let Some(sender) = guard.as_ref() {
        tauri::async_runtime::block_on(async move {
            sender.send(json).await.map_err(|_| "send failed".to_string())
        })
    } else {
        Err("Not connected".to_string())
    }
}

pub async fn start_host(app: AppHandle) -> Result<(), String> {
    if HOSTING.swap(true, Ordering::SeqCst) {
        return Err("Already hosting".to_string());
    }

    let host_name = hostname::get()
        .map(|h| h.to_string_lossy().into_owned())
        .unwrap_or_else(|_| "Mac".to_string());

    let listener = TcpListener::bind(("0.0.0.0", WS_PORT))
        .await
        .map_err(|e| e.to_string())?;

    let local_ip = local_ip_address::local_ip()
        .map(|ip| ip.to_string())
        .unwrap_or_else(|_| "127.0.0.1".to_string());
    let host_domain = format!("{}.local.", host_name);

    let daemon = ServiceDaemon::new().map_err(|e| e.to_string())?;
    let service_name = format!("RemoteSync-{}", host_name);
    let service_info = ServiceInfo::new(
        SERVICE_TYPE,
        &service_name,
        &host_domain,
        local_ip.as_str(),
        WS_PORT,
        &[] as &[(&str, &str)],
    )
    .map_err(|e| e.to_string())?
    .enable_addr_auto();
    daemon.register(service_info.clone()).map_err(|e| e.to_string())?;

    if let Some(state) = app.try_state::<SyncState>() {
        *state.daemon.lock().map_err(|_| "lock")? = Some(daemon);
        *state.service_info.lock().map_err(|_| "lock")? = Some(service_info);
    }

    let app_accept = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Ok((stream, _)) = listener.accept().await {
            if let Ok(ws) = accept_async(stream).await {
                let (mut write, mut read) = ws.split();
                let (tx, mut rx) = mpsc::channel::<String>(32);

                if let Some(state) = app_accept.try_state::<SyncState>() {
                    *state.host_tx.lock().unwrap() = Some(tx.clone());
                }

                tauri::async_runtime::spawn(async move {
                    while let Some(msg) = rx.recv().await {
                        if write.send(Message::Text(msg)).await.is_err() {
                            break;
                        }
                    }
                });

                while let Some(Ok(msg)) = read.next().await {
                    if let Message::Text(text) = msg {
                        if let Ok(parsed) = serde_json::from_str::<WsMessage>(&text) {
                            match parsed {
                                WsMessage::Hello { name } => {
                                    if let Some(state) = app_accept.try_state::<SyncState>() {
                                        *state.peer_name.lock().unwrap() = Some(name.clone());
                                    }
                                    emit_connected(&app_accept, &name);
                                }
                                WsMessage::Clipboard { text: t } => emit_remote_clipboard(&app_accept, &t),
                                WsMessage::File { name, data } => emit_remote_file(&app_accept, &name, &data),
                                WsMessage::BringToFront => emit_bring_to_front(&app_accept),
                                _ => {}
                            }
                        }
                    }
                }

                if let Some(state) = app_accept.try_state::<SyncState>() {
                    *state.host_tx.lock().unwrap() = None;
                    *state.peer_name.lock().unwrap() = None;
                }
                emit_disconnected(&app_accept);
            }
        }
        HOSTING.store(false, Ordering::SeqCst);
    });

    Ok(())
}

pub async fn stop_host(app: AppHandle) -> Result<(), String> {
    HOSTING.store(false, Ordering::SeqCst);
    if let Some(state) = app.try_state::<SyncState>() {
        *state.host_tx.lock().map_err(|_| "lock")? = None;
        *state.daemon.lock().map_err(|_| "lock")? = None;
        *state.service_info.lock().map_err(|_| "lock")? = None;
    }
    Ok(())
}

pub async fn start_browse(app: AppHandle) -> Result<(), String> {
    if BROWSING.swap(true, Ordering::SeqCst) {
        return Ok(());
    }

    let daemon = ServiceDaemon::new().map_err(|e| e.to_string())?;
    let receiver = daemon.browse(SERVICE_TYPE).map_err(|e| e.to_string())?;

    if let Some(state) = app.try_state::<SyncState>() {
        *state.daemon.lock().map_err(|_| "lock")? = Some(daemon);
        *state.browse_receiver.lock().map_err(|_| "lock")? = Some(receiver);
    }

    let app_browse = app.clone();
    std::thread::spawn(move || {
        let state = match app_browse.try_state::<SyncState>() {
            Some(s) => s,
            None => return,
        };
        let mut guard = match state.browse_receiver.lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        let receiver = match guard.as_ref() {
            Some(r) => r.clone(),
            None => return,
        };
        drop(guard);

        let mut peers: HashMap<String, Peer> = HashMap::new();
        while BROWSING.load(Ordering::SeqCst) {
            if let Ok(event) = receiver.recv_timeout(Duration::from_millis(500)) {
                match event {
                    ServiceEvent::ServiceResolved(resolved) => {
                        let host = resolved.host.clone();
                        let port = resolved.port;
                        let fullname = resolved.fullname.clone();
                        let short = fullname
                            .strip_prefix("RemoteSync-")
                            .and_then(|s| s.split('.').next())
                            .unwrap_or(&fullname)
                            .to_string();
                        peers.insert(fullname, Peer {
                            name: short,
                            host,
                            port,
                        });
                        let list: Vec<Peer> = peers.values().cloned().collect();
                        emit_peers(&app_browse, list);
                    }
                    ServiceEvent::ServiceRemoved(_, fullname) => {
                        peers.remove(&fullname);
                        let list: Vec<Peer> = peers.values().cloned().collect();
                        emit_peers(&app_browse, list);
                    }
                    _ => {}
                }
            }
        }
        // Cleanup is done by stop_browse() when it clears state
    });

    Ok(())
}

pub async fn stop_browse(app: AppHandle) -> Result<(), String> {
    BROWSING.store(false, Ordering::SeqCst);
    tokio::time::sleep(Duration::from_millis(600)).await;
    if let Some(state) = app.try_state::<SyncState>() {
        *state.browse_receiver.lock().map_err(|_| "lock")? = None;
        *state.daemon.lock().map_err(|_| "lock")? = None;
    }
    Ok(())
}

pub async fn connect_to(host: String, port: u16, app: AppHandle) -> Result<(), String> {
    let url = format!("ws://{}:{}", host, port);
    let timeout_msg = "Connection timed out. Check that both Macs are on the same network and the other device is sharing.";

    let (mut write, mut read) = {
        let mut last_error = String::new();
        let mut ws_stream = None;

        for attempt in 1..=CONNECT_MAX_ATTEMPTS {
            let connect_fut = connect_async(&url);
            match tokio::time::timeout(
                Duration::from_secs(CONNECT_TIMEOUT_SECS),
                connect_fut,
            )
            .await
            {
                Ok(Ok(stream)) => {
                    ws_stream = Some(stream);
                    break;
                }
                Ok(Err(e)) => last_error = e.to_string(),
                Err(_) => last_error = timeout_msg.to_string(),
            }
            if attempt < CONNECT_MAX_ATTEMPTS {
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }

        let (stream, _) = ws_stream.ok_or_else(|| {
            format!(
                "Failed after {} attempts. {}",
                CONNECT_MAX_ATTEMPTS, last_error
            )
        })?;
        stream.split()
    };

    let my_name = hostname::get()
        .map(|h| h.to_string_lossy().into_owned())
        .unwrap_or_else(|_| "Mac".to_string());

    let hello = serde_json::to_string(&WsMessage::Hello { name: my_name.clone() }).unwrap();
    write
        .send(Message::Text(hello))
        .await
        .map_err(|e| e.to_string())?;

    let (tx, mut rx) = mpsc::channel::<String>(32);
    if let Some(state) = app.try_state::<SyncState>() {
        *state.client_tx.lock().unwrap() = Some(tx);
        *state.peer_name.lock().unwrap() = Some(host.clone());
    }
    emit_connected(&app, &host);

    let app_recv = app.clone();
    tauri::async_runtime::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if write.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    let app_read = app.clone();
    tauri::async_runtime::spawn(async move {
        while let Some(Ok(msg)) = read.next().await {
            if let Message::Text(text) = msg {
                if let Ok(parsed) = serde_json::from_str::<WsMessage>(&text) {
                    match parsed {
                        WsMessage::Clipboard { text: t } => emit_remote_clipboard(&app_read, &t),
                        WsMessage::File { name, data } => emit_remote_file(&app_read, &name, &data),
                        WsMessage::BringToFront => emit_bring_to_front(&app_read),
                        _ => {}
                    }
                }
            }
        }
        if let Some(state) = app_read.try_state::<SyncState>() {
            *state.client_tx.lock().unwrap() = None;
            *state.peer_name.lock().unwrap() = None;
        }
        emit_disconnected(&app_read);
    });

    Ok(())
}

pub async fn disconnect(app: AppHandle) -> Result<(), String> {
    if let Some(state) = app.try_state::<SyncState>() {
        *state.host_tx.lock().map_err(|_| "lock")? = None;
        *state.client_tx.lock().map_err(|_| "lock")? = None;
        *state.peer_name.lock().map_err(|_| "lock")? = None;
    }
    emit_disconnected(&app);
    Ok(())
}

pub async fn send_clipboard(text: String, app: AppHandle) -> Result<(), String> {
    let msg = WsMessage::Clipboard { text };
    if let Some(state) = app.try_state::<SyncState>() {
        if let Some(tx) = state.host_tx.lock().ok().and_then(|g| g.clone()) {
            let json = serde_json::to_string(&msg).map_err(|e| e.to_string())?;
            tx.send(json).await.map_err(|_| "Send failed".to_string())?;
            return Ok(());
        }
        if let Some(tx) = state.client_tx.lock().ok().and_then(|g| g.clone()) {
            let json = serde_json::to_string(&msg).map_err(|e| e.to_string())?;
            tx.send(json).await.map_err(|_| "Send failed".to_string())?;
            return Ok(());
        }
    }
    Err("Not connected".to_string())
}

pub async fn send_bring_to_front(app: AppHandle) -> Result<(), String> {
    let msg = WsMessage::BringToFront;
    if let Some(state) = app.try_state::<SyncState>() {
        if let Some(tx) = state.host_tx.lock().ok().and_then(|g| g.clone()) {
            let json = serde_json::to_string(&msg).map_err(|e| e.to_string())?;
            tx.send(json).await.map_err(|_| "Send failed".to_string())?;
            return Ok(());
        }
        if let Some(tx) = state.client_tx.lock().ok().and_then(|g| g.clone()) {
            let json = serde_json::to_string(&msg).map_err(|e| e.to_string())?;
            tx.send(json).await.map_err(|_| "Send failed".to_string())?;
            return Ok(());
        }
    }
    Err("Not connected".to_string())
}

pub async fn pick_and_send_file(app: AppHandle) -> Result<(), String> {
    let path = app.dialog().file().blocking_pick_file();

    let path = match path {
        Some(p) => p,
        None => return Ok(()),
    };

    let path_buf = match path {
        FilePath::Path(p) => p,
        _ => return Err("Invalid path".to_string()),
    };
    let bytes = tokio::fs::read(&path_buf).await.map_err(|e| e.to_string())?;
    let name = path_buf
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file")
        .to_string();
    let data = base64::engine::general_purpose::STANDARD.encode(&bytes);

    let msg = WsMessage::File { name, data };
    if let Some(state) = app.try_state::<SyncState>() {
        if let Some(tx) = state.host_tx.lock().ok().and_then(|g| g.clone()) {
            let json = serde_json::to_string(&msg).map_err(|e| e.to_string())?;
            tx.send(json).await.map_err(|_| "Send failed".to_string())?;
            return Ok(());
        }
        if let Some(tx) = state.client_tx.lock().ok().and_then(|g| g.clone()) {
            let json = serde_json::to_string(&msg).map_err(|e| e.to_string())?;
            tx.send(json).await.map_err(|_| "Send failed".to_string())?;
            return Ok(());
        }
    }
    Err("Not connected".to_string())
}

#[cfg(target_os = "macos")]
fn capture_screenshot_to_jpg() -> Result<(PathBuf, String), String> {
    let name = format!("screenshot_{}.jpg", Local::now().format("%Y-%m-%d_%H-%M-%S"));
    let temp_dir = std::env::temp_dir();
    let path = temp_dir.join(&name);

    let status = Command::new("screencapture")
        .args(["-i", "-x", "-t", "jpg", path.to_str().unwrap()])
        .status()
        .map_err(|e| e.to_string())?;

    if !status.success() {
        return Err("Screenshot cancelled or failed".to_string());
    }

    Ok((path, name))
}

#[cfg(not(target_os = "macos"))]
fn capture_screenshot_to_jpg() -> Result<(PathBuf, String), String> {
    Err("Screenshot capture is only supported on macOS".to_string())
}

fn send_file_bytes(app: &AppHandle, name: String, bytes: &[u8]) -> Result<(), String> {
    let data = base64::engine::general_purpose::STANDARD.encode(bytes);
    let msg = WsMessage::File { name, data };
    if let Some(state) = app.try_state::<SyncState>() {
        if let Some(tx) = state.host_tx.lock().ok().and_then(|g| g.clone()) {
            let json = serde_json::to_string(&msg).map_err(|e| e.to_string())?;
            tauri::async_runtime::block_on(async move {
                tx.send(json).await.map_err(|_| "Send failed".to_string())
            })?;
            return Ok(());
        }
        if let Some(tx) = state.client_tx.lock().ok().and_then(|g| g.clone()) {
            let json = serde_json::to_string(&msg).map_err(|e| e.to_string())?;
            tauri::async_runtime::block_on(async move {
                tx.send(json).await.map_err(|_| "Send failed".to_string())
            })?;
            return Ok(());
        }
    }
    Err("Not connected".to_string())
}

pub async fn capture_screenshot_and_send(app: AppHandle) -> Result<(), String> {
    let result = tokio::task::spawn_blocking(capture_screenshot_to_jpg)
        .await
        .map_err(|e| e.to_string())?;
    let (path, name) = result?;

    let bytes = tokio::fs::read(&path).await.map_err(|e| e.to_string())?;
    let _ = std::fs::remove_file(&path);

    send_file_bytes(&app, name, &bytes)
}

pub async fn save_received_file(name: String, data: String, app: AppHandle) -> Result<String, String> {
    let path = app
        .dialog()
        .file()
        .set_file_name(&name)
        .blocking_save_file();

    let path = match path {
        Some(FilePath::Path(p)) => p,
        Some(_) => return Err("Invalid path".to_string()),
        None => return Err("Cancelled".to_string()),
    };

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&data)
        .map_err(|e| e.to_string())?;
    tokio::fs::write(&path, &bytes)
        .await
        .map_err(|e| e.to_string())?;

    Ok(path.to_string_lossy().into_owned())
}
