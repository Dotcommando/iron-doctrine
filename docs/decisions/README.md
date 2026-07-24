# Architecture Decision Records

This directory contains Architecture Decision Records, or ADRs, for Iron Doctrine.

An ADR records an important architectural or technical decision together with its context, consequences, and alternatives.

## Index

- [ADR-0001: BLAKE3 State Hash](ADR-0001-BLAKE3-STATE-HASH.md)

## When to Create an ADR

Create an ADR when a decision:

- changes an architectural boundary;
- introduces a major dependency;
- defines or changes a public contract;
- defines a persistent or networked data format;
- imposes a long-term implementation constraint;
- is expensive or risky to reverse;
- supersedes an earlier accepted decision.

Do not create ADRs for routine implementation details, temporary experiments, naming of local variables, or easily reversible choices.

## Naming

ADR files use this format:

```text
ADR-NNNN-SHORT-TITLE.md
```

Examples:

```text
ADR-0001-RUST-AUTHORITATIVE-SIMULATION.md
ADR-0002-FIXED-SIMULATION-TICK.md
ADR-0003-MODULE-ASSEMBLY-AS-TREE.md
```

Numbers are assigned sequentially and are never reused.

## Status

Every ADR has one of these statuses:

- `Proposed` — under discussion and not yet binding.
- `Accepted` — approved and currently applicable.
- `Superseded` — replaced by a newer ADR.
- `Rejected` — considered but deliberately not adopted.

An accepted ADR must not be silently contradicted.

When a decision changes, create a new ADR and mark the previous one as superseded.

## ADR Template

```md
# ADR-NNNN: Title

- Status: Proposed
- Date: YYYY-MM-DD
- Supersedes: None
- Superseded by: None

## Context

Describe the problem, constraints, and forces that require a decision.

## Decision

State the chosen approach clearly.

## Alternatives Considered

Describe the meaningful alternatives and why they were not selected.

## Consequences

List the positive, negative, and operational consequences of the decision.

## Verification

Describe how compliance with the decision can be checked through code structure, tests, scenarios, tooling, or documentation.
```

## Maintenance Rules

- Keep ADRs focused on one decision.
- Record the reasoning available at the time of the decision.
- Do not rewrite the historical reasoning of an accepted ADR.
- Minor clarifications are allowed if they do not change the decision.
- Use a new ADR for material changes.
- Reference relevant ADRs from plans and technical documentation.
