// SPDX-License-Identifier: MIT
// Copyright (c) 2026 WaveCord contributors

//! Error and result types.

use thiserror::Error;

/// Everything that can go wrong talking to a Lavalink node.
#[derive(Debug, Error)]
pub enum Error {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("websocket error: {0}")]
    Ws(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("invalid header value: {0}")]
    Header(String),

    #[error("connection closed before the node sent its `ready` op")]
    ClosedBeforeReady,

    #[error("node not connected yet (no session id)")]
    NotConnected,

    #[error("lavalink returned status {status}: {body}")]
    Api { status: u16, body: String },

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;
