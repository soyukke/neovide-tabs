# 0005: Retire the macroquad frontend

Date: 2026-07-02

Status: Accepted

## Context

The terminal is intended to feel like a native macOS application while keeping
Neovide-like cursor movement, smooth scrolling, terminal correctness, and image
protocol support. The early Rust binary helped prove terminal rendering ideas,
but it kept the default launch path outside AppKit.

The native shell now links the Rust core/runtime through the C ABI and can
launch PTY-backed panes through `libghostty-vt`.

## Decision

`just terminal` launches the native AppKit host.

The Rust crate remains responsible for terminal/runtime state, session metadata,
notifications, and future protocol parsing. Swift/AppKit remains responsible for
native windows, menus, tabs, context menus, and the host-side drawing surface.

The macroquad frontend is removed from the active dependency graph and should
not be reintroduced as a product launch path.

## Consequences

Native macOS UI behavior can be improved directly instead of being bridged into
a prototype windowing layer.

Rendering work moves toward the AppKit/Metal host. Short-term terminal text may
still be drawn by AppKit while the Metal cell renderer is built.

Any missing behavior from the retired frontend must be ported into the Rust
runtime or native host with tests, not kept alive as a parallel application.
