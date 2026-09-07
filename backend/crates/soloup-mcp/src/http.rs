//! HTTP 传输层 — axum-based MCP over HTTP.
//!
//! 端点：
//! - POST /mcp   — JSON-RPC 请求（initialize, tools/list, tools/call）
//! - GET  /health — 健康检查

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use tower_http::cors::CorsLayer;

use crate::{handle_request, AppState, JsonRpcRequest, JsonRpcResponse};

pub fn start_http_transport(state: AppState, port: u16) {
    let app = Router::new()
        .route("/mcp", post(mcp_handler))
        .route("/health", get(health_handler))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let rt = tokio::runtime::Runtime::new().expect("创建 tokio runtime 失败");
    rt.block_on(async {
        let addr = format!("127.0.0.1:{}", port);
        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .unwrap_or_else(|e| panic!("绑定 {} 失败：{}", addr, e));
        eprintln!("[soloup-mcp] HTTP 传输监听 http://{}", addr);
        axum::serve(listener, app).await.unwrap();
    });
}

async fn mcp_handler(
    State(state): State<AppState>,
    Json(request): Json<JsonRpcRequest>,
) -> Json<JsonRpcResponse> {
    Json(handle_request(&state, &request))
}

async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}
