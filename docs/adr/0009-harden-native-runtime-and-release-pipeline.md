# 0009: Harden the native runtime and release pipeline

Date: 2026-07-28

Status: Accepted

## Context

The native AppKit host had reached functional terminal parity, but development
builds and a fixed display polling timer were not sufficient release
boundaries. PTY descendants could outlive a pane, reader threads were detached,
Kitty images crossed the FFI/render boundary on every frame, and saved sessions
could not preserve recursive split topology. Failures were also inconsistently
visible outside a debugger.

Shipping on macOS additionally requires a versioned application bundle,
hardened-runtime signing, notarization, a reproducible archive, a stable update
location, and an explicit trust boundary for executable updates. The project
targets Apple Silicon only.

## Decision

- Release builds are assembled as `Neovide Tabs.app` with versioned bundle
  metadata, a macOS 14 deployment target, an application icon, and
  hardened-runtime entitlements. Developer ID credentials are configuration,
  never repository state. The production path signs with a secure timestamp,
  submits to Apple's notary service, staples the ticket, and runs Gatekeeper
  assessment.
- Native development, packaging, and release scripts reject non-arm64 macOS
  hosts. `just native-release` produces an arm64 ZIP archive and
  schema-versioned SHA-256 manifest. Without Apple credentials it is explicitly
  a development artifact; with both signing and notary credentials it is a
  notarized production artifact.
- A `v<package-version>` tag runs the arm64 production gate and package smoke,
  records GitHub artifact provenance, then publishes the ZIP and manifest as a
  GitHub Release. GitHub Actions workflow artifacts are not a distribution
  channel because their identity, redirect URLs, and retention are run-scoped.
- The packaged application checks the latest GitHub Release after launch and
  from Help → Check for Updates…. Launch checks notify only for a newer semantic
  version; manual checks report all outcomes. The download action opens the
  exact arm64 Release asset.
- The archive and checksum currently share the GitHub Release trust boundary.
  Therefore update discovery and download are supported, but unattended
  replacement or execution is not. A future installer must verify an
  independently trusted signature, such as Ed25519 with a public key embedded
  in the application. This publisher signature is independent of Apple
  Developer ID signing and notarization.
- PTY and embedded-Neovim readers notify AppKit over nonblocking wakeup file
  descriptors. AppKit schedules rendering for runtime events and active
  animation deadlines only; a permanent display polling timer is not retained.
- Pane teardown terminates the PTY process group, reaps the child, and joins the
  reader after its completion signal. Renderer state is forgotten when its
  runtime is removed.
- Kitty RGBA snapshots and Skia images are cached by runtime, image ID, and
  generation. Cache entries are pruned when placements disappear.
- Restorable sessions use schema version 2 and recursively persist split axes,
  pane kinds, working directories, and the active leaf. Version 1 data migrates
  to terminal leaves, corrupt supported data is discarded with a warning, and
  unknown future schemas are preserved.
- Rust emits centrally filtered semantic logs through `NVTERM_LOG`; Swift uses
  unified logging categories for lifecycle, runtime, and session events. Fatal
  core or Metal initialization errors are visible to the user before exit.
- The macOS CI gate verifies Rust, builds the native host and release archive,
  and exercises native rendering, resize, session, and lifecycle smoke paths.
  `just native-soak` repeats high-risk native lifecycle scenarios locally.

## Consequences

Normal idle operation no longer wakes at 60 Hz. Runtime output remains
responsive because readable wakeup descriptors are dispatched on the main
queue, while cursor and scroll animation frames are scheduled by the renderer.

Unsigned/ad-hoc output remains useful for development and CI but is labeled as
such in the manifest and is not a production release. Production publication
still depends on externally supplied Apple Developer credentials and GitHub
release permissions; the repository contains the complete deterministic path
up to those external trust boundaries.

The tag workflow may publish an explicitly labeled, unnotarized arm64 release
for users who approve it through macOS Privacy & Security. GitHub-generated
provenance lets users independently verify the build, but the in-app updater
does not treat provenance or a same-origin checksum as authorization to execute
new code.

Session restore intentionally starts new shell or Neovim processes. Process
memory and PTY byte streams are not serialized.
