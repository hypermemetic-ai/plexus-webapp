# plexus-webapp

A Plexus RPC app's frontend build, served beside its service, and the whole
server started as one library call.

## Most people want plexus-rpc

[plexus-rpc](https://github.com/hypermemetic-ai/plexus-rpc) is the one
dependency a service takes, and its README builds one. This crate is what an
app adds when its service also serves a web page.

## Install

```toml
[features]
embedded-app = []

[dependencies]
plexus-webapp = "0.4"

[build-dependencies]
plexus-webapp = { version = "0.4", default-features = false, features = ["build"] }
```

## A service and its page

The build script embeds the frontend's build in the binary.

<!-- toy/build.rs -->
```rust
fn main() {
    plexus_webapp::build::embed_dir_with_feature("app", "embedded-app");
}
```

The app starts its service and the page on free loopback ports.

<!-- toy/src/lib.rs -->
```rust
/// app/, as build.rs embedded it.
pub static APP: EmbeddedTable = plexus_webapp::embedded_app!();
```

<!-- toy/src/lib.rs -->
```rust
pub async fn start() -> anyhow::Result<LocalServer> {
    let mut shell = ShellConfig::new("toy", AppSource::Embedded(APP));
    shell.stamps.push(("{{GREETING}}".into(), "hello from plexus-webapp".into()));
    LocalServer::start(service(), LocalConfig::new(shell)).await
}
```

<!-- toy/src/main.rs -->
```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server = plexus_webapp_toy::start().await?;
    println!("app {} · service {}", server.shell_url(), server.ws_url());
    tokio::signal::ctrl_c().await?;
    server.shutdown().await;
    Ok(())
}
```

[`toy/`](toy) is this app. `cargo test --workspace` serves it, reads its
page, calls its service, and holds this page to its files;
`cargo run -p plexus-webapp-toy` serves it until Ctrl-C.

## More

- [The shell](docs/shell.md): what is served where, the placeholders, a listener of your own.
- [Hosts, origins and the content policy](docs/security.md): who a page and its socket answer.
- [Features](docs/features.md): `build`, `serve`, `local`.
- [docs.rs/plexus-webapp](https://docs.rs/plexus-webapp)
- [CHANGELOG.md](CHANGELOG.md)

MIT.
