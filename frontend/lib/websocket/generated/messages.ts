export interface ActiveSpeakers {
  id?: string;
  participant_ids: string[];
  session_id: string;
  type: "active_speakers";
  v: 1;
}

export interface Auth {
  id?: string;
  ticket?: string | null;
  token?: string | null;
  type: "auth";
  v?: 1;
}

export interface BlockPrincipal {
  actor_id: string;
  expires_at?: number | null;
  id?: string;
  reason?: string | null;
  room_id?: string | null;
  session_id?: string | null;
  type: "block_principal";
  v?: 1;
}

export interface Broadcast {
  from: string;
  id?: string;
  text: string;
  type: "broadcast";
  v: 1;
}

export interface Bye {
  id?: string;
  reason: string;
  type: "bye";
  v: 1;
}

export interface ChatActivity {
  id?: string;
  online_count: number;
  room_id: string;
  type: "chat_activity";
  typing_count: number;
  v: 1;
}

export interface ChatEvent {
  event: {
    author_id: string;
    content?: string | null;
    created_at: string;
    deleted_at?: string | null;
    edited_at?: string | null;
    event_id: string;
    kind: string;
    message_id: string;
    parent_id?: string | null;
    reaction_actor_id?: string | null;
    reaction_emoji?: string | null;
    room_id: string;
    sequence: number;
  };
  id?: string;
  type: "chat_event";
  v: 1;
}

export interface ChatSubscribed {
  head_sequence: number;
  id?: string;
  room_id: string;
  type: "chat_subscribed";
  v: 1;
}

export interface ChatTyping {
  active: boolean;
  id?: string;
  room_id: string;
  type: "chat_typing";
  v?: 1;
}

export interface ClientBroadcast {
  id?: string;
  text: string;
  type: "broadcast";
  v?: 1;
}

export interface ClientEcho {
  id?: string;
  text: string;
  type: "echo";
  v?: 1;
}

export interface ConnectTransport {
  dtls_parameters: string;
  id?: string;
  room_id: string;
  transport_id: string;
  type: "connect_transport";
  v?: 1;
}

export interface Consume {
  id?: string;
  producer_id: string;
  room_id: string;
  rtp_capabilities: string;
  transport_id: string;
  type: "consume";
  v?: 1;
}

export interface Consumed {
  consumer_id: string;
  id?: string;
  paused: boolean;
  room_id: string;
  rtp_parameters: string;
  type: "consumed";
  v: 1;
}

export interface ConsumerPaused {
  consumer_id: string;
  id?: string;
  room_id: string;
  type: "consumer_paused";
  v: 1;
}

export interface ConsumerResumed {
  consumer_id: string;
  id?: string;
  room_id: string;
  type: "consumer_resumed";
  v: 1;
}

export interface CreateTransport {
  id?: string;
  room_id: string;
  type: "create_transport";
  v?: 1;
}

export interface Echo {
  id?: string;
  text: string;
  type: "echo";
  v: 1;
}

export interface Error {
  code: string;
  id?: string;
  message: string;
  type: "error";
  v: 1;
}

export interface Join {
  id?: string;
  session_id: string;
  type: "join";
  v?: 1;
}

export interface JoinRejected {
  code: string;
  id?: string;
  message: string;
  type: "join_rejected";
  v: 1;
}

export interface JoinWaiting {
  id?: string;
  request_id: string;
  session_id: string;
  type: "join_waiting";
  v: 1;
}

export interface Joined {
  id?: string;
  participant_id: string;
  session_id: string;
  type: "joined";
  v: 1;
}

export interface Leave {
  id?: string;
  type: "leave";
  v?: 1;
}

export interface Left {
  id?: string;
  participant_id: string;
  session_id: string;
  type: "left";
  v: 1;
}

export interface MuteParticipant {
  actor_id: string;
  id?: string;
  kind: string;
  session_id: string;
  source?: string | null;
  type: "mute_participant";
  v?: 1;
}

export interface ParticipantJoined {
  id?: string;
  participant_id: string;
  session_id: string;
  type: "participant_joined";
  v: 1;
}

export interface ParticipantLeft {
  id?: string;
  participant_id: string;
  session_id: string;
  type: "participant_left";
  v: 1;
}

