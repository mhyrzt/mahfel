# ADR 0009: Use coturn for STUN and TURN

## Status

Accepted

## Context

Some browsers cannot establish direct media paths because of NAT, firewalls, or restrictive
enterprise networks.

## Decision

Operate coturn as the STUN/TURN implementation with stable public addressing, authenticated
allocations, an explicit relay range, and production TLS.

## Consequences

Restricted users gain a reliable relay path, at the cost of significant relay bandwidth and public
UDP/TCP/TLS operations. TURN credentials must be short-lived in production.

## Alternatives considered

Direct-only WebRTC excludes real networks. A custom TURN server is unnecessary protocol and
security risk. Third-party TURN remains a deployment option behind the same interface.
