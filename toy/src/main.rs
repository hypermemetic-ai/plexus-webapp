//! `cargo run -p plexus-webapp-toy`: the toy until Ctrl-C.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server = plexus_webapp_toy::start().await?;
    println!("app {} · service {}", server.shell_url(), server.ws_url());
    tokio::signal::ctrl_c().await?;
    server.shutdown().await;
    Ok(())
}
