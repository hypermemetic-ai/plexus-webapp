# The shell

The frontend's build, the output of `trunk build` or `vite build`, is read
into a table and served at `/` and under `/app/`. A request path is a key
into the table, never a path on disk.

## Where the build comes from

`AppSource` names it.

| source | what it is |
| -- | -- |
| `AppSource::Dir(path)` | a directory, read again when its `index.html` changes |
| `AppSource::Embedded(table)` | the table the build script compiled into the binary |
| `AppSource::embedded_or(table, path)` | the embedded build when there is one, else the directory |

`plexus_webapp::build::embed_dir_with_feature(dir, feature)` in a build
script writes the table when `feature` is on, and
`plexus_webapp::embedded_app!()` includes it.

## Placeholders

The entrypoint is stamped as it is read.

| placeholder | becomes |
| -- | -- |
| `{{WS_PORT}}` | the port of the service's WebSocket |
| `{{TITLE}}` | the shell's title |
| any pair in `ShellConfig::stamps` | the value the app gives |

## `LocalServer`

`LocalServer::start(service, LocalConfig::new(shell))` serves a service on a
Plexus WebSocket and its shell, on loopback. Port 0 is a free port. Both
listeners are bound before anything serves. `shell_url()` and `ws_url()` are
where they are; `shutdown()` releases the ports. `TAURI_ORIGINS` are the
origins to admit when the page opens in a Tauri webview.

## A listener of your own

`router(cfg, ws_port)` is the shell's axum router, and
`serve_on(listener, router, shutdown)` (or `serve_on_hosts`) serves it.

`ShellConfig` carries the rest:

| field | what it is |
| -- | -- |
| `routes` | routes merged into the router |
| `pages` | stamped HTML at a route |
| `stamps` | the app's placeholders |
| `root_fallback` | what `/` serves when there is no build |
| `not_built` | what is said when there is no build |
| `mount` | where the build is mounted; `/app` |
| `hosts` | the `Host` names the shell answers |
| `csp` | the content policy of every page |
