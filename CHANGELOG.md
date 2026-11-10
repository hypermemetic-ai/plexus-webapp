# Changelog

All notable changes to plexus-webapp are documented here. This file starts
at 0.4.0; earlier changes are in the git history (tags `plexus-webapp-v*`).

## [0.4.0] — unreleased

KEEL-45: on the fixed Plexus line, 0.11 — plexus-rpc 0.11.0, and at least
plexus-core 0.11.1, plexus-transport 0.10.1, plexus-auth-core 0.3.2 and
plexus-macros 0.11.1, the security releases of 2026-11-10 (KEEL-19, KEEL-29,
KEEL-31, KEEL-32, KEEL-33). 0.3.x was on the 0.8 line. A new minor because
plexus-core's and plexus-transport's types are in this crate's API
(`LocalServer::start*` takes a `DynamicHub`, a `SessionValidator` and
`UpgradeOptions`; `rpc_module` answers a jsonrpsee module of plexus-core's
building). No source change was needed in the crate for the move.

### Changed

- **The local server's front door is plexus-transport 0.10.1's, as it is.**
  The socket admits a page from the shell's own origin and from
  `LocalConfig::extra_origins`, refuses any other `Origin` (403), and
  answers only `127.0.0.1:<port>` and `localhost:<port>` (any other `Host`:
  421, so a name rebound to 127.0.0.1 gets nothing). A client that sends no
  `Origin` (synapse, a native client, a test) connects as before. Nothing in
  the crate opts out: no `allow_any_origin()`, no `allow_any_host()`.
- An `extra_origins` entry `*` was a list entry that matched nothing; now
  the listener refuses to start (plexus-transport 0.10.1: `*` is no
  wildcard).
- `loopback_origins(port)` is built from plexus-transport's
  `host::loopback_hosts(port)` (the names its listeners answer to by
  default), each as an `http://` origin. Same value as before.

### Added

- **Named policies** (the consolidation review's row 16, KEEL-40):
  `Policy::{Page, Frame, Bare}` and `Policy::template`; `PAGE_CSP` (the
  app's page, the same string as `DEFAULT_CSP`), `FRAME_CSP` (a document in
  a sandboxed frame: plexus-ui's `/frame` policy, verbatim) and `BARE_CSP`
  (a served file that is not a page: menger's media policy, verbatim). A
  tenant takes one of these rather than keeping its own string.
- `tests/toy.rs`: `the_socket_admits_the_shells_page_and_refuses_any_other`
  — the shell's origins open the socket, a foreign origin and `null` are
  403, a rebound name is 421, no `Origin` is admitted.

### Moving from 0.3 (the Plexus 0.8 line)

In this order:

1. **Move the service to the 0.11 line with this crate.** `plexus-rpc =
   "0.11.0"` (or plexus-core 0.11.1, plexus-transport 0.10.1,
   plexus-auth-core 0.3.2, plexus-macros 0.11.1 named directly) and
   `plexus-webapp = "0.4"` in one change: the `DynamicHub` you hand
   `LocalServer` must be the plexus-core this crate links. Run `cargo update
   -p plexus-core -p plexus-transport -p plexus-auth-core -p plexus-macros`
   so the lockfile holds the security releases, not 0.11.0 / 0.10.0.
2. **Your service's own code, across plexus-core 0.9, 0.10 and 0.11** — the
   breaks are plexus-core's, read its CHANGELOG for each; the ones a tenant
   of this crate meets:
   - 0.9 (RETURNS-47): `current_reach()` is `Result<Reach, PlexusError>`
     (add `?`); `Activation::call` called straight runs in no call context,
     so a test that calls a method outside a service goes through a
     `DynamicHub`, `call::ungated`, or `call::test::in_call` (feature
     `test-util`).
   - 0.10 (ASK-1, ASK-2): the in-call question is gone — `BidirChannel`,
     `#[method(bidirectional)]`, `{ns}.respond`. A method that needs a yes
     takes it as a parameter.
   - 0.11 (NEST-16, NEST-17): every `#[activation]` emits a trait,
     `{Name}ActivationSchema`, into its module (rename a clashing item, or
     `trait = "…"`); `#[child(list = "…")]`'s string form is deprecated.
3. **Mark every method that acts on the whole service** `#[method(everywhere)]`
   (plexus-macros 0.11.1, KEEL-31): the gate then admits it only for a
   holder on `*`. Without the word a method with no target is admitted to a
   role held on any scope. Remove the hand-written `is_everywhere()` check
   it replaces.
4. **A page served from somewhere other than the shell** (a dev server on
   another port, a webview) names its origin in
   `LocalConfig::extra_origins`; `*` refuses the start. Tauri's are
   `TAURI_ORIGINS`.
5. **If your app read `<ns>.list_children` / `<ns>.search_children`, or a
   `….schema` path the caller holds nothing under, on a gated service**:
   they are not served now (KEEL-33) or not found (KEEL-32). Call the list
   method through `{ns}.call` by its own grant, and expect a schema only for
   what the caller's view shows.
