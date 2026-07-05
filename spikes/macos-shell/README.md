# macOS Native Shell

This native shell is the first executable AppKit/Metal path for ADR 0002's
target architecture:

```text
Swift/AppKit shell
  native windows, tabs, menus, settings, keybinding UI, command routing

Rust terminal core/runtime
  PTY, libghostty-vt terminal state, sessions, panes, notifications, image state

AppKit shell + MTKView surface
  native tabs/menus plus Rust Skia/Metal terminal and Neovim surfaces
```

This path is launched by `just terminal` and `just native-spike`. The Rust
library owns terminal parsing and PTY lifecycle; Swift owns the native macOS UI
and presents Rust Skia/Metal-rendered terminal panes exposed through the C ABI.

## Build

Use the repository command so Rust is built inside the Nix shell while Swift is
compiled with the host Xcode toolchain and SDK:

```sh
just native-build
```

That builds:

- `target/debug/libneovide_tabs.a`
- `spikes/macos-shell/.build/NativeShellSpike`

Run the native host with:

```sh
just terminal
```

Run a non-interactive GUI smoke check with:

```sh
just native-smoke
```

The smoke check launches the app briefly, writes
`spikes/macos-shell/.build/native-smoke.png`, and exits.

## Current Coverage

- Swift calls the Rust C ABI through a thin `RustCore` wrapper.
- Rust exposes a JSON snapshot of tabs, panes, active tab, layout, and theme.
- Rust exposes a PTY-backed `NativeTerminalRuntime` per pane.
- AppKit owns the native tab bar, main menu, rename dialog, and context menu.
- The context menu updates tab name and color theme through Rust.
- `TerminalMetalView` is an `MTKView` that presents the Rust Skia/Metal renderer.
- The host starts login shells through the Rust runtime and renders terminal
  text cells from `libghostty-vt` frame snapshots in Rust/Skia.
- Native wheel events scroll the Rust `libghostty-vt` viewport and animate the
  retained terminal window in Rust/Skia.
- Swift reads `RendererContract` from Rust to configure the Metal surface.
- `just native-smoke` verifies that the native host can launch and render a
  window snapshot, native tabs, dark terminal surface, and PTY text without
  manual interaction. It also requires `skia-frames=yes`.

## Next Questions

- Should keybinding resolution move behind the Rust core so CLI/API automation
  can share the same commands?
- What is the first Kitty graphics protocol placement shape that should cross
  the C ABI?
