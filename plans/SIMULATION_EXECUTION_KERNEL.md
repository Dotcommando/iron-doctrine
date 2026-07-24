# Iron Doctrine — Simulation Execution Kernel

- Plan status: Active
- File: `plans/SIMULATION_EXECUTION_KERNEL.md`
- Previous plan: None
- Next plan: To be determined after completion
- Related general stage: `GENERAL_PLAN.md`, Stage 1

## Objective

Initialize the Rust part of Iron Doctrine and implement the first working authoritative match kernel without a robot model or gameplay commands.

After this plan is complete, the repository must support a single command that:

1. builds the Rust workspace in Docker;
2. loads a JSON scenario;
3. validates match configuration;
4. creates a deterministic match;
5. executes a specified number of fixed ticks;
6. produces a JSON result;
7. exposes a structured trace of the execution sequence;
8. proves stable state hashing;
9. runs formatting, linting, and tests.

This plan creates code and contracts that will later be used by:

- the desktop single-player client;
- the authoritative multiplayer server;
- the replay runner;
- built-in bots;
- LLM commanders;
- analysis tools.

## Runnable Result

The repository must be able to execute a scenario with the following meaning:

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

Example command:

```powershell
docker compose -f docker-compose.yml run --rm rust cargo run -p sim-cli -- run ../scenarios/empty-match.json
```

The exact Docker Compose service name and CLI syntax may be refined during implementation.

If they change, all later steps and documentation must be updated to use the accepted form.

The result must be structured JSON with approximately the following meaning:

```json
{
  "schemaVersion": 1,
  "initialTick": 0,
  "completedTicks": 3,
  "finalTick": 3,
  "stateHash": "<deterministic-hash>",
  "trace": [
    {
      "tick": 0,
      "kind": "TickStarted"
    },
    {
      "tick": 1,
      "kind": "TickCompleted"
    }
  ]
}
```

The exact JSON structure is decided during this plan, but it must be:

- versioned;
- unambiguous;
- suitable for automated processing;
- readable by a human;
- independent of Rust `Debug` representations.

## Scope

This plan includes:

- Rust toolchain;
- Docker development environment;
- Cargo workspace;
- `sim-protocol`;
- `sim-core`;
- `sim-cli`;
- input scenario format;
- execution result format;
- match configuration;
- configuration validation;
- match lifecycle;
- fixed tick;
- explicit tick execution order;
- structured trace;
- deterministic state hash;
- one quality gate;
- minimum required ADRs;
- exact instructions for running the Rust part.

## Out of Scope

This plan does not include:

- Unity project;
- C#;
- FFI;
- multiplayer transport;
- `sim-server`;
- Tokio;
- network asynchrony;
- multithreading;
- players;
- player commands;
- groups;
- robots;
- bodies;
- locomotion modules;
- weapons;
- sockets;
- map;
- physics;
- Rapier;
- movement;
- sensors;
- collisions;
- damage;
- gameplay events;
- snapshots;
- replays;
- ECS;
- plugin system;
- universal phase registry;
- placeholder modules for future gameplay phases.

Do not create fake gameplay commands or `NoOp` commands merely to fill an interface.

## Target Structure

Expected structure after completing the plan:

```text
simulation/
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── Dockerfile
├── docker-compose.yml
├── .dockerignore
├── README.md
├── AGENTS.md
├── scripts/
│   └── check.sh
├── crates/
│   ├── sim-protocol/
│   └── sim-core/
└── apps/
    └── sim-cli/

scenarios/
└── empty-match.json
```

Names may be refined when the actual implementation reveals a clearer structure.

Do not create additional crates, applications, or layers without a concrete need.

## Hexagonal Architecture

The implementation must follow the hexagonal architecture established in `GENERAL_PLAN.md`.

For this plan:

- `sim-core` contains the domain and application logic of the simulation kernel;
- `sim-protocol` defines explicit external input and output contracts;
- `sim-cli` is an inbound and outbound adapter;
- JSON scenario loading is an infrastructure concern;
- filesystem access remains outside the domain core;
- Docker remains outside the domain and application layers.

Dependencies must point inward.

The core must not depend on:

- the CLI framework;
- filesystem paths;
- JSON files;
- Docker;
- operating-system-specific APIs.

