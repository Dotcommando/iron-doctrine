# ADR-0004: Docker Rust Development Environment

- Status: Accepted
- Date: 2026-07-24
- Supersedes: None
- Superseded by: None

## Context

Iron Doctrine is developed on Windows and must remain reproducible on macOS as well.

The Rust simulation needs a consistent toolchain, cache behaviour, and verification command without requiring host Rust to be installed or configured identically.

## Decision

Use Docker Compose as the canonical Rust development and verification environment for the simulation workspace.

The implemented environment:

- pins Rust `1.97.1` in `rust-toolchain.toml`;
- builds from an official Rust image with the same version;
- mounts the repository at `/workspace`;
- keeps Cargo registry, Git cache, and build output in Docker volumes;
- runs the full Rust quality gate through `simulation/scripts/check.sh`.

The Compose file intentionally has no top-level `version` field.

## Alternatives Considered

- Requiring host Rust was rejected because it would make verification depend on each developer machine.
- Maintaining separate PowerShell, shell, Make, or Just quality gates was rejected because multiple equivalent entry points drift over time.
- Storing `target/` on the host bind mount was rejected because it would create noisy platform-specific build output in the repository tree.

## Consequences

- The documented Docker command is the source of truth for Rust verification.
- Host Rust remains optional for routine simulation work.
- Docker must be available for canonical verification.

## Verification

- The Docker image build verifies the pinned toolchain environment.
- The documented quality gate command executes `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `cargo test --workspace --all-targets --all-features`.
- Repository file scans and `git status` verify build output is not tracked.
