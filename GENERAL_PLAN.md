# Iron Doctrine — General Development Plan

## Purpose

This document defines the path from an empty repository to a working single-player and multiplayer game.

It establishes:

- the long-term technical objective;
- mandatory engineering principles;
- the order of major development stages;
- the verifiable result of each stage;
- the rules for detailed active plans.

This document does not replace:

- `docs/VISION.md` — the product vision;
- `AGENTS.md` — rules for Codex and other agents;
- ADRs — accepted architectural decisions;
- active plans under `plans/` — concrete implementation steps.

File names, types, functions, and test details belong in active plans only when the corresponding stage is ready for implementation.

## Development Objective

Create a deterministic authoritative combat simulation core in Rust.

This core is the single implementation of gameplay rules and is used by:

- the desktop single-player game;
- authoritative multiplayer servers;
- built-in game bots;
- LLM commanders;
- replay and battle-branching systems;
- executable scenarios;
- testing, balancing, and analysis tools.

The multiplayer server does not contain a separate implementation of combat. It manages connections, match placement, command delivery, persistence, and other server responsibilities, but every match is calculated by the same Rust core.

Unity is responsible for the client side:

- player input;
- presentation of authoritative state;
- animation;
- audio;
- user interface;
- camera;
- visual effects;
- game asset integration;
- tooling for authoring and exporting data used by the simulation.

Gameplay rules must not be duplicated in C#.

## Quality Requirements

The project is developed by one person, so the architecture must accelerate development rather than create organizational overhead.

At the same time, the code must remain suitable for long-term evolution.

Mandatory requirements:

- code reads from top to bottom without relying on hidden conventions;
- domain names are preferred over generic technical names;
- public contracts are explicit;
- states and transitions are explainable;
- errors are not silently ignored;
- gameplay logic does not depend on global mutable state;
- dependencies are added only for concrete needs;
- abstractions appear only after real repetition or a second implementation exists;
- optimization follows measurement;
- `unsafe` is forbidden in the simulation core without a separate accepted ADR;
- runtime code does not rely on uncontrolled `unwrap()` or `expect()`;
- a completed stage does not intentionally leave behind a temporary architecture intended for later replacement.

Do not use language or development approaches that encourage lower code quality for a supposedly temporary result.

Every stage is implemented on the main architecture of the project and must leave the system in a working state.

## Hexagonal Architecture

The Rust simulation core uses hexagonal architecture.

Every new feature must be considered through this architecture before implementation.

The domain and application core must not depend on:

- Unity;
- CLI frameworks;
- network transports;
- databases;
- filesystem layout;
- Docker;
- serialization formats used only by external adapters;
- operating-system-specific APIs.

External systems interact with the core through explicit ports.

Adapters implement those ports for concrete environments, including:

- the headless CLI;
- the Unity client;
- the multiplayer server;
- replay storage;
- scenario files;
- persistence;
- diagnostics and analysis tools.

Dependencies point inward.

The domain model must not call infrastructure code directly.

A new abstraction is not introduced merely because hexagonal terminology exists. Ports are created only where an actual external boundary or a second adapter exists.

The architecture must remain practical and readable. Hexagonal architecture is used to protect the simulation core from infrastructure concerns, not to multiply interfaces, crates, or wrapper types without purpose.

## Core Engineering Principles

### Authoritative Rust Core

The Rust core determines every result that affects the game world, including:

- match state;
- order execution;
- group state;
- robot and module state;
- movement;
- detection;
- target selection;
- firing;
- collisions;
- hits;
- damage;
- destruction;
- objective capture;
- victory;
- gameplay randomness.

Unity presents the result but does not determine it independently.

### Determinism

Identical:

- initial data;
- configuration;
- rules version;
- seed;
- ordered input command sequence

must produce identical authoritative events and state.

The result must not depend on:

- Unity frame rate;
- `deltaTime`;
- wall-clock time;
- machine performance;
- accidental collection iteration order;
- thread execution order.

### Fixed Tick

Authoritative match time is measured in integer ticks.

Game state changes only during tick execution.

A command cannot be applied at an arbitrary moment in the middle of a calculation. It is assigned to a specific tick and processed in an explicit order.

### Explicit Execution Order

The main tick pipeline must be readable and traceable.

The target order, as systems are introduced, is:

```text
1. Begin tick
2. Receive inputs assigned to the current tick
3. Normalize stable input order
4. Validate commands
5. Produce domain intents
6. Propagate commands
7. Execute group behaviour
8. Resolve movement
9. Resolve sensors and detection
10. Resolve target selection and aiming
11. Resolve weapons
12. Move projectiles
13. Detect collisions and hits
14. Calculate and apply damage
15. Resolve destruction and disabled modules
16. Resolve map objectives and victory conditions
17. Finalize authoritative events
18. Complete tick
19. Calculate state hash
20. Produce tick result
```

Only working phases are added to the code.

Do not create empty subsystems, placeholder modules, or a universal registry for future phases.

When a new phase appears, its position in the sequence must be explicit.

