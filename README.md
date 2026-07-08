# neovide-tabs

Experimental terminal built around a Rust terminal core, a Neovide-like
renderer, and a native macOS AppKit/Metal shell. The default `just terminal`
path builds the Rust core/runtime and launches the AppKit host. Terminal bytes
flow through a PTY-backed `libghostty-vt` runtime, while the native host owns
the macOS window, menu, tab UI, and keyboard handling.

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
just native-spike # compatibility alias for just terminal
just native-smoke # launch the native host briefly and write a PNG smoke shot
just terminal-vim-scroll-smoke # verify Vim-style terminal scroll animates in Skia
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
just kitty-smoke # emit a tiny Kitty protocol PNG
just kitty-render-smoke # pending native Kitty renderer smoke test
just adr         # list Architecture Decision Records
just adr-new     # create a new ADR
just check       # cargo check
just lint        # clippy with -D warnings
just test        # cargo test
just fmt         # cargo fmt
just precommit   # fmt --check + lint + test
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
which performs `cargo fmt -- --check`, Clippy with `-D warnings`, and
`cargo test`.

Native macOS shell code lives in [`spikes/macos-shell`](spikes/macos-shell).
It links the Rust core and PTY-backed terminal runtime through a small C ABI,
owns the native window/menu/tab surface, and presents Rust Skia/Metal-rendered
terminal and Neovim panes.

## Configuration

The app reads TOML config from the first available path:

- `$NVTERM_CONFIG`
- `$XDG_CONFIG_HOME/neovide-tabs/config.toml`
- `~/.config/neovide-tabs/config.toml`

See `config.example.toml` for the full shape.

```toml
[ui]
theme = "Graphite"

[font]
latin = "/path/to/CaskaydiaCoveNerdFontMono-Regular.ttf"
cjk = "/path/to/NotoSansCJK-VF.otf.ttc"

[notifications]
agents = true
osc = true
status_files = true
agent_min_busy_seconds = 8

[keybindings]
new_tab = "cmd+t"
split_vertical = "cmd+d"
split_horizontal = "cmd+shift+d"
next_tab = "cmd+shift+]"
previous_tab = "cmd+shift+["
close_active = "cmd+w"
rename_session = "cmd+r"
cycle_theme = "cmd+k"
show_keybindings = "cmd+,"
```

## Session Restore

The app saves tab metadata as TOML and restores it on the next launch. The
saved state includes tab names, active tab/pane, pane layout, theme, and each
pane's last known working directory. It does not keep shell processes alive.

State is written to the first available path:

- `$NVTERM_SESSION`
- `$XDG_STATE_HOME/neovide-tabs/session.toml`
- `~/.local/state/neovide-tabs/session.toml`

Working directory tracking uses OSC 7. Add shell integration so the shell emits
the current directory on prompt and after `cd`.

For zsh:

```sh
_nvterm_osc7() {
  printf '\e]7;file://%s%s\a' "${HOST:-localhost}" "$PWD"
}
autoload -Uz add-zsh-hook
add-zsh-hook precmd _nvterm_osc7
add-zsh-hook chpwd _nvterm_osc7
```

For bash:

```sh
__nvterm_osc7() {
  printf '\e]7;file://%s%s\a' "${HOSTNAME:-localhost}" "$PWD"
}
PROMPT_COMMAND="__nvterm_osc7${PROMPT_COMMAND:+;$PROMPT_COMMAND}"
```

## Notifications

Desktop notifications are enabled by default. On macOS, the app uses
`osascript` so no Homebrew notifier is required.

- `OSC 9;message` and `OSC 777;notify;title;message` are forwarded as desktop
  notifications.
- Each pane gets `NVTERM_AGENT_STATUS_FILE`. Processes inside the pane can write
  explicit status TOML there; terminal states `done`, `blocked`, `failed`, and
  `needs_input` produce desktop notifications.
- Claude Code / Codex panes are watched heuristically by reading terminal screen
  text. A notification is sent when a pane looked busy for at least
  `agent_min_busy_seconds` and then becomes idle or asks for input.

