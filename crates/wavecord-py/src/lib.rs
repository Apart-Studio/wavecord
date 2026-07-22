// SPDX-License-Identifier: MIT
// Copyright (c) 2026 WaveCord contributors

//! PyO3 bindings for WaveCord. All real logic lives in `wavecord-core`; async
//! Rust futures are exposed to asyncio via `pyo3-async-runtimes`.

use std::sync::Arc;

use pyo3::create_exception;
use pyo3::exceptions::{PyException, PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyList, PyString};
use pyo3_async_runtimes::tokio::future_into_py;
use pythonize::{depythonize, pythonize};
use serde_json::Value;

use wavecord_core::{LavalinkVersion, Node as CoreNode, NodeConfig};

create_exception!(
    _wavecord,
    WaveCordError,
    PyException,
    "Base error for all WaveCord failures (connection, REST, protocol)."
);

fn map_err(err: wavecord_core::Error) -> PyErr {
    WaveCordError::new_err(err.to_string())
}

fn parse_version(s: Option<String>) -> PyResult<Option<LavalinkVersion>> {
    match s.as_deref() {
        None => Ok(None),
        Some("v3") | Some("3") => Ok(Some(LavalinkVersion::V3)),
        Some("v4") | Some("4") => Ok(Some(LavalinkVersion::V4)),
        Some(other) => Err(PyValueError::new_err(format!(
            "version must be 'v3' or 'v4', got {other:?}"
        ))),
    }
}

fn version_str(v: LavalinkVersion) -> &'static str {
    match v {
        LavalinkVersion::V3 => "v3",
        LavalinkVersion::V4 => "v4",
    }
}

/// Hand-written `serde_json::Value` -> Python converter. Faster than `pythonize`
/// because it walks the value directly instead of going through serde's generic
/// serializer machinery.
fn value_to_py<'py>(py: Python<'py>, v: &Value) -> PyResult<Bound<'py, PyAny>> {
    Ok(match v {
        Value::Null => py.None().into_bound(py),
        Value::Bool(b) => (*b).into_pyobject(py)?.to_owned().into_any(),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i.into_pyobject(py)?.into_any()
            } else if let Some(u) = n.as_u64() {
                u.into_pyobject(py)?.into_any()
            } else {
                n.as_f64().unwrap_or(0.0).into_pyobject(py)?.into_any()
            }
        }
        Value::String(s) => PyString::new(py, s).into_any(),
        Value::Array(a) => {
            let list = PyList::empty(py);
            for item in a {
                list.append(value_to_py(py, item)?)?;
            }
            list.into_any()
        }
        Value::Object(o) => {
            let dict = PyDict::new(py);
            for (k, val) in o {
                dict.set_item(PyString::new(py, k), value_to_py(py, val)?)?;
            }
            dict.into_any()
        }
    })
}

#[pyfunction]
fn _bench_pythonize<'py>(py: Python<'py>, text: &str) -> PyResult<Bound<'py, PyAny>> {
    let v: Value = serde_json::from_str(text).map_err(|e| PyValueError::new_err(e.to_string()))?;
    pythonize(py, &v).map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

#[pyfunction]
fn _bench_manual<'py>(py: Python<'py>, text: &str) -> PyResult<Bound<'py, PyAny>> {
    let v: Value = serde_json::from_str(text).map_err(|e| PyValueError::new_err(e.to_string()))?;
    value_to_py(py, &v)
}

/// Wraps any `serde::Serialize` value so it is converted into a native Python
/// object lazily, when the future resolves and the GIL is reacquired - keeping
/// serialization off the async hot path. `None`/unit map to Python `None`.
struct Json<T>(T);

impl<'py, T: serde::Serialize> IntoPyObject<'py> for Json<T> {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        pythonize(py, &self.0).map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }
}

/// A single Lavalink node (v3 or v4). All methods are asyncio coroutines.
#[pyclass]
struct Node {
    inner: Arc<CoreNode>,
}

