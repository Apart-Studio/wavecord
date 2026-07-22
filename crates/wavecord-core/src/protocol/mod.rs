// SPDX-License-Identifier: MIT
// Copyright (c) 2026 WaveCord contributors

//! Version-neutral protocol handling for Lavalink v3 and v4.

pub mod v3;

use crate::error::{Error, Result};

/// Which Lavalink protocol a node speaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LavalinkVersion {
    V3,
    V4,
}

impl LavalinkVersion {
    /// Parse the plaintext body of `GET /version` (e.g. `"3.7.13"`, `"4.2.2"`).
    pub fn from_version_string(body: &str) -> Result<Self> {
        let major = body
            .trim()
            .split('.')
            .next()
            .and_then(|m| m.trim().parse::<u32>().ok())
            .ok_or_else(|| Error::Other(format!("unrecognized version string: {body:?}")))?;
        match major {
            3 => Ok(Self::V3),
            4 => Ok(Self::V4),
            other => Err(Error::Other(format!(
                "unsupported Lavalink major version {other}"
            ))),
        }
    }

    /// The WebSocket path for this version.
    pub fn ws_path(self) -> &'static str {
        match self {
            Self::V3 => "/v3/websocket",
            Self::V4 => "/v4/websocket",
        }
    }
}
