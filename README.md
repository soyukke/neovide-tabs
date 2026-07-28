# neovide-tabs

Experimental terminal built around a Rust terminal core, a Neovide-like
renderer, and a native macOS AppKit/Metal shell. The default `just terminal`
path builds the Rust core/runtime and launches the AppKit host. Terminal bytes
flow through a PTY-backed `libghostty-vt` runtime, while the native host owns
the macOS window, menu, tab UI, and AppKit event translation.

Neovide Tabs supports Apple Silicon Macs running macOS 14 or newer. Intel Macs
are not supported.

## Run

```sh
./scripts/dev
```

or:

```sh
nix develop --command just terminal
```

The dev environment is managed by Nix flakes. `libghostty-vt-sys` currently
needs Zig 0.15.2 for the Ghostty commit it builds, so the flake uses
`nixpkgs#zig_0_15`.

```sh
nix develop
just terminal
```

## Commands

```sh
just             # list recipes
just terminal    # build and launch the native AppKit terminal host
just neovim      # build and launch the native Neovim UI pane
just native-build # build the AppKit/Metal host without launching it
just native-package # build and locally verify a hardened-runtime .app
just native-release # build a ZIP plus checksummed update manifest
just native-update-test # test version ordering and GitHub update metadata
just native-notarize # Developer ID sign, notarize, staple, and assess the .app
just native-smoke # launch the native host briefly and write a PNG smoke shot
just native-package-smoke # visually verify the exact Release .app executable
just native-resize-smoke # verify window resizing reaches terminal pane grids
just native-session-smoke # verify v2 sessions, migration, and corruption policy
just native-soak # repeat high-risk native lifecycle and input smokes
just terminal-vim-scroll-smoke # verify Vim-style terminal scroll animates in Skia
just terminal-nvim-handoff-smoke # verify explicit native Neovim pane replacement
just terminal-nvim-cwd-smoke # verify native Neovim inherits the terminal cwd
just terminal-nvim-quit-smoke # verify :qa returns the pane to a terminal
just nvim-skia-smoke # screenshot smoke for the native Neovim Skia/Metal pane
just nvim-smoke-all # run deterministic Neovim scroll/jump/pane/cmdline smokes
just nvim-smoke-shaped-text-visual # verify shaped glyph pixels in a screenshot
just nvim-smoke-ui-surfaces # verify split/float/message surfaces and float blend
just nvim-smoke-popupmenu # verify popupmenu model and Skia pixels
just nvim-smoke-cursor-normal-shape # verify normal-mode block cursor in Skia
just nvim-smoke-cursor-shape # verify Neovim mode cursor shape in Skia
just nvim-smoke-cursor-replace-shape # verify replace-mode horizontal cursor in Skia
just nvim-smoke-cursor-blink # verify cursor blink off phase in Skia
just nvim-smoke-cursor-switch # verify cursor body/trail cleanup after tab switch
just adr         # list Architecture Decision Records
just adr-new     # create a new ADR
just check       # cargo check
just lint        # clippy with -D warnings
just test        # cargo test
just fmt         # cargo fmt
just precommit   # fmt --check + lint + test
just ops-lint    # ShellCheck release/smoke scripts and validate Actions YAML
just secrets-staged # scan staged changes for secrets, as the commit hook does
just secrets     # scan all Git history plus the current worktree
just install-hooks # use repo-managed Git hooks
just verify      # fmt --check + check + lint + test
just doctor      # print tool versions and selected font
```

## Architecture Decisions

Architecture Decision Records live in [`docs/adr`](docs/adr). They capture the
long-term direction for decisions such as the native macOS shell, Rust terminal
core, Metal renderer, and Kitty graphics protocol support.

The Rust lint gate is intentionally strict: `just verify` runs
`cargo clippy --all-targets --all-features -- -D warnings`. New code should
either satisfy the lint or have a narrow, explicit reason for an allow.
Function bodies are capped at 70 lines via Clippy, and Rust formatting is capped
at 100 columns via `rustfmt.toml`.

Git pre-commit hooks live in [`.githooks`](.githooks). Run `just install-hooks`
once per clone to make Git use them. The pre-commit hook runs `just precommit`,
which first scans staged changes with Gitleaks, then performs
`cargo fmt -- --check`, Clippy with `-D warnings`, `cargo test`, ShellCheck, and
GitHub Actions validation. Gitleaks is provided by the Nix development shell;
no Homebrew or globally installed hook framework is required. `just secrets`
performs the publication-grade full-history and worktree scan used by CI.