Do not introduce ports or traits solely to imitate a hexagonal diagram.

A port is added only when there is a real external boundary or a second adapter requirement.

## Initial Contracts

### Match Configuration

Configuration must contain at least:

```text
schema version;
tick rate;
seed.
```

Requirements:

- the schema version is validated explicitly;
- unsupported versions are rejected;
- tick rate is validated;
- the seed is part of the authoritative initial state;
- match creation either returns a valid simulation or a structured error;
- do not use implicit defaults when they may affect the authoritative result.

### Match Lifecycle

The first stage supports the following lifecycle:

```text
configuration validated
→ match created
→ ticks executed
→ run result produced
```

Do not introduce additional lifecycle states without behaviour that requires them.

A new simulation starts at tick `0`.

Each successful tick execution advances the authoritative tick exactly once.

### Tick Execution Operation

The first stage has no gameplay commands.

Do not create an empty `CommandEnvelope`, a fake command payload, or `NoOp` for future use.

The public operation in this stage executes one empty authoritative tick.

The command contract is introduced together with the first real group command in the next general stage.

### Tick Result

The result of execution must contain at least:

```text
the started or completed tick number;
the resulting authoritative tick;
state hash;
diagnostic trace when enabled.
```

Internal execution records must not be called gameplay events.

### Trace

Trace explains how the kernel executed.

It must be:

- structured;
- serializable;
- stable in meaning;
- separate from error output;
- enabled through the scenario or CLI;
- unable to affect the authoritative result.

The minimum trace must show:

```text
tick start;
operations actually executed;
tick completion;
calculated final hash.
```

Do not create trace records for future systems that do not exist.

### State Hash

The hash must be calculated from a canonical representation of authoritative state.

It must not be based on:

- `Debug` output;
- formatted text;
- JSON with potentially unstable ordering;
- memory addresses;
- accidental `HashMap` order;
- diagnostic trace;
- wall-clock data.

At this stage, authoritative state includes at least:

```text
supported state version;
configuration affecting the future of the match;
seed;
current tick.
```

Identical initial data and identical tick counts must produce identical hashes.

Different seeds must produce different hashes even before gameplay randomness is used.

## Tick Execution Order

The first implementation must execute only operations that actually exist.

Expected order:

```text
1. Verify that the next tick may begin.
2. Record the beginning of tick execution.
3. Calculate the authoritative tick transition.
4. Apply the state transition.
5. Calculate the hash of the new authoritative state.
6. Record tick completion in the trace.
7. Produce the tick result.
```

The code must leave a clear place for later domain phases, but it must not contain:

- empty methods for every future phase;
- a universal callback list;
- plugin registry;
- dynamic event bus;
- macros that hide execution order.

The main execution method must read from top to bottom.

## Testing Rules

Code-producing steps follow TDD.

Add only the minimum set of tests required to prove behaviour.

Do not add tests for:

- enum values;
- simple getters;
- module exports;
- constants without behaviour;
- serde library behaviour;
- standard Rust behaviour;
- the same behaviour at multiple levels without a reason.

Every test must protect a meaningful contract.

---

## Step 1 — Rust Environment and Cargo Workspace

**Status:** Done

### Goal

Create a reproducible environment in which Rust code is built and verified the same way on Windows and macOS.

### Work

Create:

- pinned `rust-toolchain.toml`;
- Cargo workspace under `simulation/`;
- Dockerfile based on an official Rust image with an explicit version;
- `compose.yaml`;
- `.dockerignore`;
- basic workspace settings;
- minimum crate manifests for `sim-protocol`, `sim-core`, and `sim-cli`;
- `simulation/README.md` containing commands that were actually verified;
- `simulation/AGENTS.md` containing rules specific to the Rust part.

Docker requirements:

- source code is mounted from the working repository;
- Cargo registry and build cache persist in Docker volumes;
- configuration contains no user-specific absolute paths;
- host Rust is not required;
- the container can create files without ownership problems;
- commands work on Windows and macOS;
- the `rustc` version inside the container matches `rust-toolchain.toml`.

The crates must compile but must not contain fake gameplay logic.

### TDD

Behavioural tests are not required because no domain behaviour is introduced in this step.

