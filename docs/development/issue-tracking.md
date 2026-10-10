# Issue tracking

[GitLab issues](https://hamgit.ir/hamdars/mahfel/-/issues) are the source of truth for executable
work. [GitLab milestones](https://hamgit.ir/hamdars/mahfel/-/milestones) group delivery scope;
the [engineering roadmap](roadmap.md) describes sequencing and product assumptions.

Search existing issues before creating a task. Record the problem, scope, acceptance criteria,
dependencies, plan, and validation evidence in the issue. Use verified GitLab usernames for
assignees and update the relevant milestone. Trivial mechanical work does not require a new issue.

Use `status::To Do` and `status::In Progress` on open tasks. After validation, record the final
summary, set `status::Done`, and close the issue. Archived work uses `status::Archived` and remains
closed; archival does not mean its acceptance criteria were completed. Domain labels and
`priority::high`, `priority::medium`, or `priority::low` retain their original meaning.

Use `glab` from this repository with host `hamgit.ir`, or the GitLab API for project `48414`.
Read command help before unfamiliar operations. GitLab owns live task state; this page is a
historical migration index, not a parallel task database.

## Migration record

On 2026-10-07, 195 task records (192 active records and three archived records), one frontend audit
document, and 18 milestone records were migrated from the retired local tracker. Existing GitLab
issues #1 through #40 were preserved: title, description, state, labels, assignees, milestone,
and update timestamp were compared against the pre-migration snapshot.

During the initial import, legacy acceptance-criteria numbers were interpreted as GitLab issue
references and generated system mention notes on some existing issues. Readable criteria were
corrected to code-formatted numbers to prevent further accidental references. GitLab rejected
removal of a generated system note with HTTP 403; these notes may remain in existing discussions.
Original issue fields and original discussion content were not edited or deleted.

Migrated issues retain legacy IDs in titles, acceptance criteria, implementation plans, final
summaries, original metadata and timestamps, complete original source, and a source SHA-256.
Dependencies and parent/child relationships have explicit links in issue descriptions. These links
preserve relationship meaning without depending on GitLab tier-specific dependency or hierarchy
features. Legacy `@mahyar` maps to the verified GitLab account `mhyrzt` (Mahyar Riazati).
Agent aliases without GitLab member matches remain in the original metadata and are unassigned.
The audit document is a closed issue labeled `record::document`.

`Done` tasks are closed; `To Do` and `In Progress` tasks remain open unless archived. Active
milestones remain active, preserving the source milestone state. The archived Darkube production
milestone (`m-16`) remains separate and closed; the current VPS/Rastploy production milestone
(`m-17`) retains the original M16 display title.

## Milestone mapping

| Legacy ID | GitLab milestone | Source disposition |
| --- | --- | --- |
| `m-0` | [M0 - Repository and engineering foundation](https://hamgit.ir/hamdars/mahfel/-/milestones/1) | Active |
| `m-1` | [M1 - Control-plane foundation](https://hamgit.ir/hamdars/mahfel/-/milestones/2) | Active |
| `m-2` | [M2 - Authentication and identity](https://hamgit.ir/hamdars/mahfel/-/milestones/3) | Active |
| `m-3` | [M3 - Media-plane proof of concept](https://hamgit.ir/hamdars/mahfel/-/milestones/4) | Active |
| `m-4` | [M4 - Realtime session engine](https://hamgit.ir/hamdars/mahfel/-/milestones/5) | Active |
| `m-5` | [M5 - Persistent chat](https://hamgit.ir/hamdars/mahfel/-/milestones/6) | Active |
| `m-6` | [M6 - Files and S3](https://hamgit.ir/hamdars/mahfel/-/milestones/7) | Active |
| `m-7` | [M7 - Collaborative notes and whiteboard](https://hamgit.ir/hamdars/mahfel/-/milestones/8) | Active |
| `m-8` | [M8 - RabbitMQ worker foundation](https://hamgit.ir/hamdars/mahfel/-/milestones/9) | Active |
| `m-9` | [M9 - Recording](https://hamgit.ir/hamdars/mahfel/-/milestones/10) | Active |
| `m-10` | [M10 - Transcription](https://hamgit.ir/hamdars/mahfel/-/milestones/11) | Active |
| `m-11` | [M11 - AI participant](https://hamgit.ir/hamdars/mahfel/-/milestones/12) | Active |
| `m-12` | [M12 - AI voice](https://hamgit.ir/hamdars/mahfel/-/milestones/13) | Active |
| `m-13` | [M13 - Scalability and resilience](https://hamgit.ir/hamdars/mahfel/-/milestones/14) | Active |
| `m-14` | [M14 - Load testing](https://hamgit.ir/hamdars/mahfel/-/milestones/15) | Active |
| `m-15` | [M15 - Observability and operations](https://hamgit.ir/hamdars/mahfel/-/milestones/16) | Active |
| `m-16` | [M16 - Production deployment (archived m-16)](https://hamgit.ir/hamdars/mahfel/-/milestones/17) | Archived |
| `m-17` | [M16 - Production deployment](https://hamgit.ir/hamdars/mahfel/-/milestones/18) | Active |

## Task and document mapping

| Legacy ID | GitLab issue | Title |
| --- | --- | --- |
| `TASK-1` | [#41](https://hamgit.ir/hamdars/mahfel/-/work_items/41) | Bootstrap Mahfel production foundation |
| `TASK-1.1` | [#42](https://hamgit.ir/hamdars/mahfel/-/work_items/42) | Scaffold repository and Rust workspace |
| `TASK-1.2` | [#43](https://hamgit.ir/hamdars/mahfel/-/work_items/43) | Scaffold Next.js Bun frontend |
| `TASK-1.3` | [#44](https://hamgit.ir/hamdars/mahfel/-/work_items/44) | Create production application images |
| `TASK-1.4` | [#45](https://hamgit.ir/hamdars/mahfel/-/work_items/45) | Create local Compose infrastructure |
| `TASK-1.5` | [#46](https://hamgit.ir/hamdars/mahfel/-/work_items/46) | Establish observability foundation |
| `TASK-1.6` | [#47](https://hamgit.ir/hamdars/mahfel/-/work_items/47) | Write architecture documentation and ADRs |
| `TASK-1.7` | [#48](https://hamgit.ir/hamdars/mahfel/-/work_items/48) | Create detailed engineering roadmap |
| `TASK-1.8` | [#49](https://hamgit.ir/hamdars/mahfel/-/work_items/49) | Validate and commit the bootstrap |
| `TASK-2` | [#50](https://hamgit.ir/hamdars/mahfel/-/work_items/50) | M1-01 Migration runner and pools |
| `TASK-3` | [#51](https://hamgit.ir/hamdars/mahfel/-/work_items/51) | M1-02 User/room/session core |
| `TASK-4` | [#52](https://hamgit.ir/hamdars/mahfel/-/work_items/52) | M1-03 HTTP contract foundation |
| `TASK-5` | [#53](https://hamgit.ir/hamdars/mahfel/-/work_items/53) | M1-04 Room endpoints |
| `TASK-6` | [#54](https://hamgit.ir/hamdars/mahfel/-/work_items/54) | M1-05 WebSocket gateway |
| `TASK-7` | [#55](https://hamgit.ir/hamdars/mahfel/-/work_items/55) | M1-06 Redis coordination adapter |
| `TASK-8` | [#56](https://hamgit.ir/hamdars/mahfel/-/work_items/56) | M1-07 Authorization/OpenAPI decisions |
| `TASK-9` | [#57](https://hamgit.ir/hamdars/mahfel/-/work_items/57) | M2-01 Identity schema |
| `TASK-10` | [#58](https://hamgit.ir/hamdars/mahfel/-/work_items/58) | M2-02 Local password lifecycle |
| `TASK-11` | [#59](https://hamgit.ir/hamdars/mahfel/-/work_items/59) | M2-03 Session security |
| `TASK-12` | [#60](https://hamgit.ir/hamdars/mahfel/-/work_items/60) | M2-04 OIDC provider abstraction |
| `TASK-13` | [#61](https://hamgit.ir/hamdars/mahfel/-/work_items/61) | M2-05 OIDC provider compatibility |
| `TASK-14` | [#62](https://hamgit.ir/hamdars/mahfel/-/work_items/62) | M2-06 Identity linking |
| `TASK-15` | [#63](https://hamgit.ir/hamdars/mahfel/-/work_items/63) | M2-07 RBAC and classroom roles |
| `TASK-16` | [#64](https://hamgit.ir/hamdars/mahfel/-/work_items/64) | M2-08 Avatar resolution |
| `TASK-17` | [#65](https://hamgit.ir/hamdars/mahfel/-/work_items/65) | M3-01 Worker supervision |
| `TASK-18` | [#66](https://hamgit.ir/hamdars/mahfel/-/work_items/66) | M3-02 Router/room lifecycle |
| `TASK-19` | [#67](https://hamgit.ir/hamdars/mahfel/-/work_items/67) | M3-03 WebRtcServer/transports |
| `TASK-20` | [#68](https://hamgit.ir/hamdars/mahfel/-/work_items/68) | M3-04 Producer lifecycle |
| `TASK-21` | [#69](https://hamgit.ir/hamdars/mahfel/-/work_items/69) | M3-05 Consumer lifecycle |
| `TASK-22` | [#70](https://hamgit.ir/hamdars/mahfel/-/work_items/70) | M3-06 Signaling integration |
| `TASK-23` | [#71](https://hamgit.ir/hamdars/mahfel/-/work_items/71) | M3-07 Frontend RTC client |
| `TASK-24` | [#72](https://hamgit.ir/hamdars/mahfel/-/work_items/72) | M3-08 Screenshare/reconnect |
| `TASK-25` | [#73](https://hamgit.ir/hamdars/mahfel/-/work_items/73) | M3-09 TURN and media metrics |
| `TASK-26` | [#74](https://hamgit.ir/hamdars/mahfel/-/work_items/74) | M4-01 Session vocabulary and lifecycle |
| `TASK-27` | [#75](https://hamgit.ir/hamdars/mahfel/-/work_items/75) | M4-02 Participant lifecycle and reconnect identity |
| `TASK-28` | [#76](https://hamgit.ir/hamdars/mahfel/-/work_items/76) | M4-05 Moderation controls |
| `TASK-29` | [#77](https://hamgit.ir/hamdars/mahfel/-/work_items/77) | M4-07 Active speaker |
| `TASK-30` | [#78](https://hamgit.ir/hamdars/mahfel/-/work_items/78) | M4-08 Media node registry |
| `TASK-31` | [#79](https://hamgit.ir/hamdars/mahfel/-/work_items/79) | M4-09 Session placement scheduler |
| `TASK-32` | [#80](https://hamgit.ir/hamdars/mahfel/-/work_items/80) | M5-01 Channel/message model |
| `TASK-33` | [#81](https://hamgit.ir/hamdars/mahfel/-/work_items/81) | M5-02 History and mutations |
| `TASK-34` | [#82](https://hamgit.ir/hamdars/mahfel/-/work_items/82) | M5-03 Realtime fanout |
| `TASK-35` | [#83](https://hamgit.ir/hamdars/mahfel/-/work_items/83) | M5-04 Reactions/mentions/threads |
| `TASK-36` | [#84](https://hamgit.ir/hamdars/mahfel/-/work_items/84) | M5-05 Typing/presence/read markers |
| `TASK-37` | [#85](https://hamgit.ir/hamdars/mahfel/-/work_items/85) | M5-06 Attachments and abuse controls |
| `TASK-38` | [#86](https://hamgit.ir/hamdars/mahfel/-/work_items/86) | M6-01 Object metadata and ACL |
| `TASK-39` | [#87](https://hamgit.ir/hamdars/mahfel/-/work_items/87) | M6-02 S3 adapter |
| `TASK-40` | [#88](https://hamgit.ir/hamdars/mahfel/-/work_items/88) | M6-03 Presigned upload |
| `TASK-41` | [#89](https://hamgit.ir/hamdars/mahfel/-/work_items/89) | M6-04 Authorized download |
| `TASK-42` | [#90](https://hamgit.ir/hamdars/mahfel/-/work_items/90) | M6-05 File contexts |
| `TASK-43` | [#91](https://hamgit.ir/hamdars/mahfel/-/work_items/91) | M6-06 Validation/security strategy |
| `TASK-44` | [#92](https://hamgit.ir/hamdars/mahfel/-/work_items/92) | M7-01 CRDT document identity |
| `TASK-45` | [#93](https://hamgit.ir/hamdars/mahfel/-/work_items/93) | M7-02 Yrs synchronization gateway |
| `TASK-46` | [#94](https://hamgit.ir/hamdars/mahfel/-/work_items/94) | M7-03 Persistence/checkpoints |
| `TASK-47` | [#95](https://hamgit.ir/hamdars/mahfel/-/work_items/95) | M7-04 Awareness/presence |
| `TASK-48` | [#96](https://hamgit.ir/hamdars/mahfel/-/work_items/96) | M7-05 Collaborative notes UI |
| `TASK-49` | [#97](https://hamgit.ir/hamdars/mahfel/-/work_items/97) | M7-06 Whiteboard object model |
| `TASK-50` | [#98](https://hamgit.ir/hamdars/mahfel/-/work_items/98) | M8-01 AMQP topology |
| `TASK-51` | [#99](https://hamgit.ir/hamdars/mahfel/-/work_items/99) | M8-02 Reliable publisher |
| `TASK-52` | [#100](https://hamgit.ir/hamdars/mahfel/-/work_items/100) | M8-03 Consumer acknowledgements |
| `TASK-53` | [#101](https://hamgit.ir/hamdars/mahfel/-/work_items/101) | M8-04 Retry and DLQ |
| `TASK-54` | [#102](https://hamgit.ir/hamdars/mahfel/-/work_items/102) | M8-05 Idempotency |
| `TASK-55` | [#103](https://hamgit.ir/hamdars/mahfel/-/work_items/103) | M8-06 Transactional outbox |
| `TASK-56` | [#104](https://hamgit.ir/hamdars/mahfel/-/work_items/104) | M9-01 Recording architecture spike |
| `TASK-57` | [#105](https://hamgit.ir/hamdars/mahfel/-/work_items/105) | M9-02 Recording notice and authorization |
| `TASK-58` | [#106](https://hamgit.ir/hamdars/mahfel/-/work_items/106) | M9-03 Recording lifecycle |
| `TASK-59` | [#107](https://hamgit.ir/hamdars/mahfel/-/work_items/107) | M9-04 Capture and upload |
| `TASK-60` | [#108](https://hamgit.ir/hamdars/mahfel/-/work_items/108) | M9-05 Finalization worker |
| `TASK-61` | [#109](https://hamgit.ir/hamdars/mahfel/-/work_items/109) | M9-06 Playback and retention |
| `TASK-62` | [#110](https://hamgit.ir/hamdars/mahfel/-/work_items/110) | M10-01 STT provider contract |
| `TASK-63` | [#111](https://hamgit.ir/hamdars/mahfel/-/work_items/111) | M10-02 Live versus post-session decision |
| `TASK-64` | [#112](https://hamgit.ir/hamdars/mahfel/-/work_items/112) | M10-03 Transcript data model |
| `TASK-65` | [#113](https://hamgit.ir/hamdars/mahfel/-/work_items/113) | M10-04 Speaker mapping |
| `TASK-66` | [#114](https://hamgit.ir/hamdars/mahfel/-/work_items/114) | M10-05 Transcription jobs |
| `TASK-67` | [#115](https://hamgit.ir/hamdars/mahfel/-/work_items/115) | M10-06 Transcript API/UI/export/search |
| `TASK-68` | [#116](https://hamgit.ir/hamdars/mahfel/-/work_items/116) | M11-01 AI participant domain |
| `TASK-69` | [#117](https://hamgit.ir/hamdars/mahfel/-/work_items/117) | M11-02 LLM provider abstraction |
| `TASK-70` | [#118](https://hamgit.ir/hamdars/mahfel/-/work_items/118) | M11-03 Mention workflow |
| `TASK-71` | [#119](https://hamgit.ir/hamdars/mahfel/-/work_items/119) | M11-04 Context assembler |
| `TASK-72` | [#120](https://hamgit.ir/hamdars/mahfel/-/work_items/120) | M11-05 Prompt safety boundaries |
| `TASK-73` | [#121](https://hamgit.ir/hamdars/mahfel/-/work_items/121) | M11-06 AI RBAC and administration |
| `TASK-74` | [#122](https://hamgit.ir/hamdars/mahfel/-/work_items/122) | M12-01 Spoken request capture |
| `TASK-75` | [#123](https://hamgit.ir/hamdars/mahfel/-/work_items/123) | M12-02 Low-latency STT path |
| `TASK-76` | [#124](https://hamgit.ir/hamdars/mahfel/-/work_items/124) | M12-03 TTS abstraction |
| `TASK-77` | [#125](https://hamgit.ir/hamdars/mahfel/-/work_items/125) | M12-04 AI mediasoup producer |
| `TASK-78` | [#126](https://hamgit.ir/hamdars/mahfel/-/work_items/126) | M12-05 Turn interruption and mute |
| `TASK-79` | [#127](https://hamgit.ir/hamdars/mahfel/-/work_items/127) | M13-01 Stateless API scaling |
| `TASK-80` | [#128](https://hamgit.ir/hamdars/mahfel/-/work_items/128) | M13-02 Worker concurrency scaling |
| `TASK-81` | [#129](https://hamgit.ir/hamdars/mahfel/-/work_items/129) | M13-03 Database capacity |
| `TASK-82` | [#130](https://hamgit.ir/hamdars/mahfel/-/work_items/130) | M13-04 Redis/RabbitMQ failure drills |
| `TASK-83` | [#131](https://hamgit.ir/hamdars/mahfel/-/work_items/131) | M13-05 Media drain and rolling deploy |
| `TASK-84` | [#132](https://hamgit.ir/hamdars/mahfel/-/work_items/132) | M13-06 Media node loss/reconnect |
| `TASK-85` | [#133](https://hamgit.ir/hamdars/mahfel/-/work_items/133) | M13-07 coturn resilience/capacity |
| `TASK-86` | [#134](https://hamgit.ir/hamdars/mahfel/-/work_items/134) | M14-01 RTC load harness |
| `TASK-87` | [#135](https://hamgit.ir/hamdars/mahfel/-/work_items/135) | M14-02 Multi-classroom/admission tests |
| `TASK-88` | [#136](https://hamgit.ir/hamdars/mahfel/-/work_items/136) | M14-03 Network impairment tests |
| `TASK-89` | [#137](https://hamgit.ir/hamdars/mahfel/-/work_items/137) | M14-04 Control/collaboration load |
| `TASK-90` | [#138](https://hamgit.ir/hamdars/mahfel/-/work_items/138) | M14-05 Resource telemetry |
| `TASK-91` | [#139](https://hamgit.ir/hamdars/mahfel/-/work_items/139) | M14-06 Scheduler thresholds report |
| `TASK-92` | [#140](https://hamgit.ir/hamdars/mahfel/-/work_items/140) | M15-01 Metrics catalog/budgets |
| `TASK-93` | [#141](https://hamgit.ir/hamdars/mahfel/-/work_items/141) | M15-02 API/WebSocket dashboards |
| `TASK-94` | [#142](https://hamgit.ir/hamdars/mahfel/-/work_items/142) | M15-03 Media quality/capacity dashboard |
| `TASK-95` | [#143](https://hamgit.ir/hamdars/mahfel/-/work_items/143) | M15-04 Infrastructure/worker dashboards |
| `TASK-96` | [#144](https://hamgit.ir/hamdars/mahfel/-/work_items/144) | M15-05 OpenTelemetry correlation |
| `TASK-97` | [#145](https://hamgit.ir/hamdars/mahfel/-/work_items/145) | M15-06 Alerts and runbooks |
| `TASK-98` | [#146](https://hamgit.ir/hamdars/mahfel/-/work_items/146) | M15-07 Error/AI usage telemetry |
| `TASK-99` | [#147](https://hamgit.ir/hamdars/mahfel/-/work_items/147) | M16-01 Platform deployment/config |
| `TASK-100` | [#148](https://hamgit.ir/hamdars/mahfel/-/work_items/148) | M16-02 HTTP scaling rollout |
| `TASK-101` | [#149](https://hamgit.ir/hamdars/mahfel/-/work_items/149) | M16-03 Media public networking |
| `TASK-102` | [#150](https://hamgit.ir/hamdars/mahfel/-/work_items/150) | M16-04 UDP platform qualification |
| `TASK-103` | [#151](https://hamgit.ir/hamdars/mahfel/-/work_items/151) | M16-05 Production coturn |
| `TASK-104` | [#152](https://hamgit.ir/hamdars/mahfel/-/work_items/152) | M16-06 Media placement/draining rollout |
| `TASK-105` | [#153](https://hamgit.ir/hamdars/mahfel/-/work_items/153) | M16-07 Production acceptance/runbooks |
| `TASK-106` | [#154](https://hamgit.ir/hamdars/mahfel/-/work_items/154) | Replace local MinIO with RustFS |
| `TASK-107` | [#155](https://hamgit.ir/hamdars/mahfel/-/work_items/155) | Split frontend and backend Just workflows |
| `TASK-108` | [#156](https://hamgit.ir/hamdars/mahfel/-/work_items/156) | Use structured PostgreSQL configuration |
| `TASK-109` | [#157](https://hamgit.ir/hamdars/mahfel/-/work_items/157) | Add trusted local HTTPS with Caddy |
| `TASK-110` | [#158](https://hamgit.ir/hamdars/mahfel/-/work_items/158) | Adopt cargo-nextest for Rust tests |
| `TASK-111` | [#159](https://hamgit.ir/hamdars/mahfel/-/work_items/159) | Strengthen frontend lint policy |
| `TASK-112` | [#160](https://hamgit.ir/hamdars/mahfel/-/work_items/160) | Introduce mahfel-application layer and invert store dependencies |
| `TASK-113` | [#161](https://hamgit.ir/hamdars/mahfel/-/work_items/161) | Extract shared serve/probe runtime and slim API composition root |
| `TASK-114` | [#162](https://hamgit.ir/hamdars/mahfel/-/work_items/162) | M2-09 Service principals |
| `TASK-115` | [#163](https://hamgit.ir/hamdars/mahfel/-/work_items/163) | OpenAPI generation and Bruno collection |
| `TASK-116` | [#164](https://hamgit.ir/hamdars/mahfel/-/work_items/164) | Replace Dex with Keycloak as the local OIDC provider |
| `TASK-117` | [#165](https://hamgit.ir/hamdars/mahfel/-/work_items/165) | ADR 0019 and re-scope M2 authentication tasks |
| `TASK-118` | [#166](https://hamgit.ir/hamdars/mahfel/-/work_items/166) | API resource-server authentication and identity provisioning |
| `TASK-119` | [#167](https://hamgit.ir/hamdars/mahfel/-/work_items/167) | Seed Keycloak users and mirror them into Postgres |
| `TASK-120` | [#168](https://hamgit.ir/hamdars/mahfel/-/work_items/168) | Frontend authentication with NextAuth, Keycloak, and a BFF |
| `TASK-121` | [#169](https://hamgit.ir/hamdars/mahfel/-/work_items/169) | Verify Keycloak tokens on the WebSocket gateway |
| `TASK-122` | [#170](https://hamgit.ir/hamdars/mahfel/-/work_items/170) | LMS-style provisioning with an app_user_id claim |
| `TASK-123` | [#171](https://hamgit.ir/hamdars/mahfel/-/work_items/171) | M4-03 Participation records |
| `TASK-124` | [#172](https://hamgit.ir/hamdars/mahfel/-/work_items/172) | M4-04 Session capabilities and authorization |
| `TASK-125` | [#173](https://hamgit.ir/hamdars/mahfel/-/work_items/173) | M4-06 Admission, lobby, and session access overrides |
| `TASK-126` | [#174](https://hamgit.ir/hamdars/mahfel/-/work_items/174) | M4-10 Frontend session state and roster |
| `TASK-127` | [#175](https://hamgit.ir/hamdars/mahfel/-/work_items/175) | M4-11 Frontend moderation and admission UI |
| `TASK-128` | [#176](https://hamgit.ir/hamdars/mahfel/-/work_items/176) | M4-12 Frontend session lifecycle and active speaker UI |
| `TASK-129` | [#177](https://hamgit.ir/hamdars/mahfel/-/work_items/177) | M4-13 Invite link model and management API |
| `TASK-130` | [#178](https://hamgit.ir/hamdars/mahfel/-/work_items/178) | M4-14 Link join and approval flow |
| `TASK-131` | [#179](https://hamgit.ir/hamdars/mahfel/-/work_items/179) | M4-15 Frontend invite links UI |
| `TASK-132` | [#180](https://hamgit.ir/hamdars/mahfel/-/work_items/180) | M4-16 Session scheduling and title |
| `TASK-133` | [#181](https://hamgit.ir/hamdars/mahfel/-/work_items/181) | Cross-replica session connection coordination |
| `TASK-134` | [#182](https://hamgit.ir/hamdars/mahfel/-/work_items/182) | Add Minikube development deployment |
| `TASK-135` | [#183](https://hamgit.ir/hamdars/mahfel/-/work_items/183) | Make Minikube startup diagnostics actionable |
| `TASK-136` | [#184](https://hamgit.ir/hamdars/mahfel/-/work_items/184) | M3-10 Media-plane producer control |
| `TASK-137` | [#185](https://hamgit.ir/hamdars/mahfel/-/work_items/185) | Frontend i18n foundation for English and Persian |
| `TASK-138` | [#186](https://hamgit.ir/hamdars/mahfel/-/work_items/186) | Generate WebSocket AsyncAPI contract from Rust |
| `TASK-139` | [#187](https://hamgit.ir/hamdars/mahfel/-/work_items/187) | Use typed TOML fixtures for development data |
| `TASK-140` | [#188](https://hamgit.ir/hamdars/mahfel/-/work_items/188) | Redesign room sessions and chat attachments |
| `TASK-141` | [#189](https://hamgit.ir/hamdars/mahfel/-/work_items/189) | Move room management into a settings dialog |
| `TASK-142` | [#190](https://hamgit.ir/hamdars/mahfel/-/work_items/190) | Complete responsive room management and session files |
| `TASK-143` | [#191](https://hamgit.ir/hamdars/mahfel/-/work_items/191) | Apply explicit development seeds in local stacks |
| `TASK-144` | [#192](https://hamgit.ir/hamdars/mahfel/-/work_items/192) | Recover from rejected browser access tokens |
| `TASK-145` | [#193](https://hamgit.ir/hamdars/mahfel/-/work_items/193) | Use Storybook workspace design in the live application |
| `TASK-146` | [#194](https://hamgit.ir/hamdars/mahfel/-/work_items/194) | Restore RFC 3339 API timestamps for room and session flows |
| `TASK-147` | [#195](https://hamgit.ir/hamdars/mahfel/-/work_items/195) | Keep browser login active through Keycloak token refresh |
| `TASK-148` | [#196](https://hamgit.ir/hamdars/mahfel/-/work_items/196) | Restore authenticated API requests after fresh Compose startup |
| `TASK-149` | [#197](https://hamgit.ir/hamdars/mahfel/-/work_items/197) | Refine live session workspace layout and files |
| `TASK-150` | [#198](https://hamgit.ir/hamdars/mahfel/-/work_items/198) | Support multiple concurrent live sessions in one room |
| `TASK-151` | [#199](https://hamgit.ir/hamdars/mahfel/-/work_items/199) | Manage per-participant live session access |
| `TASK-152` | [#200](https://hamgit.ir/hamdars/mahfel/-/work_items/200) | Repair workspace calendar and session shell polish |
| `TASK-153` | [#201](https://hamgit.ir/hamdars/mahfel/-/work_items/201) | Polish room chat and live message notices |
| `TASK-154` | [#202](https://hamgit.ir/hamdars/mahfel/-/work_items/202) | Refresh landing and room detail presentation |
| `TASK-155` | [#203](https://hamgit.ir/hamdars/mahfel/-/work_items/203) | Simplify account menu and preferences |
| `TASK-156` | [#204](https://hamgit.ir/hamdars/mahfel/-/work_items/204) | Refine presentation carousel and sidebar conversations |
| `TASK-157` | [#205](https://hamgit.ir/hamdars/mahfel/-/work_items/205) | Replace sidebar browsers with navigation menus |
| `TASK-158` | [#206](https://hamgit.ir/hamdars/mahfel/-/work_items/206) | Polish chat, session cards, calendar, and uploads |
| `TASK-159` | [#207](https://hamgit.ir/hamdars/mahfel/-/work_items/207) | Add validation and image publishing CI |
| `TASK-160` | [#208](https://hamgit.ir/hamdars/mahfel/-/work_items/208) | Add synchronized video presentation with Vidstack |
| `TASK-161` | [#209](https://hamgit.ir/hamdars/mahfel/-/work_items/209) | Align frontend AGENTS.md and DESIGN.md with the implemented design |
| `TASK-162` | [#210](https://hamgit.ir/hamdars/mahfel/-/work_items/210) | Add client-side noise suppression |
| `TASK-163` | [#211](https://hamgit.ir/hamdars/mahfel/-/work_items/211) | Add client-side virtual background and blur |
| `TASK-164` | [#212](https://hamgit.ir/hamdars/mahfel/-/work_items/212) | Add session live streaming over RTMP |
| `TASK-165` | [#213](https://hamgit.ir/hamdars/mahfel/-/work_items/213) | Build deterministic recording replay and MP4 renderer |
| `TASK-166` | [#214](https://hamgit.ir/hamdars/mahfel/-/work_items/214) | Automatically generate durable recording MP4 exports |
| `TASK-167` | [#215](https://hamgit.ir/hamdars/mahfel/-/work_items/215) | Expose recording exports and visual stories |
| `TASK-168` | [#216](https://hamgit.ir/hamdars/mahfel/-/work_items/216) | Add browser-free authenticated recording image rendering |
| `TASK-169` | [#217](https://hamgit.ir/hamdars/mahfel/-/work_items/217) | Publish corrected release images for all apps |
| `TASK-170` | [#218](https://hamgit.ir/hamdars/mahfel/-/work_items/218) | Prepare Kubernetes and Argo CD production deployment |
| `TASK-171` | [#219](https://hamgit.ir/hamdars/mahfel/-/work_items/219) | Prepare in-cluster Coturn and document production GitOps configuration |
| `TASK-172` | [#220](https://hamgit.ir/hamdars/mahfel/-/work_items/220) | Automate fresh VPS Kubernetes bootstrap |
| `TASK-173` | [#221](https://hamgit.ir/hamdars/mahfel/-/work_items/221) | Replace Sealed Secrets with SOPS and age |
| `TASK-174` | [#222](https://hamgit.ir/hamdars/mahfel/-/work_items/222) | Build uv deployment management CLI with TOML config |
| `TASK-175` | [#223](https://hamgit.ir/hamdars/mahfel/-/work_items/223) | Consolidate deployment helpers in the Python CLI |
| `TASK-176` | [#224](https://hamgit.ir/hamdars/mahfel/-/work_items/224) | Restore shell Argo renderer and add deployment CLI CI |
| `TASK-177` | [#225](https://hamgit.ir/hamdars/mahfel/-/work_items/225) | Review Hamravesh networking requirements against official docs |
| `TASK-178` | [#226](https://hamgit.ir/hamdars/mahfel/-/work_items/226) | Remove retired deployment tooling from Mahfel |
| `TASK-179` | [#227](https://hamgit.ir/hamdars/mahfel/-/work_items/227) | Integrate Mahfel production deployment with Rastploy |
| `TASK-180` | [#228](https://hamgit.ir/hamdars/mahfel/-/work_items/228) | Restore release publishing and complete production deployment |
| `TASK-181` | [#229](https://hamgit.ir/hamdars/mahfel/-/work_items/229) | Fix Persian shell sidebar overlap |
| `TASK-182` | [#230](https://hamgit.ir/hamdars/mahfel/-/work_items/230) | Show session connection health in the header |
| `TASK-183` | [#231](https://hamgit.ir/hamdars/mahfel/-/work_items/231) | Polish session connection hover card |
| `TASK-184` | [#232](https://hamgit.ir/hamdars/mahfel/-/work_items/232) | Apply shadcn maxing to forms and shared primitives |
| `TASK-185` | [#233](https://hamgit.ir/hamdars/mahfel/-/work_items/233) | Apply shadcn maxing to permissions playback and remaining controls |
| `TASK-186` | [#234](https://hamgit.ir/hamdars/mahfel/-/work_items/234) | Evaluate shadcn Calendar and Message Scroller compatibility |
| `TASK-187` | [#235](https://hamgit.ir/hamdars/mahfel/-/work_items/235) | Add session polls quizzes and raise hand |
| `doc-1` | [#236](https://hamgit.ir/hamdars/mahfel/-/work_items/236) | Shadcn maxing frontend audit and migration proposal |
