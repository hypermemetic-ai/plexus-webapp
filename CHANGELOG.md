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
- **The shell answers only its names, with no open default** (the
  review's F4 and its follow-up). `ShellConfig::hosts` is a `HostList`:
  `HostList::named([...])`, `HostList::loopback(port)` (plexus-transport's
  `host::loopback_hosts`), or `HostList::any()`, the explicit open state.
  `router` refuses a request naming any other `Host` (421, `HOST_REFUSAL`,
  plexus-transport's words) before anything routes it; **with no hosts
  named, `router` answers no one** (it does not know its port).
  `serve_on` checks `Host` again, **outermost**, so a route merged after
  `router` is covered; with no list it answers the loopback names for the
  listener's port, as plexus-transport does; `serve_on_hosts(listener,
  router, hosts, shutdown)` names others. `LocalServer` fills an empty list
  with the shell's loopback names and serves with it. `only_hosts(router,
  &hosts)` is the check alone — a layer covers only the routes present when
  it is added, so a tenant serving the router itself calls it after merging.
  The comparison is written here once, by plexus-transport's rule, because
  `host::refused` (and `host::normalized`, `host::page_hosts`) are
  crate-private there; were `host::refused` public (or a public tower
  layer), this would call it. Test changed: `shell::tests::a_root_file_is_the_builds_file_at_the_root`
  names its loopback hosts (`HostList::loopback(port)`), since a router
  naming none now answers no one.
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
4. **Name your shell's hosts.** A `router(cfg, …)` whose `cfg.hosts` names
   none answers every request 421: set `cfg.hosts =
   HostList::named([...])` (the names a browser uses, loopback ones
   included if you want them; `HostList::loopback(port)` for those alone),
   and serve with `serve_on_hosts(listener, app, cfg_hosts, shutdown)` so
   routes merged after `router` (plexus-login's pages, your own) are
   checked too — `serve_on` alone answers only the loopback names of its
   port. A host that checks `Host` itself says `HostList::any()` in both.
   menger's `shell.rs` and plexus-ui's `serve.rs` call `router` and
   `serve_on` today: both need this step. `LocalServer` does it for you.
5. **A page served from somewhere other than the shell** (a dev server on
   another port, a webview) names its origin in
   `LocalConfig::extra_origins`; `*` refuses the start. Tauri's are
   `TAURI_ORIGINS`.
6. **If your app read `<ns>.list_children` / `<ns>.search_children`, or a
   `….schema` path the caller holds nothing under, on a gated service**:
   they are not served now (KEEL-33) or not found (KEEL-32). Call the list
   method through `{ns}.call` by its own grant, and expect a schema only for
   what the caller's view shows.
