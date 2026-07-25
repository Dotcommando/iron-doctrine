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

## Robot Identifier

A stable `RobotId` that identifies a robot in the match roster. This stage does not define robot configuration, body, locomotion, modules, sockets, weapons, statistics, position, damage, or destruction.

## State Hash

A deterministic BLAKE3-256 hash calculated from canonical authoritative state. Trace records, JSON formatting, filesystem paths, collection iteration order, memory addresses, and wall-clock data are excluded.
