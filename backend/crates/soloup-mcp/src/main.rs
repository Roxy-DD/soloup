//! soloup-mcp —— MCP (Model Context Protocol) server.
//!
//! 双传输模式：
//! - stdio（默认）：JSON-RPC 2.0 over stdin/stdout，供 Claude Desktop 等本地 AI 客户端
//! - HTTP：axum HTTP server，供 Tauri sidecar 和网络 AI 客户端
//!
//! 协议版本：2024-11-05

use std::io::{self, BufRead, Write};
use std::sync::{Arc, Mutex};

use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use soloup_solver::Solver;
use soloup_store::open_store;

mod http;
mod tools;

// ─── CLI 参数 ──────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "soloup-mcp", about = "Soloup MCP Server")]
struct Args {
    /// 传输模式: stdio 或 http
    #[arg(long, default_value = "stdio")]
    transport: String,

    /// HTTP 模式端口号
    #[arg(long, default_value_t = 8788)]
    port: u16,
}

// ─── JSON-RPC 2.0 协议类型 ───────────────────────────────────────────────────

#[derive(Deserialize)]
#[allow(dead_code)]
pub(crate) struct JsonRpcRequest {
    pub(crate) jsonrpc: String,
    pub(crate) id: Option<Value>,
    pub(crate) method: String,
    #[serde(default)]
    pub(crate) params: Value,
}

#[derive(Serialize, Clone)]
pub(crate) struct JsonRpcResponse {
    pub(crate) jsonrpc: String,
    pub(crate) id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) error: Option<JsonRpcError>,
}

#[derive(Serialize, Clone)]
pub(crate) struct JsonRpcError {
    pub(crate) code: i32,
    pub(crate) message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) data: Option<Value>,
}

// ─── MCP 协议类型 ────────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[allow(dead_code)]
struct InitializeParams {
    #[serde(rename = "protocolVersion")]
    protocol_version: String,
    #[serde(default)]
    capabilities: Value,
}

#[derive(Deserialize)]
struct ToolCallParams {
    name: String,
    #[serde(default)]
    arguments: Value,
}

// ─── 应用状态 ────────────────────────────────────────────────────────────────

pub(crate) type AppState = Arc<Mutex<Solver>>;

// ─── 主函数 ──────────────────────────────────────────────────────────────────

fn main() {
    let args = Args::parse();

    let mut store = open_store(Default::default()).expect("打开数据库失败");

    let seeded = store
        .settings()
        .get("seeded")
        .unwrap_or(None)
        .is_some();
    if !seeded {
        soloup_server::seed::seed_demo(&mut store).expect("写入演示种子失败");
        eprintln!("[soloup-mcp] 首次启动：已写入演示种子");
    }

    let state: AppState = Arc::new(Mutex::new(Solver::new(store)));

    match args.transport.as_str() {
        "http" => http::start_http_transport(state, args.port),
        _ => run_stdio(state),
    }
}

// ─── stdio 传输 ──────────────────────────────────────────────────────────────

fn run_stdio(state: AppState) {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout_lock = stdout.lock();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("[soloup-mcp] stdin 读取错误：{}", e);
                continue;
            }
        };

        if line.trim().is_empty() {
            continue;
        }

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let response = JsonRpcResponse {
                    jsonrpc: "2.0".into(),
                    id: None,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: format!("JSON 解析错误：{}", e),
                        data: None,
                    }),
                };
                writeln!(stdout_lock, "{}", serde_json::to_string(&response).unwrap()).unwrap();
                stdout_lock.flush().unwrap();
                continue;
            }
        };

        let response = handle_request(&state, &request);

        writeln!(stdout_lock, "{}", serde_json::to_string(&response).unwrap()).unwrap();
        stdout_lock.flush().unwrap();
    }
}

// ─── 请求处理（stdio 和 HTTP 共用） ──────────────────────────────────────────

pub(crate) fn handle_request(state: &AppState, request: &JsonRpcRequest) -> JsonRpcResponse {
    let id = request.id.clone();

    match request.method.as_str() {
        "initialize" => handle_initialize(request, id),
        "tools/list" => handle_tools_list(id),
        "tools/call" => handle_tools_call(state, request, id),
        _ => JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: format!("未知方法：{}", request.method),
                data: None,
            }),
        },
    }
}

fn handle_initialize(request: &JsonRpcRequest, id: Option<Value>) -> JsonRpcResponse {
    let _params: InitializeParams = match serde_json::from_value(request.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse {
                jsonrpc: "2.0".into(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: format!("initialize 参数错误：{}", e),
                    data: None,
                }),
            };
        }
    };

    JsonRpcResponse {
        jsonrpc: "2.0".into(),
        id,
        result: Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "soloup-mcp",
                "version": "0.1.0"
            }
        })),
        error: None,
    }
}

fn handle_tools_list(id: Option<Value>) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".into(),
        id,
        result: Some(json!({
            "tools": tools::get_tool_definitions()
        })),
        error: None,
    }
}

fn handle_tools_call(state: &AppState, request: &JsonRpcRequest, id: Option<Value>) -> JsonRpcResponse {
    let params: ToolCallParams = match serde_json::from_value(request.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse {
                jsonrpc: "2.0".into(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: format!("tools/call 参数错误：{}", e),
                    data: None,
                }),
            };
        }
    };

    let result = tools::call_tool(state, &params.name, &params.arguments);

    match result {
        Ok(data) => JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id,
            result: Some(json!({
                "content": [
                    {
                        "type": "text",
                        "text": serde_json::to_string_pretty(&data).unwrap_or_else(|_| data.to_string())
                    }
                ]
            })),
            error: None,
        },
        Err(e) => JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id,
            result: Some(json!({
                "content": [
                    {
                        "type": "text",
                        "text": format!("错误：{}", e.message)
                    }
                ],
                "isError": true
            })),
            error: None,
        },
    }
}
