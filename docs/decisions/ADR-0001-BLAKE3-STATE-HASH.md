# ADR-0001: BLAKE3 State Hash

- Status: Accepted
- Date: 2026-07-24
- Supersedes: None
- Superseded by: None

## Context

Iron Doctrine needs a deterministic state hash so scenarios, tests, replays, analysis tools, and future adapters can verify that identical authoritative inputs produce identical authoritative state.

The hash must be calculated from canonical authoritative state, not from debug output, JSON formatting, memory addresses, collection iteration order, wall-clock data, trace records, or runtime environment details.

## Decision

Use BLAKE3-256 as the state hash algorithm for the first authoritative simulation kernel.

The canonical state bytes are assembled explicitly by `sim-core` in a fixed order from authoritative data only. In the first execution kernel that data was:

- authoritative state version;
- tick rate;
- seed;
- current authoritative tick.

Trace records are diagnostic output and are excluded from the hash.

ADR-0005 extends the canonical state bytes with match roster data while preserving the BLAKE3-256 algorithm selected here.

## Alternatives Considered

- Rust standard hashing was rejected because it is not a stable external contract and is not intended for reproducible persisted hashes.
- Hashing JSON was rejected because it would make hash correctness depend on serialization details rather than the authoritative state contract.
- A custom non-cryptographic hash was rejected because it would provide less confidence for replay and analysis use while still becoming a compatibility burden.

## Consequences

- The project adds a focused dependency on the `blake3` crate in `sim-core`.
- State hashes are deterministic across supported platforms when canonical bytes are unchanged.
- Any future change to the canonical state representation or hash algorithm must be treated as a compatibility decision and reflected in versioning or a superseding ADR.

## Verification

- `sim-core` tests prove identical runs produce the same hash after every tick.
- `sim-core` tests prove different seeds and different tick counts produce different hashes.
- `sim-core` tests prove enabling trace does not change the hash.
- The Rust quality gate runs formatting, clippy with warnings denied, and workspace tests inside Docker.
