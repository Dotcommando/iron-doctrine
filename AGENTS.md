# Iron Doctrine — Agent Instructions

## Scope

This repository contains:

- `simulation/` — authoritative Rust combat simulation.
- `client/` — Unity client and presentation layer.
- `plans/` — implementation plans.
- `docs/GLOSSARY.md` — canonical project terminology.
- `docs/decisions/` — accepted architectural and technical decisions.

This file defines repository-wide rules. Follow the nearest nested `AGENTS.md` for component-specific instructions.

## Architecture

- The simulation is the single source of truth for movement, targeting, collisions, hits, damage, module state, random outcomes, victory conditions, and replays.
- The client owns input, rendering, animation, audio, UI, camera, and cosmetic effects.
- The client sends intent commands and renders simulation events and snapshots. It must not decide authoritative outcomes.
- The simulation must remain headless, testable, and independent of Unity, UI, transport, and server infrastructure.
- Robots are modular assemblies connected through explicit module and socket contracts.

## Simulation Guarantees

Preserve these properties in all relevant changes:

- Fixed simulation ticks.
- Seeded and controlled randomness only.
- No wall-clock time or render-frame delta in authoritative logic.
- Stable processing order where order can affect results.
- Commands are inputs; events and snapshots are outputs.
- Persistent state changes only during explicit simulation phases.
- Authoritative behaviour must be reproducible and explainable through structured traces or state diffs.

## Project Glossary

`docs/GLOSSARY.md` is the canonical source for project terminology.

- Read it before introducing domain concepts or names.
- Use glossary terms consistently in code, plans, tests, traces, and documentation.
- Update it when a new domain term becomes part of the project.
- Do not introduce synonyms for an existing term without an explicit reason.
- Prefer names from the glossary over generic names such as `Manager`, `Helper`, or `Processor`.

## Decisions

Accepted architectural and technical decisions are stored in:

```text
docs/decisions/
```

The directory index and decision-writing rules are stored in:

```text
docs/decisions/README.md
```

Decision files use this naming format:

```text
ADR-NNNN-SHORT-TITLE.md
```

Examples:

```text
ADR-0001-RUST-AUTHORITATIVE-SIMULATION.md
ADR-0002-FIXED-SIMULATION-TICK.md
ADR-0003-MODULE-ASSEMBLY-AS-TREE.md
```

Create or update an ADR when a decision:

- affects architectural boundaries;
- introduces a major dependency;
- changes a public contract or persistent format;
- imposes a long-term implementation constraint;
- is expensive or risky to reverse;
- deliberately supersedes an earlier decision.

Before making such a change, read the relevant ADRs.

Do not silently contradict an accepted ADR. Supersede it with a new ADR and mark the old one as superseded.

Do not create ADRs for routine implementation details or easily reversible local choices.

## Plan-Driven Development

Development is performed through plans stored in:

```text
plans/
```

Plan filenames use descriptive uppercase names, for example:

```text
SIMULATION_RUST_REPO_START.md
COLLISION_DETECTION.md
```

The active plan is identified by:

```text
plans/ACTIVE_PLAN.md
```

`ACTIVE_PLAN.md` must contain the filename of the currently active plan. Work on only one active plan unless explicitly instructed otherwise.

Before starting implementation:

1. Read `plans/ACTIVE_PLAN.md`.
2. Read the active plan.
3. Read relevant glossary entries, ADRs, specifications, and contracts.
4. Identify the first non-completed step.
5. Confirm that the step still matches the current repository state.

Do not implement work that is not covered by the active plan unless explicitly requested.

### Plan Steps

Each plan must be divided into ordered steps with an explicit status:

- `Pending` — work has not started.
- `In Progress` — the current step.
- `Done` — the step is complete and verified.

A step must represent one coherent, reviewable increment.

Avoid steps that are too small:

- creating directories only;
- adding one trivial declaration;
- making a mechanical change with no independently useful result.

Avoid steps that are too large:

- implementing an entire subsystem at once;
- combining several independently testable behaviours;
- mixing unrelated refactoring, infrastructure, and gameplay changes.

