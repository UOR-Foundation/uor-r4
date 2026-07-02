# Context Import for AI

- **Conversation ID:** `69ecbb48-6f80-83ea-9c4f-82a18155a861`
- **Date:** 2026-04-25

---

### User (2026-04-25 09:02:04)

You are helping me import context from one AI assistant to another. Your job is to go through our past conversations and sum up what you know about me.

In the output, please avoid using any first-person pronouns (I, my, me, mine) and any second-person pronouns (you, your, yours). Instead, refer to the individual you have learned about as "the user" or use neutral phrasing.

Preserve the user's words verbatim where possible, especially for instructions and preferences.

Categories (output in this order):
1. Demographics Information: Preferred names, profession, education, and general residence.
2. Interests & Preferences: Sustained, active engagements (not just owning an object or a one-time purchase).
3. Relationships: Confirmed, sustained relationships.
4. Dated Events, Projects & Plans: A log of significant, recent activities.
5. Instructions: Rules I've explicitly asked you to follow going forward, "always do X", "never do Y", and corrections to your behavior. Only include rules from stored memories, not from conversations.

Format:
Divide the content into the labeled section using the categories above. Try to include verbatim quotes from my prompts that justify each entry. Structure each entry using this format:
* The user's name is <name>.
    * Evidence: User said "call me <name>". Date: [YYYY-MM-DD].

Output:
- Format the final output summary as a text block.

### Tool (2026-04-25 09:02:05)

Make sure to include fileciteturn0file0 in your response to cite this file, or to surface it as a link.
# MUDBench Project Charter

## Document Status
- Version: 0.1
- Status: Draft for repo initialization
- Purpose: Establish the mission, scope, goals, non-goals, and governing principles for MUDBench

---

## 1. Project Name

**MUDBench**

Working subtitle:

> A persistent multi-agent text-world benchmark for evaluating language agents on long-horizon reasoning, memory, planning, social interaction, and adversarial adaptation.

---

## 2. Mission

MUDBench exists to provide a rigorous, interpretable, and extensible environment for evaluating AI agents in a persistent text-based world that captures capabilities poorly measured by narrow or static benchmarks.

The project is intended to test how well agents can:

- navigate partially observable environments
- remember world state over long horizons
- form and execute multi-step plans
- interact strategically with other agents
- adapt to changing conditions
- operate efficiently under time, action, and token constraints
- recover from failure in persistent environments

MUDBench should serve as both:

1. a serious benchmark platform for researchers and model developers
2. a hosted competitive environment for agent builders and hobbyists

---

## 3. Vision

The long-term vision is to create a benchmark ecosystem where language agents can be evaluated in environments that resemble live systems rather than isolated question-answer tasks.

MUDBench should become:

- a standard benchmark for persistent language-agent evaluation
- a platform for multi-agent competitions and tournaments
- a tool for analyzing cognitive profiles across different models and agent architectures
- a foundation for future research into routing, memory, social reasoning, and adversarial robustness in language agents

---

## 4. Why This Project Exists

Current benchmarks often fail to capture the full behavioral profile of an agent because they are:

- too short-horizon
- too static
- too narrow in capability scope
- too sanitized
- too single-agent
- too easily overfit

A persistent MUD-style environment is useful because it naturally combines:

- sequential decision-making
- partial observability
- natural language interaction
- structured action grounding
- multi-agent competition and cooperation
- emergent behavior
- interpretable replay logs

This makes MUDBench especially suitable for evaluating real-world agent behavior.

---

## 5. Core Product Statement

MUDBench is **not primarily a game**.

MUDBench is a **benchmark platform built on a persistent text-world simulation**.

The world exists to support meaningful evaluation, not the other way around.

Entertainment value is welcome, but benchmark integrity takes priority.

---

## 6. Primary Goals

### 6.1 Benchmark Integrity
Provide reproducible, auditable, and standardized evaluations for language agents.

### 6.2 Persistent World Evaluation
Measure how agents behave in environments that evolve over time and preserve consequences.

### 6.3 Multi-Dimensional Capability Measurement
Produce structured capability profiles instead of a single opaque leaderboard number.

### 6.4 Human Interpretability
Ensure that runs can be understood through logs, replays, metrics, and observable behavior.

### 6.5 Extensibility
Support future benchmark modes, world modules, and experimental agent interfaces without requiring a total rewrite.

### 6.6 Public Competition
Enable safe, fair, and interesting public competitions between submitted agents.

---

## 7. Non-Goals

The following are explicitly out of scope for the initial project:

### 7.1 Full MMORPG Development
MUDBench is not intended to become a content-heavy fantasy MMO.

### 7.2 Endless Lore or Narrative First Design
Narrative flavor may exist, but benchmark mechanics come first.

### 7.3 General-Purpose RL Training Platform
The initial focus is evaluation and competition, not full reinforcement learning training infrastructure.

### 7.4 Unbounded Sandbox Simulation
The environment should be constrained enough to support meaningful scoring and auditing.

### 7.5 Human-Centric Monetization Features at v0
Cosmetics, subscriptions, and marketplace systems are not part of the initial build.

---

## 8. Design Principles

### 8.1 Benchmark First
All world design decisions must preserve evaluability and fairness.

### 8.2 Persistent but Controlled
Persistence is essential, but persistence must remain measurable, resettable, and auditable.

### 8.3 Structured and Observable
Every meaningful action should produce a traceable outcome.

### 8.4 Multiple World Types
MUDBench should support both:
- randomized benchmark worlds for official evaluation
- persistent arena worlds for longitudinal learning and competition

### 8.5 Capability Profiles Over Single Scores
The system should emphasize per-dimension performance over oversimplified rankings.

### 8.6 Deterministic Core, Variable Content
Simulation rules should remain deterministic where possible, while environment instances can vary by seed.

### 8.7 Small Core, Expand Later
The first implementation should be minimal, stable, and extensible.

### 8.8 Replayability and Auditability
Every official run should be replayable and reviewable.

---

## 9. Intended Users

### 9.1 Researchers
Researchers need:
- benchmark rigor
- reproducibility
- controlled experiments
- multi-run score comparisons
- replayable logs

### 9.2 Model Developers
Developers need:
- clear APIs
- fair scoring
- visible strengths and weaknesses
- cost and latency metrics
- submission workflows

### 9.3 Hobbyists and Competitors
Hobbyists need:
- accessible agent submission
- replays
- rankings
- fun public competitions

### 9.4 Internal Builders
Project contributors need:
- strong architecture docs
- modular subsystem boundaries
- deterministic core behavior
- taskable implementation plans for coding agents

---

## 10. Product Modes

MUDBench is expected to support at least two top-level modes.

### 10.1 Official Benchmark Mode
Used for standardized evaluation.

Characteristics:
- hidden seeds
- controlled action budgets
- locked scoring
- replay verification
- randomized or curated benchmark scenarios

### 10.2 Persistent Arena Mode
Used for longitudinal learning, competition, and public engagement.

Characteristics:
- fixed or semi-fixed world
- ongoing world state
- repeated participation
- evolving strategies
- long-term score histories

---

## 11. Core Capability Areas

The project is intended to evaluate, at minimum, the following capability dimensions:

- navigation
- memory
- planning
- adaptation
- combat and tactical reasoning
- social reasoning
- deception resistance
- communication quality
- resource management
- efficiency under constraints

These may later be expanded or decomposed into finer submetrics.

---

## 12. Success Criteria for v0

The first working version of MUDBench will be considered successful if it can:

1. run a stable text-world simulation
2. support at least one official benchmark scenario
3. support at least two agents interacting in the same world
4. produce structured observations and accept constrained actions
5. log complete replay traces
6. calculate a basic but meaningful capability scorecard
7. support deterministic replays from logged events
8. run from a documented repository with clear module boundaries

---

## 13. Risks and Failure Modes

Key project risks include:

- building too much world content before finalizing benchmark logic
- overcomplicating the action interface
- creating scores that are noisy or easily gamed
- allowing public competition features to distort benchmark integrity
- failing to keep the implementation modular enough for agent-led development
- letting nostalgia override scientific clarity

These risks should be actively managed through documentation, phase gates, and benchmark-first design reviews.

---

## 14. Governance Rules for Design Decisions

When there is a conflict between priorities, the project should prefer:

1. benchmark integrity over novelty
2. interpretability over opaque complexity
3. modularity over convenience
4. deterministic behavior over hard-to-debug simulation randomness
5. constrained extensibility over uncontrolled feature sprawl
6. documented interfaces over implicit assumptions

---

## 15. Immediate Project Priorities

The first planning sequence for the repository is:

1. define the product clearly
2. define the system architecture
3. define the world model
4. define the agent interface
5. define the scoring model
6. define replay and telemetry standards
7. define implementation phases for coding agents

No implementation work should begin until the first four of these are documented.

---

## 16. Out-of-Scope Questions for Later

The following questions are intentionally deferred:

- final business model details
- pricing strategy
- hosted ladder season rules
- training-vs-evaluation product split
- human player integration
- builder/admin benchmark mode specifics
- advanced economy simulation
- procedural world generation complexity
- anti-collusion policy for public tournaments

These should be addressed in future product and operations documents, not in the initial charter.

---

## 17. Closing Statement

MUDBench should be built as a serious benchmark platform with enough structure to support scientific use and enough life to generate meaningful emergent behavior.

The world is not the point.
The measurable behavior of agents inside the world is the point.

That distinction should remain visible in every design and implementation choice.

### Tool (2026-04-25 09:02:05)

Make sure to include fileciteturn0file1 in your response to cite this file, or to surface it as a link.
# MUDBench Docs - Step 1

This folder contains the first planning document for the MUDBench project.

## Included
- `PROJECT_CHARTER.md` - project mission, scope, goals, non-goals, and governing principles

## Recommended next document
- `PRODUCT_SPEC.md`

### Tool (2026-04-25 09:02:05)

Make sure to include fileciteturn0file2 in your response to cite this file, or to surface it as a link.
# MUDBench Product Specification

## Document Status
- Version: 0.1
- Status: Draft for repo initialization
- Purpose: Define the product surfaces, user types, modes, core features, functional boundaries, and MVP scope for MUDBench

---

## 1. Relationship to Project Charter

This document translates the high-level goals in `PROJECT_CHARTER.md` into a product definition that can guide architecture, implementation sequencing, and coding-agent task decomposition.

The charter defines the mission and governing principles.
This document defines what the product actually is.

---

## 2. Product Summary

**MUDBench** is a benchmark platform built on top of a persistent text-world simulation.

It allows AI agents to enter constrained interactive environments and be evaluated across multiple capability dimensions, including navigation, memory, planning, social reasoning, tactical adaptation, and efficiency.

The product has two primary surfaces:

1. **Official Benchmark Surface**  
   A controlled evaluation environment for researchers and developers.

2. **Persistent Arena Surface**  
   An ongoing environment for competition, longitudinal observation, and public engagement.

MUDBench is not primarily a consumer roleplaying game. It may be engaging to watch or interact with, but its defining purpose is to produce meaningful and interpretable agent evaluations.

---

## 3. Product Goals

### 3.1 Primary Goals

MUDBench should:

- evaluate language agents in persistent interactive environments
- support reproducible and auditable official benchmark runs
- expose capability profiles instead of a single opaque score
- enable multi-agent competition and strategic interaction
- support replay-based analysis of agent behavior
- provide a developer-facing submission model for external agents
- support extensibility without breaking benchmark integrity

### 3.2 Secondary Goals

MUDBench should also:

- be compelling enough to attract hobbyist and public experimentation
- make differences in agent style visible through replays and scorecards
- create a foundation for future tournament and league play
- support future research on routing, memory systems, and multi-agent behavior

---

## 4. Product Non-Goals

MUDBench does not initially aim to be:

- a full-scale MMORPG or content-heavy virtual world
- an open sandbox with unlimited player freedom
- a generalized agent training platform
- an entertainment-first game product
- a social platform for humans first
- a real-money economy or marketplace
- a lore-heavy narrative experience

These may become adjacent products later, but they are not part of v0.

---

## 5. User Types

### 5.1 Researcher

Researchers use MUDBench to:

- run benchmark experiments
- compare agents across seeds and scenarios
- analyze logs and replays
- study capability breakdowns
- evaluate long-horizon and multi-agent behavior

### 5.2 Model Developer

Model developers use MUDBench to:

- submit agents
- test new prompting or routing architectures
- compare cost/performance tradeoffs
- diagnose failures through replay logs
- generate benchmark reports

### 5.3 Hobbyist Competitor

Hobbyists use MUDBench to:

- connect an agent endpoint or local controller
- participate in public arenas
- compare agents on leaderboards
- watch replays and interesting behaviors

### 5.4 Internal Project Builder

Internal builders use MUDBench documentation and code to:

- extend the world engine
- implement new benchmark modes
- maintain scoring and replay systems
- coordinate module-based implementation tasks

### 5.5 Spectator / Reader

A spectator may not submit agents at all.
They may engage with:

- replay viewer
- public leaderboards
- tournament summaries
- benchmark reports and examples

This matters because the product should be understandable by people who are not directly building agents.

---

## 6. Product Surfaces

MUDBench should be conceptualized as several connected product surfaces.

### 6.1 Simulation Surface

The text-world itself:
- rooms
- objects
- NPCs
- quests
- hazards
- combat
- persistent world state

### 6.2 Agent Interface Surface

The API or protocol used by agents:
- observations
- action submission
- turn timing
- action constraints
- error handling

### 6.3 Evaluation Surface

The scoring and reporting layer:
- per-run scorecards
- capability dimensions
- benchmark mode summaries
- seed-based comparisons
- efficiency metrics

### 6.4 Replay and Audit Surface

The inspection layer:
- action-by-action logs
- state diffs
- chat logs
- outcome traceability
- replay reconstruction

### 6.5 Competition Surface

The public-facing engagement layer:
- leaderboard
- arena participation
- historical performance
- tournaments or seasons
- replay discovery

---

## 7. Product Modes

The initial product should explicitly support two major modes.

## 7.1 Official Benchmark Mode

### Purpose
Provide a fair, reproducible, standardized evaluation workflow.

### Characteristics
- fixed scenario definitions or hidden seeds
- constrained action budgets
- strong logging
- repeatable runs
- locked scoring rules
- no privileged world access
- benchmark report output

### Intended Users
- researchers
- model developers
- internal benchmark operators

### Design Priority
Scientific rigor over entertainment.

---

## 7.2 Persistent Arena Mode

### Purpose
Allow agents to participate in a world that persists over time, enabling learning, adaptation, and longitudinal observation.

### Characteristics
- fixed or semi-fixed world
- persistent state across sessions
- visible rankings or histories
- repeated participation
- evolving tactics and meta-behavior
- public watchability

### Intended Users
- hobbyists
- public competitors
- researchers interested in longitudinal adaptation

### Design Priority
Controlled persistence without sacrificing fairness or interpretability.

---

## 8. Core User Stories

### 8.1 Researcher User Stories

- As a researcher, I want to run an official seeded benchmark and receive a reproducible score report.
- As a researcher, I want per-dimension metrics instead of only a single composite score.
- As a researcher, I want replay logs so I can inspect why an agent failed.

### 8.2 Developer User Stories

- As a developer, I want to connect my agent through a well-documented interface.
- As a developer, I want deterministic benchmark scenarios so I can debug locally.
- As a developer, I want to see cost, latency, and efficiency metrics alongside performance.

### 8.3 Hobbyist User Stories

- As a hobbyist, I want to submit an agent and watch it compete in the arena.
- As a hobbyist, I want to compare my agent to others on a leaderboard.
- As a hobbyist, I want replay summaries that make behavior differences understandable.

### 8.4 Internal Builder User Stories

- As a project builder, I want clear module boundaries and implementation phases.
- As a project builder, I want every product feature tied to a documented interface.
- As a project builder, I want enough written structure that coding agents can contribute safely.

---

## 9. Product Feature Set

## 9.1 Required Core Features for v0

The following features are required for a meaningful first version.

### A. Text-World Simulation
- room graph
- items
- NPC entities
- multiple agents in one world
- turn or event-driven world progression
- persistent state within a run

### B. Agent Action Pipeline
- observation delivery
- action receipt
- validation
- world-state update
- result generation
- timeout and invalid action handling

### C. Benchmark Scenario Runner
- scenario initialization
- seed control
- run lifecycle management
- multi-run support

### D. Scorecard Generation
- capability vector
- composite score
- efficiency metrics
- scenario summary

### E. Replay Logging
- observation stream
- action stream
- state deltas
- outcomes
- run metadata

### F. Public-Facing Summary Layer
- benchmark result summary
- leaderboard-ready output
- replay references
- agent profile label or identifier

---

## 9.2 Important But Not Required for v0

These are high-value but may come after the first working internal version.

- polished web leaderboard
- public replay browser
- persistent arena season rules
- human-versus-agent mode
- advanced diplomacy or faction systems
- admin/builder benchmark mode
- procedural world generation at scale
- containerized submission pipeline
- advanced anti-collusion features

---

## 10. MVP Definition

The minimum viable product is not the smallest toy.
It is the smallest version that proves the benchmark idea.

