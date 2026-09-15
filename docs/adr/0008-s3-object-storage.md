# ADR 0008: Use S3-compatible object storage

## Status

Accepted

## Context

Files, recordings, transcripts, thumbnails, and exports are large artifacts unsuited to container
filesystems or relational byte columns.

## Decision

Use the AWS S3 API with configurable compatible endpoints; use MinIO locally. Browsers normally
upload/download using short-lived presigned operations authorized by `mahfel-api`.

## Consequences

Application containers stay stateless and avoid proxying large bodies. Metadata/ACL consistency,
orphan cleanup, checksums, MIME validation, and lifecycle policy need explicit implementation.

## Alternatives considered

Local persistent volumes conflict with horizontal stateless deployment. API-proxied storage wastes
bandwidth. Database blobs complicate backup and scaling.
