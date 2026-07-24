# Iron Doctrine — Product Vision

## Product

Iron Doctrine is a session-based team strategy game about modular combat robots.

The game is designed primarily for men aged 25–55 who want meaningful tactical decisions without the constant micro-management expected by traditional real-time strategy games.

The player commands groups of autonomous robots rather than individual units. Direct control of a single robot is not supported.

## Core Player Role

The player acts as the command intelligence of a private military corporation.

The player's primary responsibilities are:

- assembling and developing robot forces;
- dividing robots into combat groups;
- selecting equipment, technologies, and doctrine;
- issuing high-level intent orders;
- reacting to changes in the battlefield;
- analysing completed battles and improving future decisions.

Execution belongs to the robots and their commanders. The game is about directing a combat organisation, not manually controlling every machine.

## Match Structure

The primary target format is a 5-versus-5 team match.

Additional supported formats may include:

- 3 versus 3;
- 4 versus 4.

Each player controls:

- from 1 to 3 groups;
- no more than 30 robots in total.

Matches are intended to be relatively short sessions. The current target is approximately 8–12 minutes, but exact duration and pacing remain subject to playtesting.

## Group-Level Command

Orders are issued to groups, not individual robots.

A group order describes intent and doctrine rather than a sequence of low-level actions. Examples include:

- move as quickly as possible;
- preserve formation and move at the speed of the slowest robot;
- perform reconnaissance by combat;
- assault through resistance;
- occupy and hold a position;
- retreat or regroup.

Robots must remain useful without constant player input. A player should be able to briefly look away from the game without the controlled force immediately becoming helpless.

Autonomy must not remove meaningful decisions. The player's choices of composition, doctrine, objectives, and timing must remain decisive.

## Modular Robots

Robots are assembled from compatible modules.

Their visual form, available module categories, weapons, chassis, bodies, and other content will depend substantially on the commercial Unity asset packs selected for the project.

The project does not commit in advance to a fixed catalogue of robot parts. Available asset packs will be evaluated, purchased selectively, and turned into functional game content.

Visual assets do not define gameplay rules by themselves. Each supported asset must be integrated into the authoritative simulation through explicit module, socket, collision, animation, and gameplay definitions.

## Command, Communication, and Electronic Warfare

Command transmission is intended to be an explicit gameplay system rather than an invisible abstraction.

Possible mechanics include:

- command propagation delays;
- incomplete or distorted commands;
- communication range and quality;
- enemy jamming;
- electronic warfare;
- technologies that improve command reliability and autonomy.

These mechanics should explain why robots operate in groups, why commands express intent, and why execution may not always be immediate or perfect.

Exact communication mechanics remain subject to design and playtesting.

## Artificial Intelligence

Iron Doctrine includes several distinct forms of artificial intelligence.

### Built-In Game AI

Conventional game AI is required to:

- fill empty player slots;
- reduce matchmaking wait times;
- replace disconnected or absent players;
- support single-player and cooperative modes;
- provide appropriate opposition during the early life of the game.

Bots must be competent without consistently overwhelming ordinary players. Their strength, available information, reaction frequency, and decision complexity must be controllable.

### LLM Commander

A player may connect an LLM-based commander that can play while the player is away.

The commander may:

- inspect only the game state legitimately available to that player;
- issue the same group-level commands available to a human player;
- continue account progression within defined limits;
- accumulate some experience toward technology development.

A practical use case is allowing a commander to participate during the week so that the player returns by Friday with additional progress.

LLM command is a legal, visible game mechanic. It is not treated as cheating or hidden automation.

LLM commanders must not receive unrestricted internal simulation state, unlimited reaction speed, or superhuman control frequency. Their API must preserve fair play and the intended pace of the game.

### AI-Controlled Public Players

The game may contain complete public player accounts controlled by well-known AI systems, including systems associated with ChatGPT and Claude.

These players exist partly to create persistent public narratives and community events, for example:

- one AI commander achieving a long winning streak against another;
- competing claims that one model consistently outperforms another;
- organised rematches under changed conditions;
- community attempts to analyse and overturn an AI victory.

Such matches must still follow normal authoritative rules and must not be scripted to produce predetermined winners.

## Battle as an Artifact

Every battle is a persistent, reproducible artifact rather than a disposable match result.

A battle artifact contains enough authoritative information to:

