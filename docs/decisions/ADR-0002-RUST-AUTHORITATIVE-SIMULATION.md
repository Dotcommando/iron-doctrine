# ADR-0002: Rust Authoritative Simulation

- Status: Accepted
- Date: 2026-07-24
- Supersedes: None
- Superseded by: None

## Context

Iron Doctrine needs one implementation of authoritative match behaviour that can be used by the desktop client, headless tools, replays, bots, LLM commanders, and future multiplayer servers.

Duplicating authoritative combat rules between Unity, server code, CLI tools, and replay tooling would make deterministic verification and long-term maintenance fragile.

## Decision

The Rust workspace under `simulation/` contains the authoritative simulation implementation.

The current implemented boundary is:

- `sim-core` contains authoritative match lifecycle, fixed tick execution, state hashing, and trace production;
- `sim-protocol` contains explicit external scenario and configuration contracts;
- `sim-cli` is a headless adapter that reads files, invokes protocol validation and core execution, and writes JSON output.

Unity, future servers, and other adapters must use the Rust simulation contracts rather than reimplement authoritative gameplay outcomes.

## Alternatives Considered

- Implementing gameplay rules in Unity first was rejected because it would make later headless execution, replay verification, and multiplayer authority depend on client-side code.
- Implementing a separate multiplayer server simulation was rejected because it would duplicate combat behaviour and make divergence likely.
- Keeping the first scenario entirely inside the CLI was rejected because it would put authoritative lifecycle and hashing into an adapter.

## Consequences

- Authoritative behaviour belongs in `sim-core`, not in adapters.
- File I/O, command-line arguments, Docker, and JSON stdout remain outside `sim-core`.
- Future gameplay systems must preserve dependency direction toward the Rust core and protocol contracts.

## Verification

- `sim-cli` integration tests prove the adapter uses the protocol and core pipeline to run `scenarios/empty-match.json`.
- `sim-core` tests prove lifecycle, fixed ticks, state hash, and trace behaviour.
- The Docker quality gate runs formatting, clippy with warnings denied, and all workspace tests.