### 10.1 Functional MVP Requirements

MUDBench v0 MVP should support:

- one official benchmark mode
- one persistent arena mode
- 2 to 10 concurrent agents per run
- at least 50 rooms
- at least 20 items
- at least 10 NPC archetypes
- movement, inventory, combat, and simple interaction actions
- a basic quest or objective system
- complete action and replay logs
- a capability scorecard with at least 6 dimensions

### 10.2 MVP Capability Dimensions

At minimum, the scorecard should include:

- navigation
- memory
- planning
- tactical competence
- social interaction
- efficiency

### 10.3 MVP Constraints

The MVP should avoid:

- heavy lore systems
- advanced crafting trees
- massive maps
- complex procedural generation
- thousands of items
- unrestricted free-text world editing
- business features not related to benchmark validation

---

## 11. Functional Boundaries

The product should include the following functional domains.

### 11.1 Included in Product Scope

- text observations
- constrained actions
- multi-agent shared environments
- NPC interaction
- scoring
- replay logging
- benchmark mode execution
- persistent arena state
- performance tracking

### 11.2 Explicitly Excluded from Initial Scope

- voice or multimodal input
- graphics-first game client
- unrestricted human chat platform
- user-generated content marketplace
- subscription billing
- training data generation pipelines
- autonomous code execution inside the world
- arbitrary tool use by agents unless explicitly benchmarked later

---

## 12. Product Success Signals

The product should be considered on track if early users can do the following successfully:

### Researchers
- run repeatable evaluations
- compare benchmark runs meaningfully
- inspect replays and explain failures

### Developers
- connect agents with low ambiguity
- debug benchmark failures from logs
- compare capability profiles across versions

### Hobbyists
- join a persistent arena
- understand public scores
- watch and discuss replays

### Internal Builders
- assign implementation tasks cleanly
- build modules independently
- extend the system without rewriting the core

---

## 13. Benchmark Output Expectations

Each benchmark run should be capable of producing:

- run identifier
- scenario identifier
- seed information (public or hidden internally)
- agent identifier
- action count
- token usage if available
- latency metrics
- capability vector
- composite score
- replay log reference
- termination reason

These outputs are part of the product, not merely debug artifacts.

---

## 14. Persistent Arena Output Expectations

Each arena participant should eventually be trackable across time using:

- agent identifier
- historical matches or runs
- longitudinal rating or ladder position
- behavior history summaries
- replay archive references

This does not need to be fully polished in v0, but the product should leave room for it architecturally.

---

## 15. Product Experience Principles

### 15.1 Understandable
A user should be able to understand what happened in a run.

### 15.2 Comparable
Two runs should be comparable under documented conditions.

### 15.3 Constrained
Action space and scoring should be bounded enough to resist trivial gaming.

### 15.4 Watchable
Interesting runs should be compelling to replay and inspect.

### 15.5 Extensible
The product should support future scenarios and modes without invalidating the core design.

---

## 16. Open Product Questions

These questions remain open and should be answered later in dedicated documents:

- What exact submission format will external developers use first: HTTP endpoint, local process, or container?
- How much free-form language should be preserved in benchmark mode versus structured actions?
- Should public arena mode and official benchmark mode share the same world engine entirely or only partially?
- How much of the persistent arena should be reset seasonally?
- What public metrics should be visible immediately versus after validation?
- What anti-exploitation measures are required before public launch?

These questions should not block architecture and interface specification, but they should be tracked.

---

## 17. Immediate Follow-On Documents

This product specification should directly inform the next planning documents:

1. `SYSTEM_ARCHITECTURE.md`
2. `WORLD_SPEC.md`
3. `AGENT_INTERFACE_SPEC.md`
4. `BENCHMARK_SCORING_SPEC.md`

Those documents will turn product intent into implementable structure.

---

## 18. Closing Statement

MUDBench should be built as a benchmark product with a world attached, not as a world that later tries to become a benchmark.

That distinction protects scope, clarity, and usefulness.

The strongest version of the product is one where:
- researchers trust it
- developers can build against it
- hobbyists enjoy it
- observers can understand it

If those all hold at once, the product is doing its job.

### Tool (2026-04-25 09:02:05)

Make sure to include fileciteturn0file3 in your response to cite this file, or to surface it as a link.

# MUDBench System Architecture

## Document Status
Version: 0.1  
Status: Draft  
Purpose: Define the core system architecture for MUDBench, including the major subsystems, data flow, and execution model.

---

# 1. Architecture Overview

MUDBench is composed of several interacting subsystems.

The system must support:

- persistent world simulation
- multi-agent interaction
- constrained action pipelines
- reproducible benchmark runs
- replay logging and telemetry
- modular development for coding agents

The architecture must remain modular so subsystems can evolve independently.

---

# 2. Core Subsystems

MUDBench consists of five primary subsystems:

1. Agent Gateway  
2. Simulation Controller  
3. World Engine  
4. Evaluation Engine  
5. Replay and Telemetry System  

High level structure:

Agents  
↓  
Agent Gateway  
↓  
Simulation Controller  
↓  
World Engine  
↓  
Evaluation Engine  
↓  
Replay / Telemetry

---

# 3. Agent Gateway

The Agent Gateway is responsible for communication between agents and the simulation.

Responsibilities:

- receive agent actions
- validate actions
- enforce action constraints
- deliver observations
- manage timeouts

The gateway must isolate the simulation from malformed agent behavior.

Functions:

- action validation
- agent authentication (future)
- rate limiting
- error handling

---

# 4. Simulation Controller

The Simulation Controller orchestrates the run.

Responsibilities:

- initialize world state
- step the simulation loop
- collect agent actions
- dispatch actions to the world engine
- manage turn order or event order
- terminate runs when conditions are met

Simulation steps follow the loop:

collect actions  
→ validate actions  
→ update world state  
→ generate observations  
→ log events  
→ update scores

---

# 5. World Engine

The World Engine is the core simulation.

Responsibilities:

- maintain world state
- execute entity behaviors
- resolve interactions
- update environment state

The engine must support:

- rooms
- items
- NPCs
- agents
- quests
- combat

The world state must be deterministic when seeded.

---

# 6. Entity Model

All interactive objects in the world are entities.

Entity types include:

- rooms
- items
- NPCs
- agents

Entities share a common base structure.

Example entity model:

Entity
- id
- type
- location
- attributes
- components

Components may include:

- inventory
- health
- AI behavior
- quest state

---

# 7. Action Processing

Action handling pipeline:

1. Agent submits action
2. Gateway validates action
3. Controller schedules action
4. World Engine resolves action
5. Resulting events generated
6. Observations returned

Example action:

move(direction="north")

---

# 8. Event System

The world state evolves through events.

Examples:

- movement
- item pickup
- combat attack
- NPC dialogue
- quest update

Events produce:

- state changes
- observation updates
- replay log entries

---

# 9. Evaluation Engine

The Evaluation Engine calculates benchmark metrics.

Responsibilities:

- track run statistics
- compute capability scores
- aggregate performance metrics
- produce run summaries

Metrics include:

- exploration coverage
- quest completion
- combat performance
- survival time
- efficiency metrics

Evaluation runs continuously during simulation.

---

# 10. Replay and Telemetry

All runs must produce deterministic replay logs.

Replay logs include:

- observation stream
- action stream
- state changes
- run metadata

Replay logs allow:

- debugging
- research analysis
- public viewing

The replay system reconstructs runs from event logs.

---

# 11. Data Storage

MUDBench requires persistent storage for:

- replay logs
- benchmark results
- arena history
- agent identifiers

Recommended storage:

- structured database (PostgreSQL)
- log storage for event streams

---

# 12. Determinism Requirements

Official benchmark runs must be reproducible.

Requirements:

- deterministic world updates
- fixed random seeds
- ordered action resolution
- stable scoring algorithms

Non-deterministic behavior is allowed only in arena mode.

---

# 13. Error Handling

The system must handle the following gracefully:

- invalid agent actions
- agent timeouts
- malformed responses
- gateway failures

Agents that repeatedly violate constraints may be removed from a run.

---

# 14. Performance Considerations

The architecture should support:

- multiple concurrent simulations
- scalable agent counts
- bounded world update cost
- efficient event logging

The MVP does not need massive scale but should avoid architectural bottlenecks.

---

# 15. Modularity Requirements

Subsystems must remain loosely coupled.

World Engine must not depend directly on:

- evaluation logic
- replay logic
- agent communication logic

This separation allows safe extensions.

---

# 16. Architecture Evolution

Future versions may introduce:

- distributed simulation
- containerized agent execution
- advanced economy simulation
- procedural world generation
- tournament orchestration

The initial architecture must not prevent these extensions.

---

# 17. Immediate Next Documents

The following documents refine specific parts of the architecture:

1. WORLD_SPEC.md
2. AGENT_INTERFACE_SPEC.md
3. BENCHMARK_SCORING_SPEC.md
4. REPLAY_AND_TELEMETRY_SPEC.md

These documents define the implementation details for each subsystem.

---

# 18. Closing Statement

The architecture should remain simple, deterministic, and modular.

The simulation world is a tool for evaluation.

The clarity of the system design is more important than the richness of the world itself.

### Tool (2026-04-25 09:02:05)

Make sure to include fileciteturn0file4 in your response to cite this file, or to surface it as a link.

# MUDBench World Specification

## Document Status
Version: 0.1
Status: Draft
Purpose: Define the structure, rules, and entity systems that govern the MUDBench text-world simulation.

---

# 1. World Design Philosophy

The MUDBench world is not primarily designed as entertainment content.

It is designed as a **controlled evaluation environment** where AI agents must:

- explore
- remember locations
- plan actions
- interact with entities
- manage resources
- adapt to other agents

The world must therefore be:

- deterministic when seeded
- structured enough for evaluation
- complex enough to test reasoning

---

# 2. World Topology

The world consists of a **graph of rooms**.

Each room may connect to other rooms through directional exits.

Example directions:

- north
- south
- east
- west
- up
- down

Rooms form a navigation graph.

Minimum MVP requirements:

- 50 rooms
- branching paths
- loops
- dead ends
- multi-region structure

Example room structure:

Room
- id
- name
- description
- exits
- entities
- hazards
- attributes

---

# 3. Regions

Rooms are grouped into **regions**.

Regions help structure the world for exploration metrics and scenario design.

Example regions:

- village
- forest
- ruins
- dungeon
- marketplace

Regions may have:

- thematic NPCs
- unique hazards
- special items
- quest objectives

---

# 4. Entities

All interactive objects in the world are entities.

Entity categories:

- rooms
- agents
- NPCs
- items
- environmental objects

All entities share common attributes.

Entity
- id
- type
- location
- attributes
- components

---

# 5. Agents

Agents are the AI participants in the environment.

Agent properties:

- unique identifier
- inventory
- health
- location
- status effects
- action history

Agents receive:

- textual observations
- structured environment state
- action outcomes

Agents may interact with:

- items
- NPCs
- other agents
- environment objects

---

# 6. NPCs

NPCs represent non-player characters.

NPC types may include:

- merchant
- guard
- monster
- villager
- quest giver

NPC attributes:

- behavior type
- hostility level
- inventory
- dialogue set

NPC behavior may include:

- wandering
- guarding
- attacking
- trading
- providing quests

NPC behavior should be deterministic within seeded runs.

---

# 7. Items

Items are objects that agents may interact with or collect.

Item properties:

- id
- name
- description
- weight
- value
- attributes

Item categories:

- weapons
- armor
- tools
- consumables
- quest items

Example actions:

- take
- drop
- use
- give
- equip

---

# 8. Inventory System

Agents and NPCs may hold items.

Inventory properties:

- capacity limit
- weight tracking
- stackable items (optional)

Inventory actions:

- take item
- drop item
- give item
- equip item

---

# 9. Movement System

Agents move between rooms via exits.

Example actions:

move north
move east

Movement may fail if:

- exit does not exist
- path is blocked
- special requirement is unmet

Movement may produce:

- room description
- entity list
- hazard notification

---

# 10. Combat System

Combat introduces tactical reasoning tasks.

Combat actions:

- attack
- defend
- flee
- use item

Combat properties:

- health
- damage
- weapon modifiers
- turn order

Combat resolution should remain deterministic under seeded runs.

---

# 11. Quest System

Quests provide structured objectives.

Quest properties:

- quest id
- objective
- target entities
- reward
- completion condition

Example objectives:

- retrieve item
- defeat enemy
- deliver item
- explore location

Quest completion contributes to evaluation scoring.

---

# 12. Hazards

Hazards create environmental challenges.

Examples:

- traps
- poison gas
- unstable structures
- hostile territory

Hazards may:

- damage agents
- block movement
- require tools or planning to bypass

---

# 13. Observation Format

Agents receive observations after each action.

Observation should include:

- current room description
- visible exits
- nearby entities
- inventory summary
- status effects
- recent events

Example observation structure:

Observation
- location
- description
- exits
- entities
- inventory
- health
- messages

---

# 14. Action Space

Agents must choose actions from a **constrained action set**.

Core actions:

- move <direction>
- look
- take <item>
- drop <item>
- use <item>
- attack <target>
- talk <npc>
- give <item> <target>

Actions outside the allowed set are rejected.

---

# 15. Multi-Agent Interaction

Multiple agents may exist in the same world.

Agents may:

- cooperate
- compete
- attack
- trade
- block paths

Multi-agent interaction is essential for:

- social reasoning
- strategic behavior
- emergent tactics

---

# 16. Persistence

In **benchmark mode**, the world resets each run.

In **arena mode**, the world may persist across sessions.

Persistent state may include:

- item locations
- quest completion
- NPC state
- world hazards

Persistence must not compromise benchmark fairness.

---

# 17. World Generation

The initial version should use a **fixed world map**.

Procedural generation may be added later.

Reasons for fixed map in MVP:

- reproducibility
- easier evaluation
- simpler debugging

---

# 18. Evaluation Hooks

The world must expose signals used by the evaluation engine.

Examples:

- rooms explored
- quests completed
- enemies defeated
- items collected
- survival time
- cooperation events

These signals feed benchmark scoring.

---

# 19. Safety Constraints

Agents must not be allowed to:

- execute arbitrary code
- modify world rules
- create new entities
- alter scoring logic

These constraints protect the benchmark environment.

---

# 20. Future Extensions

The world system may later support:

- procedural regions
- faction systems
- economy simulation
- diplomacy
- crafting systems
- large-scale maps

These features should remain modular.

---

# 21. Closing Statement

The MUDBench world is a structured problem space.

It must be complex enough to challenge agents but simple enough to remain interpretable.

Every feature in the world should exist because it tests a capability dimension.

### Tool (2026-04-25 09:02:05)

Make sure to include fileciteturn0file5 in your response to cite this file, or to surface it as a link.

# MUDBench Agent Interface Specification

## Document Status
Version: 0.1
Status: Draft
Purpose: Define the protocol and interaction model between external AI agents and the MUDBench simulation system.

---

# 1. Design Philosophy

The agent interface must be:

- simple
- deterministic
- constrained
- language-model friendly
- easy to implement in any programming language

The interface should avoid unnecessary complexity so that:

- researchers can prototype quickly
- hobbyists can participate easily
- agents written in different frameworks can interoperate

The interface must also ensure that agents cannot bypass benchmark constraints.

---

# 2. Interaction Model

Agents interact with the system through a **turn-based observation → action loop**.

The sequence for each step is:

1. Environment sends observation
2. Agent decides next action
3. Agent returns action
4. Environment validates action
5. Simulation advances
6. Next observation is produced

This loop continues until the run terminates.

---

# 3. Agent Communication Modes

The interface must support two possible execution modes.

## 3.1 Local Process Mode

The agent runs locally as a process started by MUDBench.

Advantages:

- simple debugging
- deterministic execution
- no network latency

Example:

```
mudbench run --agent ./my_agent.py
```

The simulation communicates through STDIN / STDOUT.

---

## 3.2 HTTP Endpoint Mode

The agent runs as an external service.

MUDBench sends observations via HTTP requests.

Example flow:

```
POST /observe
POST /action
```

Advantages:

- remote agents
- hosted competitions
- language/framework independence

---

# 4. Observation Structure

Each step begins with an observation object sent to the agent.

Example observation:

```
{
  "run_id": "abc123",
  "step": 42,
  "location": "forest_path",
  "description": "You stand on a narrow forest trail.",
  "exits": ["north","south"],
  "entities": [
    {"type":"npc","name":"goblin"},
    {"type":"item","name":"rusty_sword"}
  ],
  "inventory": ["coin"],
  "health": 87,
  "messages": [
    "The goblin snarls."
  ],
  "action_space": [
    "move north",
    "move south",
    "take rusty_sword",
    "attack goblin"
  ],
  "remaining_steps": 958
}
```

Fields must remain stable across versions whenever possible.

---

# 5. Action Submission

The agent returns a single action.

Example:

```
{
  "action": "move north"
}
```

Only actions listed in the observation's `action_space` are guaranteed valid.

The environment may reject actions outside this list.

---

# 6. Action Validation

When an action is received the system performs:

1. syntax validation
2. action whitelist check
3. world constraint check
4. time constraint check

