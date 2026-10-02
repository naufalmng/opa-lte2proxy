use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use serde::Deserialize;
use serde_json::json;
use std::net::SocketAddr;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tracing::info;

use crate::modem::ModemController;
use crate::socks5::Socks5Server;

#[derive(Clone)]
pub struct AppState {
    pub modem: ModemController,
    pub socks: Arc<Socks5Server>,
}

#[derive(Deserialize, Debug, Default)]
pub struct RotateQuery {
    pub force: Option<bool>,
    pub session: Option<String>,
}

pub async fn run_api_server(
    addr: SocketAddr,
    state: AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = Router::new()
        .route("/status", get(get_status))
        .route("/ip", get(get_ip))
        .route("/rotate", get(trigger_rotate).post(trigger_rotate))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("REST API Control Server running on http://{}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}

async fn get_status(State(state): State<AppState>) -> impl IntoResponse {
    let modem_status = state.modem.get_status();
    let active_conns = state.socks.active_conns.load(Ordering::Relaxed);
    let total_conns = state.socks.total_conns.load(Ordering::Relaxed);

    Json(json!({
        "status": if modem_status.connected { "online" } else { "disconnected" },
        "modem": modem_status,
        "proxy": {
            "active_connections": active_conns,
            "total_connections": total_conns,
            "rotate_every_reqs": state.socks.rotate_every_reqs,
        }
    }))
}

async fn get_ip(State(state): State<AppState>) -> impl IntoResponse {
    let ip = state.modem.get_internal_ip();
    if ip.is_empty() {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "No cellular IP".to_string(),
        )
    } else {
        (StatusCode::OK, ip)
    }
}

async fn trigger_rotate(
    State(state): State<AppState>,
    Query(query): Query<RotateQuery>,
) -> impl IntoResponse {
    let force = query.force.unwrap_or(false);
    let session = query.session.unwrap_or_else(|| "default".to_string());

    info!(
        "[API] Rotate requested (session='{}', force={})",
        session, force
    );

    match state.modem.rotate_ip(force).await {
        Ok((old_ip, new_ip, elapsed_ms)) => (
            StatusCode::OK,
            Json(json!({
                "status": "success",
                "session": session,
                "old_ip": old_ip,
                "new_ip": new_ip,
                "duration_ms": elapsed_ms,
                "rotation_count": state.modem.rotation_count.load(Ordering::Relaxed)
            })),
        ),
        Err(e) => {
            let is_failover_block = e.starts_with("FAILOVER_GUARD");
            let code = if is_failover_block {
                StatusCode::LOCKED // 423 Locked
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };

            (
                code,
                Json(json!({
                    "status": "blocked",
                    "error": e,
                    "tip": if is_failover_block { "Kirim ?force=true jika ingin memaksa rotasi meski server sedang mode failover" } else { "Periksa koneksi ADB atau modem" }
                })),
            )
        }
    }
}
