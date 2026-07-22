// SPDX-License-Identifier: MIT
// Copyright (c) 2026 WaveCord contributors

//! A single Lavalink node: version detection, a version-neutral command surface
//! for v3 and v4, and a background supervisor that reconnects and resumes.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

use crate::error::{Error, Result};
use crate::model::{
    LoadResult, Player, ServerMessage, UpdatePlayer, UpdatePlayerTrack, VoiceState,
};
use crate::protocol::{v3, LavalinkVersion};
use crate::rest::{seg, RestClient};
use crate::ws::build_ws_request;

/// Connection parameters for a node.
#[derive(Debug, Clone)]
pub struct NodeConfig {
    pub host: String,
    pub port: u16,
    pub password: String,
    pub secure: bool,
    pub user_id: String,
    pub client_name: String,
    pub session_id: Option<String>,
    /// Force a protocol version instead of auto-detecting via `GET /version`.
    pub force_version: Option<LavalinkVersion>,
    /// Reconnect automatically after an unexpected disconnect (default true).
    pub reconnect: bool,
    /// Configure session resuming so players survive a reconnect (default true).
    pub resume: bool,
    /// How long the node keeps players alive while we're gone, in seconds.
    pub resume_timeout: u64,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 2333,
            password: String::new(),
            secure: false,
            user_id: String::new(),
            client_name: concat!("WaveCord/", env!("CARGO_PKG_VERSION")).into(),
            session_id: None,
            force_version: None,
            reconnect: true,
            resume: true,
            resume_timeout: 60,
        }
    }
}

impl NodeConfig {
    fn scheme(&self, ws: bool) -> &'static str {
        match (ws, self.secure) {
            (true, true) => "wss",
            (true, false) => "ws",
            (false, true) => "https",
            (false, false) => "http",
        }
    }

    pub(crate) fn http_base(&self) -> String {
        format!("{}://{}:{}", self.scheme(false), self.host, self.port)
    }

    pub(crate) fn ws_url(&self, version: LavalinkVersion) -> String {
        format!(
            "{}://{}:{}{}",
            self.scheme(true),
            self.host,
            self.port,
            version.ws_path()
        )
    }
}

/// State shared between the public `Node` handle and its background supervisor.
struct Shared {
    config: NodeConfig,
    rest: RestClient,
    version: Mutex<Option<LavalinkVersion>>,
    session_id: Mutex<Option<String>>,
    /// v3 resume key (v4 resumes via the session id instead).
    resume_key: Mutex<Option<String>>,
    /// Sender to the *current* connection's writer task; swapped on reconnect.
    ws_tx: Mutex<Option<mpsc::UnboundedSender<Message>>>,
    /// Stable event sink carrying already-normalized JSON *strings* (v4 frames
    /// pass through as-is; v3 frames are rewritten to the canonical shape). The
    /// Python side decodes these with msgspec - far cheaper than building Python
    /// objects in Rust.
    events_tx: mpsc::UnboundedSender<String>,
    /// Whether a live WebSocket connection is currently up.
    connected: AtomicBool,
}

/// A connected (or connectable) Lavalink node.
pub struct Node {
    shared: Arc<Shared>,
    events: Mutex<Option<mpsc::UnboundedReceiver<String>>>,
}

impl Node {
    pub fn new(config: NodeConfig) -> Result<Self> {
        let rest = RestClient::new(config.http_base(), config.password.clone())?;
        let (events_tx, events_rx) = mpsc::unbounded_channel();
        let shared = Arc::new(Shared {
            version: Mutex::new(config.force_version),
            session_id: Mutex::new(config.session_id.clone()),
            resume_key: Mutex::new(None),
            ws_tx: Mutex::new(None),
            events_tx,
            connected: AtomicBool::new(false),
            rest,
            config,
        });
        Ok(Self {
            shared,
            events: Mutex::new(Some(events_rx)),
        })
    }