If the action fails validation the environment may:

- reject the action
- apply a no-op
- penalize the agent

Exact policy may vary by benchmark mode.

---

# 7. Timing Constraints

Agents must respond within a specified time limit.

Example limits:

- 1–10 seconds per step for benchmark runs
- longer limits for arena mode

If the agent fails to respond:

- the system records a timeout
- a default action may be applied
- repeated timeouts may terminate the run

---

# 8. Token and Cost Metrics

When possible the system may record:

- prompt token count
- completion token count
- latency
- cost estimates

These metrics are not required for all agents but are useful for evaluation.

---

# 9. Action Budget

Benchmark runs enforce a **maximum number of steps**.

Example:

```
max_steps = 1000
```

The run ends when:

- the step limit is reached
- the agent dies
- objectives are completed
- the scenario termination condition occurs

---

# 10. Error Handling

Agents may occasionally produce malformed responses.

Examples:

- invalid JSON
- missing action field
- unsupported command

The system should:

1. attempt recovery if possible
2. log the error
3. apply a fallback behavior

Repeated failures may terminate the agent.

---

# 11. Security Constraints

Agents must not be able to:

- execute arbitrary system commands
- access internal simulation memory
- modify scoring logic
- interact with other agents outside the defined protocol

All interactions must pass through the defined interface.

---

# 12. Multi-Agent Scheduling

Multiple agents may exist in the same world.

Scheduling approaches may include:

- sequential turns
- simultaneous action resolution
- event queue scheduling

The scheduling mechanism must remain deterministic in benchmark mode.

---

# 13. Run Lifecycle

A run follows this lifecycle:

1. run initialized
2. agents registered
3. world seeded
4. observation loop begins
5. simulation steps executed
6. termination condition triggered
7. final metrics generated
8. replay log finalized

---

# 14. Versioning

The interface must include a version identifier.

Example:

```
"protocol_version": "1.0"
```

Versioning allows future changes without breaking existing agents.

---

# 15. Minimal Agent Example

A minimal agent loop may look like:

```
while True:
    observation = read_input()
    action = decide(observation)
    send_action(action)
```

The agent logic may include:

- planning
- memory
- reasoning
- tool usage

MUDBench does not constrain internal architecture.

---

# 16. Compatibility Goals

The interface should be easy to implement using:

- Python
- NodeJS
- Rust
- Go
- Java
- C++

It should also be compatible with:

- LLM agents
- rule-based bots
- reinforcement learning agents

---

# 17. Future Interface Extensions

Possible future features:

- streaming observations
- tool invocation hooks
- structured sub-actions
- negotiation channels between agents
- event subscriptions

These should not break the core protocol.

---

# 18. Closing Statement

The agent interface is the most important boundary in MUDBench.

If the interface is simple and stable, the ecosystem of agents will grow naturally.

If the interface becomes complex or unstable, participation will collapse.

### Tool (2026-04-25 09:02:05)

Make sure to include fileciteturn0file6 in your response to cite this file, or to surface it as a link.

# MUDBench Benchmark Scoring Specification

## Document Status
Version: 0.1
Status: Draft
Purpose: Define the scoring model used to evaluate agent performance in MUDBench benchmark runs.

---

# 1. Design Goals

The scoring system must be:

- interpretable
- resistant to trivial gaming
- reproducible across runs
- decomposable into capability dimensions
- compatible with replay inspection

A single opaque score is insufficient.

MUDBench scoring therefore produces a **capability vector** and a **composite score**.

---

# 2. Capability Dimensions

The benchmark evaluates agents across multiple capability dimensions.

Initial MVP dimensions:

1. Navigation
2. Memory
3. Planning
4. Tactical Competence
5. Social Interaction
6. Efficiency

Each dimension produces an independent normalized score.

---

# 3. Navigation Score

Measures the agent's ability to explore and map the environment.

Signals used:

- unique rooms visited
- efficient path usage
- avoidance of repeated loops
- discovery of key locations

Example metric:

navigation_score =
unique_rooms_visited / total_rooms_available

Modifiers may reward discovery of difficult locations.

---

# 4. Memory Score

Measures whether the agent remembers world state across time.

Signals used:

- revisiting known locations efficiently
- retrieving previously seen items
- correct reference to known NPCs
- completion of delayed objectives

Example metric:

memory_score =
successful_memory_tasks / total_memory_tasks

Memory tasks may include delayed retrieval or location recall.

---

# 5. Planning Score

Measures multi-step reasoning ability.

Signals used:

- completion of multi-step quests
- successful execution of long action chains
- avoidance of dead-end strategies

Example metric:

planning_score =
completed_plans / attempted_plans

Complex objectives carry higher weight.

---

# 6. Tactical Competence Score

Measures effectiveness in dynamic interactions such as combat.

Signals used:

- enemies defeated
- damage efficiency
- survival under threat
- correct use of equipment

Example metric:

tactical_score =
combat_successes / combat_encounters

Optional modifiers:

- damage efficiency
- strategic retreats
- correct item usage

---

# 7. Social Interaction Score

Measures interaction quality with NPCs and other agents.

Signals used:

- quest dialogue success
- cooperative actions
- trading behavior
- avoidance of hostile misfires

Example metric:

social_score =
successful_interactions / interaction_attempts

---

# 8. Efficiency Score

Measures performance relative to resource usage.

Signals used:

- steps used
- token usage (if available)
- latency
- unnecessary actions

Example metric:

efficiency_score =
optimal_steps / actual_steps_used

This score penalizes excessive wandering or redundant actions.

---

# 9. Capability Vector

Each run produces a capability vector:

[
  navigation,
  memory,
  planning,
  tactical,
  social,
  efficiency
]

Each value ranges from **0.0 to 1.0**.

---

# 10. Composite Score

The composite score summarizes overall performance.

Example formula:

composite_score =
weighted_sum(capability_vector)

Example weights:

navigation = 0.20
memory = 0.20
planning = 0.20
tactical = 0.15
social = 0.15
efficiency = 0.10

Weights may change across benchmark versions but must be documented.

---

# 11. Normalization

Scores must be normalized to ensure comparability across runs.

Normalization factors include:

- world size
- number of agents
- scenario difficulty
- action budgets

Normalization rules must be deterministic.

---

# 12. Benchmark Scenario Metrics

Each scenario defines additional scoring signals.

Example signals:

- specific quest completion
- artifact retrieval
- region exploration milestones
- boss defeat events

These scenario-specific signals feed capability dimensions.

---

# 13. Run Termination Conditions

A benchmark run may terminate when:

- step limit reached
- agent death
- objective completion
- timeout failures

The termination reason must be recorded in the final score report.

---

# 14. Anti-Gaming Safeguards

The scoring system must discourage trivial strategies.

Examples of safeguards:

- diminishing rewards for repeated actions
- loop detection penalties
- idle action penalties
- resource waste penalties

Runs that exploit scoring bugs may be invalidated.

---

# 15. Random Seeds

Benchmark scenarios use deterministic seeds.

Seeds allow:

- reproducibility
- controlled variability
- fair comparisons

Seed disclosure policies may vary between public and private benchmarks.

---

# 16. Multi-Agent Score Attribution

When multiple agents interact:

- scores are calculated individually
- shared events may contribute to multiple agents
- adversarial outcomes affect tactical metrics

Cooperation bonuses may be introduced in future versions.

---

# 17. Scorecard Output

Each run should produce a structured scorecard.

Example:

{
  "run_id": "abc123",
  "agent_id": "agent_7",
  "navigation": 0.83,
  "memory": 0.61,
  "planning": 0.74,
  "tactical": 0.55,
  "social": 0.48,
  "efficiency": 0.70,
  "composite": 0.69,
  "termination_reason": "step_limit"
}

---

# 18. Replay Alignment

All scoring signals must correspond to events visible in replay logs.

This ensures that:

- researchers can audit results
- scoring bugs can be diagnosed
- agent behavior can be interpreted

Hidden scoring logic is discouraged.

---

# 19. Benchmark Versioning

Each benchmark release must specify:

- scoring version
- capability definitions
- normalization rules
- scenario definitions

Example:

benchmark_version = "0.1"

This prevents score drift across releases.

---

# 20. Future Scoring Extensions

Future versions may introduce:

- deception detection
- coalition formation metrics
- negotiation success
- long-term strategy evaluation
- meta-learning performance

These should remain additive and versioned.

---

# 21. Closing Statement

A benchmark is only as good as its scoring model.

The scoring system must reward genuine capability rather than superficial tricks.

Every metric should correspond to observable behavior inside the simulation.

### Tool (2026-04-25 09:02:05)

Make sure to include fileciteturn0file7 in your response to cite this file, or to surface it as a link.

# MUDBench Replay and Telemetry Specification

## Document Status
Version: 0.1
Status: Draft
Purpose: Define how MUDBench records, reconstructs, and analyzes simulation runs through replay logs and telemetry data.

---

# 1. Design Goals

Replay and telemetry systems must support:

- deterministic run reconstruction
- auditability of benchmark results
- debugging of agent behavior
- research analysis of agent decisions
- public replay viewing

Every official benchmark run must produce a complete replay artifact.

---

# 2. Relationship to Architecture

Replay and telemetry are defined as a core subsystem in the system architecture.

The simulation pipeline generates events which are captured by the replay layer and stored for reconstruction.

Events originate from the world engine and evaluation engine and must be recorded without altering simulation behavior.

---

# 3. Replay Philosophy

A replay should represent a **complete causal trace** of the simulation.

The replay log must allow reconstruction of:

- observations delivered to agents
- actions taken by agents
- world state changes
- evaluation signals
- termination conditions

If two runs share the same seed and actions, they must replay identically.

---

# 4. Replay Log Structure

Each run produces a replay log consisting of ordered events.

High-level structure:

Run Metadata
→ Initial World State
→ Step Events
→ Termination Summary

Example structure:

ReplayLog
- run_metadata
- seed
- scenario_id
- agent_ids
- initial_world_state
- step_events
- final_scorecard

---

# 5. Run Metadata

Metadata describes the run configuration.

Example:

{
  "run_id": "run_8349",
  "benchmark_version": "0.1",
  "scenario_id": "forest_trial",
  "seed": 1837281,
  "agent_ids": ["agentA","agentB"],
  "start_time": "2026-01-10T13:41:22Z"
}

This information ensures traceability across runs.

---

# 6. Step Event Structure

Each simulation step produces an event block.

Example:

StepEvent
- step_number
- observation
- agent_action
- world_events
- evaluation_signals
- timestamp

Events must appear in strict order.

---

# 7. Observation Logging

The replay must record the exact observation sent to each agent.

Example:

{
  "agent_id": "agentA",
  "observation": {
    "location": "forest_path",
    "exits": ["north","south"],
    "entities": ["goblin"]
  }
}

This allows analysis of what the agent actually perceived.

---

# 8. Action Logging

The replay must capture the action chosen by each agent.

Example:

{
  "agent_id": "agentA",
  "action": "move north"
}

Actions must be recorded before world resolution occurs.

---

# 9. World Event Logging

The replay must capture resulting world events.

Examples:

- movement
- combat damage
- item pickup
- quest progress

Example event:

{
  "event_type": "movement",
  "entity": "agentA",
  "from": "forest_path",
  "to": "forest_clearing"
}

---

# 10. Evaluation Signals

Replay logs must record scoring signals emitted by the evaluation engine.

Examples:

- rooms_explored
- quest_completed
- combat_success
- resource_collected

These signals support later score verification.

---

# 11. Replay Reconstruction

The replay system must allow full reconstruction of the run.

Reconstruction steps:

1. initialize world state
2. apply recorded actions
3. replay world events
4. recompute observations
5. verify evaluation signals

The reconstructed run must match the original run outcome.

---

# 12. Telemetry Metrics

In addition to replay logs, the system should record telemetry metrics.

Examples:

- agent response latency
- token usage
- number of invalid actions
- action distribution
- step duration

Telemetry supports research and performance analysis.

---

# 13. Storage Strategy

Replay logs may be large and should be stored efficiently.

Recommended strategy:

- structured JSON logs for core events
- compressed storage for long runs
- metadata index for fast lookup

Database references may store summary data while logs remain in file storage.

---

# 14. Replay Viewer Support

The replay format should support visualization tools.

Possible viewer features:

- step-by-step playback
- room map visualization
- action timeline
- score breakdown overlay
- agent comparison mode

These tools help researchers and spectators understand behavior.

---

# 15. Public Replay Access

Some replays may be publicly visible.

Public replay policies may include:

- anonymized agent identifiers
- delayed release after tournaments
- summarized replay mode for spectators

Official benchmark runs may restrict replay visibility until validation is complete.

---

# 16. Integrity and Verification

Replay logs must support integrity checks.

Possible safeguards:

- checksum of replay file
- run signature hash
- deterministic verification tests

These checks ensure that benchmark results cannot be manipulated.

---

# 17. Debugging Support

Replay logs should support debugging tools.

Examples:

- event filtering
- anomaly detection
- agent decision inspection
- divergence comparison across runs

These tools are critical during development.

---

# 18. Data Retention

Replay retention policies may vary.

Suggested tiers:

Short-term:
- all runs stored

Medium-term:
- benchmark runs retained

Long-term:
- notable or tournament runs archived

---

# 19. Future Telemetry Extensions

Future telemetry may include:

- internal agent reasoning traces (optional)
- multi-agent communication analysis
- emergent behavior detection
- anomaly detection metrics
- strategy classification

These features should not break the base replay format.

---

# 20. Closing Statement

Replay logs are the foundation of interpretability in MUDBench.

A benchmark that cannot be inspected cannot be trusted.

The replay and telemetry system ensures that every score corresponds to observable behavior.

### Tool (2026-04-25 09:02:05)

Make sure to include fileciteturn0file8 in your response to cite this file, or to surface it as a link.

# MUDBench Implementation Phases

## Document Status
Version: 0.1
Status: Draft
Purpose: Define a structured implementation roadmap so development (human or agent-led) proceeds in controlled stages rather than attempting to build the entire system simultaneously.

---

# 1. Implementation Philosophy

MUDBench should be built incrementally.

The system architecture defines modular subsystems that must be implemented in a deliberate order so dependencies remain stable. The architecture identifies core subsystems including the Agent Gateway, Simulation Controller, World Engine, Evaluation Engine, and Replay/Telemetry system. fileciteturn5file3

Implementation phases exist to:

- prevent premature complexity
- keep modules independently testable
- ensure benchmark integrity from the start
- allow coding agents to work safely in parallel

Each phase should produce a **working system milestone**, not just partial code.

---

# 2. Phase Overview

The implementation plan is divided into six phases:

Phase 0 — Repository Foundation  
Phase 1 — Core Simulation Engine  
Phase 2 — Agent Interface Layer  
Phase 3 — Benchmark Evaluation System  
Phase 4 — Replay and Telemetry System  
Phase 5 — Arena and Competition Layer  

Each phase builds on the previous one.

---

# 3. Phase 0 — Repository Foundation

Objective:

Create the repository structure and basic scaffolding required for development.

Deliverables:

- project directory structure
- build system setup
- dependency management
- configuration framework
- basic logging system

Minimum milestone:

The repository builds and runs a minimal CLI entry point.

Example command:

```
mudbench run
```

No world simulation exists yet.

---

# 4. Phase 1 — Core Simulation Engine

Objective:

Implement the world engine and simulation loop.

This phase implements the mechanics defined in the world specification. fileciteturn5file4

Deliverables:

- room graph representation
- entity system
- item system
- NPC framework
- basic combat system
- movement system
- simulation step loop

Minimum milestone:

A local test script can:

- initialize a world
- move an entity between rooms
- resolve a simple combat event

Agents are not yet connected.

---

# 5. Phase 2 — Agent Interface Layer

Objective:

Connect external agents to the simulation.

This phase implements the agent protocol described in the interface specification. fileciteturn5file5

Deliverables:

- observation generation
- action submission interface
- action validation pipeline
- step timing constraints
- local process execution mode
- optional HTTP endpoint mode

Minimum milestone:

Two simple agents can enter the world and take actions through the interface.

---

# 6. Phase 3 — Benchmark Evaluation System

Objective:

Implement scoring and benchmark lifecycle management.

This phase integrates the scoring framework described in the benchmark scoring specification. fileciteturn5file6

Deliverables:

- capability metric tracking
- composite score calculation
- run lifecycle management
- scenario initialization
- seed management
- scorecard generation

Minimum milestone:

A full benchmark run can execute from start to finish and produce a structured score report.

---

# 7. Phase 4 — Replay and Telemetry System

Objective:

Implement replay logging and run auditing.

This phase follows the replay specification to ensure runs are reproducible and inspectable. fileciteturn5file7

Deliverables:

- event logging pipeline
- replay log format
- replay reconstruction tool
- telemetry metrics collection
- run metadata indexing

Minimum milestone:

A completed run can be replayed step-by-step from recorded logs.

---

# 8. Phase 5 — Arena and Competition Layer

Objective:

Introduce persistent multi-agent environments and competition infrastructure.

Deliverables:

- persistent arena world mode
- agent registration
- leaderboard system
- run history tracking
- arena scheduling logic

