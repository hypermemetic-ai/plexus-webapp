//! SCAFFOLD-5: a tiny service and an embedded two-file app, through
//! plexus-webapp's LocalServer on free ports — `/` is the stamped page, the
//! app's asset is served byte-exact, the WebSocket answers a call, and a
//! shutdown releases both ports.

use jsonrpsee::core::client::SubscriptionClientT;
use jsonrpsee::core::params::ObjectParams;
use jsonrpsee::ws_client::WsClientBuilder;
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

async fn get(port: u16, path: &str) -> String {
    let mut s = tokio::net::TcpStream::connect(("127.0.0.1", port)).await.expect("the shell accepts");
    s.write_all(format!("GET {path} HTTP/1.0\r\nHost: 127.0.0.1:{port}\r\n\r\n").as_bytes()).await.unwrap();
    let mut resp = Vec::new();
    s.read_to_end(&mut resp).await.unwrap();
    String::from_utf8_lossy(&resp).into_owned()
}

#[tokio::test]
async fn the_toy_serves_its_embedded_app_and_its_service() {
    let keys: Vec<_> = plexus_webapp_toy::APP.iter().map(|f| f.0).collect();
    assert_eq!(keys, ["app.js", "index.html"], "build.rs embedded app/");
    let server = plexus_webapp_toy::start().await.expect("start");
    let (ws, shell) = (server.ws_port(), server.shell_port());
    assert!(ws != 0 && shell != 0 && ws != shell);

    let page = get(shell, "/").await;
    assert!(page.starts_with("HTTP/1.0 200") || page.starts_with("HTTP/1.1 200"), "{page}");
    assert!(page.contains("<title>toy</title>"), "{page}");
    assert!(page.contains(&format!("content=\"{ws}\"")), "stamped with the socket's port: {page}");
    assert!(page.contains("<body>hello from plexus-webapp</body>"), "an app's own placeholder: {page}");
    assert!(page.to_ascii_lowercase().contains("x-frame-options: deny"), "hardened: {page}");
    let js = get(shell, "/app/app.js").await;
    assert!(js.contains("text/javascript") && js.contains("toy.call"), "{js}");
    assert!(js.contains("{text: \"hi\"}") || js.contains("text: \"hi\""), "byte-exact, not stamped: {js}");
    assert!(get(shell, "/app/../Cargo.toml").await.contains("404"));
    assert!(get(shell, "/app/nothing.js").await.contains("404"));

    let client = WsClientBuilder::default().build(server.ws_url()).await.expect("the socket");
    let mut p = ObjectParams::new();
    p.insert("method", "echo.say").unwrap();
    p.insert("params", serde_json::json!({ "text": "hi" })).unwrap();
    let mut sub = client.subscribe::<Value, _>("toy.call", p, "toy.call_unsub").await.expect("subscribe");
    let item = sub.next().await.expect("an item").expect("ok");
    assert_eq!(item["type"], "data", "{item}");
    assert_eq!(item["content"], "echo: hi", "{item}");

    server.shutdown().await;
    for port in [ws, shell] {
        tokio::net::TcpListener::bind(("127.0.0.1", port)).await.unwrap_or_else(|e| panic!("port {port} released: {e}"));
    }
}

/// The status line of a WebSocket upgrade to `port` naming `host` and, when
/// given, `origin` — as a browser page would send it.
async fn upgrade(port: u16, host: &str, origin: Option<&str>) -> String {
    let mut s = tokio::net::TcpStream::connect(("127.0.0.1", port)).await.expect("the socket accepts");
    let origin = origin.map(|o| format!("Origin: {o}\r\n")).unwrap_or_default();
    let req = format!(
        "GET / HTTP/1.1\r\nHost: {host}\r\n{origin}Connection: Upgrade\r\nUpgrade: websocket\r\n\
         Sec-WebSocket-Version: 13\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\r\n"
    );
    s.write_all(req.as_bytes()).await.unwrap();
    let mut buf = vec![0u8; 512];
    let n = tokio::time::timeout(std::time::Duration::from_secs(5), s.read(&mut buf)).await.expect("an answer").unwrap();
    String::from_utf8_lossy(&buf[..n]).lines().next().unwrap_or_default().to_string()
}

/// KEEL-45: the local server takes plexus-transport 0.10.1's front door as
/// it is (KEEL-29) — the shell's own page opens the socket, a page from any
/// other origin is refused 403, a name rebound to 127.0.0.1 is refused 421,
/// and a client with no `Origin` (synapse, a native client) is admitted.
#[tokio::test]
async fn the_socket_admits_the_shells_page_and_refuses_any_other() {
    let server = plexus_webapp_toy::start().await.expect("start");
    let (ws, shell) = (server.ws_port(), server.shell_port());
    let own = format!("127.0.0.1:{ws}");
    for origin in plexus_webapp::loopback_origins(shell) {
        let line = upgrade(ws, &own, Some(&origin)).await;
        assert!(line.contains(" 101 "), "the shell's own page ({origin}): {line}");
    }
    assert!(upgrade(ws, &own, None).await.contains(" 101 "), "no Origin: a native client");
    assert!(upgrade(ws, &format!("localhost:{ws}"), None).await.contains(" 101 "), "the loopback name");
    for origin in ["http://evil.example", &format!("http://127.0.0.1:{ws}1"), "null"] {
        let line = upgrade(ws, &own, Some(origin)).await;
        assert!(line.contains(" 403 "), "a page from {origin}: {line}");
    }
    let line = upgrade(ws, &format!("evil.example:{ws}"), None).await;
    assert!(line.contains(" 421 "), "a rebound name: {line}");
    server.shutdown().await;
}

/// The status line of `GET path` on `port` naming `host`.
async fn get_as(port: u16, host: &str, path: &str) -> String {
    let mut s = tokio::net::TcpStream::connect(("127.0.0.1", port)).await.expect("the shell accepts");
    s.write_all(format!("GET {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n").as_bytes()).await.unwrap();
    let mut resp = Vec::new();
    s.read_to_end(&mut resp).await.unwrap();
    String::from_utf8_lossy(&resp).lines().next().unwrap_or_default().to_string()
}

/// KEEL-45 (review F4): the shell answers the names its socket does — its
/// loopback names — and a rebound name gets 421, as at the socket.
#[tokio::test]
async fn the_shell_answers_its_loopback_names_and_refuses_any_other() {
    let server = plexus_webapp_toy::start().await.expect("start");
    let shell = server.shell_port();
    for host in [format!("127.0.0.1:{shell}"), format!("localhost:{shell}"), format!("LOCALHOST:{shell}")] {
        assert!(get_as(shell, &host, "/").await.contains(" 200 "), "{host}");
    }
    for host in [format!("evil.example:{shell}"), "evil.example".to_string(), format!("127.0.0.1:{}", server.ws_port())] {
        let line = get_as(shell, &host, "/").await;
        assert!(line.contains(" 421 "), "{host}: {line}");
        assert!(get_as(shell, &host, "/app/app.js").await.contains(" 421 "), "{host}: an asset too");
    }
    server.shutdown().await;
}
