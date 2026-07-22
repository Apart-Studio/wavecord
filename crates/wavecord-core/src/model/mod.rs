// SPDX-License-Identifier: MIT
// Copyright (c) 2026 WaveCord contributors

//! Serde models mirroring the Lavalink wire format.

pub mod events;
pub mod load;
pub mod player;
pub mod stats;
pub mod track;

pub use events::{Event, EventTrack, PlayerUpdate, Ready, ServerMessage, TrackEndReason};
pub use load::{Exception, LoadResult, Playlist, PlaylistInfo, Severity};
pub use player::{Player, PlayerState, UpdatePlayer, UpdatePlayerTrack, VoiceState};
pub use stats::{Cpu, FrameStats, Memory, Stats};
pub use track::{Track, TrackInfo};

#[cfg(test)]
mod tests;
