# ADR-0003: Fixed Simulation Tick

- Status: Accepted
- Date: 2026-07-24
- Supersedes: None
- Superseded by: None

## Context

Iron Doctrine must produce reproducible authoritative results for scenarios, replays, tests, analysis, bots, and future multiplayer matches.

Wall-clock time, render-frame delta, and machine performance would make authoritative state depend on runtime environment rather than explicit inputs.

## Decision

Authoritative match time is represented as integer ticks.

The first implemented simulation starts at tick `0`. Each successful call to the public tick execution operation advances the authoritative tick exactly once. The operation does not accept `deltaTime`, wall-clock time, frame time, or gameplay commands in this stage.

## Alternatives Considered

- Frame-delta simulation was rejected because it would couple authoritative state to render timing and machine performance.
- Wall-clock scheduling inside `sim-core` was rejected because scheduling is an infrastructure concern, not authoritative simulation state.
- Placeholder command or phase systems were rejected because this stage has no real gameplay commands or gameplay phases yet.

## Consequences

- All state changes occur during explicit tick execution.
- Future command processing must assign inputs to authoritative ticks rather than arbitrary moments.
- Replay and scenario verification can compare tick counts and state hashes deterministically.

## Verification

- `sim-core` tests prove a new simulation starts at tick `0`.
- `sim-core` tests prove one and three successful ticks advance to ticks `1` and `3`.
- `sim-core` tests prove tick execution requires no time input.
- The Docker quality gate runs formatting, clippy with warnings denied, and all workspace tests.