Verification is performed through build commands.

### DoD

- Docker image builds successfully.
- The container reports the pinned `rustc` and `cargo` versions.
- Cargo recognizes the three expected packages.
- `cargo check --workspace --all-targets` passes inside Docker.
- Cargo cache persists between runs.
- Files created by the container remain accessible to the Windows user.
- `simulation/README.md` contains only verified commands.
- No `target/`, IDE cache, or Docker-generated garbage is tracked.
- No unnecessary crates or dependencies are added.

### Done

- Added `simulation/rust-toolchain.toml` pinned to Rust `1.97.1`.
- Added `simulation/Dockerfile` based on the official Rust image and `simulation/docker-compose.yml` with `RUST_VERSION: "1.97.1"` and no Compose `version` field.
- Mounted the repository root into the container at `/workspace`, kept Cargo registry, Git cache, and build output in Docker volumes, and kept the Cargo working directory at `/workspace/simulation`.
- Created the Cargo workspace with `sim-protocol`, `sim-core`, and `sim-cli` manifests and minimal compilable entry points without gameplay logic.
- Added `simulation/README.md` with verified Docker Compose commands and `simulation/AGENTS.md` with Rust-specific rules.
- Generated `simulation/Cargo.lock` through the verified Cargo workflow.
- Verified `docker compose -f docker-compose.yml build rust`.
- Verified `rustc 1.97.1` and `cargo 1.97.1` inside the container.
- Verified Cargo metadata lists exactly `sim-protocol`, `sim-core`, and `sim-cli`.
- Verified `cargo check --workspace --all-targets` inside Docker and repeated it to confirm the build cache persists.
- Verified a file created by the container in `simulation/` is visible and removable from Windows.
- Reviewed Steps 2, 3, and 4; their names, paths, symbols, assumptions, dependencies, and expected outputs still match the implemented workspace.

---

## Step 2 — Configuration and Headless Scenario Formats

**Status:** Done

### Goal

Define and implement the versioned input formats for the first stage.

### Work

Implement in `sim-protocol`:

- schema version type;
- tick rate type;
- seed type;
- `MatchConfig`;
- headless input scenario structure;
- structured validation errors.

The input scenario must define:

```text
schema version;
match configuration;
number of ticks;
trace enabled or disabled.
```

Distinguish between:

- JSON reading failure;
- data shape failure;
- unsupported version;
- invalid value.

Do not use strings as the internal representation of numeric domain values without a concrete reason.

Do not introduce players, commands, or gameplay events.

### TDD

Before implementation, add the minimum behavioural tests:

1. valid configuration is accepted;
2. unsupported schema version is rejected;
3. invalid tick rate is rejected;
4. invalid tick count is rejected;
5. seed is preserved without modification.

Do not test every individual serde field when one whole-contract test is sufficient.

### DoD

- Valid JSON scenario deserializes and passes validation.
- Unsupported schema version returns a specific error.
- Invalid values return specific errors.
- Errors are not represented only as free-form text.
- All added tests failed before implementation and now pass.
- Formats contain no fake gameplay entities.
- `cargo fmt --check` passes.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes.
- `cargo test --workspace --all-targets --all-features` passes.

### Done

