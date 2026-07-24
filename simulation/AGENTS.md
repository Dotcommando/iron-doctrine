# Iron Doctrine Simulation — Agent Instructions

## Scope

These rules apply to the Rust simulation workspace under `simulation/`.

## Rust Environment

- Use the pinned Rust toolchain in `rust-toolchain.toml`.
- Prefer Docker Compose commands from `simulation/README.md` for verification.
- Do not require host Rust for routine build, lint, or test workflows.
- Keep Docker configuration free of user-specific absolute paths.

## Architecture

- `sim-core` owns authoritative simulation domain and application logic.
- `sim-protocol` owns explicit external input and output contracts.
- `sim-cli` is an adapter for headless command-line execution.
- Keep filesystem, Docker, and CLI concerns out of `sim-core`.
- Do not add placeholder gameplay phases, fake commands, or no-op command contracts.

## Rust Code

- Forbid unsafe code unless an accepted ADR explicitly allows it.
- Add dependencies only for concrete implemented behaviour.
- Keep public APIs small until a real caller or contract requires them.
- Use deterministic data structures and explicit processing order where results can depend on order.