Prefer a step that delivers one meaningful vertical or architectural increment, including its required tests and documentation.

### TDD in Plan Steps

Any step that changes behaviour or adds code must be designed around TDD:

1. Define the required observable behaviour.
2. Add the minimum failing tests or executable scenarios needed to prove it.
3. Implement the smallest complete solution.
4. Refactor only when necessary.
5. Run all relevant checks.

Tests must focus on behaviour and meaningful contracts.

Do not add tests merely to increase coverage. Avoid trivial tests such as:

- checking enum values without behavioural significance;
- checking barrel-file exports;
- testing language or framework behaviour;
- testing getters, constants, or declarations with no meaningful logic;
- duplicating the same behaviour across multiple test levels without a reason.

Every bug fix must include a test or scenario that reproduces the bug.

### Definition of Done

Every plan step must contain a `DoD` section with concrete, verifiable completion conditions.

Example:

```md
### DoD

- The command is rejected for destroyed modules.
- A behavioural test covers the rejection.
- Relevant tests, formatting, and linting pass.
- The command contract is updated.
```

A step is not complete until every applicable DoD item has been verified.

After completing the work, insert a `Done` section directly after `DoD`:

```md
### Done

- Added destroyed-module validation in `validate_fire_command`.
- Added the `rejects_destroyed_weapon` test.
- Updated the command contract.
- Ran formatting, linting, and relevant tests successfully.
```

The `Done` section must describe what was actually implemented, not repeat planned intentions.

Only after adding the `Done` section and verifying the DoD may the step status be changed to `Done`.

### Plan Maintenance

After completing each step:

1. Review the next three non-completed steps.
2. Compare them with the actual implementation.
3. Update names, paths, symbols, assumptions, dependencies, and expected outputs where necessary.
4. Split or merge future steps if their scope is no longer appropriate.
5. Preserve the original intended behaviour unless a deliberate decision changes it.

Prefer updating future plan references to match a sound implementation rather than changing working code merely to match stale planned names.

Do not silently rewrite completed steps. Their `Done` sections are historical records of what was implemented.

## Working Method

Before editing:

1. Read the nearest `AGENTS.md`.
2. Read the active plan and relevant README files, glossary entries, ADRs, specifications, and contracts.
3. Use Serena semantic navigation to understand symbols and call paths before broad text searches.
4. Identify the affected input → pipeline → state change → output flow.
5. Mark the active step as `In Progress`.

For behavioural changes:

1. Define or update the expected scenario or contract.
2. Add a failing automated test or executable scenario.
3. Implement the smallest complete change.
4. Run the relevant formatting, linting, tests, and scenarios.
5. Update documentation when behaviour, contracts, terminology, or pipeline stages change.
6. Complete the step according to the plan workflow.

## Change Discipline

- Keep changes narrow and task-focused.
- Do not perform unrelated refactoring or formatting.
- Do not invent gameplay rules silently. Use only minimal, reversible assumptions and report them.
- Prefer explicit, readable pipelines over clever or speculative abstractions.
- Avoid hidden mutation, global mutable state, and implicit cross-phase side effects.
- Do not change public contracts, data formats, dependencies, architectural boundaries, or accepted decisions without explaining the reason.
- Do not edit generated files, vendored code, or imported Unity assets unless the task explicitly requires it.
- Preserve existing user changes.
- Never commit secrets.
- Do not commit, push, rewrite Git history, or run destructive Git commands unless explicitly requested.

## Language and Naming

- Code, identifiers, comments, documentation, plans, ADRs, and commit messages are written in English.
- Use terminology from `docs/GLOSSARY.md`.
- Keep documentation, plans, ADRs, tests, and code references aligned with actual symbols and paths.

## Completion Report

When finishing a task, report:

- What changed.
- Which files changed.
- Which plan step was completed.
- Which checks were run and their results.
- Whether the next three pending steps were reviewed and updated.
- Any assumptions, unresolved questions, or known risks.

If a required check cannot be run, state exactly why.