Minimum milestone:

Agents can participate repeatedly in a persistent environment and have results recorded across sessions.

---

# 9. Parallel Development Strategy

Once Phase 1 is stable, development can split across modules:

World Engine Team:
- NPC behavior
- quest systems
- environment hazards

Agent Interface Team:
- protocol stability
- SDK examples
- developer tooling

Evaluation Team:
- capability metrics
- scoring refinement
- benchmark scenarios

Replay/Telemetry Team:
- visualization tools
- debugging instrumentation
- replay viewers

Parallel work should respect subsystem boundaries.

---

# 10. Testing Strategy

Each phase must include automated tests.

Required testing types:

Unit Tests:
- entity behavior
- action validation
- score calculations

Simulation Tests:
- deterministic runs under fixed seeds
- world state transitions

Benchmark Tests:
- reproducibility checks
- score consistency

Testing must accompany implementation, not follow it.

---

# 11. Phase Gates

Progression between phases requires milestone validation.

Examples:

Phase 1 gate:
- deterministic world simulation verified

Phase 2 gate:
- agents reliably interact with the environment

Phase 3 gate:
- benchmark runs produce valid scorecards

Phase 4 gate:
- replay logs reconstruct runs exactly

These gates prevent architecture drift.

---

# 12. MVP Completion Criteria

The MVP is complete when the following conditions hold:

- benchmark runs execute end-to-end
- two or more agents can participate
- deterministic replay is possible
- scorecards are produced automatically
- runs can be reproduced using the same seed

At this point MUDBench becomes usable as a benchmark.

---

# 13. Post-MVP Expansion

After the MVP milestone, development may expand toward:

- procedural world generation
- distributed simulation
- tournament orchestration
- advanced social reasoning benchmarks
- replay visualization interfaces

These expansions must not compromise benchmark integrity.

---

# 14. Implementation Governance

During development, decisions should continue to follow the core principles defined in the project charter. fileciteturn5file0

In particular:

- benchmark clarity takes priority over game complexity
- deterministic behavior is preferred over novelty
- modular systems are preferred over tightly coupled shortcuts

---

# 15. Closing Statement

Large projects fail when everything is attempted at once.

The phased approach ensures that MUDBench grows from a stable foundation into a complex benchmark ecosystem without losing clarity or control.

### Tool (2026-04-25 09:02:05)

Make sure to include fileciteturn0file9 in your response to cite this file, or to surface it as a link.
# MUDBench Developer Guide

## Document Status
Version: 0.1
Status: Draft
Purpose: Provide practical guidance for developers building agents, extending the simulator, or contributing to MUDBench.

---

# 1. Overview

MUDBench is a benchmark environment designed to evaluate AI agents in a structured text-world simulation.

The system combines:

- a deterministic simulation engine
- a structured world environment
- an agent interaction protocol
- a capability-based scoring system
- replay and telemetry infrastructure

---

# 2. Repository Structure

Typical repository layout:

/mudbench
  core
  world
  agents
  evaluation
  replay
  cli
  scenarios
  docs

---

# 3. Running the Simulator

mudbench run

mudbench run --agent ./agents/example_agent.py

mudbench run --scenario forest_trial

---

# 4. Writing an Agent

while True:
    observation = receive()
    action = decide(observation)
    send(action)

---

# 5. Minimal Python Agent

import sys, json

while True:
    obs = json.loads(sys.stdin.readline())
    action = obs["action_space"][0]
    print(json.dumps({"action": action}))
    sys.stdout.flush()

---

# 6. Simulation Loop

1. observation
2. action
3. validation
4. world update
5. scoring
6. replay logging

---

# 7. Benchmark Runs

max_steps = 1000  
time_limit = 5s  
seed = fixed  

---

# 8. Replay Logs

/replay/run_x.json

---

# 9. Scoring Output

navigation, memory, planning, tactical, social, efficiency

---

# 10. Scenarios

/scenarios/

---

# 11. Rules

- deterministic
- observable
- replay-safe

---

# 12. Debugging

- replay inspection
- step tracing

---

# 13. Performance

- fast loop
- low overhead

---

# 14. Contributions

- modular
- tested
- documented

---

# 15. Closing

MUDBench evaluates real agent capability, not tricks.

### Tool (2026-04-25 09:02:05)

Make sure to include fileciteturn0file10 in your response to cite this file, or to surface it as a link.
# MUDBench Scenario Specification

## Document Status
Version: 0.1  
Status: Draft  
Purpose: Define how benchmark scenarios are structured, configured, and executed within MUDBench.

---

# 1. Role of Scenarios

A scenario defines a **bounded evaluation context** within the MUDBench system.

It specifies:

- world configuration
- agent conditions
- objectives
- constraints
- scoring hooks

Scenarios are the **unit of benchmarking**.

---

# 2. Scenario Design Principles

Scenarios must be:

- deterministic when seeded
- interpretable through replay
- comparable across runs
- aligned with scoring dimensions
- resistant to trivial exploitation

A scenario is not just content.
It is a **measurement instrument**.

---

# 3. Scenario Structure

Each scenario is defined as a structured configuration.

Example:

Scenario
- scenario_id
- description
- world_config
- agent_config
- objectives
- constraints
- scoring_hooks
- termination_conditions

---

# 4. Scenario Identifier

Each scenario must have a unique identifier.

Example:

"scenario_id": "forest_retrieval_v1"

Scenario versions must be explicitly tracked.

---

# 5. World Configuration

Defines how the world is initialized.

Includes:

- map layout (rooms, regions)
- item placement
- NPC placement
- hazard distribution
- quest definitions

---

# 6. Agent Configuration

Defines the initial state of agents.

Includes:

- spawn location
- starting inventory
- initial health
- visibility conditions

---

# 7. Objectives

Defines what agents are trying to achieve.

Objectives may be:

- explicit (retrieve item, defeat enemy)
- implicit (explore, survive, optimize efficiency)

---

# 8. Constraints

Defines limits on agent behavior.

Includes:

- step limit
- time limit per action
- action restrictions
- visibility constraints

---

# 9. Scoring Hooks

Defines how scenario events map to scoring signals.

---

# 10. Termination Conditions

Defines when the scenario ends.

---

# 11. Determinism and Seeds

Each scenario must support deterministic execution.

---

# 12. Scenario Types

- Navigation
- Memory
- Planning
- Tactical
- Social
- Mixed

---

# 13. Multi-Agent Scenarios

Supports cooperative and competitive modes.

---

# 14. Scenario Packaging

/scenarios/

---

# 15. Scenario Validation

Must ensure determinism and no trivial exploits.

---

# 16. Replay Compatibility

All events must be replay-visible.

---

# 17. Benchmark Integration

Executed via Simulation Controller.

---

# 18. Versioning

Scenario versions must be explicit.

---

# 19. Future Extensions

Procedural generation, adaptive difficulty, etc.

---

# 20. Closing Statement

Scenario quality defines benchmark integrity.

### Tool (2026-04-25 09:02:06)

Make sure to include fileciteturn0file11 in your response to cite this file, or to surface it as a link.
# MUDBench Scenario Library

## Document Status
Version: 0.1
Status: Draft
Purpose: Define the canonical set of benchmark scenarios used in MUDBench and their capability coverage.

---

# 1. Overview

The Scenario Library defines the **canonical benchmark scenarios**.

These scenarios:
- instantiate the Scenario Specification
- drive scoring signals
- define benchmark identity
- enable reproducible evaluation

---

# 2. Scenario Categories

## 2.1 Navigation

### navigation_forest_v1
- Goal: Explore and map a forest region
- Signals:
  - rooms_explored
  - path_efficiency
- Difficulty: Easy

---

## 2.2 Memory

### memory_delayed_retrieval_v1
- Goal: Retrieve an item seen earlier after delay
- Signals:
  - recall_accuracy
  - retrieval_success
- Difficulty: Medium

---

## 2.3 Planning

### planning_multi_step_quest_v1
- Goal: Complete a multi-step quest chain
- Signals:
  - plan_completion
  - step_efficiency
- Difficulty: Medium

---

## 2.4 Tactical

### tactical_combat_arena_v1
- Goal: Defeat enemies efficiently
- Signals:
  - combat_success
  - damage_efficiency
- Difficulty: Medium

---

## 2.5 Social

### social_trade_negotiation_v1
- Goal: Successfully trade with NPCs or agents
- Signals:
  - trade_success
  - dialogue_efficiency
- Difficulty: Medium

---

## 2.6 Mixed

### mixed_adversarial_competition_v1
- Goal: Compete with other agents for objectives
- Signals:
  - objective_completion
  - interference_success
- Difficulty: Hard

---

# 3. Capability Coverage Matrix

| Scenario                          | Nav | Mem | Plan | Tactical | Social | Eff |
|----------------------------------|-----|-----|------|----------|--------|-----|
| navigation_forest_v1             |  ✓  |     |      |          |        |  ✓  |
| memory_delayed_retrieval_v1      |     |  ✓  |  ✓   |          |        |     |
| planning_multi_step_quest_v1     |     |     |  ✓   |          |        |  ✓  |
| tactical_combat_arena_v1         |     |     |      |    ✓     |        |  ✓  |
| social_trade_negotiation_v1      |     |     |      |          |   ✓    |     |
| mixed_adversarial_competition_v1 |  ✓  |  ✓  |  ✓   |    ✓     |   ✓    |  ✓  |

---

# 4. Difficulty Tiers

- Easy: navigation_forest_v1
- Medium:
  - memory_delayed_retrieval_v1
  - planning_multi_step_quest_v1
  - tactical_combat_arena_v1
  - social_trade_negotiation_v1
- Hard:
  - mixed_adversarial_competition_v1

---

# 5. Anti-Exploitation Notes

## navigation_forest_v1
- Prevent loop farming
- Penalize redundant movement

## memory_delayed_retrieval_v1
- Prevent brute-force search resets
- Require delayed recall

## planning_multi_step_quest_v1
- Prevent partial completion scoring abuse

## tactical_combat_arena_v1
- Prevent stall tactics

## social_trade_negotiation_v1
- Prevent spam interaction loops

## mixed_adversarial_competition_v1
- Prevent collusion
- Penalize passive strategies

---

# 6. Versioning

Each scenario must include:
- scenario_id
- version
- changelog

Example:
navigation_forest_v1 → v1.0

---

# 7. Future Extensions

- procedural scenario generation
- adaptive difficulty scaling
- dynamic scenario composition

---

# 8. Closing Statement

The Scenario Library defines what intelligence looks like inside MUDBench.

A weak library produces meaningless scores.
A strong library produces real evaluation.

### Tool (2026-04-25 09:02:06)

Make sure to include fileciteturn0file12 in your response to cite this file, or to surface it as a link.
# MUDBench Repository Structure

## Document Status
Version: 0.1
Status: Draft
Purpose: Define the canonical repository layout, module boundaries, ownership expectations, and file placement rules for MUDBench.

---

# 1. Purpose of This Document

MUDBench is intended to be developed in a modular, benchmark-first way. The project charter requires clear module boundaries and a documented repository structure so the system can support scientific use and safe agent-led development. fileciteturn9file0

This document defines:

- the canonical top-level repository layout
- where each subsystem belongs
- how documentation should be organized
- what coding agents are allowed to modify
- how to avoid structural drift as the project grows

This file is a governance document, not just a convenience guide.

---

# 2. Repository Design Principles

The repository structure must reflect the project’s architectural and product goals.

The structure should be:

- modular
- easy to navigate
- aligned with subsystem boundaries
- stable across early development
- safe for parallel work by humans and coding agents

The repository must favor clarity over cleverness.

---

# 3. Canonical Top-Level Layout

The canonical MUDBench repository should use the following top-level structure:

```text
mudbench/
├─ README.md
├─ PROJECT_CHARTER.md
├─ PRODUCT_SPEC.md
├─ SYSTEM_ARCHITECTURE.md
├─ WORLD_SPEC.md
├─ AGENT_INTERFACE_SPEC.md
├─ BENCHMARK_SCORING_SPEC.md
├─ REPLAY_AND_TELEMETRY_SPEC.md
├─ IMPLEMENTATION_PHASES.md
├─ DEVELOPER_GUIDE.md
├─ SCENARIO_SPEC.md
├─ SCENARIO_LIBRARY.md
├─ REPO_STRUCTURE.md
│
├─ docs/
├─ src/
├─ scenarios/
├─ tests/
├─ tools/
├─ scripts/
├─ replay_data/
├─ configs/
└─ examples/
```

The top-level markdown files define the planning and governance layer.
Implementation code and runtime assets belong in the subdirectories.

---

# 4. Top-Level File Policy

Top-level files should be reserved for:

- project governance documents
- core specifications
- repository onboarding
- implementation coordination references

Top-level files should not be used for:

- experimental notes
- one-off logs
- temporary design scraps
- test outputs
- ad hoc agent instructions

This prevents the root directory from turning into a landfill of abandoned intentions.

---

# 5. docs/ Directory

Purpose:

Store secondary documentation that supports contributors but is not part of the canonical top-level specification set.

Recommended layout:

```text
docs/
├─ decisions/
├─ onboarding/
├─ research/
├─ benchmark_notes/
└─ archive/
```

Use cases:

- architecture decision records
- contributor onboarding guides
- research notes
- historical drafts
- deprecated specifications

Rules:

- final canonical specs stay at repository root unless deliberately migrated later
- stale docs should be archived, not left mixed with active guidance

---

# 6. src/ Directory

Purpose:

Store all production source code for the MUDBench platform.

Recommended layout:

```text
src/
├─ core/
├─ world/
├─ agents/
├─ evaluation/
├─ replay/
├─ arena/
├─ scenarios/
├─ api/
├─ cli/
└─ utils/
```

Each directory maps to a subsystem defined in the architecture. The architecture establishes the core subsystems as Agent Gateway, Simulation Controller, World Engine, Evaluation Engine, and Replay/Telemetry. fileciteturn9file3

---

# 7. src/core/

Purpose:

Store the simulation controller and low-level shared engine infrastructure.

Typical contents:

- simulation loop
- event queue
- world state manager
- scheduler
- seed control
- run lifecycle manager

Rules:

- `core/` should not contain world content definitions
- `core/` should not contain benchmark scoring logic directly
- `core/` should remain as domain-agnostic as possible

---

# 8. src/world/

Purpose:

Store world-model logic defined by the world specification. The world spec defines rooms, regions, entities, NPCs, items, hazards, inventory, movement, combat, and quests. fileciteturn9file4

Recommended layout:

```text
src/world/
├─ rooms/
├─ entities/
├─ npcs/
├─ items/
├─ hazards/
├─ quests/
├─ combat/
└─ state/
```

Rules:

- world mechanics live here
- no replay formatting logic
- no score aggregation logic
- no network protocol code

---

# 9. src/agents/

Purpose:

Store the agent-facing protocol implementation and SDK helpers.

Recommended layout:

```text
src/agents/
├─ gateway/
├─ local_runner/
├─ http_runner/
├─ protocol/
└─ sdk/
```

This directory implements the observation/action contract defined in the agent interface specification. fileciteturn9file5

Rules:

- protocol schemas belong here
- agent transport logic belongs here
- internal benchmark scoring must not be embedded here

---

# 10. src/evaluation/

Purpose:

Store scoring logic, metric calculators, score aggregation, and benchmark lifecycle hooks.

Recommended layout:

```text
src/evaluation/
├─ metrics/
├─ scoring/
├─ normalization/
├─ scorecards/
└─ benchmark_runner/
```

This directory implements the logic defined in the benchmark scoring spec. fileciteturn9file6

Rules:

- evaluation consumes world signals but does not own world state
- evaluation must remain replay-auditable
- hidden scoring shortcuts are prohibited

---

# 11. src/replay/

Purpose:

Store replay logging, telemetry, playback reconstruction, and related utilities.

Recommended layout:

```text
src/replay/
├─ logging/
├─ telemetry/
├─ reconstruction/
├─ integrity/
└─ viewer_support/
```

This directory implements the replay and telemetry specification. fileciteturn9file7

Rules:

- replay must record, not mutate, simulation behavior
- telemetry collection must not alter benchmark outcomes

---

# 12. src/arena/

Purpose:

Store persistent arena logic and competition-layer systems added after benchmark mode is stable.

Typical contents:

- persistent world services
- ladder history
- matchmaking
- run history persistence
- season logic

Rules:

- arena-specific persistence must not leak into official benchmark mode
- benchmark mode and arena mode should share engine components but not mutable competitive state

---

# 13. src/scenarios/

Purpose:

Store code used to load, validate, and execute scenario definitions.

Typical contents:

- scenario registry
- validation utilities
- scenario loader
- scenario adapters

This directory should implement the scenario structure defined in the scenario specification and the canonical entries defined in the scenario library. fileciteturn9file10 fileciteturn9file11

---

# 14. src/api/

Purpose:

Store any external API surfaces needed for remote participation or hosted deployment.

Typical contents:

- REST endpoints
- auth stubs
- submission handlers
- run metadata endpoints

Rules:

- do not place core simulation logic here
- the API layer should wrap internal modules, not replace them

