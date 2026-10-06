# Session polls, quizzes, and raised hands

Session interactions run in `mahfel-api`. PostgreSQL owns activities, responses, the ordered hand
queue, and idempotency records. The browser uses authenticated REST on the API host and the existing
session WebSocket. No interaction traffic passes through the media plane.

## Product behavior

Managers prepare editable drafts containing one question and 2–10 distinct options. A session may
have one open activity at a time. Launching freezes the question, options, voting mode, and quiz key.
Only active, currently connected participants may respond or raise a hand. Current room/session
visibility, membership, explicit session admission, blocks, and capabilities remain authoritative.

Polls support single or multiple selections and no correct answer. After voting, participants see
option counts and horizontal bars. They can change or retract their vote while the poll is open.
Closing reveals the result to everyone with session access, including people who did not vote.
Percentages divide each option's count by respondents; a multiple-choice poll can total over 100%.

Quizzes support single or multiple correct options. Participants submit one final selection set.
Correctness requires exactly the configured set, without partial credit. Until closing, no API
response exposes the key, explanation, option distribution, or named submissions. Managers can see
the response count. Closing reveals bars, correct options, the optional explanation, and the caller's
own correctness. Ending a session also closes its open activity and reveals those results.

Creators choose anonymous or named voting before launch. Anonymous results expose aggregates and
the caller's own choices, including to managers. They never expose another person's choices through
the product API. The durable database still associates responses with actors for uniqueness and
retry protection; anonymous voting is not anonymous to database operators. Named voter lists follow
the same result-reveal rules and paginate 100 responses at a time.

Raising a hand creates one queue entry per actor across devices. Every participant sees its order;
moderators can lower a particular entry or clear the queue. Lowering a hand never unmutes anyone.
Leaving, removal, loss of the last connection, and session termination clear applicable hands.
Expired presence leases are reconciled after an API replica crashes. Closed results remain available
on the ended-session page subject to current access authorization.

## Commands and delivery

`GET /v1/sessions/{session_id}/interactions` returns an authorized, consistent snapshot with a
monotonic version. `POST /v1/sessions/{session_id}/interactions/commands` accepts a UUID `request_id`
and a tagged command: save draft, launch, close, vote, retract, set hand, lower hand, or clear hands.
The named voter route is
`GET /v1/sessions/{session_id}/interactions/activities/{activity_id}/voters?offset=0`.
All snapshot and voter responses use `Cache-Control: private, no-store`.

Commands serialize on the session row. A durable command fingerprint makes identical retries
no-ops and refuses reuse of the same identifier for a different payload. Vote versions preserve
retraction tombstones so stale requests cannot replace newer responses. Hand queue generations
prevent delayed lower commands from removing a later raise. Clearing the queue requires the current
snapshot version. Session/participant transition triggers close activities and remove hands within
the lifecycle transaction.

Each API replica broadcasts only `session_interactions_changed { session_id, version }` to its
joined sockets. Redis pub/sub wakes the replicas; a two-second PostgreSQL version poll recovers
missed notifications. Bounded socket queues retry the current version after backpressure. The
browser refreshes the personalized snapshot on newer events and every 15 seconds as a fallback.
Keys, voter identities, and selections are never broadcast over this notification channel.

Resource limits are 50 activities per session, 2,000 distinct response records per activity, 2,000
queued hands, and 2,000 successful command identifiers per actor/session. These limits bound
snapshots and retained retry state; reaching one returns a conflict rather than silently dropping
state.

## Frontend and verification

The `interactions` feature uses the official shadcn Questionnaire adapted to the existing Radix
stack, with Dialog, Field, RadioGroup, Checkbox, Switch, Card, Badge, Alert, and Tooltip. Persian and
English use the shared catalog, logical layout, isolated mixed text, locale-formatted numbers, and
mirrored bar origins. Session controls contain an outlined hand toggle and a reactions menu. Polls and
quizzes are a stage view alongside participants, notes, whiteboard, and presentation; the queue
appears above the roster.
Draft creation and editing are embedded in the activity view, with a compact scrolling form body
and a fixed save footer. They do not open a modal or cover the session toolbar. Cancelling returns
to the selected question without losing unfinished answer selections.
Questions use a slideshow with one visible slide, localized position and Previous/Next controls.
Navigation preserves unsent choices on other slides and when switching away from the activity view.
The open activity is ordered first. Questions center within the available stage height; overflowing
content scrolls independently above the bottom control toolbar. The breadcrumb includes the active
view. Session URLs retain `view` (participants, notes, whiteboard, presentation, or activities) and
`question` (activity identifier), preserving unrelated parameters and the fragment. Native history
updates integrate with Next.js navigation without reloading the session connection. Reloads and
browser history restore the view and question; unfinished selections remain in memory only.

## Live emoji reactions

`POST /v1/sessions/{session_id}/interactions/reactions` accepts a fixed eight-emoji palette. The API
requires a currently authorized, connected participant and derives the username from the verified
principal. Redis limits each actor to two reactions per two seconds across devices and each session
to 60 per two seconds. Rate-limit keys hash identifiers and expire with the window.

Reactions use the existing Redis interaction channel to deliver `session_reaction` events to joined
sockets on every API replica. They are transient notices: they are not persisted or replayed after
reconnection, and missed pub/sub events may disappear. Votes and raised hands remain durable.
The browser deduplicates event identifiers, retains at most 12 animations and 128 recently seen
identifiers, and removes each notice after 3.5 seconds. A username and emoji float from the bottom
of the stage upward; reduced-motion users see a stationary temporary notice. The reactions menu
also exposes the persistent raise/lower hand command.

The disposable PostgreSQL scenario test covers retries, retraction, concurrent replacements,
grading, hand generations, membership revocation, removal, and session end. Transport projection
tests cover quiz redaction. Polling tests run two replica registries without pub/sub wakeups;
registry tests cover scoped, deduplicated notifications and full queues.
Frontend component/hook tests cover selection sets, reveal states, creation, moderation commands,
and network retry identifiers. Storybook includes Persian/English voting, results, composer, hands,
and long mixed text. These checks do not replace authenticated multi-user browser validation.