For stable agent completion notifications, prefer the explicit status file path
over screen-text detection:

```sh
./scripts/nvterm-agent-status running "started"
./scripts/nvterm-agent-status done "implemented and tested"
./scripts/nvterm-agent-status blocked "needs user decision"
./scripts/nvterm-agent-status failed "tests failed"
```

Each terminal pane prepends a pane-local shim directory to `PATH`, so Claude
Code and Codex can be started normally while still writing to the same status
path:

```sh
claude
claude --permission-mode bypassPermissions
codex --yolo
codex --sandbox danger-full-access --ask-for-approval never
```

Inside neovide-tabs only, `claude` resolves through a wrapper that adds a
temporary `--settings` file for `UserPromptSubmit`, `Notification`, and `Stop`.
`codex` resolves through a wrapper that adds a per-invocation `-c notify=...`
override. Global `~/.claude/settings.json` and `~/.codex/config.toml` are not
modified. Existing Codex `notify` command arrays are delegated at runtime under
`$XDG_STATE_HOME/neovide-tabs` or `~/.local/state/neovide-tabs`.

The heuristic Claude Code / Codex watcher is a fallback for unwrapped shells. It
does not know task semantics and can break when an agent changes its visible
UI. The status-file path is closer to cmux's stability model because the running
process explicitly declares its state.

## Current MVP

- Launches the native AppKit terminal host through `just terminal`.
- Spawns the user's shell in a real PTY from the Rust runtime.
- Feeds PTY output through `libghostty-vt`.
- Renders terminal and Neovim panes through the Rust Skia/Metal adapter.
- Supports basic printable input, control keys, arrows, delete, home/end, and
  page up/down.
- Keeps tab/session metadata in the Rust core and native tab/menu UI.
- Animates cursor movement in the Rust Skia/Metal renderer with a
  Neovide-style trail.
- Scrolls `libghostty-vt` history from the native wheel event and animates the
  retained terminal window through the Skia/Metal renderer.
- Launches an experimental native Neovim UI pane through `just neovim`, backed
  by `nvim --embed`, `ext_multigrid`, and a Rust editor/window compositor
  instead of terminal cell diffing.

## Scroll Design

`src/scroll.rs` owns the spring model. It is intentionally independent from the
PTY, terminal emulator, and renderer.

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

The current renderer boundary also exposes `nvterm_nvim_renderer_model_json`.
Schema version 1 returns the background, cursor, and retained windows with
screen placement, window kind, z-order, hidden state, scroll animation position,
and colored cell lines. That model is the intended input for the future
Skia/Metal Neovim surface.

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
`NeovideRendererModelSnapshot` and does not call `nvterm_nvim_frame_json`.

Neovim text in the Skia/Metal path is shaped by a Neovide-derived Rust shaper
instead of direct `Canvas::draw_str` cell drawing. The shaper uses `swash` to map
grapheme clusters onto grid-cell positions, caches Skia `TextBlob`s with `lru`,
loads `$NVTERM_FONT` as the primary face, falls back through Skia `FontMgr`
character matching, and keeps bundled Neovide font assets as default and last
resort faces. Cell style now carries bold, italic, underline, and strikethrough
from Neovim highlights and terminal SGR state into the renderer; bold and italic
participate in font fallback, while underline and strikethrough are drawn as
grid-aligned decorations.
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

`nvterm_nvim_frame_json` remains only as a compatibility/debug frame output.
The smoke recipes require Skia frames; `native-smoke` also checks
`skia-frames=yes`, so frame-only or AppKit-only fallbacks no longer pass the
native smoke gate.

## Next Work

- Add a proper key encoder from `libghostty-vt::key` instead of manual escape
  strings.
- Add mouse reporting for terminal-pane alternate-screen apps.
- Expand the native Neovim pane compositor to cover more Neovide behavior:
  externalized windows and richer mouse input such as drag selection.
- Expand deterministic visual coverage for shaped Japanese text, Nerd Font
  symbols, combining marks, ambiguous-width characters, and more Neovim UI
  surface combinations in the Skia/Metal path.