    /// Detect the version (unless forced), open the WebSocket, start the
    /// supervisor, and resolve once the node has sent its first `ready` op.
    pub async fn connect(&self) -> Result<()> {
        let version = match self.shared.config.force_version {
            Some(v) => v,
            None => {
                let body = self.shared.rest.version().await?;
                LavalinkVersion::from_version_string(&body)?
            }
        };
        *self.shared.version.lock().await = Some(version);

        let (ready_tx, ready_rx) = oneshot::channel();
        tokio::spawn(supervise(self.shared.clone(), version, ready_tx));
        // The supervisor sends Ok on first ready, or Err if the initial connect
        // fails (it does not retry before we've ever connected).
        ready_rx.await.map_err(|_| Error::ClosedBeforeReady)?
    }

    pub async fn version(&self) -> Option<LavalinkVersion> {
        *self.shared.version.lock().await
    }

    pub async fn session_id(&self) -> Option<String> {
        self.shared.session_id.lock().await.clone()
    }

    /// Whether a live WebSocket connection is currently up (false while the
    /// supervisor is between reconnect attempts).
    pub fn is_connected(&self) -> bool {
        self.shared.connected.load(Ordering::Relaxed)
    }

    /// Await the next server message as a normalized JSON string (decode it on
    /// the Python side with msgspec). Returns `None` once the connection closes.
    pub async fn recv_event(&self) -> Option<String> {
        let mut guard = self.events.lock().await;
        match guard.as_mut() {
            Some(rx) => rx.recv().await,
            None => None,
        }
    }

