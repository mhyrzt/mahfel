# Files and object storage

`mahfel-api` authorizes file operations and stores metadata in PostgreSQL. The browser sends bytes
directly to S3-compatible storage. The API uses an internal S3 endpoint to verify uploads; it
signs browser URLs against a public endpoint. In local Minikube these are `http://rustfs:9000` and
`https://s3.mahfel.localhost`. Both endpoint names must address the same bucket.

## Transfer contract

1. `POST /v1/files/uploads` accepts a context kind and ID, file name, MIME type, byte count,
   and base64 SHA-256 checksum. It returns a five-minute signed PUT URL and required headers.
2. The browser sends the file directly to that URL and calls
   `POST /v1/files/{file_id}/complete`. Metadata is `pending` until this call moves it through
   `quarantined` to `ready`.
3. Completion reads at most 50 MiB from S3, checks the exact size, SHA-256 digest, declared and
   stored MIME type, and format markers. It then copies the object from `uploads/<random UUID>`
   to `files/<same UUID>`, verifies the copy, and publishes the final key in PostgreSQL.
4. `GET /v1/files?context_kind=...&context_id=...` lists ready files in one context with
   `limit` and `offset` pagination (up to 100 per request).
   `GET /v1/files/{file_id}/download` returns a one-minute signed GET URL with attachment
   disposition and private, no-store caching. Each authorization is recorded in
   `mahfel.file_download_audit` without the signed URL.
5. `DELETE /v1/files/{file_id}` detaches the file from every live presentation selection, marks the
   row as deleting, deletes both final and staging objects, then removes metadata. A file embedded
   in a collaboration document is retained and returns a conflict. Deleting a chat message queues
   one durable cleanup job per attachment in the same database transaction; the worker removes
   those objects and metadata. The existing worker sweep also retries expired pending, quarantined,
   rejected, and deleting rows in batches of 100 every five minutes. The expiry threshold is 30
   minutes.

The frontend file client performs the signed PUT directly from the browser. Its BFF only forwards
the small metadata requests. Returned signed URLs are capabilities; callers must avoid logging,
caching, or sharing them.

## Context and access rules

The file row references exactly one room, session, chat message, avatar actor, or recording
session. PostgreSQL foreign keys prevent a target from being deleted while a file row still
references it. Session and recording access resolves the room through the session; chat files
resolve it through the message's channel. A deleted chat message cannot accept a new file. Chat
messages permit at most four pending, quarantined, or ready files; only ready files are visible
to readers. Deleting a message marks its linked files for worker cleanup.

| Context | Upload and read | Delete |
| --- | --- | --- |
| Room, session, chat message | Current room members | File owner or room manager |
| Avatar | Matching actor | Matching actor |
| Recording session | Room managers | File owner or room manager |

Only ready files can be listed or downloaded. Missing contexts and non-members both receive a
not-found result, so the file API does not reveal private room existence. A principal that loses
membership also loses room file access. Recording files have a session association for M9; the
recording pipeline and retention rules are separate work.

## Validation and threat model

Client MIME types and extensions are untrusted. This release accepts PDF, PNG, JPEG, MP4, and WebM;
each file is limited to 50 MiB and each owner to 1 GiB of pending or ready files. The owner quota
is checked under a database row lock to prevent concurrent initiations from exceeding it.
Completion rejects size or checksum mismatches, MIME mismatches, missing format signatures, and
trailing bytes after the PDF EOF, PNG IEND, or JPEG EOI marker. MP4 and WebM are accepted on their
container signature (`ftyp` box and EBML header) rather than a full parse. The verification path
bounds concurrent S3 reads and blocking verification jobs to four each per API process. Invalid
objects remain rejected until worker cleanup.

Marker checks are a type gate, not a malware scanner or full PDF/image parser. Files are served as
attachments by default. The collaboration presentation view fetches authorized signed URLs and
renders PDF pages and images in the browser. No image normalization is performed; uploaded EXIF
and PDF contents therefore remain in the private artifact. A malware scanner or image
normalization stage can be added before the ready transition when inline viewing is introduced.
Presentation selection parses the PDF page tree separately and limits decks to 1000 pages or 50
ordered images. The file API itself does not create slide sequences.

Files selected for a collaboration presentation are referenced by the durable event journal.
Deleting a presented file first removes it from every live presentation selection, clearing a
selection that has no remaining assets, so the operation succeeds and the current presentation
stops referencing the deleted bytes. A file embedded as an asset in a collaboration document is
retained and the delete returns a conflict, because the document still references it.

Object storage must keep the bucket private and expire residual `uploads/` objects after one day.
The worker removes known abandoned objects; the storage lifecycle rule also catches a PUT that
finishes after its metadata has been cleaned up. The final `files/` prefix has no automatic expiry.

## Configuration and checks

The `[storage]` section has internal/public endpoints, region, bucket, credentials, and path-style
addressing; environment overrides use `MAHFEL_STORAGE__*`. Replace all local fixture credentials
in shared or production environments. The API checks the bucket at startup. Configure bucket
CORS for the browser origin, PUT and GET, and the signed request headers; Minikube bootstrap sets
this for its RustFS bucket.

`backend/crates/storage/tests/s3.rs` runs the signed PUT/GET/copy/read/delete contract against a
disposable RustFS container (`MAHFEL_TESTCONTAINERS=1`). It creates and removes a test bucket and
checks CORS and lifecycle configuration. PostgreSQL migration tests cover file constraints and
context-isolated ready-only listing.

From `backend/`, run `MAHFEL_TESTCONTAINERS=1 cargo test -p mahfel-storage --test s3` and
`MAHFEL_TESTCONTAINERS=1 cargo test -p mahfel-postgres --test migrations`. If a development shell
sets an HTTP proxy, exempt `127.0.0.1` and `localhost` so container port mappings stay local.
