# Iron Doctrine Glossary

This glossary is the canonical terminology source for Iron Doctrine.

## Match

A single authoritative simulation instance. A match has a stable `MatchId`, tick rate, seed, current authoritative tick, and match roster.

## Match Roster

The validated set of teams, participants, groups, and robot identifiers that belongs to one match. The roster is authoritative state and is immutable in the current stage.

## Team

One side in a match. A team has a stable `TeamId`. This stage does not define display names, colors, factions, spawn areas, score, or victory rules.

## Participant

A command-producing actor inside a specific match. A participant has a stable `ParticipantId` and belongs to exactly one existing team. A participant is not a persistent user account, network connection, device, Unity client, bot implementation, or LLM provider.

## Group

The public command target controlled by one participant. A group has a stable `GroupId`, references its controlling participant, and contains zero or more stable robot identifiers.

## Group Order

An order assigned to a group through a validated command. The first supported group order is `HoldPosition`. In the current stage this is a domain order only; it does not execute physical movement, formation, collision, or combat behaviour.

## Active Group Order

The persistent authoritative group order currently assigned to a group. A group may have no active group order or one active group order. Accepted `HoldPosition` commands set this state; rejected commands do not change it.

## Robot Identifier

A stable `RobotId` that identifies a robot in the match roster. This stage does not define robot configuration, body, locomotion, modules, sockets, weapons, statistics, position, damage, or destruction.

## Command

An external request submitted to the simulation. A command is carried by a command envelope and is validated by the authoritative core before it can produce an internal intent.

## Command Envelope

The deterministic wrapper around a command payload. The current command envelope contains a `CommandSequence`, `TargetTick`, `ParticipantId`, and real command payload.

## Command Sequence

The deterministic ordering and correlation value in a command envelope. Lower `CommandSequence` values are processed first for commands targeting the same tick. Duplicate command sequences are rejected deterministically.

## Target Tick

The authoritative tick that receives a command. `TargetTick` must equal the simulation `currentTick` before that tick transition is executed; commands are not silently moved to another tick.

## Command Result

A public output describing whether one submitted command was accepted or rejected. A command result includes enough information to correlate it with the submitted command and uses structured rejection reasons.

## Command Rejection Reason

A structured reason explaining why a command was rejected. The current reasons are `WrongTargetTick`, `UnknownParticipant`, `UnknownGroup`, `GroupNotControlledByParticipant`, and `DuplicateCommandSequence`.

## Intent

An internal validated instruction produced by command processing. Intents belong to `sim-core`; external adapters do not submit them directly.

## Event

An authoritative fact that occurred in the game world. Events are public gameplay outputs and are not diagnostic trace records.

## Event Ordinal

A deterministic zero-based position for a gameplay event within one authoritative tick. Event ordinals are not wall-clock timestamps and are not random identifiers.

## State Hash

A deterministic BLAKE3-256 hash calculated from canonical authoritative state. Trace records, JSON formatting, filesystem paths, collection iteration order, memory addresses, and wall-clock data are excluded.

## Trace

Diagnostic information explaining how the simulation executed. Trace records are optional diagnostics and do not affect authoritative state, events, command results, or state hashes.

## Headless Scenario

A versioned JSON input consumed by `sim-cli run`. The current accepted schema is `schemaVersion: 2`; unsupported schema versions are rejected before simulation execution.
