//! soloup-server 二进制入口：HTTP JSON-RPC（`/api/rpc`）+ 前端静态托管。
//!
//! 环境变量：
//! - `SOLOUP_PORT`    默认 8787
//! - `SOLOUP_DB_PATH` 默认 ~/.soloup/soloup.db
//! - `SOLOUP_WEB_DIR` 前端静态产物目录（`apps/web/out`），默认自动探测
//!
//! 关于监听地址：进程**始终**绑定 `0.0.0.0`，但非回环来源要经过 lan_gate 校验
//! settings 里的「局域网服务」开关。这样开关能即时生效，不必重启进程；
//! 关掉开关时局域网设备能连上端口、但拿不到任何数据。

use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};

use axum::extract::{ConnectInfo, State};
use axum::http::{Request, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};

use soloup_server::{dispatch, local_lan_ip, resolve_web_dir, seed, DEFAULT_SERVER_PORT, LAN_ENABLED_KEY};
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

/// 判断来源是否为本机回环（回环永远放行，不依赖开关）。
fn is_loopback(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => v4.is_loopback(),
        IpAddr::V6(v6) => {
            v6.is_loopback() || v6.to_ipv4_mapped().map(|v| v.is_loopback()).unwrap_or(false)
        }
    }
}

/// 局域网闸门：回环放行；其它来源必须显式开启「局域网服务」。
/// 开关状态每次请求实时从数据库读取，因此设置页一改即刻生效。
async fn lan_gate(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Response {
    if is_loopback(peer.ip()) {
        return next.run(req).await;
    }
    let allowed = {
        let solver = state.lock().unwrap();
        solver
            .store
            .settings()
            .get(LAN_ENABLED_KEY)
            .ok()
            .flatten()
            .as_deref()
            == Some("1")
    };
    if allowed {
        next.run(req).await
    } else {
        (
            StatusCode::FORBIDDEN,
            "局域网访问未开启：请在本机「设置 → 局域网服务」中打开开关",
        )
            .into_response()
    }
}

#[tokio::main]
async fn main() {
    let port: u16 = std::env::var("SOLOUP_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(DEFAULT_SERVER_PORT);

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

    // 前端静态产物：手机浏览器访问 http://<局域网IP>:<port>/ 直接拿到应用，
    // 与 API 同源同端口，免去 CORS 与额外的静态服务器。
    let web_dir = resolve_web_dir();
    let index = web_dir.join("index.html");
    let static_svc = ServeDir::new(&web_dir)
        .append_index_html_on_directories(true)
        .not_found_service(ServeFile::new(&index));

    let app = Router::new()
        .route("/api/rpc", post(rpc_handler))
        .route("/health", get(|| async { "ok" }))
        .fallback_service(static_svc)
        .layer(middleware::from_fn_with_state(state.clone(), lan_gate))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await.expect("绑定端口失败");

    println!("soloup-server 已启动");
    println!("  本机      http://localhost:{port}/");
    if let Some(ip) = local_lan_ip() {
        println!("  局域网    http://{ip}:{port}/   （需在设置中开启「局域网服务」）");
    } else {
        println!("  局域网    未探测到可用地址（可能未连接 Wi-Fi/网线）");
    }
    println!("  静态目录  {}", web_dir.display());
    if !index.is_file() {
        println!("  ⚠ 未找到 index.html —— 请先执行 pnpm --filter @soloup/web build");
    }

    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .expect("服务异常退出");
}
