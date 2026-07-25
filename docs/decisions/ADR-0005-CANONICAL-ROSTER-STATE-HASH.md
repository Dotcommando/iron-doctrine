# ADR-0005: Canonical Roster State Hash

- Status: Accepted
- Date: 2026-07-25
- Supersedes: None
- Superseded by: None

## Context

ADR-0001 selected BLAKE3-256 for deterministic authoritative state hashes. The first execution kernel hashed only state version, tick rate, seed, and current tick because no gameplay roster existed yet.

The players, commands, and groups contract introduces match identity, teams, participants, groups, and robot identifiers as persistent authoritative state. Equivalent roster data can arrive in different JSON array orders, but that input order must not affect replay verification or state hashes.

## Decision

Keep BLAKE3-256 from ADR-0001 and extend the canonical state bytes assembled by `sim-core`.

The canonical state now includes:

- authoritative state version;
- match identifier;
- tick rate;
- seed;
- current authoritative tick;
- team identifiers sorted by `TeamId`;
- participant identifiers and team membership sorted by `ParticipantId`;
- group identifiers, controlling participants, and robot identifiers sorted by `GroupId`, with each group's robot identifiers sorted by `RobotId`.

Counts and string lengths are encoded explicitly before variable-length collections and strings.

## Alternatives Considered

- Preserving JSON input order was rejected because semantically equivalent rosters would produce different hashes.
- Hashing serialized protocol JSON was rejected for the same reason recorded in ADR-0001: hash correctness would depend on adapter serialization details.
- Omitting roster data from the state hash was rejected because participant ownership and group membership are authoritative state that can affect future command validation and execution.

## Consequences

- Reordering teams, participants, groups, or robot identifiers without changing their meaning does not change the authoritative state hash.
- Changing match identity, team membership, group control, or robot membership changes the authoritative state hash.
- Future persistent authoritative fields must be added to the canonical state bytes and covered by tests. ADR-0007 applies this rule to active group orders.

## Verification

- `sim-core` tests prove equivalent roster data in different input array order produces the same initial state hash.
- `sim-core` tests continue to prove identical runs produce the same hash after every tick.
- `sim-core` tests continue to prove trace does not affect the state hash.
- The Rust quality gate runs formatting, clippy with warnings denied, and all workspace tests inside Docker.
