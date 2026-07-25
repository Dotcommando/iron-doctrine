# ADR-0008: Headless Scenario Schema Version 2

- Status: Accepted
- Date: 2026-07-25
- Supersedes: None
- Superseded by: None

## Context

The players, commands, and groups contract exposes the first complete gameplay vertical slice through `sim-cli run`.

The JSON scenario format is a public adapter contract used by executable scenarios, tests, replay-oriented tools, analysis workflows, and future adapters. The format needs to carry explicit match roster data and scheduled commands without making JSON the internal domain model.

## Decision

The accepted headless scenario input schema is `schemaVersion: 2`.

Version 2 contains:

- `match.matchId`;
- `match.tickRateHz`;
- `match.seed`;
- explicit `match.teams`;
- explicit `match.participants`;
- explicit `match.groups`;
- explicit scheduled `commands`;
- `runTicks`;
- `trace`.

Each scheduled command is a public command envelope with:

- `sequence`;
- `targetTick`;
- `participantId`;
- a real `payload`.

The first payload is `IssueGroupOrder` with a `groupId` and `HoldPosition` order.

`sim-protocol` rejects unsupported schema versions before simulation execution. `sim-cli` keeps filesystem access, JSON reading, stdout, stderr, and process exit status at the adapter boundary. Roster validation, command validation, command ordering, intent production, event production, state mutation, hashing, and trace creation remain in `sim-protocol` and `sim-core`.

Successful `sim-cli run` output includes:

- `schemaVersion`;
- `matchId`;
- `initialTick`;
- `completedTicks`;
- `finalTick`;
- `stateHash`;
- `commandResults`;
- `gameplayEvents`;
- `trace` when requested.

Rejected commands are successful simulation output when the match itself remains valid. Invalid scenario shape or invalid match configuration is a controlled non-zero CLI failure with diagnostics on stderr.

## Alternatives Considered

- Extending schema version 1 in place was rejected because schema 1 represented the earlier empty-kernel scenario shape and did not contain scheduled commands or gameplay outputs.
- Letting `sim-cli` interpret command semantics was rejected because authoritative validation, ordering, intent production, mutation, events, trace, and hashing belong inside the simulation/protocol boundary.
- Treating rejected commands as CLI process failures was rejected because command rejection is authoritative simulation output when the match configuration itself is valid.

## Consequences

- Scenario files can distinguish empty command streams from omitted command data.
- Output consumers can correlate command results by command envelope fields without relying on array position.
- Ownership violations and other command rejections are represented as structured JSON results.
- Invalid roster data remains a CLI failure because the match cannot be created.
- Future schema changes require explicit version handling and documentation.

## Verification

- `sim-cli` integration tests prove schema version 2 scenarios execute through JSON input and output.
- `sim-cli` integration tests prove accepted `HoldPosition` commands produce accepted command results and `GroupOrderAssigned` gameplay events.
- `sim-cli` integration tests prove foreign-group commands are rejected without failing valid match execution.
- `sim-cli` integration tests prove reordered command input produces identical command results, gameplay events, and final state hash.
- `sim-cli` integration tests prove invalid roster data returns a controlled non-zero CLI failure with diagnostics on stderr.
- The Rust quality gate runs formatting, clippy with warnings denied, and all workspace tests inside Docker.
