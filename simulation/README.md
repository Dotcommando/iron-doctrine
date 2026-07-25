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

`sim-protocol` accepts the first versioned headless scenario shape:

```json
{
  "schemaVersion": 1,
  "match": {
    "matchId": "match-empty-001",
    "tickRateHz": 20,
    "seed": 123456,
    "teams": [],
    "participants": [],
    "groups": []
  },
  "runTicks": 3,
  "trace": true
}
```

The match roster is explicit. `teams`, `participants`, and `groups` may be empty, but they are not inferred from missing fields.

## Command Contract

`sim-protocol` defines the first public command envelope with:

- `CommandSequence`;
- `TargetTick`;
- `ParticipantId`;
- `CommandPayload`.

The first real command payload is `IssueGroupOrder` targeting a `GroupId` with the `HoldPosition` group order.

`sim-core` validates command envelopes against the current authoritative tick and match roster. Accepted commands produce internal intents. Rejected commands produce structured public `CommandResult` values and no intent.

During tick execution, command sets are normalized into deterministic sequence order. Duplicate command sequences are rejected deterministically. Accepted `HoldPosition` commands assign the group's active group order, produce public `CommandResult` values, and emit `GroupOrderAssigned` gameplay events with zero-based ordinals inside the tick.

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
- `initialTick`;
- `completedTicks`;
- `finalTick`;
- `stateHash`;
- `trace` when requested by the scenario.

Error diagnostics are written to stderr, and failures exit with a non-zero status.
