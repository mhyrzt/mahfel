# ADR 0012: Use the Yrs and Yjs CRDT model

## Status

Accepted

## Context

Collaborative notes and whiteboards need concurrent editing, reconnection, convergence, and browser
interoperability.

## Decision

Use Yjs in the frontend and Yrs in Rust, with a documented synchronization, snapshot, and
persistence strategy built in M7.

## Consequences

Concurrent edits can converge across supported clients. Updates, awareness, compaction, recovery,
and authorization around documents still require careful protocol design.

## Alternatives considered

Last-write-wins loses concurrent intent. Operational transformation would require more custom
protocol work. A standalone collaboration service is not justified initially.
