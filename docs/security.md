# Hosts, origins and the content policy

## Hosts

A shell answers the hosts it names (`ShellConfig::hosts`, a `HostList`).

| the shell | answers |
| -- | -- |
| `router`, no host named | no one: `421` |
| `serve_on`, no host named | the loopback names for its port |
| `HostList::any()` | every host |

`serve_on` checks `Host` first, over routes merged after `router` too. A
program that serves the router itself wraps it in `only_hosts` after merging
its routes.

## The socket

The local server's socket admits a page from the shell's own origin and
from `extra_origins`; any other `Origin` is answered `403`. It answers its
loopback names; any other `Host` is answered `421`. A client that sends no
`Origin` is admitted.

## Every answer

Every answer carries `nosniff`, `Referrer-Policy: same-origin`, and no
framing.

## The content policy

Every page goes out under `ShellConfig::csp`, a template.

| in the template | becomes |
| -- | -- |
| `{{SCRIPTS}}` | the hashes of the page's inline scripts, read when the build is |
| `{{SERVICE}}` | the service's WebSocket origins, at the name the page was asked for |

`DEFAULT_CSP` allows scripts from the app's own files, the wasm and those
hashes, and connections to the page's origin and its service.

The policies are named, in `Policy`:

| policy | for |
| -- | -- |
| `PAGE_CSP` | a page; it is `DEFAULT_CSP` |
| `FRAME_CSP` | a document in a sandboxed frame |
| `BARE_CSP` | a served file that is not a page |
