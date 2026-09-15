# ADR 0014: Use Next.js, Bun, TypeScript, Tailwind, and shadcn/ui

## Status

Accepted

## Context

Mahfel needs a maintainable React UI for complex classroom, meeting, media, collaboration, file,
AI, and administration experiences.

## Decision

Use Next.js App Router, strict TypeScript, React, Tailwind CSS, and shadcn/ui. Bun is the package
manager, script runner, and compatible local/container runtime.

## Consequences

The frontend gains typed modern tooling and locally owned UI primitives. Bun/Next compatibility,
browser configuration at build time, and independent deployments need validation. Avoid arbitrary
additional component libraries.

## Alternatives considered

Backend-rendered templates conflict with ADR 0002. Other package managers and component suites add
tooling/design duplication without a current need.
