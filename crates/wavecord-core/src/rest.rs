// SPDX-License-Identifier: MIT
// Copyright (c) 2026 WaveCord contributors

//! REST client for the Lavalink HTTP API.

use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use reqwest::{Method, Response};
use serde_json::Value;

use crate::error::{Error, Result};
use crate::model::{LoadResult, Player, UpdatePlayer};

/// Percent-encode a value used as a URL path segment (guild id, session id, ...)
/// so it cannot break out of the path.
pub(crate) fn seg(value: &str) -> String {
    utf8_percent_encode(value, NON_ALPHANUMERIC).to_string()
}

#[derive(Debug, Clone)]
pub struct RestClient {
    http: reqwest::Client,
    /// e.g. `http://127.0.0.1:2333`
    base: String,
    password: String,
}

impl RestClient {
    pub fn new(base: impl Into<String>, password: impl Into<String>) -> Result<Self> {
        Ok(Self {
            http: reqwest::Client::builder().build()?,
            base: base.into(),
            password: password.into(),
        })
    }

    async fn check(resp: Response) -> Result<Response> {
        if resp.status().is_success() {
            Ok(resp)
        } else {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            Err(Error::Api { status, body })
        }
    }

    /// `PATCH /v4/sessions/{sid}/players/{guild}?noReplace=...`
    pub async fn update_player(
        &self,
        session_id: &str,
        guild_id: &str,
        update: &UpdatePlayer,
        no_replace: bool,
    ) -> Result<Player> {
        let url = format!(
            "{}/v4/sessions/{}/players/{}",
            self.base,
            seg(session_id),
            seg(guild_id)
        );
        let resp = self
            .http
            .patch(url)
            .query(&[("noReplace", no_replace)])
            .header("Authorization", &self.password)
            .json(update)
            .send()
            .await?;
        Ok(Self::check(resp).await?.json().await?)
    }

    /// `DELETE /v4/sessions/{sid}/players/{guild}`
    pub async fn destroy_player(&self, session_id: &str, guild_id: &str) -> Result<()> {
        let url = format!(
            "{}/v4/sessions/{}/players/{}",
            self.base,
            seg(session_id),
            seg(guild_id)
        );
        let resp = self
            .http
            .delete(url)
            .header("Authorization", &self.password)
            .send()
            .await?;
        Self::check(resp).await?;
        Ok(())
    }

    /// `GET /v4/loadtracks?identifier=...` - parses straight into the model.
    pub async fn load_tracks_v4(&self, identifier: &str) -> Result<LoadResult> {
        let url = format!("{}/v4/loadtracks", self.base);
        Ok(Self::check(self.get_loadtracks(&url, identifier).await?)
            .await?
            .json()
            .await?)
    }

    /// `GET /loadtracks?identifier=...` on a v3 node - normalized into the model.
    pub async fn load_tracks_v3(&self, identifier: &str) -> Result<LoadResult> {
        let url = format!("{}/loadtracks", self.base);
        let value: serde_json::Value = Self::check(self.get_loadtracks(&url, identifier).await?)
            .await?
            .json()
            .await?;
        crate::protocol::v3::normalize_load_result(&value)
    }

    async fn get_loadtracks(&self, url: &str, identifier: &str) -> Result<Response> {
        Ok(self
            .http
            .get(url)
            .query(&[("identifier", identifier)])
            .header("Authorization", &self.password)
            .send()
            .await?)
    }

    /// `PATCH /v4/sessions/{sid}` - enable session resuming so players survive a
    /// short reconnect (the node keeps them alive for `timeout` seconds).
    pub async fn configure_resuming_v4(&self, session_id: &str, timeout: u64) -> Result<()> {
        let url = format!("{}/v4/sessions/{}", self.base, seg(session_id));
        let resp = self
            .http
            .patch(url)
            .header("Authorization", &self.password)
            .json(&serde_json::json!({ "resuming": true, "timeout": timeout }))
            .send()
            .await?;
        Self::check(resp).await?;
        Ok(())
    }

    /// `GET /version` - used to detect the Lavalink major version (v3 vs v4).
    pub async fn version(&self) -> Result<String> {
        let url = format!("{}/version", self.base);
        let resp = self
            .http
            .get(url)
            .header("Authorization", &self.password)
            .send()
            .await?;
        Ok(Self::check(resp).await?.text().await?)
    }

    /// `GET {path}` with optional query, parsed as JSON. Returns `Null` for an
    /// empty body (e.g. a `204 No Content`).
    pub async fn get_value(&self, path: &str, query: &[(&str, &str)]) -> Result<Value> {
        let resp = self
            .http
            .get(format!("{}{}", self.base, path))
            .query(query)
            .header("Authorization", &self.password)
            .send()
            .await?;
        let text = Self::check(resp).await?.text().await?;
        Ok(if text.trim().is_empty() {
            Value::Null
        } else {
            serde_json::from_str(&text)?
        })
    }

    /// Send a `POST`/`PUT`/`DELETE` (optionally with a JSON body) that returns no
    /// meaningful body.
    pub async fn send(&self, method: Method, path: &str, body: Option<Value>) -> Result<()> {
        let mut req = self
            .http
            .request(method, format!("{}{}", self.base, path))
            .header("Authorization", &self.password);
        if let Some(body) = body {
            req = req.json(&body);
        }
        Self::check(req.send().await?).await?;
        Ok(())
    }
}
