# Iron Doctrine — Players, Commands, and Groups Contract

- Plan status: Pending
- File: `plans/PLAYERS_COMMANDS_AND_GROUPS_CONTRACT.md`
- Previous plan: `SIMULATION_EXECUTION_KERNEL.md`
- Next plan: Stage 3 — Modular Robot Model
- Related general stage: `GENERAL_PLAN.md`, Stage 2

## Objective

Introduce the first real gameplay vertical slice into the authoritative Rust simulation without defining the concrete robot model.

After this plan is complete, the repository must support a headless scenario in which:

1. a match has a stable identifier;
2. teams and participants are defined;
3. participants control groups;
4. groups contain stable robot identifiers without robot configuration;
5. a participant submits a real group-level command for a specific tick;
6. commands are normalized into a deterministic order;
7. each command is accepted or rejected through explicit rules;
8. an accepted command produces a domain intent;
9. the intent changes authoritative group state;
10. the simulation produces a public command result;
11. the simulation produces the first authoritative gameplay event;
12. repeated execution produces the same state hash, command results, events, and trace.

This plan establishes the gameplay command boundary used later by:

- the desktop single-player client;
- the authoritative multiplayer server;
- built-in bots;
- LLM commanders;
- replay and battle-branching systems;
- executable scenarios;
- testing and analysis tools.

The public gameplay interface must remain group-oriented. It must not provide direct command control over an individual robot.

## Runnable Result

The repository must be able to execute a versioned scenario with approximately the following meaning:

