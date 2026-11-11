# Features

| feature | gives | dependencies |
| -- | -- | -- |
| `build` | `build::embed_dir*`, for a build script | none |
| `serve` | `AppSource`, `ShellConfig`, `router`, `serve_on` | axum, tokio |
| `local` (default) | `LocalServer`, `LocalConfig`, `rpc_module` | plexus-rpc, jsonrpsee |

A build script takes the crate with `default-features = false` and
`features = ["build"]`.