    /// Await at least one message, then drain up to `max_n` that are already
    /// queued (without waiting for more). Returns `None` once closed. Batching
    /// amortizes the per-call await + Python-boundary overhead under high load.
    pub async fn recv_events(&self, max_n: usize) -> Option<Vec<String>> {
        let mut guard = self.events.lock().await;
        let rx = guard.as_mut()?;
        let first = rx.recv().await?;
        let mut batch = Vec::with_capacity(max_n.max(1));
        batch.push(first);
        while batch.len() < max_n {
            match rx.try_recv() {
                Ok(msg) => batch.push(msg),
                Err(_) => break, // empty (or disconnected) - return what we have
            }
        }
        Some(batch)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn play(
        &self,
        guild_id: &str,
        encoded: &str,
        start_ms: Option<i64>,
        end_ms: Option<i64>,
        volume: Option<i32>,
        paused: Option<bool>,
        no_replace: bool,
    ) -> Result<Option<Player>> {
        match self.require_version().await? {
            LavalinkVersion::V4 => {
                let update = UpdatePlayer {
                    track: Some(UpdatePlayerTrack {
                        encoded: Some(Some(encoded.to_owned())),
                        ..Default::default()
                    }),
                    position: start_ms,
                    end_time: end_ms.map(Some),
                    volume,
                    paused,
                    ..Default::default()
                };
                self.rest_update(guild_id, &update, no_replace)
                    .await
                    .map(Some)
            }
            LavalinkVersion::V3 => {
                self.ws_send(v3::play_op(
                    guild_id, encoded, start_ms, end_ms, volume, paused, no_replace,
                ))
                .await?;
                Ok(None)
            }
        }
    }

    pub async fn stop(&self, guild_id: &str) -> Result<Option<Player>> {
        match self.require_version().await? {
            LavalinkVersion::V4 => self
                .rest_update(guild_id, &UpdatePlayer::stop(), false)
                .await
                .map(Some),
            LavalinkVersion::V3 => {
                self.ws_send(v3::stop_op(guild_id)).await?;
                Ok(None)
            }
        }
    }

    pub async fn set_pause(&self, guild_id: &str, paused: bool) -> Result<Option<Player>> {
        match self.require_version().await? {
            LavalinkVersion::V4 => {
                let update = UpdatePlayer {
                    paused: Some(paused),
                    ..Default::default()
                };
                self.rest_update(guild_id, &update, false).await.map(Some)
            }
            LavalinkVersion::V3 => {
                self.ws_send(v3::pause_op(guild_id, paused)).await?;
                Ok(None)
            }
        }
    }

    pub async fn set_volume(&self, guild_id: &str, volume: i32) -> Result<Option<Player>> {
        match self.require_version().await? {
            LavalinkVersion::V4 => {
                let update = UpdatePlayer {
                    volume: Some(volume),
                    ..Default::default()
                };
                self.rest_update(guild_id, &update, false).await.map(Some)
            }
            LavalinkVersion::V3 => {
                self.ws_send(v3::volume_op(guild_id, volume)).await?;
                Ok(None)
            }
        }
    }

    pub async fn seek(&self, guild_id: &str, position_ms: i64) -> Result<Option<Player>> {
        match self.require_version().await? {
            LavalinkVersion::V4 => {
                let update = UpdatePlayer {
                    position: Some(position_ms),
                    ..Default::default()
                };
                self.rest_update(guild_id, &update, false).await.map(Some)
            }
            LavalinkVersion::V3 => {
                self.ws_send(v3::seek_op(guild_id, position_ms)).await?;
                Ok(None)
            }
        }
    }

    /// Hand Lavalink the Discord voice connection. `channel_id` is required by
    /// Lavalink v4 (its `VoiceState` mandates the field); it is ignored on v3.
    pub async fn update_voice(
        &self,
        guild_id: &str,
        token: &str,
        endpoint: &str,
        session_id: &str,
        channel_id: Option<&str>,
    ) -> Result<Option<Player>> {
        match self.require_version().await? {
            LavalinkVersion::V4 => {
                let update = UpdatePlayer {
                    voice: Some(VoiceState {
                        token: token.to_owned(),
                        endpoint: endpoint.to_owned(),
                        session_id: session_id.to_owned(),
                        channel_id: channel_id.map(str::to_owned),
                    }),
                    ..Default::default()
                };
                self.rest_update(guild_id, &update, false).await.map(Some)
            }
            LavalinkVersion::V3 => {
                self.ws_send(v3::voice_update_op(guild_id, token, endpoint, session_id))
                    .await?;
                Ok(None)
            }
        }
    }

    pub async fn set_filters(
        &self,
        guild_id: &str,
        filters: serde_json::Value,
    ) -> Result<Option<Player>> {
        match self.require_version().await? {
            LavalinkVersion::V4 => {
                let update = UpdatePlayer {
                    filters: Some(filters),
                    ..Default::default()
                };
                self.rest_update(guild_id, &update, false).await.map(Some)
            }
            LavalinkVersion::V3 => {
                self.ws_send(v3::filters_op(guild_id, &filters)).await?;
                Ok(None)
            }
        }
    }

    pub async fn destroy_player(&self, guild_id: &str) -> Result<()> {
        match self.require_version().await? {
            LavalinkVersion::V4 => {
                let sid = self.require_session().await?;
                self.shared.rest.destroy_player(&sid, guild_id).await
            }
            LavalinkVersion::V3 => self.ws_send(v3::destroy_op(guild_id)).await,
        }
    }

    pub async fn load_tracks(&self, identifier: &str) -> Result<LoadResult> {
        match self.require_version().await? {
            LavalinkVersion::V4 => self.shared.rest.load_tracks_v4(identifier).await,
            LavalinkVersion::V3 => self.shared.rest.load_tracks_v3(identifier).await,
        }
    }

    /// Node info (version, source managers, and loaded plugins).
    pub async fn info(&self) -> Result<serde_json::Value> {
        let path = match self.require_version().await? {
            LavalinkVersion::V4 => "/v4/info",
            LavalinkVersion::V3 => "/v3/info",
        };
        self.shared.rest.get_value(path, &[]).await
    }

    /// Decode a base64 track string into its info without playing it.
    pub async fn decode_track(&self, encoded: &str) -> Result<serde_json::Value> {
        match self.require_version().await? {
            LavalinkVersion::V4 => {
                self.shared
                    .rest
                    .get_value("/v4/decodetrack", &[("encodedTrack", encoded)])
                    .await
            }
            LavalinkVersion::V3 => {
                self.shared
                    .rest
                    .get_value("/decodetrack", &[("track", encoded)])
                    .await
            }
        }
    }

    /// LavaSearch plugin: `GET /v4/loadsearch`. `types` is a comma-separated list
    /// such as `"track,album,artist,playlist,text"` (v4 only).
    pub async fn load_search(&self, query: &str, types: &str) -> Result<serde_json::Value> {
        self.shared
            .rest
            .get_value("/v4/loadsearch", &[("query", query), ("types", types)])
            .await
    }

    /// LavaLyrics plugin: lyrics for an encoded track (v4 only).
    pub async fn lyrics(
        &self,
        encoded: &str,
        skip_track_source: bool,
    ) -> Result<serde_json::Value> {
        let skip = if skip_track_source { "true" } else { "false" };
        self.shared
            .rest
            .get_value(
                "/v4/lyrics",
                &[("track", encoded), ("skipTrackSource", skip)],
            )
            .await
    }

    /// LavaLyrics plugin: lyrics for a guild's currently playing track (v4 only).
    pub async fn current_lyrics(
        &self,
        guild_id: &str,
        skip_track_source: bool,
    ) -> Result<serde_json::Value> {
        let sid = self.require_session().await?;
        let skip = if skip_track_source { "true" } else { "false" };
        self.shared
            .rest
            .get_value(
                &format!(
                    "/v4/sessions/{}/players/{}/lyrics",
                    seg(&sid),
                    seg(guild_id)
                ),
                &[("skipTrackSource", skip)],
            )
            .await
    }

    /// SponsorBlock plugin: set the categories to skip for a guild's player (v4).
    pub async fn set_sponsorblock_categories(
        &self,
        guild_id: &str,
        categories: Vec<String>,
    ) -> Result<()> {
        let sid = self.require_session().await?;
        let path = format!(
            "/v4/sessions/{}/players/{}/sponsorblock/categories",
            seg(&sid),
            seg(guild_id)
        );
        self.shared
            .rest
            .send(
                reqwest::Method::PUT,
                &path,
                Some(serde_json::to_value(categories)?),
            )
            .await
    }

    /// Route planner status (for balancing/failing over IP blocks).
    pub async fn routeplanner_status(&self) -> Result<serde_json::Value> {
        let path = match self.require_version().await? {
            LavalinkVersion::V4 => "/v4/routeplanner/status",
            LavalinkVersion::V3 => "/routeplanner/status",
        };
        self.shared.rest.get_value(path, &[]).await
    }

    /// Unmark a single failed address in the route planner.
    pub async fn routeplanner_free(&self, address: &str) -> Result<()> {
        let path = match self.require_version().await? {
            LavalinkVersion::V4 => "/v4/routeplanner/free/address",
            LavalinkVersion::V3 => "/routeplanner/free/address",
        };
        self.shared
            .rest
            .send(
                reqwest::Method::POST,
                path,
                Some(serde_json::json!({ "address": address })),
            )
            .await
    }

    /// Unmark all failed addresses in the route planner.
    pub async fn routeplanner_free_all(&self) -> Result<()> {
        let path = match self.require_version().await? {
            LavalinkVersion::V4 => "/v4/routeplanner/free/all",
            LavalinkVersion::V3 => "/routeplanner/free/all",
        };
        self.shared
            .rest
            .send(reqwest::Method::POST, path, None)
            .await
    }

    async fn require_version(&self) -> Result<LavalinkVersion> {
        self.shared.version.lock().await.ok_or(Error::NotConnected)
    }

    async fn require_session(&self) -> Result<String> {
        self.session_id().await.ok_or(Error::NotConnected)
    }

    async fn rest_update(
        &self,
        guild_id: &str,
        update: &UpdatePlayer,
        no_replace: bool,
    ) -> Result<Player> {
        let sid = self.require_session().await?;
        self.shared
            .rest
            .update_player(&sid, guild_id, update, no_replace)
            .await
    }

    async fn ws_send(&self, value: serde_json::Value) -> Result<()> {
        let guard = self.shared.ws_tx.lock().await;
        let tx = guard.as_ref().ok_or(Error::NotConnected)?;
        let text = serde_json::to_string(&value)?;
        tx.send(Message::Text(text))
            .map_err(|_| Error::Other("websocket writer closed".into()))
    }
}

const BACKOFF_BASE: Duration = Duration::from_secs(1);
const BACKOFF_MAX: Duration = Duration::from_secs(30);

pub(crate) fn backoff(attempt: u32) -> Duration {
    let secs = BACKOFF_BASE
        .as_secs()
        .saturating_mul(1u64 << attempt.min(5));
    Duration::from_secs(secs).min(BACKOFF_MAX)
}

/// Owns the (re)connect loop for a node's WebSocket.
async fn supervise(
    shared: Arc<Shared>,
    version: LavalinkVersion,
    ready_tx: oneshot::Sender<Result<()>>,
) {
    let mut ready_tx = Some(ready_tx);
    let mut attempt: u32 = 0;

    loop {
        let first = ready_tx.is_some();
        match run_connection(&shared, version).await {
            ConnectionOutcome::Ready(reader) => {
                attempt = 0;
                shared.connected.store(true, Ordering::Relaxed);
                if let Some(tx) = ready_tx.take() {
                    let _ = tx.send(Ok(()));
                }
                // Block until this connection closes, then fall through to reconnect.
                let _ = reader.await;
                shared.connected.store(false, Ordering::Relaxed);
                tracing::warn!("lavalink connection closed");
            }
            ConnectionOutcome::Failed(err) => {
                if first {
                    // Never connected - report and give up (no retry storm on a typo).
                    if let Some(tx) = ready_tx.take() {
                        let _ = tx.send(Err(err));
                    }
                    return;
                }
                tracing::warn!(%err, "reconnect attempt failed");
            }
        }

        if !shared.config.reconnect {
            return;
        }
        let delay = backoff(attempt);
        attempt = attempt.saturating_add(1);
        tracing::info!(?delay, "reconnecting to lavalink");
        tokio::time::sleep(delay).await;
    }
}

enum ConnectionOutcome {
    /// Connected and received `ready`; the join handle completes on close.
    Ready(tokio::task::JoinHandle<()>),
    Failed(Error),
}

async fn run_connection(shared: &Arc<Shared>, version: LavalinkVersion) -> ConnectionOutcome {
    let session_id = shared.session_id.lock().await.clone();
    let resume_key = shared.resume_key.lock().await.clone();
    let req = match build_ws_request(
        &shared.config,
        version,
        session_id.as_deref(),
        resume_key.as_deref(),
    ) {
        Ok(req) => req,
        Err(err) => return ConnectionOutcome::Failed(err),
    };

    let ws = match connect_async(req).await {
        Ok((ws, _)) => ws,
        Err(err) => return ConnectionOutcome::Failed(Error::Ws(err)),
    };
    let (sink, stream) = ws.split();

    let (ws_tx, ws_rx) = mpsc::unbounded_channel::<Message>();
    *shared.ws_tx.lock().await = Some(ws_tx.clone());
    tokio::spawn(writer_task(sink, ws_rx));

    let (conn_ready_tx, conn_ready_rx) = oneshot::channel();
    let reader = tokio::spawn(reader_task(
        stream,
        version,
        shared.events_tx.clone(),
        ws_tx.clone(),
        shared.clone(),
        conn_ready_tx,
    ));

    // Wait for this connection's `ready` (or an early drop).
    if conn_ready_rx.await.is_err() {
        reader.abort();
        return ConnectionOutcome::Failed(Error::ClosedBeforeReady);
    }

    if shared.config.resume {
        configure_resuming(shared, version, &ws_tx).await;
    }
    ConnectionOutcome::Ready(reader)
}

async fn configure_resuming(
    shared: &Arc<Shared>,
    version: LavalinkVersion,
    ws_tx: &mpsc::UnboundedSender<Message>,
) {
    match version {
        LavalinkVersion::V4 => {
            if let Some(sid) = shared.session_id.lock().await.clone() {
                if let Err(err) = shared
                    .rest
                    .configure_resuming_v4(&sid, shared.config.resume_timeout)
                    .await
                {
                    tracing::warn!(%err, "failed to configure v4 resuming");
                }
            }
        }
        LavalinkVersion::V3 => {
            let key = format!("wavecord-{}", shared.config.user_id);
            *shared.resume_key.lock().await = Some(key.clone());
            let op = v3::configure_resuming_op(&key, shared.config.resume_timeout);
            if let Ok(text) = serde_json::to_string(&op) {
                let _ = ws_tx.send(Message::Text(text));
            }
        }
    }
}

/// Owns the WebSocket sink and writes every message pushed onto its channel.
async fn writer_task<S>(
    mut sink: futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<S>, Message>,
    mut rx: mpsc::UnboundedReceiver<Message>,
) where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    while let Some(msg) = rx.recv().await {
        if sink.send(msg).await.is_err() {
            break;
        }
    }
}

