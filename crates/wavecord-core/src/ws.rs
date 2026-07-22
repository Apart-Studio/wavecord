// SPDX-License-Identifier: MIT
// Copyright (c) 2026 WaveCord contributors

//! WebSocket handshake helpers for the Lavalink event stream (v3 and v4).

use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::handshake::client::Request;
use tokio_tungstenite::tungstenite::http::HeaderValue;

use crate::error::{Error, Result};
use crate::node::NodeConfig;
use crate::protocol::LavalinkVersion;

/// Build the WebSocket upgrade request with Lavalink's required headers for the
/// given version, optionally carrying a resume identifier so the node hands us
/// back the same session on reconnect (v4: `Session-Id`; v3: `Resume-Key`).
pub fn build_ws_request(
    config: &NodeConfig,
    version: LavalinkVersion,
    session_id: Option<&str>,
    resume_key: Option<&str>,
) -> Result<Request> {
    let url = config.ws_url(version);
    let mut req = url.into_client_request().map_err(Error::Ws)?;

    let headers = req.headers_mut();
    let set = |v: &str| HeaderValue::from_str(v).map_err(|e| Error::Header(e.to_string()));

    headers.insert("Authorization", set(&config.password)?);
    headers.insert("User-Id", set(&config.user_id)?);
    headers.insert("Client-Name", set(&config.client_name)?);
    match version {
        LavalinkVersion::V4 => {
            if let Some(sid) = session_id {
                headers.insert("Session-Id", set(sid)?);
            }
        }
        LavalinkVersion::V3 => {
            if let Some(key) = resume_key {
                headers.insert("Resume-Key", set(key)?);
            }
        }
    }
    Ok(req)
}
