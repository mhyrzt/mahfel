# ADR 0001: Record architecture decisions

## Status

Accepted

## Context

Mahfel spans browser, control, media, workers, state systems, and unusual network constraints.
Important choices otherwise become undocumented convention.

## Decision

Record durable decisions in numbered Markdown ADRs containing status, context, decision,
consequences, and alternatives. Supersede decisions with later records.

## Consequences

Tradeoffs and rejected options remain reviewable. Contributors must maintain a small amount of
documentation alongside architectural changes.

## Alternatives considered

Commit messages and informal chat were rejected because they are difficult to discover and do not
present context consistently.