/// Only the fields the reader needs to peek at without a full typed parse.
#[derive(serde::Deserialize)]
struct Peek {
    op: Option<String>,
    #[serde(rename = "sessionId")]
    session_id: Option<String>,
}

/// Reads WebSocket frames and forwards them to the event channel as normalized
/// JSON *strings*: v4 frames pass through untouched; v3 frames are rewritten to
/// the canonical (v4-shaped) form. Captures the session id from `ready` and
/// answers Pings via the writer channel. Building Python objects is left to the
/// Python side (msgspec), which is far cheaper than doing it in Rust.
async fn reader_task<S>(
    mut stream: futures_util::stream::SplitStream<tokio_tungstenite::WebSocketStream<S>>,
    version: LavalinkVersion,
    events_tx: mpsc::UnboundedSender<String>,
    ws_tx: mpsc::UnboundedSender<Message>,
    shared: Arc<Shared>,
    ready_tx: oneshot::Sender<()>,
) where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    let mut ready_tx = Some(ready_tx);

    while let Some(frame) = stream.next().await {
        match frame {
            Ok(Message::Text(txt)) => {
                // Produce the JSON string to hand to Python (already canonical).
                let out: Option<String> = match version {
                    LavalinkVersion::V4 => {
                        // Peek for `ready` to capture the session id; otherwise
                        // forward the frame verbatim (zero re-serialization).
                        if let Ok(peek) = serde_json::from_str::<Peek>(&txt) {
                            if peek.op.as_deref() == Some("ready") {
                                if let Some(sid) = peek.session_id {
                                    *shared.session_id.lock().await = Some(sid);
                                }
                                if let Some(rt) = ready_tx.take() {
                                    let _ = rt.send(());
                                }
                            }
                        }
                        Some(txt.to_string())
                    }
                    LavalinkVersion::V3 => match v3::normalize_message(&txt) {
                        Ok(Some(msg)) => {
                            if let ServerMessage::Ready(ready) = &msg {
                                *shared.session_id.lock().await = Some(ready.session_id.clone());
                                if let Some(rt) = ready_tx.take() {
                                    let _ = rt.send(());
                                }
                            }
                            serde_json::to_string(&msg).ok()
                        }
                        Ok(None) => None,
                        Err(err) => {
                            tracing::warn!(%err, raw = %txt, "failed to normalize v3 message");
                            None
                        }
                    },
                };
                if let Some(s) = out {
                    if events_tx.send(s).is_err() {
                        break;
                    }
                }
            }
            Ok(Message::Ping(payload)) => {
                let _ = ws_tx.send(Message::Pong(payload));
            }
            Ok(Message::Close(_)) => break,
            Ok(_) => {}
            Err(err) => {
                tracing::error!(%err, "websocket read error");
                break;
            }
        }
    }
}
