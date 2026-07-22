// SPDX-License-Identifier: MIT
// Copyright (c) 2026 WaveCord contributors

//! Node statistics models.

use serde::{Deserialize, Serialize};

/// `stats` op payload with periodic node health metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    pub players: u32,
    pub playing_players: u32,
    pub uptime: u64,
    pub memory: Memory,
    pub cpu: Cpu,
    /// Only present while at least one player is active.
    #[serde(default)]
    pub frame_stats: Option<FrameStats>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Memory {
    pub free: u64,
    pub used: u64,
    pub allocated: u64,
    pub reservable: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cpu {
    pub cores: u32,
    pub system_load: f64,
    pub lavalink_load: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FrameStats {
    pub sent: i64,
    pub nulled: i64,
    pub deficit: i64,
}
