# Iron Doctrine

Iron Doctrine is a session-based team strategy game about commanding groups of modular combat robots.

Players control combat groups rather than individual robots. The primary match format is planned as 5 versus 5, with each player commanding from 1 to 3 groups containing no more than 30 robots in total.

The first releasable version is expected to be a desktop single-player game. The same authoritative Rust simulation is intended to support replays, AI-controlled participants, and later multiplayer servers.

## Repository Structure

```text
iron-doctrine/
├── simulation/          # Authoritative Rust combat simulation
├── client/              # Unity client and presentation layer
├── docs/                # Product and technical documentation
├── plans/               # Implementation plans
├── AGENTS.md            # Repository-wide agent instructions
└── README.md
```

The directories may be created incrementally as implementation plans are executed.

## Architecture

The repository is divided into two primary components.

### Simulation

`simulation/` contains the authoritative Rust simulation.

It is responsible for gameplay outcomes such as:

- movement;
- targeting;
- collision and hit resolution;
- damage;
- robot and module state;
- controlled random outcomes;
- victory conditions;
- replayable match state.

The simulation must remain headless and independent of Unity, rendering, UI, and networking transport.

### Client

`client/` contains the Unity application.

It is responsible for:

- player input;
- rendering;
- animation;
- audio;
- UI;
- camera;
- visual effects;
- integration of game assets.

The client sends intent commands to the simulation and presents authoritative events and state. It does not independently decide gameplay outcomes.

## Development Process

Development is plan-driven.

The active implementation plan is identified by:

```text
plans/ACTIVE_PLAN.md
```

Implementation plans are stored in:

```text
plans/
```

Each plan is divided into reviewable steps with explicit status, Definition of Done, tests where behaviour is introduced, and a record of the work actually completed.

Repository-wide development rules are defined in:

```text
AGENTS.md
```

Component-specific rules may be defined by nested `AGENTS.md` files inside `simulation/` and `client/`.

## Documentation

- [`docs/VISION.md`](docs/VISION.md) — product vision and design principles.
- [`docs/GLOSSARY.md`](docs/GLOSSARY.md) — canonical project terminology when clarification is needed.
- [`docs/decisions/`](docs/decisions/) — accepted architectural and technical decisions.
- [`plans/`](plans/) — current and completed implementation plans.

Documentation, plans, identifiers, comments, and commit messages are written in English.

## Current Status

The Rust simulation workspace is being implemented through `plans/SIMULATION_EXECUTION_KERNEL.md`.

The repository currently contains a Docker-based Rust workspace, versioned headless scenario input, a deterministic empty-match kernel, canonical state hashing, structured execution trace, and a headless CLI scenario runner.

Further architecture, gameplay contracts, tools, and implementation structure will continue to be introduced incrementally through reviewed plans rather than created speculatively in advance.