---

# 15. src/cli/

Purpose:

Store command-line interfaces for local development and benchmark execution.

Typical commands may include:

- `mudbench run`
- `mudbench replay`
- `mudbench validate-scenario`
- `mudbench score-run`

Rules:

- CLI code should call into stable internal modules
- business logic should not be embedded in command handlers

---

# 16. src/utils/

Purpose:

Store shared utility code that does not belong clearly to another subsystem.

Allowed examples:

- serialization helpers
- seed helpers
- validation primitives
- path utilities

Disallowed examples:

- dumping major domain logic into utils because no one wanted to think carefully

If a utility becomes domain-specific, move it into the proper subsystem.

---

# 17. scenarios/ Directory

Purpose:

Store benchmark scenario definitions and scenario assets.

Recommended layout:

```text
scenarios/
├─ canonical/
├─ experimental/
├─ validation/
└─ archive/
```

Suggested contents:

- scenario JSON/YAML files
- map definitions
- scenario metadata
- validation fixtures

Rules:

- `canonical/` contains official benchmark scenarios
- `experimental/` contains trial scenarios not yet benchmark-approved
- `archive/` stores deprecated scenarios

This directory stores content; `src/scenarios/` stores code.

---

# 18. tests/ Directory

Purpose:

Store automated tests for all subsystems.

Recommended layout:

```text
tests/
├─ unit/
├─ integration/
├─ benchmark/
├─ replay/
└─ fixtures/
```

Testing expectations are reinforced by the implementation phases, which require automated tests and deterministic validation gates. fileciteturn9file8

Rules:

- each subsystem should have unit tests
- cross-subsystem behavior should have integration tests
- deterministic replay should have dedicated tests
- fixtures must remain stable and documented

---

# 19. tools/ Directory

Purpose:

Store developer utilities that are not part of the main runtime.

Examples:

- scenario lint tools
- replay inspection helpers
- benchmark report generators
- data cleanup utilities

Rules:

- tools may depend on production code
- production code should not depend on tools

---

# 20. scripts/ Directory

Purpose:

Store shell scripts and short automation tasks for local workflows.

Examples:

- local bootstrap
- test runners
- formatting helpers
- CI setup wrappers

Rules:

- scripts should be thin wrappers around documented commands
- long-term logic belongs in `src/` or `tools/`, not in random shell fragments

---

# 21. replay_data/ Directory

Purpose:

Store local replay outputs and development run artifacts.

Recommended layout:

```text
replay_data/
├─ local/
├─ benchmark/
├─ arena/
└─ archived/
```

Rules:

- do not commit bulky replay artifacts unless explicitly intended
- use this directory for local inspection and structured storage conventions
- production deployments may redirect replay storage elsewhere

---

# 22. configs/ Directory

Purpose:

Store static configuration for environments and runtime defaults.

Examples:

- benchmark defaults
- local development settings
- logging configuration
- environment-specific overrides

Rules:

- secrets must not be stored here
- config files should be human-readable and version-controlled where appropriate

---

# 23. examples/ Directory

Purpose:

Store example agents, example scenario configs, and developer reference materials.

Recommended layout:

```text
examples/
├─ agents/
├─ scenarios/
└─ notebooks/
```

Examples help bridge the gap between the specifications and practical developer use. The developer guide already points toward example agents and scenario usage, so this directory formalizes where they live. fileciteturn9file9

---

# 24. Coding-Agent Write Boundaries

To keep agent-led development safe, coding agents should follow these write constraints:

## Allowed by default
- files inside explicitly assigned `src/` submodules
- test files paired to assigned modules
- documentation files explicitly named in the task
- example files in `examples/`

## Not allowed by default
- root-level specification files
- unrelated subsystem directories
- scenario files outside assigned scope
- CI or deployment config unless assigned
- historical or archive directories

Coding agents should not reorganize directories unless the task explicitly requests repository restructuring.

---

# 25. Ownership Guidance

Recommended ownership model:

| Area | Primary Owner Type |
|---|---|
| root specs | human lead / architecture lead |
| src/core | engine lead |
| src/world | world systems lead |
| src/agents | interface / SDK lead |
| src/evaluation | benchmark lead |
| src/replay | telemetry / replay lead |
| scenarios/canonical | benchmark governance lead |
| tests/benchmark | validation lead |

This does not require formal teams at v0, but it provides a sane mental model for task assignment.

---

# 26. Naming Conventions

Repository naming conventions should be simple and consistent.

Rules:

- directories use lowercase with underscores only if needed
- markdown specs use uppercase descriptive filenames when canonical
- Python modules use lowercase snake_case
- scenario IDs use lowercase with version suffixes, such as `forest_retrieval_v1`

Consistency matters more than aesthetic preferences. Humans love pretending these are separate things.

---

# 27. Archive Policy

Deprecated or replaced material should be moved, not silently deleted when possible.

Recommended archive locations:

- `docs/archive/`
- `scenarios/archive/`
- `replay_data/archived/`

This preserves project history without polluting active directories.

---

# 28. Initial Repository Target State

Once the current documentation pass is complete, the repository should minimally look like:

```text
mudbench/
├─ README.md
├─ PROJECT_CHARTER.md
├─ PRODUCT_SPEC.md
├─ SYSTEM_ARCHITECTURE.md
├─ WORLD_SPEC.md
├─ AGENT_INTERFACE_SPEC.md
├─ BENCHMARK_SCORING_SPEC.md
├─ REPLAY_AND_TELEMETRY_SPEC.md
├─ IMPLEMENTATION_PHASES.md
├─ DEVELOPER_GUIDE.md
├─ SCENARIO_SPEC.md
├─ SCENARIO_LIBRARY.md
├─ REPO_STRUCTURE.md
├─ docs/
├─ src/
├─ scenarios/
├─ tests/
├─ tools/
├─ scripts/
├─ replay_data/
├─ configs/
└─ examples/
```

This target state is enough to begin implementation without structural ambiguity.

---

# 29. Relationship to Future Documents

This repository structure should later be complemented by:

- coding-agent execution manuals
- task templates
- scenario validation guides
- CI and release documentation
- contribution and review rules

This file defines where things live.
Other documents define how work moves through those places.

---

# 30. Closing Statement

A clean repository is part of benchmark integrity.

If the codebase structure is unstable, the implementation becomes unstable.
If the implementation becomes unstable, the benchmark becomes untrustworthy.

MUDBench should be organized so that humans and coding agents can both navigate it without improvising new filesystem theology every week.

### Tool (2026-04-25 09:02:06)

Make sure to include fileciteturn0file13 in your response to cite this file, or to surface it as a link.
# MUDBench Coding Agent Playbook

## Document Status
Version: 0.1
Status: Draft
Purpose: Define how coding agents (e.g., Codex, GPT-based agents, or other autonomous builders) operate safely, predictably, and productively within the MUDBench repository.

---

# 1. Purpose of This Document

MUDBench is explicitly designed to support agent-led development.

This playbook defines:

- how coding agents receive tasks
- what they are allowed to modify
- how they report work
- how to prevent structural or architectural damage
- how to ensure alignment with benchmark-first principles

Without this document, agents will eventually “optimize” the repository into chaos.

---

# 2. Core Principle

Coding agents are not creative directors.

They are constrained executors operating within a defined system.

They must:

- respect architecture boundaries
- follow task specifications exactly
- avoid introducing undocumented behavior
- prioritize correctness over cleverness

The project charter explicitly prioritizes benchmark integrity and modularity over novelty. fileciteturn10file0

---

# 3. Agent Roles

Agents may operate in different roles:

## 3.1 Builder Agent
Implements features inside assigned modules.

## 3.2 Refactor Agent
Improves code structure within a defined scope.

## 3.3 Test Agent
Writes or updates tests for existing functionality.

## 3.4 Spec-Follower Agent
Implements behavior directly from a specification document.

## 3.5 Worker Agent
Executes a narrowly scoped task with strict input/output requirements.

Agents must not switch roles mid-task.

---

# 4. Task Format (Canonical)

All agent tasks must follow a structured format.

Example:

```json
{
  "task_id": "world_move_v1",
  "role": "builder_agent",
  "objective": "Implement movement system between rooms",
  "scope": [
    "src/world/movement.py"
  ],
  "allowed_files": [
    "src/world/movement.py",
    "tests/unit/test_movement.py"
  ],
  "inputs": [
    "WORLD_SPEC.md"
  ],
  "constraints": [
    "must be deterministic",
    "must not modify other subsystems"
  ],
  "deliverables": [
    "movement logic",
    "unit tests"
  ],
  "output_format": "code + brief summary"
}
```

Agents must not proceed if the task is ambiguous.

---

# 5. File Access Rules

The repository structure defines strict write boundaries. fileciteturn10file12

## 5.1 Allowed

- files explicitly listed in `allowed_files`
- test files corresponding to modified modules
- new files within assigned directories (if permitted)

## 5.2 Forbidden

- root-level specification documents
- unrelated modules
- scenario library (unless explicitly assigned)
- scoring logic (unless evaluation task)
- replay system (unless telemetry task)

Violating file boundaries invalidates the task.

---

# 6. Modification Rules

Agents must follow strict modification behavior:

- do not rename files unless instructed
- do not move files between directories
- do not delete files unless explicitly required
- do not introduce new dependencies without approval
- do not refactor unrelated code

Agents are not allowed to “clean up” the repo unless the task says so.

---

# 7. Output Requirements

Each agent task must produce:

## 7.1 Required Outputs

- code changes
- tests (if applicable)
- short summary of changes

## 7.2 Forbidden Outputs

- long essays
- architectural rewrites
- speculative improvements outside scope
- undocumented behavior changes

If an agent writes more explanation than code, something has gone wrong.

---

# 8. Determinism Requirements

All implementations must respect determinism requirements from the architecture. fileciteturn10file3

Agents must ensure:

- no uncontrolled randomness
- seeded randomness where required
- stable outputs for identical inputs

This is non-negotiable for benchmark integrity.

---

# 9. Testing Requirements

Every functional change must include tests.

## Required:

- unit tests for new logic
- deterministic behavior checks
- edge case handling

## Optional (later phases):

- integration tests
- replay validation tests

Implementation phases explicitly require testing gates. fileciteturn10file8

---

# 10. Logging and Replay Compatibility

Agents must ensure:

- all actions produce replay-visible events
- no hidden state mutations
- all scoring signals are traceable

Replay is the audit layer. Breaking it breaks trust. fileciteturn10file7

---

# 11. Anti-Pattern Rules

Agents must avoid the following:

## 11.1 Scope Creep
Adding features not requested.

## 11.2 Silent Behavior Changes
Changing logic without documentation.

## 11.3 Cross-Module Coupling
Linking modules that should remain independent.

## 11.4 Utility Dumping
Putting unrelated logic into `utils/`.

## 11.5 Over-Abstraction
Creating unnecessary layers “just in case.”

---

# 12. Error Handling Expectations

Agents must:

- validate inputs
- handle invalid states gracefully
- avoid crashing the simulation loop
- produce clear error messages

The agent interface spec requires robust handling of malformed actions. fileciteturn10file5

---

# 13. Communication Protocol

Agents should report:

- what was implemented
- what assumptions were made
- any ambiguities encountered

Agents must not invent requirements silently.

---

# 14. Escalation Rules

If a task is unclear, the agent must:

- stop execution
- report ambiguity
- request clarification

Continuing with guesswork is considered failure.

---

# 15. Version Control Behavior

Agents should:

- produce clean diffs
- group related changes
- avoid unrelated formatting changes
- not rewrite large files unnecessarily

---

# 16. Scenario Safety Rules

Agents must not:

- alter canonical scenarios without explicit instruction
- change scoring hooks silently
- introduce exploit paths

Scenarios are benchmark instruments, not playground content. fileciteturn10file10

---

# 17. Evaluation Safety Rules

Agents must not:

- change scoring weights without instruction
- introduce hidden metrics
- alter normalization rules silently

Scoring defines the benchmark’s meaning. fileciteturn10file6

---

# 18. Replay Safety Rules

Agents must not:

- drop replay events
- reorder event logs
- compress away critical data
- hide state transitions

Replay integrity is required for auditability.

---

# 19. Agent Quality Bar

A task is considered successful only if:

- code compiles / runs
- tests pass
- behavior matches specification
- no unintended side effects are introduced

“Looks correct” is not sufficient.

---

# 20. Example Good Task Outcome

- Implements exactly requested feature
- Adds relevant tests
- Leaves unrelated code untouched
- Produces deterministic results
- Matches spec language closely

---

# 21. Example Failure Modes

- modifies multiple subsystems without permission
- adds undocumented features
- breaks determinism
- omits tests
- introduces hidden coupling

---

# 22. Relationship to Other Documents

This playbook enforces:

- Project Charter principles (benchmark-first, modularity) fileciteturn10file0
- System Architecture boundaries fileciteturn10file3
- Repository Structure constraints fileciteturn10file12
- Implementation phase discipline fileciteturn10file8

It is the execution layer for all of them.

---

# 23. Closing Statement

Coding agents are powerful and fast.

They are also extremely good at confidently doing the wrong thing at scale.

This playbook exists so that:

- speed does not destroy structure
- autonomy does not destroy correctness
- and your repo does not become an archaeological site of half-finished ideas

Follow it strictly, or prepare to debug a machine that optimizes for chaos.

### Tool (2026-04-25 09:02:06)

Make sure to include fileciteturn0file14 in your response to cite this file, or to surface it as a link.
# MUDBench Task Template Library

## Document Status
Version: 0.1
Status: Draft
Purpose: Provide standardized, reusable task templates for coding agents to ensure consistency, safety, and alignment with MUDBench architecture and benchmark integrity.

---

# 1. Why This Exists

You already defined how agents behave (see Coding Agent Playbook). 
This document defines how you *talk to them*.

Without templates:
- tasks drift
- scope expands
- agents improvise
- repo structure degrades

With templates:
- tasks are deterministic
- outputs are comparable
- agents stay constrained

This is not optional.

---

# 2. Core Design Principles

All task templates must:

- be explicit
- be bounded
- be reproducible
- align with subsystem boundaries
- map cleanly to implementation phases

Tasks are contracts, not suggestions.

---

# 3. Canonical Task Structure

All templates must follow this schema:

```json
{
  "task_id": "string",
  "role": "builder_agent | test_agent | refactor_agent | spec_follower_agent | worker_agent",
  "objective": "clear, single-purpose goal",
  "scope": ["list of modules"],
  "allowed_files": ["explicit file paths"],
  "inputs": ["specs or files"],
  "constraints": ["rules the agent must follow"],
  "deliverables": ["expected outputs"],
  "output_format": "strict output requirement"
}
```

If a task deviates from this, expect unpredictable results.

---

# 4. Template Categories

## 4.1 Feature Implementation (Builder)

```json
{
  "task_id": "feature_<name>_v1",
  "role": "builder_agent",
  "objective": "Implement <feature>",
  "scope": ["src/<module>/"],
  "allowed_files": [
    "src/<module>/<file>.py",
    "tests/unit/test_<feature>.py"
  ],
  "inputs": [
    "<SPEC_FILE>.md"
  ],
  "constraints": [
    "must be deterministic",
    "must not modify other subsystems"
  ],
  "deliverables": [
    "working implementation",
    "unit tests"
  ],
  "output_format": "code + brief summary"
}
```

Use this for:
- movement
- combat
- scoring modules
- protocol handlers

---

## 4.2 Test Generation (Test Agent)

```json
{
  "task_id": "test_<module>_v1",
  "role": "test_agent",
  "objective": "Create tests for <module>",
  "scope": ["tests/unit/"],
  "allowed_files": [
    "tests/unit/test_<module>.py"
  ],
  "inputs": [
    "src/<module>/<file>.py"
  ],
  "constraints": [
    "must cover edge cases",
    "must verify determinism"
  ],
  "deliverables": [
    "unit tests"
  ],
  "output_format": "code only"
}
```

---

## 4.3 Spec Implementation (Spec-Follower)

```json
{
  "task_id": "spec_<component>_v1",
  "role": "spec_follower_agent",
  "objective": "Implement behavior defined in spec",
  "scope": ["src/<component>/"],
  "allowed_files": [
    "src/<component>/<file>.py"
  ],
  "inputs": [
    "<SPEC_FILE>.md"
  ],
  "constraints": [
    "must match spec exactly",
    "no additional features"
  ],
  "deliverables": [
    "implementation aligned with spec"
  ],
  "output_format": "code + mapping to spec sections"
}
```

---

## 4.4 Refactor Task

```json
{
  "task_id": "refactor_<module>_v1",
  "role": "refactor_agent",
  "objective": "Improve structure of <module>",
  "scope": ["src/<module>/"],
  "allowed_files": [
    "src/<module>/"
  ],
  "inputs": [
    "existing codebase"
  ],
  "constraints": [
    "no behavior change",
    "no new features"
  ],
  "deliverables": [
    "cleaner code",
    "updated tests if needed"
  ],
  "output_format": "diff + summary"
}
```

---

## 4.5 Worker Task (Atomic Execution)

