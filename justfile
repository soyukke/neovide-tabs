set shell := ["bash", "-euo", "pipefail", "-c"]

default:
    @just --list

# Enter the Nix development shell.
shell:
    @nix develop

# Show tool versions from the flake shell.
doctor:
    @if [[ -z "${IN_NIX_SHELL:-}" ]]; then exec nix develop --command just doctor; else echo "zig:  $(zig version)"; echo "rust: $(rustc --version)"; echo "just: $(just --version)"; echo "font: ${NVTERM_FONT:-unset}"; fi

# List Architecture Decision Records.
adr:
    @find docs/adr -maxdepth 1 -name '[0-9][0-9][0-9][0-9]-*.md' -print | sort

# Create a new Architecture Decision Record from docs/adr/template.md.
adr-new title:
    @title="{{title}}"; \
    slug=$(printf '%s' "$title" | tr '[:upper:]' '[:lower:]' | sed -E 's/[^a-z0-9]+/-/g; s/^-+//; s/-+$//'); \
    last=$(find docs/adr -maxdepth 1 -name '[0-9][0-9][0-9][0-9]-*.md' -print | sed -E 's#.*/([0-9]{4})-.*#\1#' | sort -n | tail -1); \
    next=$(printf '%04d' $((10#${last:-0} + 1))); \
    path="docs/adr/${next}-${slug}.md"; \
    escaped_title=$(printf '%s' "$title" | sed -e 's/[\/&]/\\&/g'); \
    sed -e "s/NNNN/${next}/" -e "s/Title/${escaped_title}/" -e "s/YYYY-MM-DD/$(date +%F)/" docs/adr/template.md > "$path"; \
    echo "$path"

# Launch the terminal prototype.
terminal:
    @if [[ -z "${IN_NIX_SHELL:-}" ]]; then exec nix develop --command just terminal; else exec cargo run; fi

alias run := terminal
alias launch := terminal

# Update the current neovide-tabs pane's agent status.
agent-status state summary="":
    @./scripts/nvterm-agent-status "{{state}}" "{{summary}}"

# Check the Rust crate.
check:
    @if [[ -z "${IN_NIX_SHELL:-}" ]]; then exec nix develop --command just check; else cargo check; fi

# Run tests.
test:
    @if [[ -z "${IN_NIX_SHELL:-}" ]]; then exec nix develop --command just test; else cargo test; fi

# Format Rust sources.
fmt:
    @if [[ -z "${IN_NIX_SHELL:-}" ]]; then exec nix develop --command just fmt; else cargo fmt; fi

# Verify formatting, type checking, and tests.
verify:
    @if [[ -z "${IN_NIX_SHELL:-}" ]]; then exec nix develop --command just verify; else cargo fmt -- --check && cargo check && cargo test; fi

# Remove Rust build artifacts.
clean:
    @if [[ -z "${IN_NIX_SHELL:-}" ]]; then exec nix develop --command just clean; else cargo clean; fi
