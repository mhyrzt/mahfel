# ADR 0004: Use mediasoup for the SFU media engine

## Status

Accepted

## Context

Mahfel needs a proven SFU with routers, transports, simulcast/SVC, bandwidth adaptation, active
speaker support, screen sharing, statistics, and future media taps.

## Decision

Use the maintained Rust `mediasoup` bindings and their native mediasoup worker. Do not implement a
WebRTC stack.

## Consequences

The media image needs C++/Python/native build dependencies and a slower build. The mature SFU model
reduces protocol risk. mediasoup-specific types stay behind `rtc` and the internal media protocol.

## Alternatives considered

`str0m` and `webrtc-rs` offer lower-level Rust WebRTC building blocks but would require Mahfel to
construct and operate more SFU behavior. They may be reconsidered only through a future ADR.