#[pymethods]
impl Node {
    #[new]
    #[pyo3(signature = (host, port, password, user_id, *, secure = false, client_name = None, session_id = None, version = None, reconnect = true, resume = true, resume_timeout = 60))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        host: String,
        port: u16,
        password: String,
        user_id: String,
        secure: bool,
        client_name: Option<String>,
        session_id: Option<String>,
        version: Option<String>,
        reconnect: bool,
        resume: bool,
        resume_timeout: u64,
    ) -> PyResult<Self> {
        let config = NodeConfig {
            host,
            port,
            password,
            secure,
            user_id,
            client_name: client_name
                .unwrap_or_else(|| format!("WaveCord/{}", env!("CARGO_PKG_VERSION"))),
            session_id,
            force_version: parse_version(version)?,
            reconnect,
            resume,
            resume_timeout,
        };
        let inner = CoreNode::new(config).map_err(map_err)?;
        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    /// Detect the version (unless forced), open the WebSocket, resolve on ready.
    fn connect<'p>(&self, py: Python<'p>) -> PyResult<Bound<'p, PyAny>> {
        let inner = self.inner.clone();
        future_into_py(py, async move {
            inner.connect().await.map_err(map_err)?;
            Ok(())
        })
    }

    /// `"v3"`, `"v4"`, or `None` if not connected/detected yet.
    fn version<'p>(&self, py: Python<'p>) -> PyResult<Bound<'p, PyAny>> {
        let inner = self.inner.clone();
        future_into_py(
            py,
            async move { Ok(inner.version().await.map(version_str)) },
        )
    }

    fn session_id<'p>(&self, py: Python<'p>) -> PyResult<Bound<'p, PyAny>> {
        let inner = self.inner.clone();
        future_into_py(py, async move { Ok(inner.session_id().await) })
    }

    /// Whether a live WebSocket connection is currently up (sync - reads an
    /// atomic flag maintained by the reconnect supervisor).
    fn is_connected(&self) -> bool {
        self.inner.is_connected()
    }

    /// Await the next server message as a normalized JSON **string** (decode it
    /// with msgspec - see `EventDispatcher`), or `None` once closed. Returning a
    /// string keeps the GIL-side cost minimal; building Python objects in Rust is
    /// slower than msgspec.
    fn next_event<'p>(&self, py: Python<'p>) -> PyResult<Bound<'p, PyAny>> {
        let inner = self.inner.clone();
        future_into_py(py, async move { Ok(inner.recv_event().await) })
    }

    /// Await at least one message, then return up to `max_n` already-queued ones
    /// as a list of strings (or `None` once closed). Used by `EventDispatcher`
    /// to cut per-event overhead under load.
    #[pyo3(signature = (max_n = 64))]
    fn next_events<'p>(&self, py: Python<'p>, max_n: usize) -> PyResult<Bound<'p, PyAny>> {
        let inner = self.inner.clone();
        future_into_py(py, async move { Ok(inner.recv_events(max_n).await) })
    }

    /// Play an encoded track. Returns the updated player dict on v4, `None` on v3.
    #[pyo3(signature = (guild_id, encoded, *, no_replace = false, start_ms = None, end_ms = None, volume = None, paused = None))]
    #[allow(clippy::too_many_arguments)]
    fn play<'p>(
        &self,
        py: Python<'p>,
        guild_id: String,
        encoded: String,
        no_replace: bool,
        start_ms: Option<i64>,
        end_ms: Option<i64>,
        volume: Option<i32>,
        paused: Option<bool>,
    ) -> PyResult<Bound<'p, PyAny>> {
        let inner = self.inner.clone();
        future_into_py(py, async move {
            let player = inner
                .play(
                    &guild_id, &encoded, start_ms, end_ms, volume, paused, no_replace,
                )
                .await
                .map_err(map_err)?;
            Ok(Json(player))
        })
    }

    fn stop<'p>(&self, py: Python<'p>, guild_id: String) -> PyResult<Bound<'p, PyAny>> {
        let inner = self.inner.clone();
        future_into_py(py, async move {
            Ok(Json(inner.stop(&guild_id).await.map_err(map_err)?))
        })
    }

    fn set_pause<'p>(
        &self,
        py: Python<'p>,
        guild_id: String,
        paused: bool,
    ) -> PyResult<Bound<'p, PyAny>> {
        let inner = self.inner.clone();
        future_into_py(py, async move {
            Ok(Json(
                inner.set_pause(&guild_id, paused).await.map_err(map_err)?,
            ))
        })
    }

    fn set_volume<'p>(
        &self,
        py: Python<'p>,
        guild_id: String,
        volume: i32,
    ) -> PyResult<Bound<'p, PyAny>> {
        let inner = self.inner.clone();
        future_into_py(py, async move {
            Ok(Json(
                inner.set_volume(&guild_id, volume).await.map_err(map_err)?,
            ))
        })
    }

    fn seek<'p>(
        &self,
        py: Python<'p>,
        guild_id: String,
        position_ms: i64,
    ) -> PyResult<Bound<'p, PyAny>> {
        let inner = self.inner.clone();
        future_into_py(py, async move {
            Ok(Json(
                inner.seek(&guild_id, position_ms).await.map_err(map_err)?,
            ))
        })
    }

    /// Hand Lavalink the Discord voice connection. Must be called before `play`.
    /// `channel_id` is required by Lavalink v4 (ignored on v3).
    #[pyo3(signature = (guild_id, token, endpoint, session_id, channel_id = None))]
    fn update_voice<'p>(
        &self,
        py: Python<'p>,
        guild_id: String,
        token: String,
        endpoint: String,
        session_id: String,
        channel_id: Option<String>,
    ) -> PyResult<Bound<'p, PyAny>> {
        let inner = self.inner.clone();
        future_into_py(py, async move {
            let player = inner
                .update_voice(
                    &guild_id,
                    &token,
                    &endpoint,
                    &session_id,
                    channel_id.as_deref(),
                )
                .await
                .map_err(map_err)?;
            Ok(Json(player))
        })
    }

    /// Apply an audio filter set (a dict like
    /// `{"volume": 1.5, "equalizer": [{"band": 0, "gain": 0.2}]}`). Returns the
    /// updated player dict on v4, `None` on v3.
    fn set_filters<'p>(
        &self,
        py: Python<'p>,
        guild_id: String,
        filters: Bound<'p, PyAny>,
    ) -> PyResult<Bound<'p, PyAny>> {
        let value: serde_json::Value =
            depythonize(&filters).map_err(|e| PyValueError::new_err(e.to_string()))?;
        let inner = self.inner.clone();
        future_into_py(py, async move {
            Ok(Json(
                inner.set_filters(&guild_id, value).await.map_err(map_err)?,
            ))
        })
    }

    fn destroy<'p>(&self, py: Python<'p>, guild_id: String) -> PyResult<Bound<'p, PyAny>> {
        let inner = self.inner.clone();
        future_into_py(py, async move {
            inner.destroy_player(&guild_id).await.map_err(map_err)?;
            Ok(())
        })
    }

    /// Resolve/search tracks. Returns a normalized `LoadResult` dict (same shape
    /// on v3 and v4).
    fn load_tracks<'p>(&self, py: Python<'p>, identifier: String) -> PyResult<Bound<'p, PyAny>> {
        let inner = self.inner.clone();
        future_into_py(py, async move {
            Ok(Json(inner.load_tracks(&identifier).await.map_err(map_err)?))
        })
    }

    /// Node info dict (version, source managers, loaded plugins).
    fn info<'p>(&self, py: Python<'p>) -> PyResult<Bound<'p, PyAny>> {
        let inner = self.inner.clone();
        future_into_py(
            py,
            async move { Ok(Json(inner.info().await.map_err(map_err)?)) },
        )
    }

    /// Decode a base64 track string into its info without playing it.
    fn decode_track<'p>(&self, py: Python<'p>, encoded: String) -> PyResult<Bound<'p, PyAny>> {
        let inner = self.inner.clone();
        future_into_py(py, async move {
            Ok(Json(inner.decode_track(&encoded).await.map_err(map_err)?))
        })
    }

    /// LavaSearch plugin search. `types` defaults to all categories.
    #[pyo3(signature = (query, types = None))]
    fn load_search<'p>(
        &self,
        py: Python<'p>,
        query: String,
        types: Option<String>,
    ) -> PyResult<Bound<'p, PyAny>> {
        let inner = self.inner.clone();
        let types = types.unwrap_or_else(|| "track,album,artist,playlist,text".to_string());
        future_into_py(py, async move {
            Ok(Json(
                inner.load_search(&query, &types).await.map_err(map_err)?,
            ))
        })
    }

    /// LavaLyrics plugin: lyrics for an encoded track.
    #[pyo3(signature = (encoded, skip_track_source = false))]
    fn lyrics<'p>(
        &self,
        py: Python<'p>,
        encoded: String,
        skip_track_source: bool,
    ) -> PyResult<Bound<'p, PyAny>> {
        let inner = self.inner.clone();
        future_into_py(py, async move {
            Ok(Json(
                inner
                    .lyrics(&encoded, skip_track_source)
                    .await
                    .map_err(map_err)?,
            ))
        })
    }

    /// LavaLyrics plugin: lyrics for a guild's currently playing track.
    #[pyo3(signature = (guild_id, skip_track_source = false))]
    fn current_lyrics<'p>(
        &self,
        py: Python<'p>,
        guild_id: String,
        skip_track_source: bool,
    ) -> PyResult<Bound<'p, PyAny>> {
        let inner = self.inner.clone();
        future_into_py(py, async move {
            Ok(Json(
                inner
                    .current_lyrics(&guild_id, skip_track_source)
                    .await
                    .map_err(map_err)?,
            ))
        })
    }

    /// SponsorBlock plugin: set the categories to skip for a guild's player.
    fn set_sponsorblock_categories<'p>(
        &self,
        py: Python<'p>,
        guild_id: String,
        categories: Vec<String>,
    ) -> PyResult<Bound<'p, PyAny>> {
        let inner = self.inner.clone();
        future_into_py(py, async move {
            inner
                .set_sponsorblock_categories(&guild_id, categories)
                .await
                .map_err(map_err)?;
            Ok(())
        })
    }

    /// Route planner status.
    fn routeplanner_status<'p>(&self, py: Python<'p>) -> PyResult<Bound<'p, PyAny>> {
        let inner = self.inner.clone();
        future_into_py(py, async move {
            Ok(Json(inner.routeplanner_status().await.map_err(map_err)?))
        })
    }

    /// Unmark a single failed address in the route planner.
    fn routeplanner_free<'p>(&self, py: Python<'p>, address: String) -> PyResult<Bound<'p, PyAny>> {
        let inner = self.inner.clone();
        future_into_py(py, async move {
            inner.routeplanner_free(&address).await.map_err(map_err)?;
            Ok(())
        })
    }

    /// Unmark all failed addresses in the route planner.
    fn routeplanner_free_all<'p>(&self, py: Python<'p>) -> PyResult<Bound<'p, PyAny>> {
        let inner = self.inner.clone();
        future_into_py(py, async move {
            inner.routeplanner_free_all().await.map_err(map_err)?;
            Ok(())
        })
    }
}