- Added `sim-protocol` input contracts for `SchemaVersion`, `TickRateHz`, `Seed`, `RunTicks`, `MatchConfig`, and `HeadlessScenario`.
- Added private raw JSON DTOs so external JSON shape is separated from validated protocol types.
- Added `parse_headless_scenario_json`, which maps malformed or shape-invalid JSON to `ScenarioInputError::DataShape` and maps validation failures to `ScenarioInputError::Validation`.
- Added structured validation errors for unsupported schema version, invalid tick rate, and invalid tick count.
- Added behavioural tests proving valid scenarios are accepted, unsupported schema versions are rejected, invalid tick rates are rejected, invalid tick counts are rejected, and seeds are preserved without modification.
- Added `serde` and `serde_json` only where needed by `sim-protocol`.
- Updated `simulation/README.md` with the verified Rust check commands and the supported headless scenario input shape.
- Verified the added tests failed before implementation and pass after implementation.
- Verified `docker compose -f docker-compose.yml run --rm rust cargo fmt --check`.
- Verified `docker compose -f docker-compose.yml run --rm rust cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- Verified `docker compose -f docker-compose.yml run --rm rust cargo test --workspace --all-targets --all-features`.
- Reviewed Steps 3, 4, and 5; their names, paths, assumptions, dependencies, and expected outputs still match the implemented protocol contracts.

---

## Step 3 — Deterministic Kernel and Fixed Tick

**Status:** Done

### Goal

Implement the first authoritative match lifecycle and deterministic fixed tick.

### Work

Implement in `sim-core`:

- simulation creation from validated `MatchConfig`;
- authoritative state for the first stage;
- initial tick `0`;
- execution of one tick;
- explicit execution sequence;
- tick result;
- controlled failure when execution cannot continue.

The main execution method must remain short, explicit, and readable from top to bottom.

Do not introduce:

- async;
- threads;
- ECS;
- a trait for a single implementation;
- phase registry;
- event bus;
- global mutable state;
- fake commands;
- empty future systems.

### TDD

Before implementation, add the minimum behavioural tests:

1. a new simulation starts at tick `0`;
2. one successful step completes tick `1`;
3. three steps complete tick `3`;
4. independent simulations with identical configuration follow the same tick sequence;
5. execution does not accept `deltaTime` or wall-clock input.

The last contract is enforced through the public API and type structure rather than by testing system time.

### DoD

- Simulation is created only from validated configuration.
- Initial authoritative tick is `0`.
- Tick changes exactly once per successful step.
- Main pipeline reads from top to bottom.
- No placeholder phases exist.
- No async or multithreading exists.
- No hidden global state exists.
- All behavioural tests failed before implementation and now pass.
- The full Rust quality gate passes.

### Done

- Added `sim-core` dependency on `sim-protocol` so `MatchSimulation` is created from validated `MatchConfig`.
- Added first-stage authoritative state containing the validated match configuration and current authoritative tick.
- Added `MatchSimulation::new`, `current_tick`, `match_config`, and `execute_tick`.
- Added `TickResult` with started and completed tick numbers.
- Added structured `MatchCreationError` and `TickExecutionError` types, including controlled failure for tick overflow.
- Implemented the fixed tick pipeline as verify next tick, calculate next tick, apply tick transition, and produce the tick result.
- Kept the public tick execution operation free of commands, `deltaTime`, wall-clock input, async, threads, ECS, phase registries, event buses, placeholder phases, and global mutable state.
- Added behavioural tests proving a new simulation starts at tick `0`, one tick completes tick `1`, three ticks complete tick `3`, identical configurations follow the same tick sequence, and tick execution requires no time input.
- Verified the added tests failed before implementation and pass after implementation.
- Verified `docker compose -f docker-compose.yml run --rm rust cargo fmt --check`.
- Verified `docker compose -f docker-compose.yml run --rm rust cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- Verified `docker compose -f docker-compose.yml run --rm rust cargo test --workspace --all-targets --all-features`.
- Reviewed Steps 4, 5, and 6; their names, paths, assumptions, dependencies, and expected outputs still match the implemented kernel API.

---

## Step 4 — Canonical State Hash and Structured Trace

**Status:** Done

### Goal

Make execution reproducible and explainable.

### Work

Implement:

- canonical authoritative-state representation for hashing;
- stable hash after every tick;
- structured trace;
- optional trace sink;
- guarantee that trace cannot influence state or hash.

The trace must describe only operations that the kernel actually performs.

Do not include in the hash:

- trace;
- errors;
- execution duration;
- file paths;
- operating-system version;
- Docker metadata;
- JSON field order;
- any other non-authoritative data.

The hash algorithm must be selected explicitly and used through one implementation.

If the algorithm becomes a long-term external contract, record it in an ADR.

### TDD

Before implementation, add the minimum behavioural tests:

1. two identical runs produce the same hash after every tick;
2. different seeds produce different initial or final hashes;
3. different tick counts produce different final hashes;
4. enabling trace does not change the hash;
5. trace reflects the correct order of operations actually executed.

Do not compare full trace text when trace record kinds and order are sufficient.

### DoD

