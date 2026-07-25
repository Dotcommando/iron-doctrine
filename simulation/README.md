# Iron Doctrine Simulation

This directory contains the authoritative Rust simulation workspace.

## Docker Environment

The Rust toolchain is pinned to `1.97.1` in `rust-toolchain.toml`.

Verified commands:

Run these commands from this `simulation/` directory.

```powershell
docker compose -f docker-compose.yml build rust
docker compose -f docker-compose.yml run --rm rust rustc --version
docker compose -f docker-compose.yml run --rm rust cargo --version
docker compose -f docker-compose.yml run --rm rust cargo metadata --no-deps --format-version 1
docker compose -f docker-compose.yml run --rm rust cargo check --workspace --all-targets
docker compose -f docker-compose.yml run --rm rust cargo run -p sim-cli -- run ../scenarios/empty-match.json
```

## Quality Gate

Run the full Rust quality gate from this `simulation/` directory:

```powershell
docker compose -f docker-compose.yml run --rm rust sh scripts/check.sh
```

The script runs:

1. `cargo fmt --check`
2. `cargo clippy --workspace --all-targets --all-features -- -D warnings`
3. `cargo test --workspace --all-targets --all-features`

## Headless Scenario Input

`sim-protocol` accepts the Stage 2 versioned headless scenario shape:

```json
{
  "schemaVersion": 2,
  "match": {
    "matchId": "match-group-order-001",
    "tickRateHz": 20,
    "seed": 123456,
    "teams": [
      {
        "teamId": "team-blue"
      }
    ],
    "participants": [
      {
        "participantId": "participant-blue-1",
        "teamId": "team-blue"
      }
    ],
    "groups": [
      {
        "groupId": "group-blue-alpha",
        "controllerParticipantId": "participant-blue-1",
        "robotIds": [
          "robot-blue-001"
        ]
      }
    ]
  },
  "commands": [
    {
      "sequence": 1,
      "targetTick": 0,
      "participantId": "participant-blue-1",
      "payload": {
        "kind": "IssueGroupOrder",
        "groupId": "group-blue-alpha",
        "order": {
          "kind": "HoldPosition"
        }
      }
    }
  ],
  "runTicks": 1,
  "trace": true
}
```

The match roster is explicit. `teams`, `participants`, and `groups` may be empty, but they are not inferred from missing fields.
`commands` is explicit and may be empty.

The accepted schema version is `2`. Unsupported schema versions are rejected before simulation execution.

## Command Contract

`sim-protocol` defines the first public command envelope with:

- `CommandSequence`;
- `TargetTick`;
- `ParticipantId`;
- `CommandPayload`.

The first real command payload is `IssueGroupOrder` targeting a `GroupId` with the `HoldPosition` group order.

`sim-core` validates command envelopes against the current authoritative tick and match roster. Accepted commands produce internal intents. Rejected commands produce structured public `CommandResult` values and no intent.

`targetTick` must equal the simulation `currentTick` before the tick transition is executed. Commands for another tick are rejected with `WrongTargetTick`; they are not moved to the current tick.

During tick execution, command sets are normalized into deterministic sequence order. Lower `CommandSequence` values are processed first for commands targeting the same tick. Duplicate command sequences are rejected deterministically with `DuplicateCommandSequence`.

The current command rejection reasons are:

- `WrongTargetTick`;
- `UnknownParticipant`;
- `UnknownGroup`;
- `GroupNotControlledByParticipant`;
- `DuplicateCommandSequence`.

Accepted `HoldPosition` commands assign the group's active group order, produce public `CommandResult` values, and emit `GroupOrderAssigned` gameplay events with zero-based ordinals inside the tick.

`GroupOrderAssigned` means the order was assigned. It does not mean the group physically held a position; movement, formation, collision, and combat behaviour do not exist in this stage.

## State Hash and Trace

`sim-core` calculates authoritative state hashes with BLAKE3-256, as recorded in `docs/decisions/ADR-0001-BLAKE3-STATE-HASH.md`.

The first canonical state hash includes:

- authoritative state version;
- match identifier;
- tick rate;
- seed;
- current authoritative tick;
- team identifiers in canonical order;
- participant identifiers and team membership in canonical order;
- group identifiers, controlling participants, and robot identifiers in canonical order;
- active group orders in canonical order.

Execution trace records are structured diagnostics. They are not gameplay events and do not affect authoritative state hashes.

## Headless CLI Output

`sim-cli run` writes successful results to stdout as JSON with:

- `schemaVersion`;
- `matchId`;
- `initialTick`;
- `completedTicks`;
- `finalTick`;
- `stateHash`;
- `commandResults`;
- `gameplayEvents`;
- `trace` when requested by the scenario.

Accepted commands are represented as:

```json
{
  "sequence": 1,
  "targetTick": 0,
  "participantId": "participant-blue-1",
  "status": {
    "kind": "Accepted"
  }
}
```

Rejected commands are successful simulation output when the match itself remains valid:

```json
{
  "sequence": 1,
  "targetTick": 0,
  "participantId": "participant-blue-1",
  "status": {
    "kind": "Rejected",
    "reason": "GroupNotControlledByParticipant"
  }
}
```

The first gameplay event shape is:

```json
{
  "kind": "GroupOrderAssigned",
  "tick": 0,
  "ordinal": 0,
  "groupId": "group-blue-alpha",
  "participantId": "participant-blue-1",
  "order": {
    "kind": "HoldPosition"
  }
}
```

Event `ordinal` is a deterministic zero-based position within the authoritative tick.

Error diagnostics are written to stderr, and failures exit with a non-zero status.

Verified scenario commands from this directory:

```powershell
cargo run -p sim-cli -- run ../scenarios/empty-match.json
cargo run -p sim-cli -- run ../scenarios/group-order.json
cargo run -p sim-cli -- run ../scenarios/foreign-group-order.json
cargo run -p sim-cli -- run ../scenarios/two-group-orders-ordered.json
cargo run -p sim-cli -- run ../scenarios/two-group-orders-reordered.json
cargo run -p sim-cli -- run ../scenarios/unknown-participant-command.json
```

`../scenarios/invalid-roster.json` is expected to fail with a controlled non-zero CLI result.
