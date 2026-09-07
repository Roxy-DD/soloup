//! soloup-server 二进制入口：HTTP JSON-RPC（`/api/rpc`）。
//! 环境变量：`SOLOUP_PORT`（默认 8787）、`SOLOUP_DB_PATH`（默认 ~/.soloup/soloup.db）。

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use tower_http::cors::CorsLayer;

use soloup_server::{dispatch, seed};
use soloup_solver::Solver;
use soloup_store::open_store;

#[derive(Deserialize)]
struct RpcReq {
    op: String,
    #[serde(default)]
    args: Value,
}

type AppState = Arc<Mutex<Solver>>;

async fn rpc_handler(State(state): State<AppState>, Json(req): Json<RpcReq>) -> Json<Value> {
    // 作用域隔离：MutexGuard 不跨越 await，保证 handler future 是 Send。
    let result = {
        let mut solver = state.lock().unwrap();
        dispatch(&mut *solver, &req.op, &req.args)
    };
    match result {
        Ok(data) => Json(json!({ "ok": true, "data": data })),
        Err(e) => Json(json!({ "ok": false, "error": { "code": e.code, "message": e.message } })),
    }
}

#[tokio::main]
async fn main() {
    let port: u16 = std::env::var("SOLOUP_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8787);

    let mut store = open_store(Default::default()).expect("打开数据库失败");
    let seeded = store
        .settings()
        .get("seeded")
        .unwrap_or(None)
        .is_some();
    if !seeded {
        seed::seed_demo(&mut store).expect("写入演示种子失败");
        println!("首次启动：已写入演示种子");
    }

    let state: AppState = Arc::new(Mutex::new(Solver::new(store)));
    let app = Router::new()
        .route("/api/rpc", post(rpc_handler))
        .route("/health", get(|| async { "ok" }))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("soloup-server 监听 http://{addr}  (POST /api/rpc)");
    let listener = tokio::net::TcpListener::bind(addr).await.expect("绑定端口失败");
    axum::serve(listener, app).await.expect("服务异常退出");
}