- Hash is built from a canonical authoritative representation.
- Hash does not depend on trace.
- Hash does not depend on wall-clock time or environment.
- Identical runs are reproducible.
- Different seeds are distinguishable.
- Trace is structured and serializable.
- Trace is not called gameplay events.
- All behavioural tests failed before implementation and now pass.
- The full Rust quality gate passes.

### Done

- Added BLAKE3-256 state hashing in `sim-core` through one implementation over explicit canonical authoritative-state bytes.
- Included authoritative state version, tick rate, seed, and current authoritative tick in the canonical hash input.
- Excluded trace, errors, elapsed time, file paths, operating system details, Docker metadata, JSON ordering, and non-authoritative data from the hash.
- Added `StateHash` and exposed hashes through `MatchSimulation::state_hash` and `TickResult::state_hash`.
- Added structured `ExecutionTrace`, `TraceRecord`, and `TraceRecordKind` diagnostics.
- Added optional trace collection through `MatchSimulation::execute_tick_with_trace`.
- Updated tick execution order to record tick start, tick transition calculation, tick transition application, state hash calculation, and tick completion.
- Added behavioural tests proving identical runs produce the same hash after every tick, different seeds produce different hashes, different tick counts produce different hashes, trace does not affect the hash, and trace order matches operations actually executed.
- Added `docs/decisions/ADR-0001-BLAKE3-STATE-HASH.md` and indexed it in `docs/decisions/README.md`.
- Updated `simulation/README.md` with the state hash and trace contract.
- Verified the added tests failed before implementation and pass after implementation.
- Verified `docker compose -f docker-compose.yml run --rm rust cargo fmt --check`.
- Verified `docker compose -f docker-compose.yml run --rm rust cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- Verified `docker compose -f docker-compose.yml run --rm rust cargo test --workspace --all-targets --all-features`.
- Reviewed Steps 5, 6, and 7; Step 6 was updated to account for ADR-0001 already recording the state hash algorithm.

---

## Step 5 — Headless CLI and End-to-End Scenario

**Status:** Done

### Goal

Connect input JSON, protocol, simulation core, trace, and output JSON into one runnable pipeline.

### Work

Implement a scenario-running command in `sim-cli`.

The CLI must:

1. receive a scenario path;
2. read the file;
3. deserialize it;
4. validate it;
5. create the simulation;
6. execute the requested number of ticks;
7. collect the result;
8. write successful output to stdout as JSON;
9. write error diagnostics to stderr;
10. exit with a non-zero code on failure.

Successful output must contain at least:

```text
schema version;
initial tick;
completed tick count;
final tick;
state hash;
trace when requested.
```

Do not duplicate in the CLI:

- protocol validation;
- lifecycle;
- tick calculation;
- hashing;
- trace creation.

### TDD

Before implementation, add the minimum integration tests:

1. `empty-match.json` executes three ticks and returns final tick `3`;
2. repeated execution returns the same hash;
3. a scenario with invalid configuration fails;
4. successful stdout is valid JSON;
5. errors are not mixed into successful JSON output.

Tests must verify meaningful JSON fields rather than whitespace and formatting.

### DoD

- The run command works inside Docker.
- `scenarios/empty-match.json` executes successfully.
- Final tick matches `runTicks`.
- Repeated runs produce the same hash.
- Invalid scenario returns a non-zero exit code.
- Successful stdout contains only the JSON result.
- stderr contains useful error diagnostics.
- CLI contains no duplicated gameplay logic.
- All integration tests failed before implementation and now pass.
- The full Rust quality gate passes.

### Done

- Added `scenarios/empty-match.json` with schema version `1`, tick rate `20`, seed `123456`, `runTicks` `3`, and trace enabled.
- Implemented `sim-cli run <scenario-path>` as the headless CLI entry point.
- Kept filesystem reading, argument handling, stdout JSON writing, stderr diagnostics, and exit status handling in `sim-cli`.
- Reused `sim-protocol` for JSON deserialization and validation instead of duplicating validation in the CLI.
- Reused `sim-core` for match creation, tick execution, state hashing, and trace creation instead of duplicating lifecycle or simulation logic in the CLI.
- Added JSON run output containing `schemaVersion`, `initialTick`, `completedTicks`, `finalTick`, `stateHash`, and `trace` when requested.
- Added integration tests proving `empty-match.json` executes three ticks, repeated execution returns the same hash, invalid configuration fails, successful stdout is valid JSON, and errors are not mixed into successful JSON output.
- Updated `simulation/README.md` with the verified CLI run command and output contract.
- Updated the plan runnable command to `docker compose -f docker-compose.yml run --rm rust cargo run -p sim-cli -- run ../scenarios/empty-match.json`.
- Verified the added integration tests failed before implementation and pass after implementation.
- Verified `docker compose -f docker-compose.yml run --rm rust cargo run -p sim-cli -- run ../scenarios/empty-match.json`.
- Verified an invalid scenario returns a non-zero exit code and writes diagnostics to stderr.
- Verified `docker compose -f docker-compose.yml run --rm rust cargo fmt --check`.
- Verified `docker compose -f docker-compose.yml run --rm rust cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- Verified `docker compose -f docker-compose.yml run --rm rust cargo test --workspace --all-targets --all-features`.
- Reviewed Steps 6 and 7; their names, paths, assumptions, dependencies, and expected outputs still match the implemented CLI pipeline.