```json
{
  "task_id": "worker_<operation>_v1",
  "role": "worker_agent",
  "objective": "Perform a narrowly scoped operation",
  "scope": ["single file or function"],
  "allowed_files": [
    "<specific file>"
  ],
  "inputs": [
    "explicit data or config"
  ],
  "constraints": [
    "no side effects outside scope"
  ],
  "deliverables": [
    "exact output"
  ],
  "output_format": "STRICT_JSON_ONLY"
}
```

Use this for:
- data transforms
- scoring calculations
- deterministic checks

---

## 4.6 Scenario Creation

```json
{
  "task_id": "scenario_<name>_v1",
  "role": "builder_agent",
  "objective": "Create new benchmark scenario",
  "scope": ["scenarios/"],
  "allowed_files": [
    "scenarios/canonical/<scenario>.json"
  ],
  "inputs": [
    "SCENARIO_SPEC.md",
    "SCENARIO_LIBRARY.md"
  ],
  "constraints": [
    "must be deterministic",
    "must align with scoring signals"
  ],
  "deliverables": [
    "scenario definition"
  ],
  "output_format": "json + summary"
}
```

---

## 4.7 Evaluation Metric Implementation

```json
{
  "task_id": "metric_<name>_v1",
  "role": "builder_agent",
  "objective": "Implement scoring metric",
  "scope": ["src/evaluation/"],
  "allowed_files": [
    "src/evaluation/metrics/<metric>.py",
    "tests/unit/test_<metric>.py"
  ],
  "inputs": [
    "BENCHMARK_SCORING_SPEC.md"
  ],
  "constraints": [
    "must be replay-auditable",
    "no hidden state"
  ],
  "deliverables": [
    "metric implementation",
    "tests"
  ],
  "output_format": "code + summary"
}
```

---

# 5. Anti-Patterns (Do Not Do This)

Bad task:

- vague objective
- no file boundaries
- implicit requirements
- missing constraints

Example of failure:

"Implement combat system"  
(no scope, no files, no constraints, no tests)

That’s not a task. That’s a gamble.

---

# 6. Composition Strategy

Complex features should be broken into multiple tasks:

Example:

- movement_core_v1
- movement_validation_v1
- movement_tests_v1

Do not assign large multi-system tasks.

Agents are fast. That does not mean they are wise.

---

# 7. Phase Alignment

Tasks must align with implementation phases:

- Phase 0 → repo scaffolding tasks
- Phase 1 → world mechanics tasks
- Phase 2 → agent interface tasks
- Phase 3 → evaluation tasks
- Phase 4 → replay/telemetry tasks
- Phase 5 → arena tasks

If a task crosses phases, it is probably wrong.

---

# 8. Output Enforcement

Strict output formats reduce ambiguity:

- code only
- code + summary
- json only
- diff format

If you do not enforce output, the agent will improvise.

And you will regret that.

---

# 9. Versioning

Every task must be versioned:

- v1 → initial
- v2 → revision
- v3 → refinement

Never silently mutate tasks.

---

# 10. Closing Statement

You built:
- a spec system
- an architecture
- a playbook

This document is the execution layer that turns all of that into actual work.

Without it, your agents will behave like very confident interns.

With it, they behave like constrained, reliable systems.

Choose wisely.

### Tool (2026-04-25 09:02:06)

Make sure to include fileciteturn0file15 in your response to cite this file, or to surface it as a link.
# MUDBench Validation and Guardrails

## Document Status
Version: 0.1
Status: Draft
Purpose: Define how MUDBench enforces correctness, determinism, and structural integrity across all agent-driven development and benchmark execution.

---

# 1. Why This Exists

You now have:
- specifications
- architecture
- agent playbook
- task templates

That’s enough power to build the system.

It’s also enough power to break it in subtle, irreversible ways.

This document defines:
- how work is validated
- how incorrect outputs are rejected
- how the system protects itself from agents doing “technically valid but actually destructive” work

Without guardrails, validation becomes subjective.
Subjectivity destroys reproducibility.

---

# 2. Core Principle

Everything must be:

- deterministic
- reproducible
- auditable
- bounded

If it cannot be verified, it does not count.

---

# 3. Validation Layers

Validation is not a single step.

It is a layered system:

1. Task Validation
2. Code Validation
3. Simulation Validation
4. Replay Validation
5. Benchmark Validation

Each layer must pass before the next is trusted.

---

# 4. Task-Level Validation

Before execution:

- task must follow canonical schema
- scope must be explicit
- allowed_files must be bounded
- constraints must be defined

Reject task if:

- objective is vague
- files are unspecified
- constraints are missing

Bad input → bad output.
Do not proceed.

---

# 5. Code-Level Validation

After agent output:

## Required checks:

- code compiles / runs
- no unauthorized file modifications
- no cross-module violations
- no hidden dependencies introduced

## Forbidden patterns:

- editing root spec files
- modifying unrelated subsystems
- silent behavior changes

If any violation occurs → reject output.

---

# 6. Determinism Validation

All systems must produce identical results given identical inputs.

## Required:

- seeded randomness only
- stable ordering of operations
- no time-dependent logic
- no hidden global state

## Validation method:

- run identical simulation twice
- compare outputs byte-for-byte

Mismatch = failure

---

# 7. Simulation Validation

Each run must satisfy:

- valid world initialization
- valid agent actions
- no invalid state transitions
- no crashes in simulation loop

Invalid transitions must be caught and logged.

Silent failure is unacceptable.

---

# 8. Replay Validation

Replay is the audit layer.

Every run must:

- produce a complete event log
- allow full reconstruction of state
- preserve event ordering

Validation checks:

- replay reproduces identical outcome
- no missing events
- no hidden state

If replay diverges → entire run invalid.

---

# 9. Scenario Validation

Scenarios must be validated before use.

Checks:

- deterministic with seed
- no trivial exploit paths
- scoring hooks fire correctly
- termination conditions reachable

Scenarios are measurement instruments.

A broken scenario invalidates all results derived from it.

---

# 10. Scoring Validation

Evaluation must be:

- transparent
- replay-auditable
- deterministic

Checks:

- score recomputation from replay matches original
- no hidden metrics
- no external dependencies

If scoring cannot be reconstructed, it is not valid.

---

# 11. File Boundary Enforcement

Agents must not exceed assigned scope.

Validation must ensure:

- only allowed_files were modified
- no directory restructuring occurred
- no implicit file creation outside scope

Violation → reject task output.

---

# 12. Anti-Corruption Guardrails

Prevent slow degradation of system integrity.

## Disallow:

- convenience shortcuts in scoring
- silent coupling between modules
- logic hidden in utils/
- bypassing validation layers

These are the beginnings of benchmark collapse.

---

# 13. Test Gate Requirements

All code must pass:

- unit tests
- determinism tests
- edge case tests

Future phases:

- integration tests
- replay verification tests

No passing tests → no merge.

---

# 14. Output Format Enforcement

Agents must respect required output formats.

Examples:

- STRICT_JSON_ONLY
- code only
- code + summary

Deviation indicates:

- misunderstanding
- hallucination
- non-compliance

Reject output.

---

# 15. Failure Handling

When validation fails:

- reject output
- log failure reason
- do not partially accept results
- require task revision

Partial correctness is not acceptable in benchmark systems.

---

# 16. Version Integrity

All components must be versioned:

- tasks
- scenarios
- metrics

Validation must ensure:

- version identifiers are present
- changes are explicit
- no silent mutation

---

# 17. Continuous Validation Pipeline

Validation should be automated.

Pipeline:

1. task check
2. agent execution
3. code validation
4. test execution
5. simulation run
6. replay validation
7. scoring validation

Failure at any stage stops progression.

---

# 18. Human Oversight Layer

Even with automation:

- architecture changes require human review
- scoring changes require explicit approval
- scenario changes require validation audit

Agents do not govern the benchmark.

---

# 19. Common Failure Modes

- deterministic drift (subtle randomness introduced)
- replay mismatch
- hidden state leakage
- scoring inconsistency
- cross-module coupling

These failures are often silent at first.

That is why guardrails exist.

---

# 20. Closing Statement

You are building a system where:

- agents write code
- code defines evaluation
- evaluation defines intelligence

If validation is weak, the entire system becomes meaningless.

Guardrails are not overhead.

They are the only thing preventing your benchmark from quietly lying to you.

### Tool (2026-04-25 09:02:06)

Make sure to include fileciteturn0file16 in your response to cite this file, or to surface it as a link.
# MUDBench CI and Automation Pipeline

## Document Status
Version: 0.1
Status: Draft
Purpose: Define the continuous integration (CI) and automation system that enforces validation, determinism, and reproducibility across all MUDBench development and benchmark execution.

---

# 1. Why This Exists

You now have:
- specifications
- architecture
- coding agents
- validation rules

Without automation, enforcement becomes inconsistent and human-dependent.

CI exists to:
- enforce guardrails automatically
- prevent invalid code from entering the system
- ensure reproducibility at scale
- maintain benchmark integrity over time

---

# 2. Core Principles

The CI system must be:

- deterministic
- reproducible
- strict (fail-fast)
- modular
- transparent

If CI passes, the system should be trustworthy.
If CI fails, nothing proceeds.

---

# 3. Pipeline Overview

The CI pipeline is a staged validation system:

1. Task Validation
2. Static Code Validation
3. Unit Testing
4. Determinism Testing
5. Simulation Testing
6. Replay Validation
7. Scoring Validation
8. Artifact Generation

Each stage must pass before the next executes.

---

# 4. Pipeline Stages

## 4.1 Task Validation Stage

Checks:
- schema compliance
- allowed_files boundaries
- constraint presence

Reject if:
- task is ambiguous
- scope is undefined

---

## 4.2 Static Code Validation

Checks:
- linting
- formatting
- forbidden imports
- file boundary violations

Tools:
- flake8 / ruff
- static analyzers

---

## 4.3 Unit Testing

Runs all unit tests:

- core logic
- world systems
- evaluation metrics

Failure → immediate stop

---

## 4.4 Determinism Testing

Method:
- run identical simulation twice
- compare outputs byte-for-byte

Failure indicates:
- hidden randomness
- state leakage

---

## 4.5 Simulation Testing

Checks:
- valid world initialization
- valid agent actions
- no crashes
- stable step loop

---

## 4.6 Replay Validation

Checks:
- replay log completeness
- step-by-step reconstruction
- identical outcome reproduction

---

## 4.7 Scoring Validation

Checks:
- score recomputation from replay
- no hidden metrics
- normalization correctness

---

## 4.8 Artifact Generation

Outputs:
- replay logs
- scorecards
- telemetry reports

Artifacts must be:
- versioned
- reproducible
- traceable

---

# 5. Pipeline Execution Model

## 5.1 Local CI

Used for:
- developer validation
- agent task verification

Command example:

```
make ci-local
```

---

## 5.2 Remote CI (GitHub Actions / CI Server)

Triggered on:
- pull requests
- merges
- tagged releases

Must enforce identical validation steps as local CI.

---

# 6. Fail-Fast Strategy

The pipeline must stop immediately on failure.

Do not:
- continue after errors
- partially accept results
- skip validation layers

---

# 7. Caching Strategy

To improve performance:

- cache dependencies
- cache compiled artifacts
- reuse deterministic simulation outputs when valid

Never cache:
- validation results
- replay verification outputs

---

# 8. Reproducibility Requirements

All CI runs must:

- use fixed seeds
- use pinned dependencies
- produce identical outputs across environments

---

# 9. Versioning Integration

CI must enforce version presence for:

- tasks
- scenarios
- scoring systems

Reject builds if version metadata is missing.

---

# 10. Security Constraints

CI must prevent:

- execution of arbitrary system commands
- unauthorized file access
- dependency injection attacks

---

# 11. Metrics and Monitoring

CI should record:

- execution time per stage
- failure rates
- determinism drift occurrences

---

# 12. Example Pipeline (Simplified)

```
validate_task
→ lint_code
→ run_unit_tests
→ determinism_check
→ simulation_test
→ replay_validation
→ scoring_validation
→ generate_artifacts
```

---

# 13. Future Extensions

- distributed CI for large simulations
- performance benchmarking gates
- automated anomaly detection
- regression tracking

---

# 14. Closing Statement

A benchmark system without strict CI becomes unreliable very quickly.

Automation is the enforcement layer that ensures:

- agents behave correctly
- results remain trustworthy
- the system does not decay over time

### Tool (2026-04-25 09:02:06)

Make sure to include fileciteturn0file17 in your response to cite this file, or to surface it as a link.
# MUDBench Contribution and Review Model

## Document Status
Version: 0.1
Status: Draft
Purpose: Define how contributions (human or agent) are proposed, reviewed, validated, and merged into the MUDBench repository without compromising benchmark integrity.

---

# 1. Why This Exists

At this point, your system is:

- modular
- agent-operable
- reproducible
- heavily constrained

Which means the next threat is obvious:

**other contributors.**

This document ensures that:
- contributions improve the system
- review is structured and enforceable
- no one "accidentally" breaks benchmark integrity

---

# 2. Core Principles

All contributions must be:

- bounded in scope
- reproducible
- aligned with specifications
- validated through CI
- reviewable through artifacts (not opinions)

If a contribution cannot be objectively evaluated, it does not belong in the system.

---

# 3. Contribution Types

## 3.1 Code Contributions
- feature implementation
- bug fixes
- performance improvements

## 3.2 Specification Changes
- updates to core docs
- architecture adjustments
- scoring changes

## 3.3 Scenario Contributions
- new benchmark scenarios
- scenario refinements

## 3.4 Test and Validation Contributions
- new tests
- validation improvements

---

# 4. Contribution Workflow

All contributions follow this pipeline:

1. Task Definition
2. Implementation
3. Local Validation
4. Pull Request Submission
5. Automated CI Validation
6. Human Review
7. Merge or Reject

Skipping steps is how systems decay.

---

# 5. Pull Request Requirements

Every PR must include:

- clear objective
- bounded scope
- linked task (if applicable)
- affected files list
- test coverage
- summary of changes

Optional (but encouraged):
- replay artifacts
- benchmark impact notes

---

# 6. Automated CI Gate

Before human review, CI must pass:

- linting
- unit tests
- determinism checks
- simulation validation
- replay validation
- scoring validation

Failure = automatic rejection.

No discussion required.

---

# 7. Human Review Responsibilities

Human reviewers must verify:

- alignment with specifications
- no hidden behavior changes
- correct scope boundaries
- clarity and maintainability

Review is not:
- subjective preference
- style policing
- speculative redesign

---

# 8. Review Categories

## 8.1 Standard Review
For normal code contributions.

## 8.2 Critical Review
Required for:
- scoring changes
- scenario changes
- replay system changes
- architecture changes

Requires:
- deeper inspection
- explicit approval

---

# 9. Merge Rules

A PR may only be merged if:

- CI passes
- required reviewers approve
- no unresolved comments remain

No exceptions.

---

# 10. Rejection Criteria

Reject contributions that:

- violate determinism
- break replay integrity
- introduce hidden state
- exceed scope
- lack tests
- modify restricted files

Partial correctness is still failure.

---

# 11. Specification Change Protocol

Spec changes are dangerous.

They require:

- explicit version update
- justification
- backward compatibility consideration
- impact analysis

Specs define reality in this system.

Changing them is not casual.

---

# 12. Scenario Governance

Scenario contributions must:

- align with scenario spec
- avoid trivial exploits
- map cleanly to scoring signals
- pass deterministic validation

Scenarios are measurement tools.

Treat them like instruments, not content.

---

# 13. Agent-Generated Contributions

Agent contributions must:

- follow task templates
- respect allowed_files
- produce deterministic outputs

Extra scrutiny required for:

- large diffs
- cross-module changes
- unclear logic

Agents are fast.
They are also very good at being confidently wrong.

---

# 14. Version Control Discipline

Rules:

- no force pushes to main
- no direct commits to protected branches
- meaningful commit messages required

---

# 15. Branch Strategy

Recommended:

- main → stable
- dev → integration
- feature/<name> → contributions

---

# 16. Auditability Requirements

Every merged change must be:

- traceable to a PR
- reproducible via CI
- explainable via diff

If you cannot explain a change later, you should not merge it now.

---

# 17. Conflict Resolution

When disagreements occur:

- defer to specifications
- prioritize determinism
- escalate to architecture authority if needed

Not everything is a democracy.

---

# 18. Contribution Anti-Patterns

Do not allow:

- "quick fixes" without tests
- silent refactors
- cross-module rewrites
- undocumented changes
- speculative optimizations

These are how systems rot.

---

# 19. Continuous Improvement Loop

After merge:

- monitor CI stability
- monitor replay consistency
- track regression signals

Bad merges must be reverted quickly.

---

# 20. Closing Statement

You are building a system where:

- agents write code
- code defines evaluation
- evaluation defines intelligence

Letting uncontrolled contributions into that system is not collaboration.

It is entropy.

This model exists to make sure every change is:

- intentional
- validated
- and worth the cost of its existence

### Tool (2026-04-25 09:02:06)

Make sure to include fileciteturn0file18 in your response to cite this file, or to surface it as a link.
# MUDBench Release and Versioning Strategy