### Separate Calculation from Application

Where order affects the result, a system should, when practical:

1. read the state at the beginning of the phase;
2. calculate intents or expected changes;
3. order the results;
4. apply them in a stable order.

This supports:

- determinism;
- readable tracing;
- correct handling of simultaneous actions;
- possible future parallel calculation.

### Parallel Matches

One match is initially calculated sequentially.

Different matches may run in parallel through a thread pool.

Asynchronous code is used for:

- networking;
- waiting for incoming commands;
- persistence;
- infrastructure timers;
- other I/O work.

CPU-heavy simulation must not run directly on an async runtime worker.

Parallel execution inside one match is introduced only after profiling and only if deterministic result application remains guaranteed.

### Traceability

For meaningful scenarios, the following path must be observable:

```text
inputs
→ processing stages
→ decisions
→ state changes
→ authoritative events
→ final hash
```

A trace explains how the simulation worked.

An event is a fact in the game world.

Trace and Event must remain separate concepts.

## Rust Project Structure

The structure is created incrementally as real responsibilities appear.

Expected components:

```text
simulation/
├── crates/
│   ├── sim-core/
│   ├── sim-protocol/
│   ├── sim-replay/
│   └── sim-testkit/
└── apps/
    ├── sim-cli/
    └── sim-server/
```

Responsibilities:

- `sim-core` — match state and authoritative gameplay rules;
- `sim-protocol` — external input and output contracts of the core;
- `sim-replay` — battle recording, playback, and branching;
- `sim-testkit` — executable scenarios and verification tools;
- `sim-cli` — headless execution, diagnostics, and analysis;
- `sim-server` — multiplayer infrastructure around `sim-core`.

Do not create all components in advance.

A component appears only when its responsibility exists and should not belong elsewhere.

## Stage 1 — Simulation Execution Kernel

Create a reproducible Rust environment and the smallest working match kernel without a robot model.

Stage result:

- Cargo workspace;
- pinned Rust toolchain;
- Docker environment for Windows and macOS;
- `sim-core`;
- `sim-protocol`;
- `sim-cli`;
- validated match configuration format;
- fixed tick;
- explicit match lifecycle;
- explicit execution order for one tick;
- deterministic state hash;
- structured execution trace;
- JSON headless scenario format;
- JSON execution result;
- one command for formatting, linting, and tests.

The first scenario contains no robots and no gameplay commands.

It creates an empty match from configuration, executes a specified number of ticks, and proves:

- lifecycle correctness;
- stable phase order;
- determinism;
- reproducible state hash;
- correct external formats;
- headless execution.

## Stage 2 — Players, Commands, and Groups Contract

Add the first real gameplay vertical slice without a concrete robot model.

Stage result:

- match identifier;
- player teams;
- match participants;
- groups;
- ownership of controllable entities by groups;
- command envelope;
- assignment of a command to a specific tick;
- stable command ordering;
- command acceptance and rejection;
- the first real group command;
- production of a domain intent;
- public command results;
- the first authoritative gameplay event.

The public gameplay interface must not provide direct control over an individual robot.

## Stage 3 — Modular Robot Model

This stage starts after the purchased Unity assets have been studied.

Stage result:

- body as the root of robot assembly;
- replaceable locomotion module;
- weapons and other modules;
- sockets;
- installed-module tree;
- socket compatibility;
- stable instance identifiers;
- derived assembly properties;
- configuration validation;
- readable assembly trace.

A robot is assembled around its body.

Tracked, wheeled, legged, antigravity, or stationary bases are replaceable locomotion modules.

Losing the locomotion module does not necessarily destroy the body.

The architecture must allow future mechanics such as:

- leaving a damaged body immobile;
- repairing it;
- evacuating it;
- installing a different base;
- converting the body into a stationary turret.

Exact module categories depend on available assets and real gameplay requirements.

## Stage 4 — Space, Map, and Group Movement

Stage result:

- coordinate system;
- position and orientation of entities;
- map and authoritative world geometry;
- locomotion parameters;
- group movement;
- fast movement;
- movement while preserving group cohesion;
- holding position;
- obstacles;
- movement collisions;
- stable results from identical inputs.

Visual animation of legs, wheels, or tracks does not determine authoritative movement.

## Stage 5 — Sensors, Detection, and Target Selection

Stage result:

- visibility and detection;
- participant-visible information;
- module sensors;
- contact loss;
- target selection;
- group reaction to detection;
- separation between true world state and participant-known state;
- information restrictions for built-in AI and future LLM commanders.

## Stage 6 — First Complete Combat Pipeline

Stage result:

```text
two groups receive orders
→ move
→ detect each other
→ select targets
→ aim weapons
→ fire
→ resolve trajectories
→ determine hits
→ apply damage
→ disable modules and robots
→ determine the battle result
```

Only mechanics required by this vertical slice are introduced:

- weapons;
- reload;
- range;
- hitscan;
- authoritative projectiles;
- broad phase;
- precise hitboxes;
- projectile sweeps;
- damage zones;
- damage;
- module disablement;
- destruction;
- battle completion.

