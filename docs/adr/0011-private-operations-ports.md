# ADR 0011: Use dedicated private operations ports

## Status

Accepted

## Context

Metrics and health probes serve operators/orchestrators and have different exposure requirements
from product APIs.

## Decision

Expose `/metrics`, `/health/live`, and `/health/ready` on 9101 (API), 9102 (media), and 9103
(worker). Keep these listeners private; never put metrics on the public API listener.

## Consequences

Ingress and network policy can isolate operations traffic. Every deployment must scrape/probe a
second listener. Separate ports alone are not access control.

## Alternatives considered

Public API metrics risk information exposure. Sidecar-only metrics would add bootstrap complexity
without removing the need for in-process instrumentation.