Please report vulnerabilities through GitHub's private vulnerability reporting
flow as described in [`SECURITY.md`](SECURITY.md), not in a public issue.

Native macOS shell code lives in [`spikes/macos-shell`](spikes/macos-shell).
It links the Rust core and PTY-backed terminal runtime through a small C ABI,
owns the native window/menu/tab surface, and presents Rust Skia/Metal-rendered
terminal and Neovim panes.

## Release

`just native-release` creates the Apple Silicon archive
`spikes/macos-shell/.build/release/Neovide-Tabs-<version>-macOS-arm64.zip` and
`latest.json`. With no Apple credentials it produces an ad-hoc signed artifact
and labels the manifest as `development`.

Pushing a tag that exactly matches the Cargo package version publishes a GitHub
Release after the full verification and packaged-application smoke gates pass:

```sh
git tag v0.1.0
git push origin v0.1.0
```

The tag workflow runs on GitHub's Apple Silicon runner, attaches the arm64 ZIP
and manifest to the Release, generates release notes, and records GitHub build
provenance. Workflow artifacts remain CI diagnostics; the stable distribution
and update channel is GitHub Releases.

For a production artifact, configure a Developer ID Application identity in
`APPLE_SIGNING_IDENTITY` and a `notarytool` keychain profile name in
`APPLE_NOTARY_PROFILE`, then run `just native-release`. That route adds a secure
timestamp, notarizes and staples the bundle, validates the ticket, runs
Gatekeeper assessment, and only then writes the release archive and SHA-256
manifest. Signing credentials and notary secrets are never stored in this
repository.

The packaged application checks GitHub's latest Release once after launch. It
stays silent when current or offline and presents an alert only when a newer
semantic version is available. Help → Check for Updates… performs the same
check interactively and reports every outcome. Download Update opens the exact
arm64 Release asset.

The manifest checksum permits manual detection of accidental corruption but is
fetched from the same GitHub trust boundary as the archive. It is not an
independent publisher signature, so the application does not silently replace
or execute itself. Automatic installation requires a separately trusted updater
signature, such as an embedded Ed25519 public key; this does not require an
Apple Developer ID.

Rust diagnostic verbosity is controlled by
`NVTERM_LOG=off|error|warn|info|debug|trace`; native lifecycle/runtime/session
events use macOS unified logging.

## Terminal Features

- Launches the native AppKit terminal host through `just terminal`.
- Spawns the user's shell in a real PTY from the Rust runtime.
- Feeds PTY output through `libghostty-vt`.
- Renders terminal and Neovim panes through the Rust Skia/Metal adapter.
- Encodes keys and mouse events with `libghostty-vt` using active terminal
  modes, including application cursor mode, bracketed paste, and focus events.
- Supports AppKit IME composition, native copy/paste, drag/rectangular
  selection, select-all, scrollback search, command-clickable OSC 8 and plain
  URL links, OSC title updates, bell feedback, and a native scroll indicator.
- Keeps recursive split layout and tab metadata in the Rust core. AppKit renders
  every split leaf into a clipped region of one shared Metal drawable and routes
  focus/input to the selected pane.
- Inherits the active pane working directory for new tabs, splits, and explicit
  native Neovim replacement.
- Stores font size, Option-as-Alt, bell attention, session-restore preferences,
  and versioned recursive tab/pane session metadata in macOS `UserDefaults`.
- Uses runtime wakeup descriptors and renderer animation deadlines instead of
  permanent 60 Hz polling, and tears down PTY process groups and reader threads
  when panes close.
- Animates cursor movement in the Rust Skia/Metal renderer with a
  Neovide-style trail.
- Scrolls `libghostty-vt` history from the native wheel event and animates the
  retained terminal window through the Skia/Metal renderer.
- Launches an experimental native Neovim UI pane through `just neovim` or the
  File → Open Native Neovim command, backed by `nvim --embed`, `ext_multigrid`,
  and a Rust editor/window compositor instead of terminal cell diffing. Typing
  `nvim` in a shell remains ordinary terminal input and is never intercepted.
- Preserves terminal faint, blink, overline, strikethrough, underline variants,
  and underline colors through the retained model.
- Decodes Kitty graphics through `libghostty-vt` and composites visible image
  placements in Skia/Metal with pane clipping and z-order. RGBA and Skia image
  objects are cached by image generation and released with their pane runtime.

## Native Shortcuts