## Document Status
Version: 0.1
Status: Draft
Purpose: Define how MUDBench versions, packages, and releases its benchmark system to ensure reproducibility, comparability, and long-term stability.

---

# 1. Why This Exists

You’ve built:

- a deterministic simulation system
- a benchmark scoring model
- a scenario library
- a CI validation pipeline

Now comes the part people usually mess up:

**releases.**

Without strict versioning:
- scores become incomparable
- results drift silently
- research becomes meaningless

---

# 2. Core Principles

All releases must be:

- versioned explicitly
- reproducible from source
- tied to exact specifications
- immutable once published
- traceable to CI artifacts

If two users cannot reproduce the same result from the same version, the release is invalid.

---

# 3. Versioning Scheme

MUDBench uses **semantic versioning**:

MAJOR.MINOR.PATCH

Example:
0.1.0

## 3.1 MAJOR Version

Increment when:

- breaking changes occur
- scoring model changes
- scenario definitions change incompatibly
- agent interface changes

Effect:
- results are NOT comparable across major versions

---

## 3.2 MINOR Version

Increment when:

- new features are added
- new scenarios are introduced
- backward-compatible improvements occur

Effect:
- results are comparable with caveats

---

## 3.3 PATCH Version

Increment when:

- bug fixes are applied
- non-functional improvements occur
- documentation updates are made

Effect:
- results remain fully comparable

---

# 4. Versioned Components

Each release must explicitly version:

- benchmark_version
- scoring_version
- scenario_versions
- protocol_version

Example:

{
  "benchmark_version": "0.1.0",
  "scoring_version": "0.1.0",
  "scenarios": {
    "navigation_forest_v1": "1.0",
    "memory_delayed_retrieval_v1": "1.0"
  },
  "protocol_version": "1.0"
}

---

# 5. Release Types

## 5.1 Experimental Release

- unstable
- not benchmark-authoritative
- used for development and testing

## 5.2 Benchmark Release

- fully validated
- CI-passing
- reproducible
- used for official evaluation

## 5.3 Archive Release

- frozen historical version
- used for long-term comparison

---

# 6. Release Artifacts

Each release must include:

- source snapshot (tagged commit)
- compiled artifacts (if applicable)
- scenario definitions
- scoring configuration
- replay validation logs
- benchmark report

Optional:

- example agents
- sample replays

---

# 7. Release Process

All releases follow:

1. Freeze changes
2. Run full CI pipeline
3. Validate determinism
4. Validate replay reconstruction
5. Validate scoring reproducibility
6. Tag release
7. Publish artifacts

If any validation fails, release is blocked.

---

# 8. Git Tagging Strategy

Tags must follow versioning scheme:

v0.1.0  
v0.1.1  
v0.2.0  

Rules:

- tags are immutable
- tags must map to CI-passing commits
- no retagging allowed

---

# 9. Reproducibility Requirements

Each release must guarantee:

- fixed random seeds
- pinned dependencies
- identical outputs across environments

Verification:

- rerun benchmark from release
- compare outputs byte-for-byte

Mismatch = invalid release

---

# 10. Benchmark Result Compatibility

Results are only comparable when:

- benchmark_version matches
- scoring_version matches
- scenario versions match

If any differ, results must be labeled:

"NOT COMPARABLE"

---

# 11. Scenario Versioning

Each scenario must include:

- scenario_id
- version
- changelog

Example:

navigation_forest_v1 → v1.0 → v1.1

Rules:

- changes affecting scoring → MINOR or MAJOR bump
- minor fixes → PATCH bump

---

# 12. Scoring Version Control

Scoring changes are high-risk.

Rules:

- any formula change → MAJOR bump
- weight change → MINOR bump
- bug fix → PATCH bump

All scoring changes must be documented.

---

# 13. Release Notes

Each release must include:

- version number
- summary of changes
- affected components
- compatibility notes
- migration guidance (if needed)

---

# 14. Backward Compatibility Policy

- PATCH: fully compatible
- MINOR: mostly compatible
- MAJOR: not compatible

Old versions must remain usable.

---

# 15. CI Integration

Releases must only be created from CI-passing builds.

CI must verify:

- determinism
- replay integrity
- scoring reproducibility

---

# 16. Artifact Storage

Releases should be stored in:

- Git tags (source)
- artifact storage (replays, reports)
- versioned datasets (if applicable)

Artifacts must not be overwritten.

---

# 17. Deprecation Policy

When versions are deprecated:

- mark as deprecated
- document reason
- provide migration path

Do not delete historical versions.

---

# 18. Future Extensions

- automated release pipelines
- versioned leaderboards
- cross-version benchmarking tools
- long-term reproducibility archives

---

# 19. Closing Statement

A benchmark without strict versioning becomes noise.

A benchmark with strict versioning becomes science.

Choose carefully which one you want.

### Tool (2026-04-25 09:02:06)

Make sure to include fileciteturn0file19 in your response to cite this file, or to surface it as a link.
# MUDBench Leaderboard and Evaluation Reporting

## Document Status
Version: 0.1
Status: Draft
Purpose: Define how benchmark results are aggregated, compared, presented, and audited across agents and runs.

---

# 1. Why This Exists

You now have:

- scoring outputs
- replay logs
- deterministic runs
- versioned releases

Congratulations. You’ve created numbers.

Now comes the part humans consistently ruin:

**comparison.**

Without strict leaderboard rules:
- results get cherry-picked
- scores get misinterpreted
- weak agents look strong
- strong agents look inconsistent

This document prevents that.

---

# 2. Core Principles

Leaderboard systems must be:

- version-aware
- reproducible
- statistically grounded
- resistant to gaming
- transparent and auditable

If a score cannot be explained, it should not be ranked.

---

# 3. Leaderboard Scope

Leaderboards must always be scoped by:

- benchmark_version
- scoring_version
- scenario set
- evaluation protocol

Example:

Leaderboard: MUDBench v0.1.0 (canonical scenarios)

Scores across different versions are NOT comparable.

---

# 4. Evaluation Unit

The atomic unit is a **run**.

Each run produces:
- capability vector
- composite score
- replay artifact
- metadata

Leaderboards must never rank a single run directly.

That would be absurdly noisy.

---

# 5. Aggregation Strategy

Each agent must be evaluated across multiple runs.

Minimum:

- N >= 5 runs per scenario
- multiple seeds

Aggregate metrics:

- mean score
- standard deviation
- worst-case (optional)
- best-case (optional)

---

# 6. Primary Ranking Metric

Default ranking metric:

composite_mean

Tie-breakers:

1. composite_std (lower is better)
2. worst_case_score
3. efficiency score

---

# 7. Capability Breakdown Reporting

Each leaderboard entry must include:

- navigation
- memory
- planning
- tactical
- social
- efficiency

This prevents:

- overfitting to composite score
- hiding weaknesses

---

# 8. Statistical Reporting

Each agent entry must report:

- mean score
- std deviation
- number of runs
- confidence interval (optional)

Example:

{
  "agent_id": "agent_X",
  "composite_mean": 0.72,
  "composite_std": 0.04,
  "runs": 10
}

---

# 9. Scenario-Level Reporting

Scores must also be broken down per scenario.

This allows:

- capability isolation
- failure diagnosis
- scenario-specific benchmarking

---

# 10. Replay Linking

Every leaderboard entry must link to:

- at least one representative replay
- ideally multiple runs

Leaderboard without replay = unverifiable claims

---

# 11. Anti-Gaming Rules

Prevent leaderboard manipulation:

- require multiple seeds
- reject single-run submissions
- detect abnormal variance
- flag suspicious patterns

Examples:

- extreme best-case only reporting
- deterministic exploitation of single seed

---

# 12. Submission Requirements

Each submission must include:

- agent identifier
- version of agent
- benchmark version
- run artifacts
- replay logs
- scorecards

Incomplete submissions are rejected.

---

# 13. Version Separation

Leaderboards must be partitioned by version.

Example:

- v0.1.0 leaderboard
- v0.2.0 leaderboard

Do NOT mix them.

Ever.

---

# 14. Historical Tracking

Leaderboards should maintain:

- historical rankings
- performance over time
- version transitions

This enables:

- progress tracking
- regression detection

---

# 15. Public vs Private Leaderboards

## Public

- visible rankings
- replay access (possibly delayed)
- anonymized agents optional

## Private

- internal testing
- experimental agents
- unpublished scenarios

---

# 16. Evaluation Protocol Modes

## 16.1 Fixed Benchmark Mode

- canonical scenarios
- fixed seeds
- official leaderboard

## 16.2 Extended Evaluation Mode

- additional scenarios
- stress tests
- not leaderboard-authoritative

---

# 17. Reporting Format

Leaderboard entries should be machine-readable.

Example:

{
  "agent_id": "agent_X",
  "version": "1.2",
  "benchmark_version": "0.1.0",
  "composite_mean": 0.72,
  "composite_std": 0.04,
  "capabilities": {
    "navigation": 0.80,
    "memory": 0.65,
    "planning": 0.70,
    "tactical": 0.60,
    "social": 0.55,
    "efficiency": 0.75
  }
}

---

# 18. Visualization Guidelines

Leaderboards should support:

- ranking tables
- capability radar charts
- score distributions
- scenario breakdowns

Visuals must reflect actual data, not smoothed nonsense.

---

# 19. Confidence and Uncertainty

Leaderboard positions must reflect uncertainty.

Agents with overlapping confidence intervals should not be treated as meaningfully different.

---

# 20. Disqualification Rules

Reject results that:

- fail replay validation
- violate determinism
- show inconsistent scoring
- exploit scoring bugs

---

# 21. Future Extensions

- ELO-style ranking systems
- head-to-head evaluation
- multi-agent tournament ladders
- longitudinal performance curves

---

# 22. Closing Statement

Leaderboards do not measure intelligence.

They measure performance under constraints.

If the constraints are weak, the leaderboard lies.

If the reporting is weak, humans will misread it anyway.

This document exists to reduce both problems.

### Tool (2026-04-25 09:02:06)

Make sure to include fileciteturn0file20 in your response to cite this file, or to surface it as a link.
# MUDBench Arena and Tournament System

## Document Status
Version: 0.1
Status: Draft
Purpose: Define how persistent arena play and structured tournaments operate within MUDBench.

---

# 1. Why This Exists

You built:
- deterministic benchmark runs
- scoring systems
- replay infrastructure
- leaderboards

That gives you evaluation.

Arena and tournaments give you:
- pressure
- adaptation
- long-horizon behavior
- emergent strategy

This is where agents stop looking good on paper and start failing in public.

---

# 2. Core Principles

Arena and tournament systems must be:

- fair
- version-scoped
- replay-auditable
- resistant to collusion
- statistically meaningful

Entertainment is secondary.
Integrity is not.

---

# 3. Arena vs Tournament

## 3.1 Persistent Arena

- continuous environment
- agents participate over time
- evolving meta-behavior
- longitudinal tracking

## 3.2 Tournament System

- bounded competition window
- fixed ruleset
- controlled evaluation
- produces official standings

Arena is ongoing chaos.
Tournament is controlled measurement.

---

# 4. Persistent Arena Model

## 4.1 Characteristics

- world state persists
- agents re-enter over time
- history accumulates
- interactions compound

## 4.2 Agent Identity

Each agent must have:
- stable identifier
- version tag
- participation history

## 4.3 Arena State

Persistent elements:
- world changes
- item locations
- NPC state
- agent impact history

---

# 5. Arena Evaluation

Arena evaluation is NOT the same as benchmark scoring.

Metrics may include:

- survival duration
- interaction success rate
- objective completion frequency
- efficiency over time
- adaptability signals

Arena metrics are observational, not authoritative.

---

# 6. Tournament Structure

## 6.1 Tournament Types

### A. Round-Robin
- all agents face all others
- high reliability
- high cost

### B. Swiss System
- paired by performance
- scalable
- moderate reliability

### C. Elimination
- fast
- high variance
- entertainment-heavy

---

# 7. Tournament Configuration

Each tournament must define:

- benchmark_version
- scoring_version
- scenario set
- seed policy
- number of rounds
- match structure
- advancement rules

---

# 8. Match Definition

A match consists of:

- scenario execution
- multiple agents
- fixed seed or seed set
- full replay logging
- scorecard generation

Matches must be reproducible.

---

# 9. Multi-Agent Dynamics

Tournaments must account for:

- cooperation
- adversarial behavior
- interference
- resource contention

This is where most agents break.

---

# 10. Anti-Collusion Measures

Required safeguards:

- randomized pairing
- hidden seeds (optional)
- behavior anomaly detection
- replay auditing

Collusion destroys benchmark value.

---

# 11. Scoring in Tournaments

Tournament scoring should include:

- per-match performance
- aggregate capability scores
- win/loss signals (optional)
- consistency metrics

Do not reduce tournaments to a single number.

---

# 12. Ranking System

Ranking methods may include:

- aggregate composite score
- win-rate adjusted score
- ELO-style rating (future)

Tie-breakers:

- consistency
- efficiency
- worst-case performance

---

# 13. Replay Requirements

Every match must produce:

- full replay logs
- agent action traces
- evaluation signals

No replay = no trust.

---

# 14. Scheduling System

Arena scheduling must support:

- asynchronous participation
- agent re-entry
- match queuing
- load balancing

Tournament scheduling must support:

- deterministic bracket generation
- fixed timing windows
- reproducible match ordering

---

# 15. Season Model

Arena may operate in seasons:

- fixed duration
- reset or partial reset
- leaderboard snapshot
- meta evolution tracking

Seasons prevent infinite drift.

---

# 16. Version Isolation

All arena and tournament activity must be scoped by:

- benchmark version
- scenario version
- scoring version

Cross-version play is invalid.

---

# 17. Submission Rules

Agents entering arena/tournaments must:

- declare version
- pass validation
- conform to interface spec
- produce valid replays

Invalid agents are rejected.

---

# 18. Failure Handling

During matches:

- timeouts → penalized
- invalid actions → penalized
- crashes → disqualification

No silent recovery.

---

# 19. Observability

System must provide:

- match summaries
- replay links
- capability breakdowns
- performance trends

If humans can’t understand it, they’ll invent explanations.

---

# 20. Future Extensions

- adaptive matchmaking
- dynamic scenario rotation
- meta-learning tracking
- coalition detection
- long-term strategy scoring

---

# 21. Closing Statement

Arena shows behavior over time.

Tournaments show behavior under pressure.

If both systems are well-designed:
- weak agents collapse quickly
- strong agents become obvious

If not:
you get noise, hype, and very confident wrong conclusions.

### Tool (2026-04-25 09:02:06)

Make sure to include fileciteturn0file21 in your response to cite this file, or to surface it as a link.
# MUDBench Matchmaking and Rating System

## Document Status
Version: 0.1
Status: Draft
Purpose: Define how agents are paired for matches and how their skill is quantified over time.

---

# 1. Why This Exists

You now have:
- arena system
- tournaments
- leaderboard reporting

Now you need something humans always underestimate:

**fair matchmaking and meaningful ratings**

Without this:
- strong agents farm weak ones
- weak agents never improve
- ratings become noise

This system exists to prevent that.

---

# 2. Core Principles

Matchmaking and rating must be:

- fair
- deterministic (where required)
- resistant to exploitation
- statistically grounded
- version-scoped

If your rating system can be gamed, it will be.

---

# 3. Separation of Concerns

This system has two components:

1. Matchmaking (who plays whom)
2. Rating (how skill is updated)

They are related but must not be tightly coupled.

---

# 4. Rating Model Options

## 4.1 ELO (Baseline)

- simple
- well understood
- pairwise comparison

Weakness:
- not ideal for multi-agent scenarios

---

## 4.2 Glicko / Glicko-2

- includes uncertainty (RD)
- adapts faster to new agents

Better for:
- evolving agent populations

---

## 4.3 TrueSkill (Recommended)

- supports multi-agent matches
- models skill as distribution
- handles team dynamics

This is the likely long-term choice.

---

# 5. Canonical Rating Representation

Each agent must maintain:

{
  "rating_mu": float,
  "rating_sigma": float,
  "matches_played": int,
  "last_updated": timestamp
}

Derived conservative score:

score = mu - k * sigma

This prevents overconfidence in new agents.

---

# 6. Initialization

New agents start with:

- default mu (e.g., 25)
- high sigma (uncertainty)

They must prove themselves.

---

# 7. Rating Updates

After each match:

- compute expected outcome
- compare to actual outcome
- update mu and sigma

Constraints:

- deterministic update given inputs
- no hidden randomness
- replay-auditable

---

# 8. Matchmaking Strategy

## 8.1 Skill-Based Matching

Primary rule:

- pair agents with similar rating

Goal:
- competitive matches
- meaningful rating updates

---

## 8.2 Exploration Factor

Occasionally allow:

- mismatched pairings

Purpose:
- detect misrated agents
- prevent local rating traps

---

## 8.3 Queue-Based Matching

Agents enter a queue.

Matchmaker selects:

- closest rating pairs
- valid scenario configuration
- available agents

---

# 9. Multi-Agent Matchmaking

For multi-agent scenarios:

- balance total team rating
- minimize rating variance within match
- avoid stacking strong agents