- replay the complete battle;
- inspect commands and significant events;
- seek to an arbitrary point;
- explain important outcomes;
- create a new branch from a selected point;
- compare the original and alternative outcomes;
- export a presentation suitable for video sharing.

All authoritative randomness must derive from controlled seeds. The same initial state, configuration, ordered commands, and seed must reproduce the same battle result.

## Replay and Branching

A replay is an interactive part of the game.

A player should be able to:

1. open a completed battle;
2. select a meaningful point in time;
3. take control from that point;
4. replace later commands with new decisions;
5. let built-in AI control missing participants;
6. produce a new battle artifact as an alternative history.

This supports arguments, tactical study, experimentation, and questions such as:

- Could this position have been saved?
- Was the defeat caused by composition, doctrine, or timing?
- Could another player have executed the same situation better?
- Was an AI commander's decision actually optimal?

The project should support replay seeking through snapshots or checkpoints rather than requiring every battle to be simulated from its first tick whenever it is opened.

## Experience from Battles

Experience may be awarded for:

- normal live participation;
- replaying a battle from a selected point;
- watching and analysing another battle.

The intended reward order is:

1. live participation gives the most experience;
2. replaying and branching gives less;
3. passive viewing gives the least.

Replay rewards should make analysis feel valuable without making passive progression more efficient than playing the game.

Within the game world, this can be explained as a command intelligence learning from recorded combat experience.

Exact values, limits, anti-abuse rules, and technology progression remain subject to design and balancing.

## Video and Distribution

Battle artifacts should be suitable for export into shareable video.

A future export tool may reproduce:

- the camera position used by a selected player;
- relevant interface information;
- key tactical events;
- an overview or observer camera;
- selected parts of a replay branch.

Replay-derived videos are expected to be an important organic distribution channel for the game.

## Factions and Corporations

Private military corporations may function as factions with different:

- technologies;
- visual identities;
- doctrines;
- command styles;
- strategic strengths and weaknesses.

The exact number and structure of corporations are not fixed yet.

## Technology Development

Players develop technology trees that may improve areas such as:

- robot modules;
- command reliability;
- autonomy;
- sensors;
- electronic warfare;
- group doctrine;
- LLM commander capabilities.

Technology choices should create different strategic identities rather than only linear numerical growth.

The exact trees, currencies, progression speed, and account economy are not yet fixed.

## Product and Technical Evolution

The first releasable game is expected to be a desktop single-player product.

It must use the same authoritative Rust simulation core intended for later multiplayer execution.

The Unity client is responsible for:

- rendering;
- animation;
- audio;
- UI;
- input;
- camera;
- visual effects;
- authoring and integrating purchased assets.

The Rust simulation is responsible for authoritative gameplay outcomes, including:

- movement;
- targeting;
- collision and hit resolution;
- damage;
- module state;
- random combat outcomes;
- victory conditions;
- replayable match state.

The simulation must remain headless and independent of Unity so it can later run:

- inside the desktop single-player game;
- in automated tests and scenarios;
- in replay and analysis tools;
- on authoritative multiplayer servers.

The transition from single-player to multiplayer must reuse the same gameplay rules rather than reimplementing them.

## Design Principles

Iron Doctrine follows these principles:

- Command groups, not individual robots.
- Prefer meaningful decisions over mechanical click speed.
- Make robot autonomy useful but not strategically self-sufficient.
- Treat AI and LLM participation as explicit game mechanics.
- Make every battle reproducible, inspectable, and reusable.
- Turn replay analysis and alternative outcomes into gameplay.
- Keep live participation more valuable than passive progression.
- Use purchased visual assets pragmatically without letting them define architecture.
- Build the authoritative simulation once for single-player, replay, AI, and multiplayer use.
- Prefer explainable and traceable outcomes over opaque simulation behaviour.

## Not Yet Fixed

The following remain hypotheses or later design decisions:

- exact match duration and phase timings;
- final victory conditions;
- exact experience values and progression speed;
- monetisation and distribution model;
- number and identity of corporations;
- final technology trees;
- precise communication and electronic-warfare rules;
- final LLM commander limitations;
- exact match formats beyond the primary formats;
- detailed module catalogue;
- specific robot, weapon, chassis, and body content;
- final set of game modes.

These decisions must be validated through plans, prototypes, executable scenarios, and playtesting rather than treated as established facts.
