# ADR-0007: Active Group Order State

- Status: Accepted
- Date: 2026-07-25
- Supersedes: None
- Superseded by: None

## Context

The first gameplay command assigns a `HoldPosition` group order. Accepting that command must have an authoritative effect without inventing movement, formations, collisions, or combat behaviour before those systems exist.

The assigned group order can affect future execution, replay verification, and branch comparison, so it must be persistent authoritative state and included in the canonical state hash.

## Decision

An accepted `IssueGroupOrder` command with `HoldPosition` assigns the group's active group order in `sim-core`.

The active group order is persistent authoritative state. It is included in the canonical BLAKE3 state bytes after the match roster, sorted by `GroupId`.

Applying the intent emits a public `GroupOrderAssigned` gameplay event. The event records the authoritative tick, zero-based ordinal within the tick, `GroupId`, `ParticipantId`, and `GroupOrder`.

The event means that the order was assigned. It does not mean the group physically held a position.

## Alternatives Considered

- Treating `HoldPosition` as validation-only was rejected because accepted commands need a real authoritative effect.
- Emitting only a trace record was rejected because trace is diagnostic output, not a gameplay event stream.
- Adding physical movement or formation behaviour was rejected because no movement or map model exists yet.
- Omitting active group orders from the state hash was rejected because persistent authoritative state must be reflected in replay verification.

## Consequences

- Accepted group-order commands change persistent authoritative state.
- Rejected commands do not change active group orders and do not emit gameplay events.
- Reordered equivalent command sets produce the same command results, gameplay events, final state, and state hash.
- Future group orders must define their state-hash encoding and gameplay event meaning when they are introduced.

## Verification

- `sim-core` tests prove an accepted `HoldPosition` becomes the group's active order.
- `sim-core` tests prove an accepted command emits one `GroupOrderAssigned` gameplay event.
- `sim-core` tests prove rejected commands do not mutate group state and emit no gameplay event.
- `sim-core` tests prove reordered equivalent command sets produce identical outputs and hashes.
- The Rust quality gate runs formatting, clippy with warnings denied, and all workspace tests inside Docker.