```json
{
  "schemaVersion": 2,
  "match": {
    "matchId": "match-001",
    "tickRateHz": 20,
    "seed": 123456,
    "teams": [
      {
        "teamId": "team-blue"
      },
      {
        "teamId": "team-red"
      }
    ],
    "participants": [
      {
        "participantId": "participant-blue-1",
        "teamId": "team-blue"
      },
      {
        "participantId": "participant-red-1",
        "teamId": "team-red"
      }
    ],
    "groups": [
      {
        "groupId": "group-blue-alpha",
        "controllerParticipantId": "participant-blue-1",
        "robotIds": [
          "robot-blue-001",
          "robot-blue-002"
        ]
      },
      {
        "groupId": "group-red-alpha",
        "controllerParticipantId": "participant-red-1",
        "robotIds": [
          "robot-red-001",
          "robot-red-002"
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

The exact JSON representation may be refined during implementation.

If it changes, later plan steps, scenarios, tests, and documentation must be updated to use the accepted form.

The successful JSON result must contain at least:

```text
schema version;
match identifier;
initial tick;
completed tick count;
final tick;
state hash;
command results;
authoritative gameplay events;
trace when requested.
```

The result must make it possible to distinguish:

- a command accepted by the authoritative core;
- a command rejected by the authoritative core;
- the reason for rejection;
- the authoritative gameplay event produced by an accepted command;
- diagnostic trace records explaining the execution path.

## Scope

This plan includes:

- match identifier;
- team identifiers;
- match participants;
- group identifiers;
- stable robot identifiers without a robot model;
- participant membership in a team;
- participant control of groups;
- robot membership in groups;
- validation of match roster and ownership relationships;
- command envelope;
- command sequence;
- assignment of commands to a specific tick;
- deterministic command ordering;
- the first real group command;
- the first group order;
- command acceptance and rejection;
- structured rejection reasons;
- production of a domain intent;
- authoritative application of the intent;
- persistent active group order;
- public command results;
- the first authoritative gameplay event;
- deterministic event ordering;
- state hash extension;
- structured execution trace extension;
- versioned headless scenario and result formats;
- end-to-end CLI execution;
- glossary updates for domain terms that become real;
- ADRs only for decisions that become binding through implementation.

## Out of Scope

This plan does not include:

- a concrete robot model;
- robot bodies;
- locomotion modules;
- weapons;
- sockets;
- module trees;
- robot attributes;
- health;
- damage;
- destruction;
- map;
- coordinates;
- orientation;
- movement;
- formations;
- obstacles;
- collision detection;
- sensors;
- target selection;
- firing;
- projectiles;
- physics libraries;
- Rapier;
- Unity;
- C#;
- FFI;
- networking;
- authentication;
- user accounts;
- persistence;
- replay storage;
- bots;
- LLM integration;
- async execution;
- multithreading;
- ECS;
- plugin systems;
- a universal command bus;
- a dynamic event bus;
- placeholder commands for future systems.

Do not introduce a direct command for an individual robot.

Do not implement physical holding, movement prevention, formation behaviour, or combat behaviour. In this stage, `HoldPosition` is an assigned group order stored in authoritative state. Its physical execution begins only when movement exists.

## Hexagonal Architecture

The implementation must follow the hexagonal architecture established in `GENERAL_PLAN.md`.

For this plan:

- `sim-core` owns match roster state, group ownership, command validation, intent production, intent application, command results, gameplay events, deterministic ordering, and authoritative state hashing;
- `sim-protocol` owns explicit external scenario, command, event, and result contracts;
- `sim-cli` remains a filesystem, JSON, stdout, stderr, and process-exit adapter;
- JSON shape must not become the internal domain model by accident;
- network concepts, sessions, authenticated users, and transport metadata must remain outside the core.

Dependencies must continue to point inward.

The domain and application core must not depend on:

- JSON files;
- filesystem paths;
- CLI arguments;
- network connections;
- databases;
- Docker;
- operating-system-specific APIs.

Do not create ports or traits solely to imitate a hexagonal architecture diagram.

Introduce a port only if a real external boundary or second adapter requires it.

## Domain Terminology

The following terms become real in this plan and must be defined in `docs/GLOSSARY.md` using their accepted final meanings:

- Match;
- Team;
- Participant;
- Group;
- Group Order;
- Command;
- Command Envelope;
- Command Result;
- Intent;
- Event;
- Trace.

The glossary must clearly preserve these distinctions:

- a **Command** is an external request submitted to the simulation;
- an **Intent** is an internal validated instruction produced by command processing;
- an **Event** is an authoritative fact that occurred in the game world;
- a **Trace** is diagnostic information explaining how the simulation executed.

Do not use these terms interchangeably.

## Initial Domain Contracts

### Match Identifier

Every non-empty gameplay scenario must identify the match through a stable `MatchId`.

Requirements:

- the identifier is preserved without modification;
- it is part of the authoritative initial state;
- it is included in the canonical state hash;
- it is present in successful public execution results;
- it is not inferred from a file name, filesystem path, or process state.

The exact identifier representation must be explicit and validated.

### Team

A `Team` represents one side in a match.

At this stage, a team requires only a stable `TeamId`.

Do not add:

- display names;
- colors;
- factions;
- corporations;
- spawn areas;
- score;
- victory rules.

Those concepts are added only when their behaviour exists.

### Participant

A `Participant` represents one command-producing actor inside a specific match.

A participant is not:

- a persistent user account;
- an authenticated network connection;
- a device;
- a Unity client;
- a bot implementation;
- an LLM provider.

A participant belongs to exactly one team.

Requirements:

- every `ParticipantId` is unique within a match;
- every participant references an existing team;
- participant membership is immutable during this stage;
- a participant may control zero or more groups.

### Group

A `Group` is the public command target controlled by one participant.

Requirements:

- every `GroupId` is unique within a match;
- every group references one existing controlling participant;
- a group contains zero or more stable `RobotId` values;
- every `RobotId` is unique within the match roster;
- one robot cannot belong to multiple groups;
- group membership is immutable during this stage;
- the group may store an active group order after a command is accepted.

A group with zero robots may remain valid unless implementation reveals a concrete behavioural reason to reject it.

Do not invent minimum or maximum group sizes in this plan.

### Robot Identifier

`RobotId` provides stable identity only.

This plan must not define:

- robot configuration;
- body;
- locomotion;
- modules;
- sockets;
- weapons;
- statistics;
- position;
- damage state.

Do not introduce an abstract `ControllableEntityId` unless a concrete second controllable entity exists.

### Active Group Order

A group may have no active order or one active order.

The first supported order is:

```text
HoldPosition
```

At this stage, accepting `HoldPosition` means:

- the order becomes the group’s authoritative active order;
- the state change is included in the canonical state hash;
- a gameplay event records that the order was assigned.

It does not yet mean that movement, collision, formation, or combat systems execute physical holding behaviour.

## Command Contract

### Command Envelope

The first command envelope must contain at least:

```text
sequence;
target tick;
participant identifier;
payload.
```

The exact Rust and JSON types are decided during implementation.

Requirements:

- the envelope is versioned through the surrounding protocol schema;
- the payload contains a real command;
- there is no `NoOp` command;
- there is no fake placeholder payload;
- transport timestamps are not part of authoritative ordering;
- wall-clock arrival time is not part of authoritative ordering;
- client frame order is not part of authoritative ordering.

### Command Sequence

`CommandSequence` establishes deterministic authoritative order.

The initial contract is:

- sequences are unique within the match command stream;
- lower sequence values are processed first when commands target the same tick;
- input collection order must not affect the result;
- duplicate sequences are rejected deterministically;
- the authoritative adapter that submits commands to the core is responsible for assigning sequence values;
- the public client must not be trusted to determine final authoritative ordering in future multiplayer use.

If implementation makes this sequence a binding public contract, record the decision in an ADR.

### Target Tick

`targetTick` identifies the tick whose execution receives the command.

The initial semantic contract is:

> `targetTick` equals the simulation’s `currentTick` before that tick transition is executed.

Therefore:

- a new simulation starts at authoritative tick `0`;
- a command for the first execution step targets tick `0`;
- after successful execution of that step, `currentTick` becomes `1`.

Commands for a different tick are not silently moved to the current tick.

This plan does not introduce command buffering across multiple future ticks inside the core unless required by the accepted scenario runner design.

### First Command

The first real command is conceptually:

```text
IssueGroupOrder {
    group_id,
    order: HoldPosition
}
```

Requirements:

- the target is a group;
- the participant must control the group;
- the command produces an internal intent only after validation;
- the public command does not directly mutate authoritative state;
- the public command cannot target an individual robot.

## Command Validation

Each command must produce exactly one public result:

```text
Accepted
```

or:

```text
Rejected {
    reason
}
```

The minimum structured rejection reasons are:

```text
WrongTargetTick
UnknownParticipant
UnknownGroup
GroupNotControlledByParticipant
DuplicateCommandSequence
```

Names may be refined if the final terminology becomes clearer.

Requirements:

- rejection reasons are structured, not free-form strings only;
- rejection does not partially mutate authoritative state;
- one rejected command does not prevent unrelated valid commands from being processed unless a documented invariant requires full tick failure;
- validation order is deterministic;
- repeated execution returns the same rejection reason for the same input;
- diagnostic trace may explain validation but must not replace the public command result.

Do not invent gameplay-specific rejections that require systems not yet implemented.

## Intent Contract

An accepted `IssueGroupOrder` command produces an internal group-order intent.

The intent must contain only information required to apply the validated state change.

Requirements:

- intents are internal to `sim-core`;
- external adapters do not submit intents directly;
- intent production is separate from state mutation;
- accepted intents are applied in deterministic command order;
- intent application does not repeat ownership validation already completed for the command unless an invariant check is required for safety;
- no intent exists for a rejected command.

Do not expose a universal public intent registry.

## Command Result Contract

A public command result must contain enough information to correlate it with the submitted command without relying on array position.

It must contain at least:

```text
command sequence;
target tick;
participant identifier;
accepted or rejected status;
structured rejection reason when rejected.
```

The exact representation may include the command kind when useful.

Command results are outputs of execution.

They must not be confused with gameplay events.

## Gameplay Event Contract

The first authoritative gameplay event records that an accepted group order was assigned.

A suitable conceptual event is:

```text
GroupOrderAssigned {
    tick,
    ordinal,
    group_id,
    participant_id,
    order
}
```

The final name may be refined.

Do not emit an event claiming that the group physically held a position, because no movement system exists yet.

Requirements:

- events represent authoritative facts;
- rejected commands do not produce `GroupOrderAssigned`;
- event ordering is stable;
- every event has a deterministic position within the tick;
- event meaning is versioned through the public protocol;
- events do not contain diagnostic-only data;
- trace records are not reused as gameplay events.

### Event Ordinal

Events produced during one tick require a stable order.

The simplest accepted design should be used, such as a zero-based or one-based ordinal within the tick.

The exact convention must be explicit and tested.

Do not use wall-clock timestamps or random identifiers to order events.

## Match Roster Validation

Match creation must reject structurally invalid roster data before simulation execution begins.

The minimum validation rules are:

- duplicate `TeamId` values are rejected;
- duplicate `ParticipantId` values are rejected;
- a participant referencing an unknown team is rejected;
- duplicate `GroupId` values are rejected;
- a group referencing an unknown participant is rejected;
- duplicate `RobotId` values inside one group are rejected;
- the same `RobotId` appearing in multiple groups is rejected.

Validation errors must be structured.

Do not silently repair identifiers, ownership, membership, or order.

Do not infer missing references.

## Deterministic Processing Order

For one tick, command processing must be explicit and readable from top to bottom.

Target execution order:

```text
1. Verify that the next tick may begin.
2. Select commands assigned to the current tick.
3. Normalize commands into stable sequence order.
4. Detect duplicate command sequences deterministically.
5. Validate command envelopes and payloads.
6. Produce internal group-order intents for accepted commands.
7. Apply accepted intents in stable order.
8. Produce public command results.
9. Finalize authoritative gameplay events in stable order.
10. Apply the authoritative tick transition.
11. Calculate the hash of the resulting authoritative state.
12. Complete structured trace records.
13. Produce the tick result.
```

The exact placement of command results and event finalization may be refined if implementation reveals a clearer pipeline, but the following guarantees must remain:

- validation precedes mutation;
- intents are produced before they are applied;
- state mutation follows deterministic order;
- events describe applied authoritative changes;
- rejected commands do not mutate state;
- state hashing occurs after all authoritative changes for the tick;
- trace cannot affect state, events, command results, or hash.

Do not add empty phases for future gameplay systems.

## Authoritative State Hash

The canonical state hash must be extended to include every new persistent authoritative value that can affect future execution.

At minimum, the hash must include:

```text
authoritative state version;
match identifier;
tick rate;
seed;
current tick;
teams in canonical order;
participants in canonical order;
groups in canonical order;
robot identifiers in canonical order;
group ownership;
active group orders.
```

Canonical ordering must not depend on:

- JSON input array order;
- `HashMap` iteration;
- memory addresses;
- filesystem order;
- trace;
- command results;
- gameplay event serialization order when events are not persistent state;
- wall-clock time;
- execution duration.

The plan must preserve the existing BLAKE3 hashing decision unless a deliberate accepted ADR supersedes it.

If the canonical representation changes materially, update the existing state-hash ADR or create a new ADR according to the repository decision rules.

## Trace Extension

The structured trace must be extended only for operations that actually exist.

Useful trace record meanings may include:

```text
commands selected;
commands normalized;
command validation completed;
intent produced;
intent applied;
gameplay events finalized.
```

Exact record kinds are decided during implementation.

Requirements:

- trace remains optional;
- enabling trace does not change command results;
- enabling trace does not change gameplay events;
- enabling trace does not change authoritative state;
- enabling trace does not change the state hash;
- trace remains diagnostic output, not a gameplay event stream.

## Protocol Versioning

The headless scenario contract changes materially in this plan.

The implementation must make version handling explicit.

The expected result is:

- the new scenario and result contract uses schema version `2`;
- unsupported versions are rejected through structured errors;
- old version `1` scenarios are not silently interpreted as version `2`;
- backward compatibility is not added unless a concrete need appears;
- all repository scenarios and documentation are updated to the accepted current version.

If implementation reveals a better versioning design, document the reason and update the plan before applying it.

## Target Repository Changes

Expected changes may include:

```text
simulation/
├── crates/
│   ├── sim-protocol/
│   └── sim-core/
├── apps/
│   └── sim-cli/
└── README.md

