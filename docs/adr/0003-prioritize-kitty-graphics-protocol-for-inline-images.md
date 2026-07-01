# 0003: Prioritize Kitty graphics protocol for inline images

Date: 2026-07-02

Status: Accepted

## Context

The terminal should support inline images. Common terminal image protocols
include Kitty graphics protocol, iTerm2 inline images, and Sixel.

The renderer also needs Neovide-like scroll behavior, so image placements must
belong to terminal state and scroll with text instead of being drawn as an
unstructured overlay.

## Decision

Prioritize Kitty graphics protocol as the first inline-image protocol.

Kitty support should define the internal image model: image resources,
placements, deletion semantics, pane isolation, scroll interaction, resize
behavior, and renderer texture uploads.

iTerm2 inline images can be added later as a compatibility layer. Sixel is not a
near-term target.

## Consequences

Image protocol parsing and image placement state should live in the Rust
terminal core, not only in the renderer.

The renderer must support textured image placement in cell coordinates and make
those placements participate in smooth scroll animation.

Compatibility with iTerm2 and Sixel is intentionally deferred so the first
implementation can focus on one modern protocol and a clean internal model.
