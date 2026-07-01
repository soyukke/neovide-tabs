# neovide-tabs

Experimental terminal renderer built around `libghostty-vt`, with Neovide-like
scroll animation as a first-class part of the architecture.

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
just terminal    # launch the terminal prototype
just adr         # list Architecture Decision Records
just adr-new     # create a new ADR
just check       # cargo check
just test        # cargo test
just fmt         # cargo fmt
just verify      # fmt --check + check + test
just doctor      # print tool versions and selected font
```

## Architecture Decisions

Architecture Decision Records live in [`docs/adr`](docs/adr). They capture the
long-term direction for decisions such as the native macOS shell, Rust terminal
core, Metal renderer, and Kitty graphics protocol support.

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

- Spawns the user's shell in a real PTY.
- Feeds PTY output through `libghostty-vt`.
- Renders `RenderState` rows/cells with `macroquad`.
- Supports basic printable input, control keys, arrows, delete, home/end, and
  page up/down.
- Smooths scrollback wheel movement with fractional row offsets.
- Detects simple vertical row shifts between frames and animates them. This is
  the first hook for Neovide-style output/TUI scrolling.

## Scroll Design

`src/scroll.rs` owns the spring model. It is intentionally independent from the
PTY, terminal emulator, and renderer.

There are two scroll sources:

- History scroll: integer rows are applied to `libghostty-vt`; fractional rows
  are held in the renderer and settled back to a cell boundary after wheel idle.
- Screen shift: when the visible rows look like they moved up/down between two
  frames, the renderer starts from the old visual position and springs to the new
  one.

The second path is approximate. A better version should use terminal mutation
events or a libghostty-side scroll-region signal instead of row-string diffing.

## Next Work

- Replace `macroquad` text drawing with a real terminal renderer, likely wgpu or
  Metal on macOS.
- Add a proper key encoder from `libghostty-vt::key` instead of manual escape
  strings.
- Add mouse reporting for alternate-screen apps.
- Add tabs as separate PTY + `Terminal` + `RenderState` instances.
- Improve scroll detection for scroll regions, alternate screen, and Neovim.
