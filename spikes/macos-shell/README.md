# macOS Native Shell Spike

This spike is the first step toward ADR 0002's target architecture:

```text
Swift/AppKit shell
  native windows, tabs, menus, settings, keybinding UI, command routing

Rust terminal core
  PTY, terminal state, sessions, panes, notifications, image protocol state

MTKView/Metal renderer
  terminal cells, cursor animation, smooth scroll, inline images/textures
```

The spike intentionally does not replace the current `macroquad` prototype. Its
job is to prove that the future host can own native macOS UI while rendering the
terminal through an `MTKView`.

## Build

On macOS with Xcode command line tools installed:

```sh
swiftc NativeShellSpike.swift \
  -framework AppKit \
  -framework MetalKit \
  -o .build/NativeShellSpike
```

Then run:

```sh
./.build/NativeShellSpike
```

## Next Questions

- How should the Swift host call the Rust core: C ABI, UniFFI, or a thin manual
  FFI layer?
- Which event boundary should own keybinding resolution: AppKit shell or Rust
  core?
- What is the minimum renderer contract needed to draw cells, animated cursor,
  smooth scroll offsets, and Kitty image placements?
