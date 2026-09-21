# ADR 0021: Invite links are revocable bearer capabilities

## Status

Accepted

## Context

Rooms are reachable through membership, and `public` rooms through the room link (`RoomVisibility`).
That is not enough to invite someone the way people expect from a meeting product: share a short,
readable code, let the holder join, and confirm them on entry unless they were already added. M4 adds
sessions on top of rooms, and both need this entry path, so the mechanism has to be designed once.

Constraints that shaped the decision:

- The code is typed or pasted by humans, so it must be short, readable, and unambiguous. The chosen
  shape is `xxx-yyyy-zzz`: 3 + 4 + 3 lowercase `a-z`, 10 letters, case-insensitive on input.
- A shared code is a capability: anyone holding an active code can attempt entry. That is the point,
  but it means the code must never be treated as a secret to be logged, and redemption must be rate
  limited and must not help an attacker enumerate valid codes.
- Links must be retractable: rooms and sessions change, and a leaked code must be invalidatable
  without changing everything else.

## Decision

- An invite link is a durable record with a globally unique code, exactly one target (one room or one
  session), an expiry, a status (`active`, `revoked`, `expired`), and a creator. Room links and
  session links share one namespace and one table, so a code resolves deterministically.
- Codes use the `xxx-yyyy-zzz` lowercase-letter format. Generation is atomic with a bounded collision
  retry, and redemption normalizes case and separators before lookup.
- The default expiry is 7 days and is bounded by policy. Links are revocable, and rotate/regenerate
  invalidates the previous code atomically. Expiry and revocation are enforced at redemption, not
  only in the UI.
- Codes are stored plaintext because redemption looks them up directly. Protection comes from rate
  limiting, bounded attempts, audit, non-enumerating errors, and log/metric hygiene (codes are never
  logged and are never metric labels), not from hashing a 10-letter alphabet-constrained value.
- Only authenticated principals (users or service principals) may redeem a link. Anonymous guests are
  out of scope.
- Redeeming a link never grants immediate access. The holder enters the waiting/confirmation state
  unless it already holds explicit membership in the target. A principal with the `admit_participant`
  capability approves or rejects.
- Approval semantics depend on the target:
  - a **session link** approval grants session participation only and does not create durable room
    membership;
  - a **room link** approval creates durable room membership, and later joins to that room bypass
    confirmation.
- Link management (create, list, revoke, rotate, regenerate) requires the `manage_session`
  capability or the equivalent room-management authority for the target. Authorization stays
  deny-by-default and server-side.

## Consequences

- Invite links reuse the admission and capability seams from M4-04 and M4-06 instead of a parallel
  access system. The confirmation flow is the lobby with an invite-link origin.
- A session link is intentionally not a membership grant, so a holder may be asked to confirm again
  in a later session. A room link is an onboarding grant and behaves like an explicit add.
- Operators get a clear lifecycle: visible expiry, explicit revoke, and rotation without breaking
  other links or memberships.
- The code is a low-entropy bearer value by design; the security posture depends on rate limiting and
  audit being implemented and tested. This is a deliberate trade for usability.
- Codes must not appear in structured logs, traces, or Prometheus labels. Audit records store the link
  identity, not the raw code where the code is not required.

## Alternatives considered

- **Signed or encrypted self-contained tokens.** Rejected: no revocation without a store, long opaque
  strings, and worse usability for the intended share-a-code flow.
- **Random opaque identifiers (UUID).** Rejected: unusable when typed, and the chosen format keeps a
  similar practical guessability only because rate limiting exists.
- **Hashed codes with a separate lookup key.** Rejected: the code is the lookup key, so this adds a
  second identifier without removing brute-force exposure on a 10-letter alphabetic code.
- **A third room visibility (`link`) instead of a link entity.** Rejected: it cannot express expiry,
  revocation, rotation, per-session targets, or the origin of an admission request.
