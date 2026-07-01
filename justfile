set shell := ["bash", "-euo", "pipefail", "-c"]

default:
    @just --list

# Enter the Nix development shell.
shell:
    @nix develop

# Show tool versions from the flake shell.
doctor:
    @if [[ -z "${IN_NIX_SHELL:-}" ]]; then exec nix develop --command just doctor; else echo "zig:  $(zig version)"; echo "rust: $(rustc --version)"; echo "just: $(just --version)"; echo "font: ${NVTERM_FONT:-unset}"; fi

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
