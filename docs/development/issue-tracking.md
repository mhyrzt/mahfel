# Issue tracking

[GitHub issues](https://github.com/mhyrzt/mahfel/issues) are the source of truth for
work. [GitHub milestones](https://github.com/mhyrzt/mahfel/milestones) group delivery
scope; the [engineering roadmap](roadmap.md) describes sequencing and product assumptions.

Search existing issues before creating a task. Record the problem, scope, acceptance
criteria, dependencies, and validation evidence. Use verified GitHub usernames for
assignees and select the relevant milestone. Trivial mechanical changes do not need
an issue.

Use `status::To Do` and `status::In Progress` on open tasks. After validation, record
the final summary, set `status::Done`, and close the issue. Archived work retains
`status::Archived` and stays closed; archival does not imply acceptance criteria were
completed. Domain and priority labels retain their original meanings.

Use `gh --repo mhyrzt/mahfel` where the command supports it, or the GitHub API for
`repos/mhyrzt/mahfel`. Read command help before unfamiliar operations. Prefer JSON
request bodies or body files so Markdown and newlines remain intact. GitHub owns live
task state; this page is a historical migration index, not a parallel task database.

## Public tracker migration

The public migration includes 220 issues (54 open and 166 closed), 16 milestones,
and 51 human discussion comments. Issue labels, milestone assignments, and the
verified `mhyrzt` assignee are retained. Original authors and timestamps are recorded
in issue and comment text; GitHub's native authors and timestamps belong to the
importing account and migration time.

One confidential issue remains exclusively in the former tracker. Twenty issues
about former private deployments, registries, or retired CI tooling, and two
production-deployment milestones, are excluded from this public backlog.

Reusable work has been edited to remove organization-specific names and private
infrastructure references. Code and retained issue links point to GitHub. References
to excluded tasks are omitted. Duplicate raw records from the earlier local-tracker
migration and generated system notes are not imported. Historical completion and
validation notes are retained as history; they are not fresh verification of the
current checkout. Security scan findings require current applicability checks.

Legacy task IDs remain in titles where available. Issue numbers changed during the
move; use the mapping below when reading older commit messages or task references.
No source issues were modified or deleted.

## Issue mapping

| Earlier issue ID | GitHub issue | Title |
| --- | --- | --- |
| `2` | [#1](https://github.com/mhyrzt/mahfel/issues/1) | Keep participant active while another device remains connected |
| `3` | [#2](https://github.com/mhyrzt/mahfel/issues/2) | Prevent sequential WebSocket joins from orphaning participation |
| `4` | [#3](https://github.com/mhyrzt/mahfel/issues/3) | Bound or remove unused WebSocket generation state |
| `5` | [#4](https://github.com/mhyrzt/mahfel/issues/4) | Clean up media transports and tracks when participants disconnect |
| `6` | [#5](https://github.com/mhyrzt/mahfel/issues/5) | Release media map locks before awaiting mediasoup operations |
| `7` | [#6](https://github.com/mhyrzt/mahfel/issues/6) | PATCH room member role change is silently not persisted |
| `8` | [#7](https://github.com/mhyrzt/mahfel/issues/7) | Room-scoped admission lookups ignore room_id |
| `9` | [#8](https://github.com/mhyrzt/mahfel/issues/8) | Admission grant happens before the conditional decision (reject race) |
| `10` | [#9](https://github.com/mhyrzt/mahfel/issues/9) | Session list endpoint is unbounded |
| `11` | [#10](https://github.com/mhyrzt/mahfel/issues/10) | Session roster performs N+1 lookups and a presign per participant |
| `12` | [#11](https://github.com/mhyrzt/mahfel/issues/11) | Pagination offset can overflow on large page values |
| `13` | [#12](https://github.com/mhyrzt/mahfel/issues/12) | Capability introspection and moderation enforcement disagree for global roles |
| `14` | [#13](https://github.com/mhyrzt/mahfel/issues/13) | Session roster is readable by blocked or lobby-waiting non-members |
| `15` | [#14](https://github.com/mhyrzt/mahfel/issues/14) | session_participation aggregation is O(n^2) |
| `16` | [#15](https://github.com/mhyrzt/mahfel/issues/15) | Chat slow-consumer eviction skips disconnect cleanup and leaks open participation |
| `17` | [#16](https://github.com/mhyrzt/mahfel/issues/16) | Collaboration WebSocket bypasses the connection cap and has no idle timeout |
| `18` | [#17](https://github.com/mhyrzt/mahfel/issues/17) | Main WebSocket has no inbound rate limit; Broadcast amplifies every frame |
| `19` | [#18](https://github.com/mhyrzt/mahfel/issues/18) | Chat fanout issues one event_head query per subscriber per tick |
| `20` | [#19](https://github.com/mhyrzt/mahfel/issues/19) | Moderation and presence frames are replica-local; cross-replica targets never see them |
| `21` | [#20](https://github.com/mhyrzt/mahfel/issues/20) | Speaker projection and collaboration fanout maps are never pruned |
| `22` | [#21](https://github.com/mhyrzt/mahfel/issues/21) | Collaboration checkpoints are never pruned and store full snapshots |
| `23` | [#22](https://github.com/mhyrzt/mahfel/issues/22) | A transient Redis error during presence renewal disconnects every socket |
| `24` | [#23](https://github.com/mhyrzt/mahfel/issues/23) | Media placement is re-resolved with two Redis calls on every RTC frame |
| `25` | [#24](https://github.com/mhyrzt/mahfel/issues/24) | Non-idempotent media create RPCs are retried, orphaning transports/producers/consumers |
| `26` | [#25](https://github.com/mhyrzt/mahfel/issues/25) | Media control-plane calls have no deadline and can hang the API |
| `27` | [#26](https://github.com/mhyrzt/mahfel/issues/26) | close_participant leaves stale consumers in the consumer map |
| `28` | [#27](https://github.com/mhyrzt/mahfel/issues/27) | One job persistence error tears down the AMQP consumer and aborts concurrent jobs |
| `29` | [#28](https://github.com/mhyrzt/mahfel/issues/28) | Claim recovery only runs while the AMQP relay is connected |
| `30` | [#29](https://github.com/mhyrzt/mahfel/issues/29) | GetNodeStats always reports participants = 0 |
| `31` | [#30](https://github.com/mhyrzt/mahfel/issues/30) | Media control gRPC service is unauthenticated |
| `32` | [#31](https://github.com/mhyrzt/mahfel/issues/31) | Outbox relay ignores the mark_published result |
| `33` | [#32](https://github.com/mhyrzt/mahfel/issues/32) | BFF request-size limit can be bypassed with chunked bodies |
| `34` | [#33](https://github.com/mhyrzt/mahfel/issues/33) | BFF does not strip forwarding and hop-by-hop headers |
| `35` | [#34](https://github.com/mhyrzt/mahfel/issues/34) | BFF allowlist relies on URL normalization for containment |
| `36` | [#35](https://github.com/mhyrzt/mahfel/issues/35) | Federated logout open redirect via protocol-relative callbackUrl |
| `37` | [#36](https://github.com/mhyrzt/mahfel/issues/36) | Send-side RTC resources leak when unmount races async setup |
| `38` | [#37](https://github.com/mhyrzt/mahfel/issues/37) | Chat refetches full history on every incoming event |
| `39` | [#38](https://github.com/mhyrzt/mahfel/issues/38) | Collaboration/signaling client robustness: parse guard, handshake timeout, duplicate sockets |
| `40` | [#39](https://github.com/mhyrzt/mahfel/issues/39) | Presentation buffers full file in React state and ICE restart reuses stale parameters |
| `41` | [#40](https://github.com/mhyrzt/mahfel/issues/40) | [TASK-1] Bootstrap Mahfel production foundation |
| `42` | [#41](https://github.com/mhyrzt/mahfel/issues/41) | [TASK-1.1] Scaffold repository and Rust workspace |
| `43` | [#42](https://github.com/mhyrzt/mahfel/issues/42) | [TASK-1.2] Scaffold Next.js Bun frontend |
| `44` | [#43](https://github.com/mhyrzt/mahfel/issues/43) | [TASK-1.3] Create production application images |
| `45` | [#44](https://github.com/mhyrzt/mahfel/issues/44) | [TASK-1.4] Create local Compose infrastructure |
| `46` | [#45](https://github.com/mhyrzt/mahfel/issues/45) | [TASK-1.5] Establish observability foundation |
| `47` | [#46](https://github.com/mhyrzt/mahfel/issues/46) | [TASK-1.6] Write architecture documentation and ADRs |
| `48` | [#47](https://github.com/mhyrzt/mahfel/issues/47) | [TASK-1.7] Create detailed engineering roadmap |
| `49` | [#48](https://github.com/mhyrzt/mahfel/issues/48) | [TASK-1.8] Validate and commit the bootstrap |
| `50` | [#49](https://github.com/mhyrzt/mahfel/issues/49) | [TASK-2] M1-01 Migration runner and pools |
| `51` | [#50](https://github.com/mhyrzt/mahfel/issues/50) | [TASK-3] M1-02 User/room/session core |
| `52` | [#51](https://github.com/mhyrzt/mahfel/issues/51) | [TASK-4] M1-03 HTTP contract foundation |
| `53` | [#52](https://github.com/mhyrzt/mahfel/issues/52) | [TASK-5] M1-04 Room endpoints |
| `54` | [#53](https://github.com/mhyrzt/mahfel/issues/53) | [TASK-6] M1-05 WebSocket gateway |
| `55` | [#54](https://github.com/mhyrzt/mahfel/issues/54) | [TASK-7] M1-06 Redis coordination adapter |
| `56` | [#55](https://github.com/mhyrzt/mahfel/issues/55) | [TASK-8] M1-07 Authorization/OpenAPI decisions |
| `57` | [#56](https://github.com/mhyrzt/mahfel/issues/56) | [TASK-9] M2-01 Identity schema |
| `58` | [#57](https://github.com/mhyrzt/mahfel/issues/57) | [TASK-10] M2-02 Local password lifecycle |
| `59` | [#58](https://github.com/mhyrzt/mahfel/issues/58) | [TASK-11] M2-03 Session security |
| `60` | [#59](https://github.com/mhyrzt/mahfel/issues/59) | [TASK-12] M2-04 OIDC provider abstraction |
| `61` | [#60](https://github.com/mhyrzt/mahfel/issues/60) | [TASK-13] M2-05 OIDC provider compatibility |
| `62` | [#61](https://github.com/mhyrzt/mahfel/issues/61) | [TASK-14] M2-06 Identity linking |
| `63` | [#62](https://github.com/mhyrzt/mahfel/issues/62) | [TASK-15] M2-07 RBAC and room roles |
| `64` | [#63](https://github.com/mhyrzt/mahfel/issues/63) | [TASK-16] M2-08 Avatar resolution |
| `65` | [#64](https://github.com/mhyrzt/mahfel/issues/64) | [TASK-17] M3-01 Worker supervision |
| `66` | [#65](https://github.com/mhyrzt/mahfel/issues/65) | [TASK-18] M3-02 Router/room lifecycle |
| `67` | [#66](https://github.com/mhyrzt/mahfel/issues/66) | [TASK-19] M3-03 WebRtcServer/transports |
| `68` | [#67](https://github.com/mhyrzt/mahfel/issues/67) | [TASK-20] M3-04 Producer lifecycle |
| `69` | [#68](https://github.com/mhyrzt/mahfel/issues/68) | [TASK-21] M3-05 Consumer lifecycle |
| `70` | [#69](https://github.com/mhyrzt/mahfel/issues/69) | [TASK-22] M3-06 Signaling integration |
| `71` | [#70](https://github.com/mhyrzt/mahfel/issues/70) | [TASK-23] M3-07 Frontend RTC client |
| `72` | [#71](https://github.com/mhyrzt/mahfel/issues/71) | [TASK-24] M3-08 Screenshare/reconnect |
| `73` | [#72](https://github.com/mhyrzt/mahfel/issues/72) | [TASK-25] M3-09 TURN and media metrics |
| `74` | [#73](https://github.com/mhyrzt/mahfel/issues/73) | [TASK-26] M4-01 Session vocabulary and lifecycle |
| `75` | [#74](https://github.com/mhyrzt/mahfel/issues/74) | [TASK-27] M4-02 Participant lifecycle and reconnect identity |
| `76` | [#75](https://github.com/mhyrzt/mahfel/issues/75) | [TASK-28] M4-05 Moderation controls |
| `77` | [#76](https://github.com/mhyrzt/mahfel/issues/76) | [TASK-29] M4-07 Active speaker |
| `78` | [#77](https://github.com/mhyrzt/mahfel/issues/77) | [TASK-30] M4-08 Media node registry |
| `79` | [#78](https://github.com/mhyrzt/mahfel/issues/78) | [TASK-31] M4-09 Session placement scheduler |
| `80` | [#79](https://github.com/mhyrzt/mahfel/issues/79) | [TASK-32] M5-01 Channel/message model |
| `81` | [#80](https://github.com/mhyrzt/mahfel/issues/80) | [TASK-33] M5-02 History and mutations |
| `82` | [#81](https://github.com/mhyrzt/mahfel/issues/81) | [TASK-34] M5-03 Realtime fanout |
| `83` | [#82](https://github.com/mhyrzt/mahfel/issues/82) | [TASK-35] M5-04 Reactions/mentions/threads |
| `84` | [#83](https://github.com/mhyrzt/mahfel/issues/83) | [TASK-36] M5-05 Typing/presence/read markers |
| `85` | [#84](https://github.com/mhyrzt/mahfel/issues/84) | [TASK-37] M5-06 Attachments and abuse controls |
| `86` | [#85](https://github.com/mhyrzt/mahfel/issues/85) | [TASK-38] M6-01 Object metadata and ACL |
| `87` | [#86](https://github.com/mhyrzt/mahfel/issues/86) | [TASK-39] M6-02 S3 adapter |
| `88` | [#87](https://github.com/mhyrzt/mahfel/issues/87) | [TASK-40] M6-03 Presigned upload |
| `89` | [#88](https://github.com/mhyrzt/mahfel/issues/88) | [TASK-41] M6-04 Authorized download |
| `90` | [#89](https://github.com/mhyrzt/mahfel/issues/89) | [TASK-42] M6-05 File contexts |
| `91` | [#90](https://github.com/mhyrzt/mahfel/issues/90) | [TASK-43] M6-06 Validation/security strategy |
| `92` | [#91](https://github.com/mhyrzt/mahfel/issues/91) | [TASK-44] M7-01 CRDT document identity |
| `93` | [#92](https://github.com/mhyrzt/mahfel/issues/92) | [TASK-45] M7-02 Yrs synchronization gateway |
| `94` | [#93](https://github.com/mhyrzt/mahfel/issues/93) | [TASK-46] M7-03 Persistence/checkpoints |
| `95` | [#94](https://github.com/mhyrzt/mahfel/issues/94) | [TASK-47] M7-04 Awareness/presence |
| `96` | [#95](https://github.com/mhyrzt/mahfel/issues/95) | [TASK-48] M7-05 Collaborative notes UI |
| `97` | [#96](https://github.com/mhyrzt/mahfel/issues/96) | [TASK-49] M7-06 Whiteboard object model |
| `98` | [#97](https://github.com/mhyrzt/mahfel/issues/97) | [TASK-50] M8-01 AMQP topology |
| `99` | [#98](https://github.com/mhyrzt/mahfel/issues/98) | [TASK-51] M8-02 Reliable publisher |
| `100` | [#99](https://github.com/mhyrzt/mahfel/issues/99) | [TASK-52] M8-03 Consumer acknowledgements |
| `101` | [#100](https://github.com/mhyrzt/mahfel/issues/100) | [TASK-53] M8-04 Retry and DLQ |
| `102` | [#101](https://github.com/mhyrzt/mahfel/issues/101) | [TASK-54] M8-05 Idempotency |
| `103` | [#102](https://github.com/mhyrzt/mahfel/issues/102) | [TASK-55] M8-06 Transactional outbox |
| `104` | [#103](https://github.com/mhyrzt/mahfel/issues/103) | [TASK-56] M9-01 Recording architecture spike |
| `105` | [#104](https://github.com/mhyrzt/mahfel/issues/104) | [TASK-57] M9-02 Recording notice and authorization |
| `106` | [#105](https://github.com/mhyrzt/mahfel/issues/105) | [TASK-58] M9-03 Recording lifecycle |
| `107` | [#106](https://github.com/mhyrzt/mahfel/issues/106) | [TASK-59] M9-04 Capture and upload |
| `108` | [#107](https://github.com/mhyrzt/mahfel/issues/107) | [TASK-60] M9-05 Finalization worker |
| `109` | [#108](https://github.com/mhyrzt/mahfel/issues/108) | [TASK-61] M9-06 Playback and retention |
| `110` | [#109](https://github.com/mhyrzt/mahfel/issues/109) | [TASK-62] M10-01 STT provider contract |
| `111` | [#110](https://github.com/mhyrzt/mahfel/issues/110) | [TASK-63] M10-02 Live versus post-session decision |
| `112` | [#111](https://github.com/mhyrzt/mahfel/issues/111) | [TASK-64] M10-03 Transcript data model |
| `113` | [#112](https://github.com/mhyrzt/mahfel/issues/112) | [TASK-65] M10-04 Speaker mapping |
| `114` | [#113](https://github.com/mhyrzt/mahfel/issues/113) | [TASK-66] M10-05 Transcription jobs |
| `115` | [#114](https://github.com/mhyrzt/mahfel/issues/114) | [TASK-67] M10-06 Transcript API/UI/export/search |
| `116` | [#115](https://github.com/mhyrzt/mahfel/issues/115) | [TASK-68] M11-01 AI participant domain |
| `117` | [#116](https://github.com/mhyrzt/mahfel/issues/116) | [TASK-69] M11-02 LLM provider abstraction |
| `118` | [#117](https://github.com/mhyrzt/mahfel/issues/117) | [TASK-70] M11-03 Mention workflow |
| `119` | [#118](https://github.com/mhyrzt/mahfel/issues/118) | [TASK-71] M11-04 Context assembler |
| `120` | [#119](https://github.com/mhyrzt/mahfel/issues/119) | [TASK-72] M11-05 Prompt safety boundaries |
| `121` | [#120](https://github.com/mhyrzt/mahfel/issues/120) | [TASK-73] M11-06 AI RBAC and administration |
| `122` | [#121](https://github.com/mhyrzt/mahfel/issues/121) | [TASK-74] M12-01 Spoken request capture |
| `123` | [#122](https://github.com/mhyrzt/mahfel/issues/122) | [TASK-75] M12-02 Low-latency STT path |
| `124` | [#123](https://github.com/mhyrzt/mahfel/issues/123) | [TASK-76] M12-03 TTS abstraction |
| `125` | [#124](https://github.com/mhyrzt/mahfel/issues/124) | [TASK-77] M12-04 AI mediasoup producer |
| `126` | [#125](https://github.com/mhyrzt/mahfel/issues/125) | [TASK-78] M12-05 Turn interruption and mute |
| `127` | [#126](https://github.com/mhyrzt/mahfel/issues/126) | [TASK-79] M13-01 Stateless API scaling |
| `128` | [#127](https://github.com/mhyrzt/mahfel/issues/127) | [TASK-80] M13-02 Worker concurrency scaling |
| `129` | [#128](https://github.com/mhyrzt/mahfel/issues/128) | [TASK-81] M13-03 Database capacity |
| `130` | [#129](https://github.com/mhyrzt/mahfel/issues/129) | [TASK-82] M13-04 Redis/RabbitMQ failure drills |
| `131` | [#130](https://github.com/mhyrzt/mahfel/issues/130) | [TASK-83] M13-05 Media drain and rolling deploy |
| `132` | [#131](https://github.com/mhyrzt/mahfel/issues/131) | [TASK-84] M13-06 Media node loss/reconnect |
| `133` | [#132](https://github.com/mhyrzt/mahfel/issues/132) | [TASK-85] M13-07 coturn resilience/capacity |
| `134` | [#133](https://github.com/mhyrzt/mahfel/issues/133) | [TASK-86] M14-01 RTC load harness |
| `135` | [#134](https://github.com/mhyrzt/mahfel/issues/134) | [TASK-87] M14-02 Concurrent-session admission tests |
| `136` | [#135](https://github.com/mhyrzt/mahfel/issues/135) | [TASK-88] M14-03 Network impairment tests |
| `137` | [#136](https://github.com/mhyrzt/mahfel/issues/136) | [TASK-89] M14-04 Control/collaboration load |
| `138` | [#137](https://github.com/mhyrzt/mahfel/issues/137) | [TASK-90] M14-05 Resource telemetry |
| `139` | [#138](https://github.com/mhyrzt/mahfel/issues/138) | [TASK-91] M14-06 Scheduler thresholds report |
| `140` | [#139](https://github.com/mhyrzt/mahfel/issues/139) | [TASK-92] M15-01 Metrics catalog/budgets |
| `141` | [#140](https://github.com/mhyrzt/mahfel/issues/140) | [TASK-93] M15-02 API/WebSocket dashboards |
| `142` | [#141](https://github.com/mhyrzt/mahfel/issues/141) | [TASK-94] M15-03 Media quality/capacity dashboard |
| `143` | [#142](https://github.com/mhyrzt/mahfel/issues/142) | [TASK-95] M15-04 Infrastructure/worker dashboards |
| `144` | [#143](https://github.com/mhyrzt/mahfel/issues/143) | [TASK-96] M15-05 OpenTelemetry correlation |
| `145` | [#144](https://github.com/mhyrzt/mahfel/issues/144) | [TASK-97] M15-06 Alerts and runbooks |
| `146` | [#145](https://github.com/mhyrzt/mahfel/issues/145) | [TASK-98] M15-07 Error/AI usage telemetry |
| `154` | [#146](https://github.com/mhyrzt/mahfel/issues/146) | [TASK-106] Replace local MinIO with RustFS |
| `155` | [#147](https://github.com/mhyrzt/mahfel/issues/147) | [TASK-107] Split frontend and backend Just workflows |
| `156` | [#148](https://github.com/mhyrzt/mahfel/issues/148) | [TASK-108] Use structured PostgreSQL configuration |
| `157` | [#149](https://github.com/mhyrzt/mahfel/issues/149) | [TASK-109] Add trusted local HTTPS with Caddy |
| `158` | [#150](https://github.com/mhyrzt/mahfel/issues/150) | [TASK-110] Adopt cargo-nextest for Rust tests |
| `159` | [#151](https://github.com/mhyrzt/mahfel/issues/151) | [TASK-111] Strengthen frontend lint policy |
| `160` | [#152](https://github.com/mhyrzt/mahfel/issues/152) | [TASK-112] Introduce mahfel-application layer and invert store dependencies |
| `161` | [#153](https://github.com/mhyrzt/mahfel/issues/153) | [TASK-113] Extract shared serve/probe runtime and slim API composition root |
| `162` | [#154](https://github.com/mhyrzt/mahfel/issues/154) | [TASK-114] M2-09 Service principals |
| `163` | [#155](https://github.com/mhyrzt/mahfel/issues/155) | [TASK-115] OpenAPI generation and Bruno collection |
| `164` | [#156](https://github.com/mhyrzt/mahfel/issues/156) | [TASK-116] Replace Dex with Keycloak as the local OIDC provider |
| `165` | [#157](https://github.com/mhyrzt/mahfel/issues/157) | [TASK-117] ADR 0019 and re-scope M2 authentication tasks |
| `166` | [#158](https://github.com/mhyrzt/mahfel/issues/158) | [TASK-118] API resource-server authentication and identity provisioning |
| `167` | [#159](https://github.com/mhyrzt/mahfel/issues/159) | [TASK-119] Seed Keycloak users and mirror them into Postgres |
| `168` | [#160](https://github.com/mhyrzt/mahfel/issues/160) | [TASK-120] Frontend authentication with NextAuth, Keycloak, and a BFF |
| `169` | [#161](https://github.com/mhyrzt/mahfel/issues/161) | [TASK-121] Verify Keycloak tokens on the WebSocket gateway |
| `170` | [#162](https://github.com/mhyrzt/mahfel/issues/162) | [TASK-122] Claim-based provisioning with an app_user_id claim |
| `171` | [#163](https://github.com/mhyrzt/mahfel/issues/163) | [TASK-123] M4-03 Participation records |
| `172` | [#164](https://github.com/mhyrzt/mahfel/issues/164) | [TASK-124] M4-04 Session capabilities and authorization |
| `173` | [#165](https://github.com/mhyrzt/mahfel/issues/165) | [TASK-125] M4-06 Admission, lobby, and session access overrides |
| `174` | [#166](https://github.com/mhyrzt/mahfel/issues/166) | [TASK-126] M4-10 Frontend session state and roster |
| `175` | [#167](https://github.com/mhyrzt/mahfel/issues/167) | [TASK-127] M4-11 Frontend moderation and admission UI |
| `176` | [#168](https://github.com/mhyrzt/mahfel/issues/168) | [TASK-128] M4-12 Frontend session lifecycle and active speaker UI |
| `177` | [#169](https://github.com/mhyrzt/mahfel/issues/169) | [TASK-129] M4-13 Invite link model and management API |
| `178` | [#170](https://github.com/mhyrzt/mahfel/issues/170) | [TASK-130] M4-14 Link join and approval flow |
| `179` | [#171](https://github.com/mhyrzt/mahfel/issues/171) | [TASK-131] M4-15 Frontend invite links UI |
| `180` | [#172](https://github.com/mhyrzt/mahfel/issues/172) | [TASK-132] M4-16 Session scheduling and title |
| `181` | [#173](https://github.com/mhyrzt/mahfel/issues/173) | [TASK-133] Cross-replica session connection coordination |
| `182` | [#174](https://github.com/mhyrzt/mahfel/issues/174) | [TASK-134] Add Minikube development deployment |
| `183` | [#175](https://github.com/mhyrzt/mahfel/issues/175) | [TASK-135] Make Minikube startup diagnostics actionable |
| `184` | [#176](https://github.com/mhyrzt/mahfel/issues/176) | [TASK-136] M3-10 Media-plane producer control |
| `185` | [#177](https://github.com/mhyrzt/mahfel/issues/177) | [TASK-137] Frontend i18n foundation for English and Persian |
| `186` | [#178](https://github.com/mhyrzt/mahfel/issues/178) | [TASK-138] Generate WebSocket AsyncAPI contract from Rust |
| `187` | [#179](https://github.com/mhyrzt/mahfel/issues/179) | [TASK-139] Use typed TOML fixtures for development data |
| `188` | [#180](https://github.com/mhyrzt/mahfel/issues/180) | [TASK-140] Redesign room sessions and chat attachments |
| `189` | [#181](https://github.com/mhyrzt/mahfel/issues/181) | [TASK-141] Move room management into a settings dialog |
| `190` | [#182](https://github.com/mhyrzt/mahfel/issues/182) | [TASK-142] Complete responsive room management and session files |
| `191` | [#183](https://github.com/mhyrzt/mahfel/issues/183) | [TASK-143] Apply explicit development seeds in local stacks |
| `192` | [#184](https://github.com/mhyrzt/mahfel/issues/184) | [TASK-144] Recover from rejected browser access tokens |
| `193` | [#185](https://github.com/mhyrzt/mahfel/issues/185) | [TASK-145] Use Storybook workspace design in the live application |
| `194` | [#186](https://github.com/mhyrzt/mahfel/issues/186) | [TASK-146] Restore RFC 3339 API timestamps for room and session flows |
| `195` | [#187](https://github.com/mhyrzt/mahfel/issues/187) | [TASK-147] Keep browser login active through Keycloak token refresh |
| `196` | [#188](https://github.com/mhyrzt/mahfel/issues/188) | [TASK-148] Restore authenticated API requests after fresh Compose startup |
| `197` | [#189](https://github.com/mhyrzt/mahfel/issues/189) | [TASK-149] Refine live session workspace layout and files |
| `198` | [#190](https://github.com/mhyrzt/mahfel/issues/190) | [TASK-150] Support multiple concurrent live sessions in one room |
| `199` | [#191](https://github.com/mhyrzt/mahfel/issues/191) | [TASK-151] Manage per-participant live session access |
| `200` | [#192](https://github.com/mhyrzt/mahfel/issues/192) | [TASK-152] Repair workspace calendar and session shell polish |
| `201` | [#193](https://github.com/mhyrzt/mahfel/issues/193) | [TASK-153] Polish room chat and live message notices |
| `202` | [#194](https://github.com/mhyrzt/mahfel/issues/194) | [TASK-154] Refresh landing and room detail presentation |
| `203` | [#195](https://github.com/mhyrzt/mahfel/issues/195) | [TASK-155] Simplify account menu and preferences |
| `204` | [#196](https://github.com/mhyrzt/mahfel/issues/196) | [TASK-156] Refine presentation carousel and sidebar conversations |
| `205` | [#197](https://github.com/mhyrzt/mahfel/issues/197) | [TASK-157] Replace sidebar browsers with navigation menus |
| `206` | [#198](https://github.com/mhyrzt/mahfel/issues/198) | [TASK-158] Polish chat, session cards, calendar, and uploads |
| `208` | [#199](https://github.com/mhyrzt/mahfel/issues/199) | [TASK-160] Add synchronized video presentation with Vidstack |
| `209` | [#200](https://github.com/mhyrzt/mahfel/issues/200) | [TASK-161] Align frontend AGENTS.md and DESIGN.md with the implemented design |
| `210` | [#201](https://github.com/mhyrzt/mahfel/issues/201) | [TASK-162] Add client-side noise suppression |
| `211` | [#202](https://github.com/mhyrzt/mahfel/issues/202) | [TASK-163] Add client-side virtual background and blur |
| `212` | [#203](https://github.com/mhyrzt/mahfel/issues/203) | [TASK-164] Add session live streaming over RTMP |
| `213` | [#204](https://github.com/mhyrzt/mahfel/issues/204) | [TASK-165] Build deterministic recording replay and MP4 renderer |
| `214` | [#205](https://github.com/mhyrzt/mahfel/issues/205) | [TASK-166] Automatically generate durable recording MP4 exports |
| `215` | [#206](https://github.com/mhyrzt/mahfel/issues/206) | [TASK-167] Expose recording exports and visual stories |
| `216` | [#207](https://github.com/mhyrzt/mahfel/issues/207) | [TASK-168] Add browser-free authenticated recording image rendering |
| `229` | [#208](https://github.com/mhyrzt/mahfel/issues/208) | [TASK-181] Fix Persian shell sidebar overlap |
| `230` | [#209](https://github.com/mhyrzt/mahfel/issues/209) | [TASK-182] Show session connection health in the header |
| `231` | [#210](https://github.com/mhyrzt/mahfel/issues/210) | [TASK-183] Polish session connection hover card |
| `232` | [#211](https://github.com/mhyrzt/mahfel/issues/211) | [TASK-184] Apply shadcn maxing to forms and shared primitives |
| `233` | [#212](https://github.com/mhyrzt/mahfel/issues/212) | [TASK-185] Apply shadcn maxing to permissions playback and remaining controls |
| `234` | [#213](https://github.com/mhyrzt/mahfel/issues/213) | [TASK-186] Evaluate shadcn Calendar and Message Scroller compatibility |
| `235` | [#214](https://github.com/mhyrzt/mahfel/issues/214) | [TASK-187] Add session polls quizzes and raise hand |
| `236` | [#215](https://github.com/mhyrzt/mahfel/issues/215) | [doc-1] Shadcn maxing frontend audit and migration proposal |
| `237` | [#216](https://github.com/mhyrzt/mahfel/issues/216) | Address High web image advisories from web-v1.1.1 |
| `238` | [#217](https://github.com/mhyrzt/mahfel/issues/217) | Preserve upload MIME metadata during S3 promotion |
| `239` | [#218](https://github.com/mhyrzt/mahfel/issues/218) | Triage Critical and High backend runtime image findings |
| `240` | [#219](https://github.com/mhyrzt/mahfel/issues/219) | Parto procedural mascot, playful motion and landing companion |
| `241` | [#220](https://github.com/mhyrzt/mahfel/issues/220) | Expose frontend application metadata in the health response |
