use std::collections::HashMap;
use std::convert::Infallible;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use axum::extract::{Query, State};
use axum::http::{StatusCode, Uri, header};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use futures_util::Stream;
use tokio::net::TcpListener;
use tokio::sync::watch;

use crate::assets::Assets;
use crate::dto::SnapshotDto;
use crate::key::AccessKey;

/// Port of the phone page on every private address.
pub const DEFAULT_PORT: u16 = 4243;

/// Same policy as the desktop panel (`tauri.conf.json`), minus Tauri's IPC.
const CONTENT_SECURITY_POLICY: &str = "default-src 'self'; img-src 'self' data:; \
     style-src 'self' 'unsafe-inline'; connect-src 'self'; frame-ancestors 'none'";

/// The latest snapshot, as JSON. Serialized once, shared by every connected phone.
type Feed = watch::Receiver<Option<Arc<str>>>;

/// Hands snapshots to the servers. Lives as long as the app; servers come and go with the
/// network.
pub struct Publisher {
    sender: watch::Sender<Option<Arc<str>>>,
}

impl Default for Publisher {
    fn default() -> Self {
        Self::new()
    }
}

impl Publisher {
    pub fn new() -> Self {
        Self {
            sender: watch::Sender::new(None),
        }
    }

    /// Every connected page receives it right away.
    pub fn publish(&self, snapshot: &SnapshotDto) {
        if let Ok(json) = serde_json::to_string(snapshot) {
            self.sender.send_replace(Some(json.into()));
        }
    }

    fn subscribe(&self) -> Feed {
        self.sender.subscribe()
    }
}

#[derive(Clone)]
struct ServerState {
    feed: Feed,
    assets: Arc<dyn Assets>,
    key: Arc<AccessKey>,
    stop: watch::Receiver<bool>,
}

/// The phone page and its API, listening on a set of addresses until stopped or dropped.
///
/// ```text
/// GET /                     the UI (same build as the desktop panel)
/// GET /api/snapshot?key=…   the latest snapshot, 204 before the first one
/// GET /api/events?key=…     server-sent events: one `snapshot` event per change
/// ```
pub struct WebServer {
    addresses: Vec<SocketAddr>,
    stop: watch::Sender<bool>,
}

impl WebServer {
    /// Binds every address, then serves in the background. Must run inside a Tokio runtime.
    pub async fn start(
        addresses: &[SocketAddr],
        publisher: &Publisher,
        assets: Arc<dyn Assets>,
        key: AccessKey,
    ) -> io::Result<Self> {
        let mut listeners = Vec::with_capacity(addresses.len());
        for address in addresses {
            let listener = TcpListener::bind(address)
                .await
                .map_err(|error| io::Error::new(error.kind(), format!("{address}: {error}")))?;
            listeners.push(listener);
        }
        let bound = listeners
            .iter()
            .map(TcpListener::local_addr)
            .collect::<io::Result<Vec<_>>>()?;

        let (stop_sender, stop) = watch::channel(false);
        let app = router(ServerState {
            feed: publisher.subscribe(),
            assets,
            key: Arc::new(key),
            stop: stop.clone(),
        });
        for listener in listeners {
            let app = app.clone();
            let mut stop = stop.clone();
            tokio::spawn(async move {
                let shutdown = async move {
                    let _ = stop.wait_for(|stopped| *stopped).await;
                };
                let _ = axum::serve(listener, app)
                    .with_graceful_shutdown(shutdown)
                    .await;
            });
        }
        Ok(Self {
            addresses: bound,
            stop: stop_sender,
        })
    }

    /// The addresses actually bound (a `:0` port is replaced by the one the OS picked).
    pub fn addresses(&self) -> &[SocketAddr] {
        &self.addresses
    }

    /// Closes the listeners and the open event streams.
    pub fn stop(&self) {
        self.stop.send_replace(true);
    }
}

impl Drop for WebServer {
    fn drop(&mut self) {
        self.stop();
    }
}