---

## Step 6 — Single Quality Gate and Recorded Decisions

**Status:** Pending

### Goal

Complete the stage with reproducible verification and record only decisions that became real through implementation.

### Work

Create one script as the source of truth for Rust verification:

```text
format check
→ clippy with warnings denied
→ workspace tests
```

Provide one Docker command for running the complete quality gate.

Do not create several equivalent implementations through:

- PowerShell;
- Make;
- Just;
- separate local shell scripts;
- CI YAML.

Create ADRs only for decisions that became binding through implementation.

Expected candidates:

- the Rust core as the authoritative simulation used by the multiplayer server;
- fixed-tick execution;
- Docker as the canonical Rust development and verification environment;
- additional state hash decisions only if they supersede or materially extend `ADR-0001-BLAKE3-STATE-HASH.md`.

Do not automatically create an ADR for every candidate.

Update:

- `simulation/README.md`;
- `simulation/AGENTS.md`;
- root `README.md` only if its current status became outdated;
- later pending steps if actual names or commands changed.

Leave `docs/GLOSSARY.md` empty unless a real terminology ambiguity appeared during implementation.

### TDD

No new behavioural tests are required.

If final verification reveals a defect, add a reproducing test before fixing it.

### DoD

- One documented Docker command runs the full quality gate.
- Quality gate exits on the first failed check.
- All checks pass.
- Documented commands were actually executed.
- Only necessary ADRs were created.
- ADRs describe implemented decisions rather than intentions.
- Documentation contains no stale crate names, commands, or paths.
- `docs/GLOSSARY.md` is not populated with obvious terms.
- No generated build output, IDE cache, or secrets are tracked.
- `git status` contains only expected changes.

---

## Step 7 — End-to-End Verification and Plan Completion

**Status:** Pending

### Goal

Prove that the first stage is complete as one coherent working capability.

### Work

From a clean Docker state, execute:

1. development image build;
2. Rust toolchain verification;
3. complete quality gate;
4. `scenarios/empty-match.json`;
5. the same scenario a second time;
6. an invalid scenario;
7. `git status`.

Verify the complete pipeline:

```text
JSON scenario
→ sim-protocol
→ validation
→ sim-core
→ fixed ticks
→ state hash
→ structured trace
→ sim-cli
→ JSON result
```

After verification:

- add a factual `Done` section after the DoD of every completed step;
- change completed step statuses to `Done`;
- review the next three pending steps after every completion where applicable;
- write the completion report;
- identify the subject of the next active plan;
- do not modify `plans/ACTIVE_PLAN.md` without explicit user instruction.

### TDD

Use the existing unit and integration tests.

Do not add tests solely to increase test count.

### DoD

- Development image builds from the repository.
- Rust version matches the pinned version.
- Complete quality gate passes.
- Valid scenario executes successfully.
- Two identical runs produce the same hash.
- Invalid scenario ends with a controlled error.
- Trace matches the actual execution order.
- Every completed step contains a factual `Done` section.
- Every completed step has status `Done`.
- Completion report lists changed files, checks, assumptions, and risks.
- The next plan does not invent a robot model before purchased assets have been studied.
