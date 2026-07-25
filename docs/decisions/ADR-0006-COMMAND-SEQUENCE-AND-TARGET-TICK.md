# ADR-0006: Command Sequence and Target Tick

- Status: Accepted
- Date: 2026-07-25
- Supersedes: None
- Superseded by: None

## Context

The first gameplay command contract introduces external commands submitted to the authoritative Rust simulation. Commands need deterministic correlation, deterministic ordering within a tick, and an explicit tick assignment that does not depend on wall-clock arrival time, render frames, filesystem order, or adapter collection order.

The command contract is public through `sim-protocol`, so the sequence and target tick semantics must be stable enough for future CLI scenarios, replay tools, clients, servers, and analysis adapters.

## Decision

Each command envelope contains:

- `CommandSequence`;
- `TargetTick`;
- `ParticipantId`;
- a real `CommandPayload`.

`CommandSequence` is the deterministic ordering key for commands in one match command stream. Lower sequence values are processed first when commands target the same tick. Duplicate sequences are rejected deterministically by command execution.

`TargetTick` identifies the authoritative tick that receives the command. Its value must equal the simulation `currentTick` before that tick transition is executed. Commands are not silently moved to another tick.

The authoritative adapter that submits commands to `sim-core` is responsible for assigning sequence values. Future public clients must not be trusted to determine final authoritative ordering in multiplayer use.

## Alternatives Considered

- Using adapter input order was rejected because collection order can differ across files, clients, servers, and replay tools.
- Using wall-clock timestamps was rejected because authoritative simulation state must not depend on runtime timing.
- Using client frame numbers was rejected because render frames are not authoritative simulation ticks.
- Buffering or rescheduling commands inside `sim-core` was deferred because the current stage only validates commands for the current tick.

## Consequences

- Command results can be correlated with submitted commands without relying on array position.
- Wrong-tick commands are rejected explicitly rather than moved.
- Duplicate-sequence handling is part of deterministic command execution. All commands sharing a duplicate sequence are rejected with `DuplicateCommandSequence`.
- Command validation rejects wrong-tick, unknown participant, unknown group, and ownership violations before any state mutation.

## Verification

- `sim-core` tests prove a `HoldPosition` command from the controlling participant is accepted and produces an internal intent.
- `sim-core` tests prove wrong-tick, unknown participant, unknown group, and foreign-group commands are rejected with structured reasons.
- `sim-core` tests prove rejected commands produce no internal intent.
- `sim-core` tests prove duplicate sequences are rejected deterministically independent of input order.
- The Rust quality gate runs formatting, clippy with warnings denied, and all workspace tests inside Docker.