fn router(state: ServerState) -> Router {
    Router::new()
        .route("/api/snapshot", get(snapshot))
        .route("/api/events", get(events))
        .fallback(get(asset))
        .with_state(state)
}

fn authorized(state: &ServerState, query: &HashMap<String, String>) -> bool {
    query
        .get("key")
        .is_some_and(|candidate| state.key.matches(candidate))
}

async fn snapshot(
    State(state): State<ServerState>,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    if !authorized(&state, &query) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let latest = state.feed.borrow().clone();
    match latest {
        Some(json) => (
            [
                (header::CONTENT_TYPE, "application/json"),
                (header::CACHE_CONTROL, "no-store"),
            ],
            json.to_string(),
        )
            .into_response(),
        None => StatusCode::NO_CONTENT.into_response(),
    }
}

async fn events(
    State(state): State<ServerState>,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    if !authorized(&state, &query) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    // Comments every 15 s keep proxies and iOS from closing an idle connection.
    Sse::new(snapshot_events(state.feed, state.stop))
        .keep_alive(KeepAlive::default())
        .into_response()
}

/// The current snapshot first, then one event per change, until the server stops.
fn snapshot_events(
    mut feed: Feed,
    stop: watch::Receiver<bool>,
) -> impl Stream<Item = Result<Event, Infallible>> {
    feed.mark_changed();
    futures_util::stream::unfold((feed, stop), |(mut feed, mut stop)| async move {
        loop {
            let changed = tokio::select! {
                changed = feed.changed() => changed.is_ok(),
                _ = stop.wait_for(|stopped| *stopped) => false,
            };
            if !changed {
                return None;
            }
            let latest = feed.borrow_and_update().clone();
            if let Some(json) = latest {
                let event = Event::default().event("snapshot").data(&*json);
                return Some((Ok(event), (feed, stop)));
            }
        }
    })
}

