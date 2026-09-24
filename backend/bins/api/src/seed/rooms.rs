use anyhow::{Context, Result};
use mahfel_domain::{
    Room, RoomId, RoomMember, RoomRole, RoomTitle, RoomVisibility, Session, SessionId, SessionTitle,
};
use mahfel_postgres::{PostgresRoomStore, PostgresSessionStore};
use serde::Deserialize;

use crate::seed::fixture::parse_actor_id;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DevelopmentRoom {
    id: String,
    title: String,
    manager_id: String,
    visibility: RoomVisibility,
    #[serde(default)]
    members: Vec<String>,
    #[serde(default)]
    sessions: Vec<DevelopmentSession>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DevelopmentSession {
    id: String,
    title: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct StandaloneSession {
    id: String,
    room_id: String,
    title: String,
}

pub(super) struct ValidatedRoom {
    room: Room,
    members: Vec<RoomMember>,
    sessions: Vec<Session>,
}

pub(super) fn validate_rooms(rooms: Vec<DevelopmentRoom>) -> Result<Vec<ValidatedRoom>> {
    if rooms.is_empty() {
        anyhow::bail!("rooms seed must contain at least one room");
    }
    rooms.into_iter().map(validate_room).collect()
}

pub(super) fn validate_sessions(sessions: Vec<StandaloneSession>) -> Result<Vec<Session>> {
    if sessions.is_empty() {
        anyhow::bail!("sessions seed must contain at least one session");
    }
    sessions
        .into_iter()
        .map(|fixture| {
            let room_id = RoomId::from_uuid(
                uuid::Uuid::parse_str(&fixture.room_id).context("validate session room id")?,
            );
            validate_session(&fixture.id, fixture.title, room_id)
        })
        .collect()
}

fn validate_room(fixture: DevelopmentRoom) -> Result<ValidatedRoom> {
    let now = time::OffsetDateTime::now_utc();
    let manager_id = parse_actor_id(&fixture.manager_id).context("validate room manager id")?;
    let mut room = Room::create(
        RoomTitle::new(fixture.title).context("validate room title")?,
        manager_id,
        fixture.visibility,
        now,
    );
    room.id = RoomId::from_uuid(uuid::Uuid::parse_str(&fixture.id).context("validate room id")?);
    let mut members = vec![RoomMember::new(room.id, manager_id, RoomRole::Manager, now)];
    for member in fixture.members {
        let actor_id = parse_actor_id(&member).context("validate room member id")?;
        if members.iter().any(|existing| existing.actor_id == actor_id) {
            anyhow::bail!("duplicate room member {actor_id}");
        }
        members.push(RoomMember::new(room.id, actor_id, RoomRole::Member, now));
    }
    let sessions = fixture
        .sessions
        .into_iter()
        .map(|fixture| validate_session(&fixture.id, fixture.title, room.id))
        .collect::<Result<Vec<_>>>()?;
    Ok(ValidatedRoom {
        room,
        members,
        sessions,
    })
}

fn validate_session(id: &str, title: String, room_id: RoomId) -> Result<Session> {
    let now = time::OffsetDateTime::now_utc();
    let mut session = Session::schedule(room_id, None, now);
    session.id = SessionId::from_uuid(uuid::Uuid::parse_str(id).context("validate session id")?);
    session.set_details(
        Some(SessionTitle::new(title).context("validate session title")?),
        None,
        None,
        now,
    );
    Ok(session)
}

pub(super) async fn apply_rooms(
    store: &PostgresRoomStore,
    fixtures: &[ValidatedRoom],
    name: &str,
) -> Result<()> {
    for fixture in fixtures {
        store
            .seed_development_room(&fixture.room, &fixture.members, &fixture.sessions)
            .await
            .with_context(|| format!("apply room {} from seed {name}", fixture.room.id))?;
    }
    Ok(())
}

pub(super) async fn apply_sessions(
    store: &PostgresSessionStore,
    fixtures: &[Session],
    name: &str,
) -> Result<()> {
    for fixture in fixtures {
        store
            .seed_development_session(fixture)
            .await
            .with_context(|| format!("apply session {} from seed {name}", fixture.id))?;
    }
    Ok(())
}