scenarios/
├── empty-match.json
├── group-order.json
└── invalid-group-order.json

docs/
├── GLOSSARY.md
└── decisions/

plans/
└── PLAYERS_COMMANDS_AND_GROUPS_CONTRACT.md
```

Exact file names may be refined during implementation.

Do not create a new crate unless an actual responsibility cannot remain in the existing crates.

## Testing Rules

All code-producing steps follow TDD.

Add the minimum tests required to protect meaningful behaviour.

Do not add tests for:

- enum discriminants without behavioural meaning;
- simple getters;
- constructors that only assign fields;
- serde library behaviour in isolation;
- module exports;
- constants;
- language behaviour;
- every rejection branch at every test level;
- identical behaviour duplicated across unit, integration, and CLI tests without a reason.

Every defect discovered during implementation must receive a reproducing test or executable scenario before it is fixed.

---

## Step 1 — Match Identity, Teams, Participants, and Groups

**Status:** Done

### Goal

Introduce the authoritative match roster and group ownership model without creating the concrete robot model.

### Work

Implement validated protocol and core representations for:

- `MatchId`;
- `TeamId`;
- `ParticipantId`;
- `GroupId`;
- `RobotId`;
- teams;
- participants;
- groups;
- participant-to-team membership;
- group-to-participant control;
- robot-to-group membership.

Extend match creation so it receives validated roster data.

Preserve support for the existing empty kernel scenario if this can be done without ambiguous defaults or weakening the new contract.

If an empty roster remains supported, it must be represented explicitly rather than inferred from missing fields.

Extend the canonical state hash with the accepted roster model.

Update `docs/GLOSSARY.md` with the domain terms introduced by this step.

### TDD

Before implementation, add the minimum behavioural tests:

1. a valid roster with two teams, two participants, and two groups creates a simulation;
2. a participant referencing an unknown team is rejected;
3. a group controlled by an unknown participant is rejected;
4. a robot assigned to more than one group is rejected;
5. equivalent roster data in different input array order produces the same authoritative state hash.

Add additional tests only when needed to distinguish materially different validation behaviour.

### DoD

- Match identity is explicit and preserved.
- Teams, participants, groups, and robot identifiers use stable validated types.
- Every participant belongs to an existing team.
- Every group is controlled by an existing participant.
- One robot cannot belong to multiple groups.
- Duplicate identifiers are rejected through structured errors.
- No concrete robot model is introduced.
- No direct individual-robot command API is introduced.
- Canonical roster ordering is independent of JSON input order.
- The state hash includes all persistent roster data.
- Glossary terminology matches code and protocol names.
- All added tests failed before implementation and now pass.
- The full Rust quality gate passes.

### Done

- Added validated `MatchId`, `TeamId`, `ParticipantId`, `GroupId`, and `RobotId` protocol types.
- Added `TeamConfig`, `ParticipantConfig`, `GroupConfig`, and validated `MatchConfig` roster construction.
- Added structured roster validation errors for duplicate teams, participants, groups, and robots, unknown participant teams, and unknown group controllers.
- Extended `MatchSimulation` and authoritative state hashing to preserve match identity and canonical roster data.
- Updated the explicit empty-match scenario shape with `matchId`, `teams`, `participants`, and `groups`.
- Updated `docs/GLOSSARY.md`, `simulation/README.md`, ADR-0001, and added ADR-0005 for canonical roster state hashing.
- Added and verified behavioural tests for valid roster creation, invalid roster references, duplicate roster identifiers, duplicate robot membership, and roster-order-independent hashing.
- Reviewed the next three pending steps and found no required scope or naming updates.
- Ran `docker compose -f docker-compose.yml run --rm rust sh scripts/check.sh` successfully.

---

## Step 2 — First Group Command and Command Envelope

**Status:** Pending

### Goal

Define the first real public gameplay command and its deterministic envelope without applying it to state yet.

### Work

Implement protocol and core representations for:

- `CommandSequence`;
- `targetTick`;
- `CommandEnvelope`;
- `IssueGroupOrder`;
- `HoldPosition`;
- structured command validation result;
- structured rejection reasons;
- internal group-order intent.

The command envelope and real payload must be introduced together.

Do not create:

- `NoOp`;
- placeholder command variants;
- direct robot commands;
- map coordinates;
- movement parameters;
- transport timestamps;
- client frame numbers.

Define the exact semantic meaning of `targetTick`.

Define the deterministic sequence-order contract.

Update `docs/GLOSSARY.md` for Command, Command Envelope, Command Result, Intent, Event, and Trace.

Create an ADR only if the command sequence contract becomes a binding architectural or public decision.

### TDD

Before implementation, add the minimum behavioural tests:

1. a participant may issue `HoldPosition` to a group they control;
2. a command for an unknown participant is rejected;
3. a command for an unknown group is rejected;
4. a participant commanding another participant’s group is rejected;
5. a command assigned to a different tick is rejected;
6. a rejected command produces no intent.

Do not test JSON formatting details when whole-contract protocol tests already prove the shape.

### DoD

- The command envelope contains sequence, target tick, participant, and a real payload.
- `HoldPosition` is the first supported group order.
- The public command target is a group.
- Ownership validation is explicit.
- Wrong-tick validation is explicit.
- Rejection reasons are structured.
- Rejected commands produce no intent.
- Accepted commands produce an internal intent.
- No command mutates state directly.
- No placeholder commands exist.
- No direct individual-robot command exists.
- Glossary distinctions are explicit and consistent.
- Necessary ADRs describe implemented decisions only.
- All added tests failed before implementation and now pass.
- The full Rust quality gate passes.

---

## Step 3 — Deterministic Command Execution and Authoritative Event

**Status:** Pending

### Goal

Process commands during a tick, apply accepted intents, and produce the first authoritative gameplay event.

### Work

Extend tick execution to:

- receive commands assigned to the current tick;
- normalize them by deterministic sequence order;
- detect duplicate sequences;
- validate each command;
- produce intents for accepted commands;
- apply accepted intents in stable order;
- store the active group order in authoritative state;
- produce public command results;
- produce `GroupOrderAssigned` or the accepted final equivalent;
- assign deterministic event ordinals;
- include active group orders in the state hash;
- extend trace with real command-processing operations.

The main tick method must remain explicit and readable from top to bottom.

Define behaviour for multiple commands affecting the same group in one tick.

The preferred minimal rule is that accepted commands are applied in sequence order, so the last accepted command in that order becomes the active order at the end of the tick.

Do not add a special conflict-resolution abstraction unless real behaviour requires it.

### TDD

Before implementation, add the minimum behavioural tests:

1. an accepted `HoldPosition` command becomes the group’s active order;
2. an accepted command produces one authoritative group-order event;
3. a rejected command does not change group state and produces no gameplay event;
4. commands supplied in different input order are processed in the same sequence order;
5. equivalent command sets in different input order produce identical command results, events, final state, and state hash;
6. duplicate command sequences are rejected deterministically;
7. enabling trace does not change command results, events, state, or hash.

Add a focused test for multiple accepted commands targeting the same group if the final command model allows this case.

### DoD

- Commands are selected for the current tick.
- Stable ordering is independent of input collection order.
- Duplicate sequences have deterministic behaviour.
- Validation occurs before state mutation.
- Accepted commands produce intents.
- Intents are applied in stable order.
- Rejected commands do not partially mutate state.
- Active group orders are persistent authoritative state.
- The first gameplay event represents an order assignment, not physical movement.
- Event ordinals are stable.
- Command results and gameplay events are separate contracts.
- Trace and gameplay events remain separate concepts.
- State hash includes active group orders.
- Trace does not affect authoritative outputs.
- All added tests failed before implementation and now pass.
- The full Rust quality gate passes.

---

## Step 4 — Versioned Headless Scenario and CLI Result

**Status:** Pending

### Goal

Expose the complete Stage 2 vertical slice through versioned JSON input and output.

### Work

Extend `sim-protocol` and `sim-cli` so a headless scenario can define:

- schema version;
- match identifier;
- tick rate;
- seed;
- teams;
- participants;
- groups;
- robot identifiers;
- scheduled commands;
- number of ticks;
- trace enabled or disabled.

Extend successful CLI output with:

- schema version;
- match identifier;
- initial tick;
- completed tick count;
- final tick;
- final state hash;
- command results;
- authoritative gameplay events;
- trace when requested.

Create repository scenarios covering:

- a valid group order;
- a command targeting a foreign group;
- an unknown participant or group;
- deterministic input reordering;
- invalid roster data.

Keep filesystem access, JSON reading, stdout, stderr, and process exit status in `sim-cli`.

Do not duplicate in the CLI:

- roster validation;
- ownership validation;
- command ordering;
- command acceptance;
- intent production;
- event production;
- state mutation;
- hashing;
- trace creation.

### TDD

Before implementation, add the minimum integration tests:

1. a valid scenario assigns `HoldPosition` and returns an accepted command result;
2. the valid scenario returns the authoritative group-order event;
3. a foreign-group command is rejected without failing the entire valid match execution;
4. reordered input commands produce identical normalized command results, events, and final hash;
5. invalid roster data returns a controlled non-zero CLI result;
6. successful stdout contains only valid JSON;
7. diagnostics remain on stderr.

Tests must verify meaningful fields, not whitespace or formatting.

### DoD

- The accepted scenario schema version is explicit.
- Unsupported versions are rejected.
- The Stage 2 scenario runs inside Docker.
- Successful output contains command results and gameplay events.
- Rejected commands are represented as successful simulation output when the match itself remains valid.
- Invalid match configuration returns a controlled CLI failure.
- Reordered command input produces identical authoritative output.
- stdout contains only successful JSON output.
- stderr contains useful diagnostics.
- CLI contains no duplicated simulation logic.
- Documentation contains the verified command and accepted JSON shapes.
- All integration tests failed before implementation and now pass.
- The full Rust quality gate passes.

---

## Step 5 — Contract Documentation and Recorded Decisions

**Status:** Pending

### Goal

Align permanent documentation with the implemented gameplay boundary and record only decisions that became binding.

### Work

Review and update:

- `simulation/README.md`;
- `simulation/AGENTS.md` only if component rules changed;
- root `README.md` if current project status became outdated;
- `docs/GLOSSARY.md`;
- relevant existing ADRs;
- `docs/decisions/README.md`;
- later pending plan assumptions affected by implementation.

Expected decision review:

- deterministic global command sequence;
- exact `targetTick` semantics;
- canonical roster and active-order state hashing;
- separation of Command, Intent, Event, and Trace.

Do not automatically create one ADR per topic.

Create or update an ADR only when the implemented decision meets the repository criteria for long-term architectural decisions.

Document the exact accepted behaviour for:

- participant versus user account;
- group ownership;
- robot identity without robot configuration;
- command sequence;
- target tick;
- command rejection;
- event ordering;
- active group order;
- schema version support.

### TDD

No new behavioural tests are required.

If documentation review exposes a behavioural inconsistency, add a reproducing test before changing code.

### DoD

- Documentation matches actual symbols, paths, and JSON contracts.
- `docs/GLOSSARY.md` contains the real domain distinctions introduced by this plan.
- No documentation claims physical group-order execution.
- No documentation describes an individual-robot command API.
- ADRs describe implemented decisions rather than intentions.
- Existing ADRs are updated or superseded according to repository rules.
- No stale schema version, scenario, or CLI examples remain.
- The full Rust quality gate passes after documentation changes.

---

## Step 6 — End-to-End Verification and Plan Completion

**Status:** Pending

### Goal

Prove that Stage 2 is complete as one coherent gameplay capability.

### Work

From a clean Docker state, execute:

1. development image build;
2. Rust toolchain verification;
3. the complete Rust quality gate;
4. the valid group-order scenario;
5. the same scenario a second time;
6. a semantically equivalent scenario with commands in a different input order;
7. a scenario containing an ownership rejection;
8. an invalid roster scenario;
9. `git status --short`.

Verify the complete pipeline:

```text
versioned JSON scenario
→ protocol deserialization
→ roster validation
→ authoritative match creation
→ command selection
→ deterministic command ordering
→ command validation
→ intent production
→ authoritative group-state change
→ command results
→ gameplay events
→ state hash
→ structured trace
→ CLI JSON result
```

After verification:

- add a factual `Done` section directly after every step’s DoD;
- change each completed step status to `Done`;
- review the next three pending steps after each completion where applicable;
- update future plan references to match sound implemented names;
- preserve completed steps as historical records;
- mark the plan status as `Complete`;
- write the completion report;
- identify the next active plan subject;
- do not modify `plans/ACTIVE_PLAN.md` without explicit user instruction.

### TDD

Use existing unit, integration, and executable scenario tests.

Do not add tests solely to increase test count.

### DoD

- Development image builds from the repository.
- Rust and Cargo versions match the pinned toolchain.
- The complete quality gate passes.
- The valid group-order scenario executes successfully.
- Repeated identical runs produce identical JSON output or identical authoritative fields where formatting is intentionally non-canonical.
- Reordered command input produces identical command results, gameplay events, final state, and hash.
- An ownership violation produces a structured rejected command result without unauthorized state mutation.
- Invalid roster data produces a controlled CLI failure.
- Trace order matches the operations actually executed.
- Trace does not change command results, events, state, or hash.
- Every completed step contains a factual `Done` section.
- Every completed step has status `Done`.
- The plan is marked `Complete`.
- The completion report lists changed files, checks, assumptions, unresolved questions, and known risks.
- The next plan does not invent the concrete robot model before purchased assets have been studied.

## Expected Stage Result

This plan is complete when the following statement is true:

> A match participant can submit a real group-level order for a specific tick. The authoritative Rust core deterministically accepts or rejects the command, produces an internal intent for an accepted command, changes persistent group state, emits a public command result and authoritative gameplay event, and returns a reproducible headless JSON result without introducing the concrete robot model or direct individual-robot control.
