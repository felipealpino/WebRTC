// Servidor HTTP (páginas) + WebSocket (sinalização WebRTC).
// O vídeo não passa por aqui: vai direto do transmissor para cada espectador.
use std::collections::HashMap;
use std::net::{SocketAddr, TcpListener, UdpSocket};
use std::sync::{Arc, Mutex};

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{ConnectInfo, State};
use axum::http::header;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{json, Value};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

const VIEWER_PAGE: &str = include_str!("../../public/index.html");
const BROADCAST_PAGE: &str = include_str!("../../public/broadcast.html");

pub struct Config {
    pub port: u16,
    fps: u64,
    max_bitrate: u64,
    ice_servers: Value,
}

impl Config {
    pub fn from_env() -> Self {
        let num = |key: &str, default: u64| std::env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default);
        Self {
            port: num("PORT", 8080) as u16,
            fps: num("FPS", 30),
            max_bitrate: num("BITRATE_MBPS", 8) * 1_000_000,
            // Para uso remoto: ICE_SERVERS='[{"urls":"stun:stun.l.google.com:19302"}]' (e um TURN, se necessário)
            ice_servers: std::env::var("ICE_SERVERS")
                .map(|v| serde_json::from_str(&v).expect("ICE_SERVERS não é um JSON válido"))
                .unwrap_or(json!([])),
        }
    }
}

type Tx = UnboundedSender<Message>;

#[derive(Default)]
struct Room {
    broadcaster: Option<(u64, Tx)>,
    viewers: HashMap<u64, Tx>,
    next_id: u64,
}

#[derive(Clone)]
struct AppState {
    room: Arc<Mutex<Room>>,
    client_config: Arc<Value>,
}

enum Role {
    None,
    Broadcaster,
    Viewer,
}

pub async fn run(listener: TcpListener, config: Config) {
    listener.set_nonblocking(true).unwrap();
    let listener = tokio::net::TcpListener::from_std(listener).unwrap();
    log!("Para assistir, abra no navegador: http://{}:{}", lan_ip(), config.port);

    let state = AppState {
        room: Default::default(),
        client_config: Arc::new(json!({
            "fps": config.fps,
            "maxBitrate": config.max_bitrate,
            "iceServers": config.ice_servers,
        })),
    };
    let app = Router::new()
        .route("/", get(|| async { no_store(Html(VIEWER_PAGE)) }))
        .route("/broadcast", get(|| async { no_store(Html(BROADCAST_PAGE)) }))
        .route("/config.json", get(|State(s): State<AppState>| async move { no_store(Json((*s.client_config).clone())) }))
        .route("/log", post(log_handler))
        .route("/ws", get(ws_handler))
        .with_state(state);

    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

fn no_store(body: impl IntoResponse) -> Response {
    ([(header::CACHE_CONTROL, "no-store")], body).into_response()
}

// IP da interface usada para sair para a rede (nenhum pacote é enviado).
fn lan_ip() -> String {
    UdpSocket::bind("0.0.0.0:0")
        .and_then(|s| s.connect("8.8.8.8:80").and(s.local_addr()))
        .map(|a| a.ip().to_string())
        .unwrap_or_else(|_| "localhost".into())
}

// A página do transmissor roda oculta; ela manda o status para aparecer no console.
async fn log_handler(ConnectInfo(addr): ConnectInfo<SocketAddr>, body: String) {
    if addr.ip().is_loopback() {
        log!("[transmissor] {body}");
    }
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, addr.ip().is_loopback(), state.room))
        .into_response()
}

async fn handle_socket(mut socket: WebSocket, is_local: bool, room: Arc<Mutex<Room>>) {
    let (tx, mut rx) = unbounded_channel::<Message>();
    let id = {
        let mut r = room.lock().unwrap();
        r.next_id += 1;
        r.next_id
    };
    let mut role = Role::None;

    loop {
        tokio::select! {
            Some(out) = rx.recv() => {
                let close = matches!(out, Message::Close(_));
                if socket.send(out).await.is_err() || close {
                    break;
                }
            }
            incoming = socket.recv() => match incoming {
                Some(Ok(Message::Text(text))) => {
                    let Ok(msg) = serde_json::from_str::<Value>(&text) else { continue };
                    if !handle_message(&msg, id, &mut role, is_local, &tx, &room) {
                        break;
                    }
                }
                Some(Ok(_)) => {}
                _ => break,
            },
        }
    }

    let mut r = room.lock().unwrap();
    match role {
        Role::Viewer => {
            r.viewers.remove(&id);
            if let Some((_, b)) = &r.broadcaster {
                send(b, json!({ "type": "viewer-left", "id": id }));
            }
            log!("Espectadores: {}", r.viewers.len());
        }
        Role::Broadcaster if r.broadcaster.as_ref().is_some_and(|(bid, _)| *bid == id) => {
            r.broadcaster = None;
            for v in r.viewers.values() {
                send(v, json!({ "type": "broadcaster-left" }));
            }
            log!("Transmissão parada.");
        }
        _ => {}
    }
}

/// Retorna `false` para encerrar a conexão.
fn handle_message(msg: &Value, id: u64, role: &mut Role, is_local: bool, tx: &Tx, room: &Mutex<Room>) -> bool {
    let mut r = room.lock().unwrap();
    match (&*role, msg["type"].as_str()) {
        // Só o próprio PC pode transmitir; qualquer um na rede pode assistir.
        (Role::None, Some("broadcaster")) => {
            if !is_local {
                return false;
            }
            if let Some((_, old)) = r.broadcaster.replace((id, tx.clone())) {
                let _ = old.send(Message::Close(None));
            }
            for viewer_id in r.viewers.keys() {
                send(tx, json!({ "type": "viewer-joined", "id": viewer_id }));
            }
            *role = Role::Broadcaster;
            log!("Transmissão iniciada.");
        }
        (Role::None, Some("viewer")) => {
            r.viewers.insert(id, tx.clone());
            match &r.broadcaster {
                Some((_, b)) => send(b, json!({ "type": "viewer-joined", "id": id })),
                None => send(tx, json!({ "type": "broadcaster-left" })),
            }
            *role = Role::Viewer;
            log!("Espectadores: {}", r.viewers.len());
        }
        (Role::Broadcaster, Some("offer")) => {
            if let Some(v) = msg["to"].as_u64().and_then(|to| r.viewers.get(&to)) {
                send(v, json!({ "type": "offer", "sdp": msg["sdp"] }));
            }
        }
        (Role::Viewer, Some("answer")) => {
            if let Some((_, b)) = &r.broadcaster {
                send(b, json!({ "type": "answer", "from": id, "sdp": msg["sdp"] }));
            }
        }
        _ => {}
    }
    true
}

fn send(tx: &Tx, msg: Value) {
    let _ = tx.send(Message::Text(msg.to_string().into()));
}
