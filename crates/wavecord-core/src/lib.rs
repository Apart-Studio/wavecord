// SPDX-License-Identifier: MIT
// Copyright (c) 2026 WaveCord contributors

//! WaveCord core: a pure-Rust Lavalink v3/v4 client engine with no Python
//! dependency, so it can be tested and benchmarked on its own.

// Errors are on the cold path; boxing them to shrink the enum is not worth it.
#![allow(clippy::result_large_err, clippy::large_enum_variant)]

pub mod error;
pub mod model;
pub mod node;
pub mod protocol;
pub mod rest;
pub mod ws;

pub use error::{Error, Result};
pub use node::{Node, NodeConfig};
pub use protocol::LavalinkVersion;

use std::time::Duration;

/// Minimal async smoke test used to verify the async bridge from Python.
pub async fn ping() -> &'static str {
    tokio::time::sleep(Duration::from_millis(10)).await;
    "pong"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn ping_returns_pong() {
        assert_eq!(ping().await, "pong");
    }

    #[test]
    fn urls_are_built_correctly() {
        let cfg = NodeConfig {
            host: "localhost".into(),
            port: 2333,
            password: "pw".into(),
            user_id: "1".into(),
            client_name: "WaveCord/0.1".into(),
            ..Default::default()
        };
        assert_eq!(cfg.http_base(), "http://localhost:2333");
        assert_eq!(
            cfg.ws_url(LavalinkVersion::V4),
            "ws://localhost:2333/v4/websocket"
        );
        assert_eq!(
            cfg.ws_url(LavalinkVersion::V3),
            "ws://localhost:2333/v3/websocket"
        );
    }

    #[test]
    fn path_segments_are_percent_encoded() {
        use crate::rest::seg;
        assert_eq!(seg("123456789012345678"), "123456789012345678");
        assert!(!seg("../../secret").contains('/'));
        assert!(!seg("a/b?c#d").contains(['/', '?', '#']));
    }

    #[test]
    fn backoff_grows_then_caps() {
        use crate::node::backoff;
        assert_eq!(backoff(0), std::time::Duration::from_secs(1));
        assert_eq!(backoff(1), std::time::Duration::from_secs(2));
        assert_eq!(backoff(2), std::time::Duration::from_secs(4));
        assert_eq!(backoff(10), std::time::Duration::from_secs(30));
    }

    #[test]
    fn version_detection_from_string() {
        assert_eq!(
            LavalinkVersion::from_version_string("3.7.13").unwrap(),
            LavalinkVersion::V3
        );
        assert_eq!(
            LavalinkVersion::from_version_string("4.2.2").unwrap(),
            LavalinkVersion::V4
        );
        assert!(LavalinkVersion::from_version_string("2.0.0").is_err());
    }
}