export interface ParticipantMuted {
  actor_id: string;
  id?: string;
  kind: string;
  participant_id: string;
  session_id: string;
  source: string;
  type: "participant_muted";
  v: 1;
}

export interface ParticipantRemoved {
  actor_id: string;
  id?: string;
  participant_id: string;
  reason: string;
  session_id: string;
  type: "participant_removed";
  v: 1;
}

export interface ParticipantUnmuted {
  actor_id: string;
  id?: string;
  participant_id: string;
  session_id: string;
  source: string;
  type: "participant_unmuted";
  v: 1;
}

export interface PauseConsumer {
  consumer_id: string;
  id?: string;
  room_id: string;
  type: "pause_consumer";
  v?: 1;
}

export interface PrincipalBlocked {
  actor_id: string;
  id?: string;
  room_id?: string | null;
  session_id?: string | null;
  type: "principal_blocked";
  v: 1;
}

export interface PrincipalUnblocked {
  actor_id: string;
  id?: string;
  room_id?: string | null;
  session_id?: string | null;
  type: "principal_unblocked";
  v: 1;
}

export interface Produce {
  id?: string;
  kind: string;
  room_id: string;
  rtp_parameters: string;
  source?: string | null;
  transport_id: string;
  type: "produce";
  v?: 1;
}

export interface Produced {
  id?: string;
  producer_id: string;
  room_id: string;
  type: "produced";
  v: 1;
}

export interface RemoveParticipant {
  actor_id: string;
  id?: string;
  session_id: string;
  type: "remove_participant";
  v?: 1;
}

export interface ResumeConsumer {
  consumer_id: string;
  id?: string;
  room_id: string;
  type: "resume_consumer";
  v?: 1;
}

export interface SessionRoster {
  id?: string;
  participant_ids: string[];
  session_id: string;
  type: "session_roster";
  v: 1;
  version: number;
}

export interface SubscribeChat {
  after_sequence: number;
  id?: string;
  room_id: string;
  type: "subscribe_chat";
  v?: 1;
}

export interface TransportConnected {
  id?: string;
  room_id: string;
  transport_id: string;
  type: "transport_connected";
  v: 1;
}

export interface TransportCreated {
  dtls_parameters: string;
  ice_candidates: string;
  ice_parameters: string;
  id?: string;
  room_id: string;
  transport_id: string;
  type: "transport_created";
  v: 1;
}

export interface UnblockPrincipal {
  actor_id: string;
  id?: string;
  room_id?: string | null;
  session_id?: string | null;
  type: "unblock_principal";
  v?: 1;
}

export interface UnmuteParticipant {
  actor_id: string;
  id?: string;
  session_id: string;
  source?: string | null;
  type: "unmute_participant";
  v?: 1;
}

export interface UnsubscribeChat {
  id?: string;
  room_id: string;
  type: "unsubscribe_chat";
  v?: 1;
}

export interface Welcome {
  connection_id: string;
  id?: string;
  type: "welcome";
  v: 1;
}

export type ClientMessage =
  | Auth
  | ClientEcho
  | ClientBroadcast
  | SubscribeChat
  | UnsubscribeChat
  | ChatTyping
  | Join
  | Leave
  | MuteParticipant
  | UnmuteParticipant
  | RemoveParticipant
  | BlockPrincipal
  | UnblockPrincipal
  | CreateTransport
  | ConnectTransport
  | Produce
  | Consume
  | PauseConsumer
  | ResumeConsumer;
export type ServerMessage =
  | Welcome
  | Echo
  | Broadcast
  | ChatSubscribed
  | ChatActivity
  | ChatEvent
  | Error
  | Bye
  | Joined
  | Left
  | JoinWaiting
  | ParticipantJoined
  | ParticipantLeft
  | SessionRoster
  | ActiveSpeakers
  | JoinRejected
  | ParticipantMuted
  | ParticipantUnmuted
  | ParticipantRemoved
  | PrincipalBlocked
  | PrincipalUnblocked
  | TransportCreated
  | TransportConnected
  | Produced
  | Consumed
  | ConsumerPaused
  | ConsumerResumed;