/// `await wavecord.ping()` -> `"pong"` - smoke test for the async bridge.
#[pyfunction]
fn ping(py: Python<'_>) -> PyResult<Bound<'_, PyAny>> {
    future_into_py(
        py,
        async move { Ok(wavecord_core::ping().await.to_string()) },
    )
}

/// Decode a raw Lavalink WebSocket message into the normalized dict WaveCord
/// delivers via `next_event`. Handy for testing/debugging and used by the
/// benchmarks to measure the real Rust->Python decode path. `version` is
/// `"v3"` or `"v4"`.
#[pyfunction]
#[pyo3(signature = (text, version = "v4"))]
fn decode_message(
    text: &str,
    version: &str,
) -> PyResult<Json<Option<wavecord_core::model::ServerMessage>>> {
    let msg = match parse_version(Some(version.to_string()))? {
        Some(LavalinkVersion::V4) => serde_json::from_str(text)
            .map(Some)
            .map_err(|e| PyValueError::new_err(e.to_string()))?,
        Some(LavalinkVersion::V3) => wavecord_core::protocol::v3::normalize_message(text)
            .map_err(|e| PyValueError::new_err(e.to_string()))?,
        None => return Err(PyValueError::new_err("version must be 'v3' or 'v4'")),
    };
    Ok(Json(msg))
}

#[pymodule]
fn _wavecord(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("WaveCordError", m.py().get_type::<WaveCordError>())?;
    m.add_function(wrap_pyfunction!(ping, m)?)?;
    m.add_function(wrap_pyfunction!(decode_message, m)?)?;
    m.add_function(wrap_pyfunction!(_bench_pythonize, m)?)?;
    m.add_function(wrap_pyfunction!(_bench_manual, m)?)?;
    m.add_class::<Node>()?;
    Ok(())
}
