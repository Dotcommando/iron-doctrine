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
    "tickRateHz": 20,
    "seed": 123456
  },
  "runTicks": 3,
  "trace": true
}
```

## State Hash and Trace

`sim-core` calculates authoritative state hashes with BLAKE3-256, as recorded in `docs/decisions/ADR-0001-BLAKE3-STATE-HASH.md`.

The first canonical state hash includes:

- authoritative state version;
- tick rate;
- seed;
- current authoritative tick.

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