- `Command-T`: new tab
- `Command-D` / `Command-Shift-D`: vertical / horizontal split
- `Command-W`: close the active pane
- `Command-N`: explicitly replace the active terminal pane with native Neovim
- `Command-C`, `Command-V`, `Command-A`, `Command-F`: copy, paste, select all,
  and scrollback search
- `Command-+`, `Command--`, `Command-0`: terminal font size

## Scroll Design

`src/neovide_render.rs` owns the retained-window spring model. The terminal and
Neovim runtimes feed scroll events into the same renderer state.

There are two scroll sources targeted by the renderer:

- History scroll: integer rows are applied to `libghostty-vt`; fractional rows
  are held in the renderer and settled back to a cell boundary after wheel idle.
- Screen shift: when the visible rows look like they moved up/down between two
  frames, the renderer starts from the old visual position and springs to the new
  one.

The native host keeps AppKit responsible for tabs, menus, input routing, and
context menus. Cell drawing is owned by the Rust Skia/Metal adapter for both
normal terminal panes and native Neovim panes.

The native Neovim pane uses Neovim `ext_multigrid` redraw events to keep
separate grids for editor windows, floating windows, messages, cmdline, and file
tree panes. Rust now emits a Neovide-derived retained command batch from those
events: `grid_line` produces `DrawLine`, `grid_scroll` produces `Scroll`, and
`win_viewport` produces `Viewport`. Neovim scroll animation is driven by
event-origin command hints instead of snapshot-diff guessing.

The renderer model contains the background, cursor, and retained windows with
screen placement, window kind, z-order, hidden state, scroll animation position,
and colored cell lines. The Skia/Metal adapter consumes this model directly.

`just terminal` and `just neovim` draw content from retained renderer models.
The Rust Skia/Metal adapter wraps the current `MTKView` drawable, draws the
retained terminal or Neovim windows, and owns cursor body/trail rendering. The
AppKit overlay is limited to native UI such as tabs, menus, dialogs, and context
menus.

The Rust renderer model also carries viewport margins, scrollback line source,
scroll position, and event-origin scroll hint metadata. The Skia/Metal adapter
uses those fields to draw animated Neovim scrollback inside the scrollable inner
region while fixed rows such as statusline-like margins stay outside the
scrolling clip. The native nvim path reads those hints from
`NeovideRendererModelSnapshot`.

Neovim text in the Skia/Metal path is shaped by a Neovide-derived Rust shaper
instead of direct `Canvas::draw_str` cell drawing. The shaper uses `swash` to map
grapheme clusters onto grid-cell positions, caches Skia `TextBlob`s with `lru`,
loads `$NVTERM_FONT` as the primary face, falls back through Skia `FontMgr`
character matching, and keeps bundled Neovide font assets as default and last
resort faces. Cell style carries bold, italic, faint, blink, underline
variants/colors, strikethrough, and overline from terminal SGR state into the
renderer; bold and italic participate in font fallback while decorations are
grid-aligned.
`nvim-shaped-text-visual` captures the Skia/Metal surface and checks that the
Japanese, Nerd Font, combining-mark, and ambiguous-width fixture cells contain
visible glyph pixels at the retained-model coordinates.
`nvim-smoke-cursor-switch` captures the Skia/Metal surface after switching tabs
and checks that the active tab's cursor body is visible while the previous tab's
cursor/trail and marker text are absent.
The cursor shape smokes drive Neovim `mode_info_set` / `mode_change` through
normal block, insert `ver25`, and replace `hor20` modes and check both the
renderer model and captured pixels. `nvim-smoke-cursor-blink` verifies the
Skia/Metal cursor body is hidden during the configured blink off phase without
requiring continuous redraw.
`nvim-smoke-ui-surfaces` also captures the Skia/Metal surface and verifies that
floating-window highlight `blend` values become alpha-composited background
pixels instead of opaque cells.
`nvim-smoke-popupmenu` drives command-line completion through Neovim
`ext_popupmenu`, then checks both the retained popupmenu model and captured
Skia/Metal glyph pixels.

The smoke recipes require Skia frames; `native-smoke` also checks
`skia-frames=yes`.

## Remaining Direction

The native Neovim compositor can continue toward deeper Neovide parity,
including externalized windows and richer editor-side drag selection. Those
renderer extensions remain event-driven; terminal behavior stays owned by
`libghostty-vt`.

## License

The neovide-tabs source is available under the [MIT License](LICENSE),
copyright © 2026 soyukke. Adapted code, bundled fonts, and other third-party
components remain under their respective licenses as documented in
[`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md).
