# plexus-webapp

Formerly `plexus-shell`, published on crates.io up to 0.1.3.

A Plexus RPC app's frontend, served beside its service — and the whole server
started as a library call.

A repository of its own beside plexus-core. It began inside
[menger](https://github.com/hypermemetic-ai/menger), which still takes it
(by sibling path in a checkout); any Plexus service can.

- **The shell.** The frontend's build (a `trunk build` or `vite build`
  output) is read into a table and served at `/` and under `/app/`. A
  request path is a key into the table, never a path on disk. The
  entrypoint is stamped as it is read: `{{WS_PORT}}` (the service's WebSocket
  port), `{{TITLE}}`, and any placeholder the app adds. The build comes from
  an `AppSource`: `Dir(path)` (read again when `index.html` changes) or
  `Embedded(table)` (compiled into the binary).
- **The embedding.** The app crate's build script writes the table; the
  app's code includes it.
- **`LocalServer`.** Any `DynamicHub` on a Plexus WebSocket, plus the shell,
  on loopback. Port 0 picks a free port. Both listeners are bound before
  anything serves. Shut it down to release the ports. `TAURI_ORIGINS` are
  the origins to admit when the page opens in a Tauri webview.

## Use

```toml
[features]
embedded-app = []

[dependencies]
plexus-webapp = "0.4"

[build-dependencies]
plexus-webapp = { version = "0.4", default-features = false, features = ["build"] }
```

```rust
// build.rs: embed frontend/dist when `embedded-app` is on
fn main() {
    plexus_webapp::build::embed_dir_with_feature("frontend/dist", "embedded-app");
}
```

```rust
use plexus_webapp::{AppSource, EmbeddedTable, LocalConfig, LocalServer, ShellConfig};

static APP: EmbeddedTable = plexus_webapp::embedded_app!();

async fn run(service: std::sync::Arc<plexus_core::plexus::DynamicHub>) -> anyhow::Result<()> {
    // the embedded build when there is one, else the checkout's dist
    let app = AppSource::embedded_or(APP, concat!(env!("CARGO_MANIFEST_DIR"), "/frontend/dist"));
    let server = LocalServer::start(service, LocalConfig::new(ShellConfig::new("myapp", app))).await?;
    println!("{} over {}", server.shell_url(), server.ws_url());
    server.shutdown().await;
    Ok(())
}
```

To use your own listener, routes or layers, call `router(cfg, ws_port)` and
`serve_on(listener, router, shutdown)`. `ShellConfig` carries the extras:
`routes` (merged in), `pages` (stamped HTML at a route), `stamps`,
`root_fallback` (served at `/` when there is no build), `not_built`,
`mount` (default `/app`), and `csp`.

Every answer is hardened (`nosniff`, `Referrer-Policy: same-origin`, no
framing), and every page the shell serves goes out under `csp` (default
`DEFAULT_CSP`), a template: `{{SCRIPTS}}` becomes the page's inline-script
hashes (read when the build is), `{{SERVICE}}` the service's WebSocket origins at the
name the page was asked for. The default allows scripts from the app's own
files, the wasm and those hashes, and connections to the page's origin and
its service.

The policies a Plexus app serves are named (`Policy`): `PAGE_CSP` (the
page, `DEFAULT_CSP`), `FRAME_CSP` (a document in a sandboxed frame) and
`BARE_CSP` (a served file that is not a page). Take one rather than writing
your own string.

The local server's socket admits a page only from the shell's own origin
and `extra_origins` (any other `Origin`: 403) and answers only its loopback
names (any other `Host`: 421) — plexus-transport's defaults, which nothing
here opens. A client with no `Origin` is admitted.

## Features

| feature | gives | dependencies |
|---|---|---|
| `build` | `build::embed_dir*` for a build script | none (always compiled) |
| `serve` | `AppSource`, `ShellConfig`, `router`, `serve_on` | axum, tokio |
| `local` (default) | `LocalServer`, `LocalConfig`, `rpc_module` | + plexus-rpc (plexus-core, plexus-transport), jsonrpsee |

A worked example lives in this repository at `toy/` (a workspace member). It is a service
with one activation and an embedded two-file app, and its test runs it.

## Build and test

A checkout builds against the plexus 0.11 line from crates.io, through the
`plexus-rpc` 0.11 umbrella, with the security releases as its floor:
plexus-core 0.11.1, plexus-transport 0.10.1, plexus-auth-core 0.3.2 and
plexus-macros 0.11.1 (KEEL-45). Moving from 0.3 (the 0.8 line): CHANGELOG.md,
"Moving from 0.3".

```sh
cargo test --workspace   # the crate's tests and the toy's
cargo run -p plexus-webapp-toy
```

MIT.