A physics library is added only after the exact queries it must solve have been defined.

## Stage 7 — Unity and Rust Integration

The Unity project is created after Rust can independently execute a complete simple battle.

Stage result:

- pinned Unity version;
- Unity client;
- approved integration interface;
- input command transfer;
- authoritative event reception;
- render-state reception;
- visualization of a headless scenario;
- authoritative debug geometry;
- tick-by-tick inspection;
- comparison of Unity presentation with Rust trace.

The initial integration may use primitives instead of commercial models.

Gameplay rules are not moved into Unity to simplify integration.

## Stage 8 — Game Asset Pipeline

Stage result:

- socket authoring in Unity;
- collision-shape authoring;
- damage-zone authoring;
- bone and animation binding;
- data export from Unity;
- loading exported data in the Rust core;
- identical modular robot assembly in Unity and Rust;
- visualization of hitboxes actually used by Rust;
- verification that visual and authoritative poses match.

The system is built around assets that are actually purchased and used.

## Stage 9 — Replays and Battle Branching

Stage result:

- battle artifact format;
- simulation version;
- initial configuration;
- seed;
- command journal;
- configuration hash;
- snapshots;
- playback;
- state-hash verification;
- seeking;
- restarting from a selected tick;
- replacement of later commands;
- creation of a new battle branch;
- relationship between source and derived artifacts.

A replay is repeated execution of the authoritative simulation, not a video recording.

## Stage 10 — Built-In Game AI

Stage result:

- the bot receives only permitted state;
- the bot emits ordinary group commands;
- the bot cannot modify world state directly;
- the bot can replace an absent player;
- decision frequency and strength are adjustable;
- large batches of headless battles can run;
- metrics are collected;
- AI decisions are available for analysis.

AI does not receive special commands unavailable to a human player.

## Stage 11 — Performance and Match Scale

Target upper bound for a primary match:

```text
up to 10 players;
up to 30 robots per player;
up to 300 robots per match.
```

Stage result:

- full-tick benchmarks;
- per-phase measurements;
- memory measurements;
- collision-query cost;
- replay playback speed;
- mass AI battle throughput;
- estimate of concurrent matches;
- distribution of matches across a thread pool.

Only measured bottlenecks are optimized.

## Stage 12 — Desktop Single-Player Game

The stage result is a complete working product.

Required gameplay loop:

- launch the game;
- assemble forces;
- create groups;
- select doctrine and orders;
- start a match;
- play against built-in opponents;
- receive battle results;
- gain progression;
- save progress;
- watch replays;
- configure settings;
- handle failures;
- produce a distributable desktop build.

Exact content depends on purchased assets and the quality of implemented systems.

## Stage 13 — Battle Library and Replay Progression

Stage result:

- battle artifact storage;
- viewing personal and other players' battles;
- small experience rewards for watching;
- larger rewards for branching and replaying;
- higher value for live participation;
- comparison of original and alternative outcomes;
- protection against meaningless farming;
- replay video export.

## Stage 14 — LLM Commanders

Stage result:

- constrained command API;
- access only to permitted state;
- group-level commands;
- decision-frequency limits;
- cost limits;
- decision logs;
- playing on behalf of an owner;
- limited progression gain;
- public AI players;
- matches between different LLMs;
- no hidden advantage over human players.

Built-in game AI remains mandatory and does not depend on external LLM availability.

## Stage 15 — Multiplayer Infrastructure

Stage result:

- `sim-server`;
- match creation and placement;
- player command reception;
- match execution through `sim-core`;
- event and snapshot delivery;
- reconnect;
- replacement of disconnected players;
- observers;
- battle artifact persistence;
- matchmaking;
- concurrent execution of multiple matches;
- horizontal server scaling.

Transport, authentication, and persistence do not leak into `sim-core`.

## Rules for Detailed Plans

Every active plan must:

- implement one coherent capability;
- end with a runnable and verifiable result;
- contain steps of reasonable size;
- combine necessary structure, code, tests, and documentation;
- not separate directory or empty-file creation into standalone steps;
- not combine multiple independent subsystems into one step;
- design code-producing steps from behavioural tests;
- add only the minimum required tests;
- not test enum values, barrel exports, simple constants, or language behaviour;
- include a Definition of Done for every step;
- add a factual `Done` section after completion;
- review the next three pending steps after every completed step;
- update future names and paths to match the actual implementation;
- preserve completed steps as historical records.

## Documentation

Permanent minimum:

```text
AGENTS.md
README.md
GENERAL_PLAN.md
docs/VISION.md
docs/GLOSSARY.md
docs/decisions/
plans/ACTIVE_PLAN.md
plans/<ACTIVE_PLAN>.md
```

New documentation is created only when it:

- records an important decision;
- defines an external contract;
- explains a non-obvious pipeline;
- enables implementation verification;
- resolves a real terminology ambiguity.

Do not create documents that restate existing documents in different words.