async fn asset(State(state): State<ServerState>, uri: Uri) -> Response {
    let path = match uri.path().trim_start_matches('/') {
        "" => "index.html",
        path => path,
    };
    // Built file names are plain: anything that could climb out of the UI folder is refused.
    let suspicious = path.contains('\\')
        || path.contains('%')
        || path
            .split('/')
            .any(|segment| segment.is_empty() || segment == "..");
    let Some(found) = (!suspicious).then(|| state.assets.get(path)).flatten() else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let is_page = found.mime_type.starts_with("text/html");
    let mut response = (
        [
            (header::CONTENT_TYPE, found.mime_type),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff".into()),
            // The page URL carries the key: never send it to the sites it links to.
            (header::REFERRER_POLICY, "no-referrer".into()),
        ],
        found.bytes,
    )
        .into_response();
    if is_page {
        let headers = response.headers_mut();
        headers.insert(
            header::CACHE_CONTROL,
            "no-cache".parse().expect("valid header"),
        );
        headers.insert(
            header::CONTENT_SECURITY_POLICY,
            CONTENT_SECURITY_POLICY.parse().expect("valid header"),
        );
    }
    response
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;

    use super::*;
    use crate::assets::Asset;
    use crate::dto::CountsDto;

    struct OnePage;
    impl Assets for OnePage {
        fn get(&self, path: &str) -> Option<Asset> {
            (path == "index.html").then(|| Asset {
                bytes: b"<html>cerebro</html>".to_vec(),
                mime_type: "text/html; charset=utf-8".into(),
            })
        }
    }

    fn snapshot_with(sessions: usize) -> SnapshotDto {
        SnapshotDto {
            scenes: vec![],
            warnings: vec![],
            counts: CountsDto {
                sessions,
                busy: 0,
                agents: 0,
                characters: 0,
            },
            taken_at: 1,
        }
    }

    async fn start(publisher: &Publisher) -> WebServer {
        let local: SocketAddr = "127.0.0.1:0".parse().unwrap();
        WebServer::start(&[local], publisher, Arc::new(OnePage), AccessKey::new("k"))
            .await
            .unwrap()
    }

    /// Sends a request and reads the whole answer (the server closes after it).
    async fn get(server: &WebServer, path: &str) -> String {
        let mut stream = TcpStream::connect(server.addresses()[0]).await.unwrap();
        let request = format!("GET {path} HTTP/1.1\r\nHost: test\r\nConnection: close\r\n\r\n");
        stream.write_all(request.as_bytes()).await.unwrap();
        let mut answer = String::new();
        stream.read_to_string(&mut answer).await.unwrap();
        answer
    }

    /// Reads from an open stream until `needle` shows up.
    async fn read_until(stream: &mut TcpStream, received: &mut String, needle: &str) {
        let mut buffer = [0_u8; 1024];
        while !received.contains(needle) {
            let read = tokio::time::timeout(Duration::from_secs(2), stream.read(&mut buffer))
                .await
                .expect("timed out")
                .unwrap();
            assert!(read > 0, "stream closed before {needle:?}: {received}");
            received.push_str(&String::from_utf8_lossy(&buffer[..read]));
        }
    }

    #[tokio::test]
    async fn serves_the_page_to_anyone_and_the_api_to_key_holders() {
        // Arrange
        let publisher = Publisher::new();
        let server = start(&publisher).await;

        // Act
        let page = get(&server, "/").await;
        let before_any_snapshot = get(&server, "/api/snapshot?key=k").await;
        publisher.publish(&snapshot_with(3));
        let with_key = get(&server, "/api/snapshot?key=k").await;
        let without_key = get(&server, "/api/snapshot").await;
        let wrong_key = get(&server, "/api/snapshot?key=nope").await;

        // Assert
        assert!(page.starts_with("HTTP/1.1 200"), "{page}");
        assert!(page.contains("referrer-policy: no-referrer"), "{page}");
        assert!(page.contains("<html>cerebro</html>"), "{page}");
        assert!(
            before_any_snapshot.starts_with("HTTP/1.1 204"),
            "{before_any_snapshot}"
        );
        assert!(with_key.contains(r#""sessions":3"#), "{with_key}");
        assert!(without_key.starts_with("HTTP/1.1 401"), "{without_key}");
        assert!(wrong_key.starts_with("HTTP/1.1 401"), "{wrong_key}");
    }

    #[tokio::test]
    async fn refuses_paths_that_climb_out() {
        // Arrange
        let server = start(&Publisher::new()).await;

        // Act & Assert
        for path in [
            "/../secret",
            "/a/../index.html",
            "/%2e%2e/secret",
            "/missing.png",
        ] {
            let answer = get(&server, path).await;
            assert!(answer.starts_with("HTTP/1.1 404"), "{path}: {answer}");
        }
    }

    #[tokio::test]
    async fn streams_each_snapshot_then_ends_when_stopped() {
        // Arrange
        let publisher = Publisher::new();
        publisher.publish(&snapshot_with(1));
        let server = start(&publisher).await;
        let mut stream = TcpStream::connect(server.addresses()[0]).await.unwrap();
        stream
            .write_all(b"GET /api/events?key=k HTTP/1.1\r\nHost: test\r\n\r\n")
            .await
            .unwrap();
        let mut received = String::new();

        // Act & Assert: the current snapshot comes first, then every new one.
        read_until(&mut stream, &mut received, r#""sessions":1"#).await;
        publisher.publish(&snapshot_with(2));
        read_until(&mut stream, &mut received, r#""sessions":2"#).await;
        assert!(received.contains("event: snapshot"), "{received}");

        // Act & Assert: stopping the server ends the stream instead of hanging forever.
        server.stop();
        let mut rest = Vec::new();
        tokio::time::timeout(Duration::from_secs(2), stream.read_to_end(&mut rest))
            .await
            .expect("the event stream should close")
            .unwrap();
    }
}
