# Architecture Decision Records

This directory stores Architecture Decision Records for neovide-tabs.

ADRs are used for decisions that shape the long-term architecture, especially
choices that are hard to reverse or that explain why the current prototype is
allowed to differ from the target design.

## Index

- [0001: Manage architecture decisions as ADRs](0001-manage-architecture-decisions-as-adrs.md)
- [0002: Target a native macOS shell with Rust terminal core and Metal renderer](0002-target-native-macos-shell-rust-core-metal-renderer.md)
- [0003: Prioritize Kitty graphics protocol for inline images](0003-prioritize-kitty-graphics-protocol-for-inline-images.md)
- [0004: Use a warning-free Rust lint gate](0004-use-clippy-warning-free-rust-gate.md)

## Workflow

Use `just adr` to list current records.

Use `just adr-new "short decision title"` to create the next ADR from the
template. New records start as `Proposed`; change the status to `Accepted`,
`Rejected`, `Superseded`, or `Deprecated` when the decision is settled.

Keep records short. An ADR should capture the decision, the context that made it
necessary, and the consequences we are accepting.
