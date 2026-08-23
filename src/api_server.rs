use crate::error::CompileError;
use crate::fortress;
use crate::obfuscator::{self, ObfuscateMode, ObfuscateOptions};
use axum::{
    extract::State,
    http::{Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

#[derive(Clone)]
pub struct ApiState {
    pub sessions: Arc<Mutex<HashMap<String, u32>>>,
}

#[derive(Debug, Deserialize)]
pub struct ObfuscateRequest {
    pub text: String,
    #[serde(default)]
    pub mode: String,
}

#[derive(Debug, Serialize)]
pub struct ObfuscateResponse {
    pub payload: String,
    pub nonce: String,
}

#[derive(Debug, Deserialize)]
pub struct RelayRequest {
    pub session_id: Option<String>,
    pub round: u32,
    pub payload: String,
}

#[derive(Debug, Serialize)]
pub struct RelayResponse {
    pub session_id: String,
    pub round: u32,
    pub payload: String,
    pub done: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

pub async fn run_api(port: u16) -> std::result::Result<(), CompileError> {
    let state = ApiState {
        sessions: Arc::new(Mutex::new(HashMap::new())),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/obfuscate", post(api_obfuscate))
        .route("/api/reveal", post(api_reveal))
        .route("/api/relay", post(api_relay))
        .with_state(state)
        .layer(cors);

    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| CompileError::new("E060", e.to_string(), 0, 0))?;

    eprintln!("난독증 API → http://127.0.0.1:{port}");
    axum::serve(listener, app)
        .await
        .map_err(|e| CompileError::new("E060", e.to_string(), 0, 0))?;
    Ok(())
}

async fn health() -> impl IntoResponse {
    Json(serde_json::json!({ "ok": true, "service": "nandokjeung-api" }))
}

async fn api_obfuscate(
    Json(req): Json<ObfuscateRequest>,
) -> std::result::Result<impl IntoResponse, ApiError> {
    let nonce = Uuid::new_v4().to_string();
    let payload = if req.mode == "fortress" {
        fortress::fortress_obfuscate(&req.text, &nonce)?
    } else {
        let mode = parse_mode(&req.mode)?;
        obfuscator::obfuscate_text(
            &req.text,
            &ObfuscateOptions {
                mode,
                split_chunks: 4,
            },
        )
    };
    Ok(Json(ObfuscateResponse { payload, nonce }))
}

async fn api_reveal(
    Json(req): Json<RelayRequest>,
) -> std::result::Result<impl IntoResponse, ApiError> {
    let plain = fortress::fortress_reveal(&req.payload)?;
    Ok(Json(serde_json::json!({ "plain": plain })))
}

async fn api_relay(
    State(state): State<ApiState>,
    Json(req): Json<RelayRequest>,
) -> std::result::Result<impl IntoResponse, ApiError> {
    let session_id = req
        .session_id
        .clone()
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let mut sessions = state.sessions.lock().unwrap();
    let stored_round = sessions.entry(session_id.clone()).or_insert(0);
    let current = *stored_round;

    if req.round != current {
        return Err(ApiError::bad_request(format!(
            "round mismatch: expected {current}, got {}",
            req.round
        )));
    }

    let inner = fortress::fortress_reveal(&req.payload)?;
    let inner_json: serde_json::Value = serde_json::from_str(&inner).unwrap_or_else(|_| {
        serde_json::json!({ "raw": inner })
    });

    let (next_payload, done, hint) = match current {
        0 => {
            // init: client sends obfuscated bootstrap
            let ack = serde_json::json!({
                "op": "ack",
                "message": "session ready",
                "session": session_id,
            });
            let nonce = Uuid::new_v4().to_string();
            let payload = fortress::fortress_obfuscate(&ack.to_string(), &nonce)?;
            (payload, false, Some("round 1: send obfuscated text".into()))
        }
        1 => {
            let text = inner_json
                .get("text")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ApiError::bad_request("round 1 expects {text}"))?;
            let result = serde_json::json!({
                "op": "result",
                "plain_len": text.len(),
                "preview": &text[..text.len().min(32)],
            });
            let nonce = Uuid::new_v4().to_string();
            let payload = fortress::fortress_obfuscate(&result.to_string(), &nonce)?;
            (payload, false, Some("round 2: confirm".into()))
        }
        2 => {
            let confirm = inner_json
                .get("confirm")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let final_msg = serde_json::json!({
                "op": "done",
                "ok": confirm,
                "message": "난독증 순환 해독 완료",
            });
            let nonce = Uuid::new_v4().to_string();
            let payload = fortress::fortress_obfuscate(&final_msg.to_string(), &nonce)?;
            (payload, true, None)
        }
        _ => return Err(ApiError::bad_request("session complete")),
    };

    *stored_round = current + 1;

    Ok(Json(RelayResponse {
        session_id,
        round: current + 1,
        payload: next_payload,
        done,
        hint,
    }))
}

fn parse_mode(mode: &str) -> std::result::Result<ObfuscateMode, CompileError> {
    match mode.to_lowercase().as_str() {
        "print" => Ok(ObfuscateMode::Print),
        "heavy" => Ok(ObfuscateMode::Heavy),
        "literal" => Ok(ObfuscateMode::Literal),
        "fortress" => Ok(ObfuscateMode::Heavy),
        other => Err(CompileError::new(
            "E070",
            format!("unknown mode '{other}'"),
            0,
            0,
        )),
    }
}

#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn bad_request(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: msg.into(),
        }
    }
}

impl From<CompileError> for ApiError {
    fn from(e: CompileError) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: e.to_string(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let body = serde_json::json!({ "error": self.message });
        (self.status, Json(body)).into_response()
    }
}