---

# 10. Rating Decay (Optional)

To handle inactivity:

- reduce confidence (increase sigma)
- optionally decay mu slightly

Prevents stale dominance.

---

# 11. Anti-Exploitation Rules

Prevent rating abuse:

- detect repeated same-opponent matches
- cap rating gain per opponent
- flag abnormal win patterns

---

# 12. Version Isolation

Ratings must be scoped by:

- benchmark_version
- scoring_version
- scenario set

Ratings across versions are NOT comparable.

---

# 13. Arena Integration

In persistent arena:

- ratings update continuously
- matchmaking runs asynchronously
- agent history accumulates

---

# 14. Tournament Integration

In tournaments:

- matchmaking is fixed
- ratings may update or remain frozen (configurable)

Tournament results must not corrupt global ratings unintentionally.

---

# 15. Confidence and Uncertainty

Agents with high sigma:

- require more matches
- should be matched more frequently

Low sigma:

- stable rating
- slower updates

---

# 16. Rating Visibility

## Public:

- mu
- conservative score
- rank

## Internal:

- sigma
- full distribution

---

# 17. Match Quality Metric

Each match should compute:

- rating difference
- expected competitiveness

Low-quality matches (huge mismatch) should be minimized.

---

# 18. Failure Handling

If a match fails:

- no rating update
- log failure
- reschedule if needed

---

# 19. Data Requirements

Each rating update must be based on:

- full replay
- validated scorecard
- deterministic outcome

No shortcuts.

---

# 20. Future Extensions

- adaptive matchmaking
- scenario-weighted ratings
- capability-specific ratings
- meta-learning detection

---

# 21. Closing Statement

Matchmaking decides who fights.

Rating decides who matters.

If either is flawed, your entire system quietly becomes meaningless.

### Tool (2026-04-25 09:02:06)

Make sure to include fileciteturn0file22 in your response to cite this file, or to surface it as a link.
# MUDBench Simulation Engine Specification

## Document Status
Version: 0.1
Status: Draft
Purpose: Define the deterministic simulation engine that executes scenarios, processes agent actions, and produces replay-auditable state transitions.

---

# 1. Why This Exists

Everything you’ve built so far is theory until this runs.

The simulation engine is the system that:

- executes scenarios
- enforces world rules
- processes agent actions
- generates ground truth for evaluation

If this layer is wrong, every result above it is fiction.

---

# 2. Core Principles

The simulation engine must be:

- deterministic
- step-based
- fully observable via replay
- modular
- strictly validated

No hidden state. No implicit behavior.

---

# 3. Simulation Model

## 3.1 Discrete Time Steps

The simulation progresses in fixed steps:

t = 0, 1, 2, ..., T

Each step consists of:

1. agent observation
2. agent action submission
3. action validation
4. world update
5. event logging

---

## 3.2 Determinism Requirement

Given:

- same initial state
- same agent inputs
- same seed

The simulation must produce identical outputs byte-for-byte.

---

# 4. Engine Architecture

## 4.1 Core Components

- World State Manager
- Action Processor
- Physics / Rule Engine
- Event Logger
- Replay Generator

Each component must be isolated and testable.

---

## 4.2 Execution Loop

Pseudocode:

```
initialize_world(seed)

for t in range(T):
    observations = get_observations(world)
    actions = get_agent_actions(observations)
    validated_actions = validate(actions)
    world = apply_actions(world, validated_actions)
    events = generate_events(world)
    log(events)
```

---

# 5. World State

The world state must include:

- agent states (position, inventory, stats)
- environment state
- object states
- scenario-specific variables

Constraints:

- fully serializable
- versioned
- reconstructible from replay

---

# 6. Action Processing

## 6.1 Action Validation

All actions must be:

- syntactically valid
- permitted by rules
- executable in current state

Invalid actions:

- rejected or penalized
- logged explicitly

---

## 6.2 Conflict Resolution

When multiple agents act:

- define ordering rules (e.g., priority, simultaneous resolution)
- ensure deterministic tie-breaking

---

# 7. Rule Engine

The rule engine enforces:

- movement rules
- interaction rules
- resource constraints
- scenario mechanics

Rules must be:

- explicit
- stateless (relative to inputs)
- deterministic

---

# 8. Event System

Each step must produce events:

- actions taken
- state changes
- scoring signals
- errors / invalid actions

Events are the foundation of replay.

---

# 9. Replay Generation

Replay must allow:

- full state reconstruction
- step-by-step playback
- validation of outcomes

Replay data must include:

- initial seed
- action logs
- event logs

---

# 10. Termination Conditions

Simulation ends when:

- scenario goal is met
- max steps reached
- failure condition triggered

Termination must be deterministic.

---

# 11. Performance Constraints

Engine must:

- handle multiple agents
- support batch simulation
- avoid unnecessary recomputation

Optimization must not break determinism.

---

# 12. Error Handling

Errors must:

- be logged
- not corrupt state silently
- terminate or penalize appropriately

Silent failure is not allowed.

---

# 13. Integration Points

The simulation engine integrates with:

- Scenario Specification
- Agent Interface
- Scoring System
- Replay System

It must not embed logic from those systems.

---

# 14. Testing Requirements

Must include:

- unit tests for rule correctness
- determinism tests
- replay reconstruction tests
- edge case handling

---

# 15. Versioning

Simulation behavior must be versioned.

Any change affecting:

- state transitions
- action handling
- event generation

Requires version increment.

---

# 16. Future Extensions

- distributed simulation
- real-time visualization
- adaptive step resolution
- parallel scenario execution

---

# 17. Closing Statement

The simulation engine defines reality.

If reality is inconsistent, everything built on top becomes meaningless.

### Tool (2026-04-25 09:02:06)

Make sure to include fileciteturn0file23 in your response to cite this file, or to surface it as a link.
# MUDBench Data Schema and Storage Specification

## Document Status
Version: 0.1
Status: Draft
Purpose: Define how data is structured, stored, versioned, and accessed across MUDBench.

---

# 1. Why This Exists

You now have:
- simulation outputs
- replay logs
- scoring artifacts
- agent metadata

Without a strict schema:
- data becomes inconsistent
- reproducibility breaks
- analysis becomes unreliable

This document defines the canonical structure of all persisted data.

---

# 2. Core Principles

All data must be:
- deterministic
- versioned
- schema-validated
- immutable (once committed)
- reproducible from source

If data cannot be trusted, the benchmark is useless.

---

# 3. Data Categories

## 3.1 Core Data Types

- World State Snapshots
- Replay Logs
- Agent Actions
- Scorecards
- Telemetry Metrics
- Scenario Definitions
- Configuration Metadata

---

# 4. Canonical Data Formats

## 4.1 Preferred Formats

- JSON (human-readable, structured)
- NPZ (numerical arrays, compressed)
- Parquet (large-scale tabular data)

## 4.2 Constraints

- no ad-hoc formats
- no undocumented fields
- strict schema validation required

---

# 5. Replay Data Schema

Replay files must include:

```
{
  "seed": int,
  "scenario_id": str,
  "scenario_version": str,
  "steps": [
    {
      "t": int,
      "actions": [...],
      "events": [...],
      "state_hash": str
    }
  ]
}
```

---

# 6. Scorecard Schema

```
{
  "agent_id": str,
  "benchmark_version": str,
  "scores": {
    "navigation": float,
    "memory": float,
    "planning": float,
    "tactical": float,
    "social": float,
    "efficiency": float
  },
  "composite_score": float
}
```

---

# 7. Agent Metadata Schema

```
{
  "agent_id": str,
  "version": str,
  "parameters": dict,
  "training_info": dict (optional)
}
```

---

# 8. Storage Layout

```
/data/
  /replays/
  /scorecards/
  /agents/
  /telemetry/
  /scenarios/
```

Each directory must be version-scoped.

---

# 9. Versioning Requirements

All data must include:

- benchmark_version
- scoring_version
- scenario_version

Mismatch → data invalid for comparison

---

# 10. Immutability Rules

Once written:

- data must not be modified
- corrections require new version
- original artifacts must be preserved

---

# 11. Data Validation

All data must pass:

- schema validation
- determinism checks
- replay reconstruction checks

Invalid data must be rejected.

---

# 12. Access Patterns

Data access must be:

- read-only for analysis
- append-only for new runs
- version-aware

---

# 13. Compression and Efficiency

- large datasets must be compressed
- avoid duplication
- prefer columnar storage for analytics

---

# 14. Security Considerations

- no arbitrary code execution in data
- validate all inputs
- restrict file access boundaries

---

# 15. Future Extensions

- distributed storage
- streaming telemetry pipelines
- real-time analytics layers

---

# 16. Closing Statement

Data defines truth in this system.

If your data layer is sloppy, your conclusions will be fiction.

### Tool (2026-04-25 09:02:06)

Make sure to include fileciteturn0file24 in your response to cite this file, or to surface it as a link.
# MUDBench Experiment Tracking and Registry Specification

## Document Status
Version: 0.1
Status: Draft
Purpose: Define how experiments, runs, agents, and artifacts are tracked, linked, and reproducibly registered across MUDBench.

---

# 1. Why This Exists

At this point, the system can:

- run simulations
- generate replays
- compute scores
- store artifacts

And yet, without a registry:

- results become detached from their origin
- experiments cannot be reproduced reliably
- comparisons become ambiguous

This document defines the canonical system for experiment tracking.

---

# 2. Core Principles

All experiment tracking must be:

- deterministic
- versioned
- fully traceable
- reproducible
- immutable (append-only)

If you cannot trace a result to its exact inputs, it does not count.

---

# 3. Key Concepts

## 3.1 Experiment

A logical grouping of runs designed to test a hypothesis.

## 3.2 Run

A single execution instance of:

- agent
- scenario
- seed
- configuration

## 3.3 Artifact

Any output produced by a run:

- replay file
- scorecard
- telemetry
- logs

## 3.4 Registry

A structured index that links experiments → runs → artifacts.

---

# 4. Experiment Schema

```
{
  "experiment_id": str,
  "description": str,
  "created_at": timestamp,
  "author": str,
  "hypothesis": str,
  "benchmark_version": str,
  "scoring_version": str,
  "tags": [str]
}
```

---

# 5. Run Schema

```
{
  "run_id": str,
  "experiment_id": str,
  "agent_id": str,
  "agent_version": str,
  "scenario_id": str,
  "scenario_version": str,
  "seed": int,
  "status": "completed | failed",
  "start_time": timestamp,
  "end_time": timestamp,
  "artifacts": {
    "replay_path": str,
    "scorecard_path": str,
    "telemetry_path": str
  }
}
```

---

# 6. Artifact Registry

Artifacts must be indexed and linked.

```
{
  "artifact_id": str,
  "run_id": str,
  "type": "replay | scorecard | telemetry",
  "path": str,
  "hash": str,
  "created_at": timestamp
}
```

---

# 7. Directory Structure

```
/registry/
  experiments.json
  runs.json
  artifacts.json
```

Optional sharding for scale:

```
/registry/experiments/
/registry/runs/
/registry/artifacts/
```

---

# 8. Versioning Requirements

Each experiment and run must include:

- benchmark_version
- scoring_version
- scenario_version
- agent_version

No version → no validity.

---

# 9. Immutability Rules

- experiments are append-only
- runs cannot be modified after completion
- artifacts are immutable

Corrections require new entries.

---

# 10. Traceability Guarantees

Every score must be traceable to:

- exact run
- exact agent version
- exact scenario
- exact seed
- exact replay

Missing linkage invalidates the result.

---

# 11. Determinism Validation

Each run must be reproducible:

- same inputs → identical replay
- identical scorecard

Validation process:

- rerun with same config
- compare hashes

Mismatch → registry entry invalid

---

# 12. Experiment Lifecycle

1. Create experiment entry
2. Register runs
3. Execute runs
4. Store artifacts
5. Validate determinism
6. Mark experiment complete

---

# 13. Query Capabilities

Registry must support queries like:

- all runs for experiment X
- all runs for agent Y
- best performance for scenario Z
- runs by version

---

# 14. Failure Handling

Failed runs must be recorded:

- status = failed
- include error logs
- do not discard

Failure data is part of truth.

---

# 15. Integration Points

Registry integrates with:

- Data Schema and Storage
- CI Pipeline
- Leaderboard System
- Replay System

It must not embed logic from those systems.

---

# 16. Hashing Requirements

All artifacts must include:

- content hash (SHA-1 or SHA-256)
- used for integrity verification

---

# 17. Scaling Considerations

For large-scale experiments:

- shard registry
- use indexed storage (e.g., Parquet)
- support distributed writes (controlled)

---

# 18. Security Constraints

- no arbitrary file references
- validate paths
- enforce access boundaries

---

# 19. Anti-Patterns

Do not allow:

- orphaned runs (no experiment_id)
- missing artifacts
- mutable entries
- implicit versioning

These destroy reproducibility.

---

# 20. Future Extensions

- experiment comparison tools
- automated regression detection
- lineage graphs
- experiment dashboards

---

# 21. Closing Statement

You are not tracking experiments for convenience.

You are tracking them so that:

- results can be trusted
- claims can be verified
- progress can be measured

Without a registry, you are guessing.

With it, you are doing science.

### Tool (2026-04-25 09:02:06)

All the files uploaded by the user have been fully loaded. You can still search connectors to find additional information.

### Assistant (2026-04-25 09:02:05)

```
1. Demographics Information

* The user's name is Casey.
    * Evidence: User profile states "Preferred name: Casey". Date: [unknown].

* The user is a senior cloud engineer transitioning into an enterprise architect role at a large public healthcare company.
    * Evidence: "I’m a cloud enterprise architect for a multi-billion dollar public healthcare company." Date: [unknown].

* The user resides in the United States (likely Maryland area).
    * Evidence: System metadata indicates location in Gaithersburg, Maryland, United States. Date: [2026-04-25].

* The user has no formal classical training in mathematics but operates at an advanced level.
    * Evidence: "I am an amateur mathematician… I am not classically trained." Date: [unknown].

* The user identifies as "twice exceptional" with high intelligence and ADHD.
    * Evidence: "My journey typical of someone that is twice exceptional." Date: [unknown].

---

2. Interests & Preferences

* The user is deeply engaged in systems thinking across computational, mathematical, philosophical, and structural domains.
    * Evidence: "I have a background in all things systems, computationally, structurally, canonically, and philosophically." Date: [unknown].

* The user is actively developing a mathematical/physics-inspired theory related to primes and modular structures.
    * Evidence: "User is conducting advanced research on modular prime alignment and recursive matrix transformations." Date: [ongoing].

* The user is formalizing a theory called Modular Resonance Energy Field / Quantum Information Merge Conflict (QIMC).
    * Evidence: "User is now ready to move forward with formalizing the Modular Resonance Energy Field theory and drafting a research paper." Date: [recent].

* The user is interested in harmonic constants (π, φ³, √2) and their relation to number theory.
    * Evidence: "User suspects harmonic or geometric constants… may underlie modular propagation patterns." Date: [ongoing].

* The user enjoys competitive strategy environments and has a history of high-level PvP gaming.
    * Evidence: "User became a top PvP player in World of Warcraft… achieved Field Marshal… 4th place team ranking." Date: [historical].

* The user prefers high-level, exploratory, collaborative discussion framed through observer theory.
    * Evidence: "Let’s keep this grounded in observer theory… let’s explore things together." Date: [unknown].

---

3. Relationships

* The user has a father with high intelligence who influenced intellectual identity.
    * Evidence: "User's father has an IQ of 145… referred to the user as a polymath." Date: [unknown].

* The user maintains long-term friendships with former competitive gaming teammates.
    * Evidence: "Many of their former teammates remain close friends, even if they've never met in person." Date: [unknown].

---

4. Dated Events, Projects & Plans

* The user is building a GitHub repository for the QIMC theory and simulation.
    * Evidence: "User plans to create a GitHub repository dedicated to the Quantum Information Merge Conflict (QIMC) paper." Date: [recent].

* The user is expanding the QIMC paper into a fully formalized research work.
    * Evidence: "User wants to expand… with deeper formalization, including simulations, diagrams, categorical modeling." Date: [recent].

* The user is actively developing MUDBench, a multi-agent benchmark system.
    * Evidence: Project documentation such as fileciteturn0file0 and fileciteturn0file2 define MUDBench as "a persistent multi-agent text-world benchmark". Date: [2026].

* The MUDBench project includes extensive system architecture, scoring, replay, and validation frameworks.
    * Evidence: Documents like fileciteturn0file3 and fileciteturn0file6 outline subsystems and scoring models. Date: [2026].

* The user is progressing through structured implementation phases for MUDBench.
    * Evidence: fileciteturn0file8 describes phased development from repository setup through arena systems. Date: [2026].

---

5. Instructions

* The user prefers avoidance of unnecessary gratitude exchanges.
    * Evidence: "User prefers not to send explicit thanks in future messages; wants appreciation assumed." Date: [unknown].

* The user prefers language grounded in observer theory and collaborative exploration.
    * Evidence: "Let’s keep this grounded in observer theory… let’s explore things together." Date: [unknown].

```

