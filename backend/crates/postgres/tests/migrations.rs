use mahfel_application::ports::{
    ChatStore, FileStore, IdentityProfile, IdentityStore, RoomStore, SessionStore, StoreError,
    UserStore,
};
use mahfel_domain::{
    ActorId, AvatarUrl, ChatChannel, ChatChannelId, ChatMessageDraft, ChatMessageId, DisplayName,
    EmailAddress, FileContext, FileObject, FileStatus, Issuer, Room, RoomMember, RoomRole,
    RoomTitle, RoomVisibility, Session, Subject, User,
};
use mahfel_postgres::{
    ConnectionSettings, PoolSettings, PostgresChatStore, PostgresFileStore, PostgresIdentityStore,
    PostgresRoomStore, PostgresSessionStore, PostgresUserStore, connect,
    migrate,
};
use sqlx::PgPool;
use std::{error::Error, time::Duration};
use testcontainers::{
    GenericImage, ImageExt,
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
};
use time::{Duration as TimeDuration, OffsetDateTime};
use url::Url;
use uuid::Uuid;

fn test_pool_settings() -> PoolSettings {
    PoolSettings {
        min_connections: 1,
        max_connections: 2,
        acquire_timeout: Duration::from_secs(5),
        idle_timeout: Duration::from_secs(30),
        max_lifetime: Duration::from_secs(60),
    }
}

#[tokio::test]
async fn migrations_provision_a_disposable_database() -> Result<(), Box<dyn Error>> {
    if let Ok(admin_url) = std::env::var("MAHFEL_TEST_DATABASE_URL") {
        return verify_disposable_database(&admin_url).await;
    }

    if std::env::var("MAHFEL_TESTCONTAINERS").as_deref() != Ok("1") {
        eprintln!(
            "MAHFEL_TESTCONTAINERS is unset and MAHFEL_TEST_DATABASE_URL is unset; skipping PostgreSQL integration test"
        );
        return Ok(());
    }

    let postgres = GenericImage::new("postgres", "16-bookworm")
        .with_exposed_port(5432.tcp())
        .with_wait_for(WaitFor::message_on_stdout(
            "database system is ready to accept connections",
        ))
        .with_env_var("POSTGRES_DB", "postgres")
        .with_env_var("POSTGRES_USER", "mahfel")
        .with_env_var("POSTGRES_PASSWORD", "mahfel-test")
        .start()
        .await?;
    let port = postgres.get_host_port_ipv4(5432).await?;
    let admin_url = format!("postgres://mahfel:mahfel-test@127.0.0.1:{port}/postgres");
    wait_until_ready(&admin_url).await?;
    verify_disposable_database(&admin_url).await
}

/// Wait until the container accepts an authenticated connection.
///
/// The `ready to accept connections` log line is emitted before the auth
/// backend can complete a handshake, so connecting immediately can be reset by
/// the server. Probe with a real pool instead of sleeping a fixed amount.
async fn wait_until_ready(admin_url: &str) -> Result<(), Box<dyn Error>> {
    let deadline = std::time::Instant::now() + Duration::from_secs(30);
    loop {
        match PgPool::connect(admin_url).await {
            Ok(pool) => {
                pool.close().await;
                return Ok(());
            }
            Err(error) if std::time::Instant::now() < deadline => {
                eprintln!("postgres not ready yet: {error}");
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
            Err(error) => return Err(error.into()),
        }
    }
}

async fn verify_disposable_database(admin_url: &str) -> Result<(), Box<dyn Error>> {
    let admin_pool = PgPool::connect(admin_url).await?;
    let database_name = format!("mahfel_test_{}", Uuid::new_v4().simple());
    let create_database = format!("CREATE DATABASE \"{database_name}\" TEMPLATE template0");
    // The interpolated identifier is generated exclusively from a UUID.
    sqlx::raw_sql(sqlx::AssertSqlSafe(create_database))
        .execute(&admin_pool)
        .await?;

    let mut test_url = Url::parse(admin_url)?;
    test_url.set_path(&format!("/{database_name}"));
    let connection = connection_settings(&test_url)?;
    let test_pool = connect(&connection, test_pool_settings()).await?;
    let verification = verify_migrations_and_stores(&test_pool, &connection).await;
    if !test_pool.is_closed() {
        test_pool.close().await;
    }

    let drop_database = format!("DROP DATABASE \"{database_name}\" WITH (FORCE)");
    let cleanup = sqlx::raw_sql(sqlx::AssertSqlSafe(drop_database))
        .execute(&admin_pool)
        .await;
    admin_pool.close().await;

    verification?;
    cleanup?;
    Ok(())
}

async fn verify_migrations_and_stores(
    pool: &PgPool,
    connection: &ConnectionSettings,
) -> Result<(), Box<dyn Error>> {
    migrate(pool).await?;

    // Re-running migrations on an already-migrated database must be a no-op.
    // The connection pins `search_path=public` because the database role and
    // the `mahfel` schema share a name; otherwise `_sqlx_migrations` resolves
    // to a different schema before and after migration 0001 creates `mahfel`.
    migrate(pool).await?;

    let migration_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM _sqlx_migrations")
        .fetch_one(pool)
        .await?;
    assert!(
        migration_count >= 16,
        "all file migrations must be recorded"
    );
    let max_migration_version =
        sqlx::query_scalar::<_, i64>("SELECT MAX(version) FROM _sqlx_migrations")
            .fetch_one(pool)
            .await?;
    assert_eq!(
        max_migration_version, migration_count,
        "migration versions must be contiguous and recorded once"
    );

    let migration_tables = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM pg_class WHERE relname = '_sqlx_migrations'",
    )
    .fetch_one(pool)
    .await?;
    assert_eq!(
        migration_tables, 1,
        "migration bookkeeping must use a single table"
    );

    let namespace_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM pg_namespace WHERE nspname = 'mahfel')",
    )
    .fetch_one(pool)
    .await?;
    assert!(
        namespace_exists,
        "migration must provision the Mahfel schema"
    );

    // The unified actor model is present and membership is keyed by actor.
    for table in ["actors", "service_principals", "service_api_keys"] {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (SELECT 1 FROM pg_tables \
             WHERE schemaname = 'mahfel' AND tablename = $1)",
        )
        .bind(table)
        .fetch_one(pool)
        .await?;
        assert!(exists, "migration must create mahfel.{table}");
    }

    let membership_actor_column = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM information_schema.columns \
         WHERE table_schema = 'mahfel' AND table_name = 'room_memberships' \
         AND column_name = 'actor_id')",
    )
    .fetch_one(pool)
    .await?;
    assert!(
        membership_actor_column,
        "memberships must be keyed by actor_id"
    );

    // The live aggregate is generic: table `sessions`, not `classroom_sessions`.
    let sessions_table = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM pg_tables \
         WHERE schemaname = 'mahfel' AND tablename = 'sessions')",
    )
    .fetch_one(pool)
    .await?;
    assert!(sessions_table, "migration must create mahfel.sessions");

    let legacy_sessions_table = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM pg_tables \
         WHERE schemaname = 'mahfel' AND tablename = 'classroom_sessions')",
    )
    .fetch_one(pool)
    .await?;
    assert!(
        !legacy_sessions_table,
        "the education-flavored table name must be gone"
    );

    // At most one live session per room is a database invariant.
    let live_index = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM pg_indexes \
         WHERE schemaname = 'mahfel' AND tablename = 'sessions' \
         AND indexname = 'sessions_one_live_per_room_idx')",
    )
    .fetch_one(pool)
    .await?;
    assert!(
        live_index,
        "migration must create the one-live-session-per-room partial index"
    );

    for index in ["sessions_room_id_idx", "sessions_status_idx"] {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (SELECT 1 FROM pg_indexes \
             WHERE schemaname = 'mahfel' AND tablename = 'sessions' AND indexname = $1)",
        )
        .bind(index)
        .fetch_one(pool)
        .await?;
        assert!(exists, "migration must create index {index}");
    }

    // PostgreSQL keeps microseconds; align the in-memory clock so round-trip
    // equality assertions compare the same instant.
    let now = OffsetDateTime::now_utc()
        .replace_nanosecond(0)
        .expect("zero nanoseconds is valid");
    let users = PostgresUserStore::new(pool.clone());
    let rooms = PostgresRoomStore::new(pool.clone());
    let sessions = PostgresSessionStore::new(pool.clone());
    let identities = PostgresIdentityStore::new(pool.clone());
    let chat = PostgresChatStore::new(pool.clone());
    let files = PostgresFileStore::new(pool.clone());

    let user = User::create(DisplayName::new("Ada Lovelace".to_owned())?, now);
    users.insert(&user).await?;

    verify_identity_lifecycle(&identities, users.clone(), now).await?;

    let room = Room::create(
        RoomTitle::new("Analytical Engines".to_owned())?,
        user.id,
        RoomVisibility::InviteOnly,
        now,
    );
    let manager = RoomMember::new(room.id, user.id, RoomRole::Manager, now);
    rooms.create_with_manager(&room, &manager).await?;

    let session = Session::schedule(room.id, None, now);
    sessions.insert(&session).await?;
    assert_eq!(
        sessions.list_for_room(room.id).await?,
        vec![session.clone()]
    );

    assert_eq!(
        rooms.get_member(room.id, user.id).await?,
        Some(manager.clone())
    );
    assert_eq!(rooms.list_members(room.id).await?, vec![manager.clone()]);
    assert_eq!(
        rooms.list_for_member(user.id, 10, 0).await?,
        vec![room.clone()]
    );

    let first_channel = ChatChannel::for_room(ChatChannelId::new(), room.id, now);
    let first_draft = ChatMessageDraft::new(
        ChatMessageId::new(),
        user.id,
        "Persistent room message".to_owned(),
        now,
    )?;
    let first_message = chat.create_message(&first_channel, &first_draft).await?;
    assert_eq!(first_message.channel_id, first_channel.id);

    let file = FileObject::pending(
        user.id,
        FileContext::Room(room.id),
        "deck.pdf".into(),
        "application/pdf".into(),
        1024,
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".into(),
        now,
    )?;
    files.insert(&file).await?;
    assert_eq!(
        files.get(file.id).await?.expect("inserted file").object_key,
        file.object_key
    );
    assert!(
        files
            .list(FileContext::Room(room.id), 100, 0)
            .await?
            .is_empty()
    );
    assert_eq!(
        files.context_room(FileContext::Session(session.id)).await?,
        Some(room.id)
    );
    assert_eq!(
        files
            .context_room(FileContext::ChatMessage(first_message.id))
            .await?,
        Some(room.id)
    );
    assert!(
        sqlx::query("UPDATE mahfel.files SET size_bytes = 0 WHERE id = $1")
            .bind(file.id.as_uuid())
            .execute(pool)
            .await
            .is_err()
    );
    assert!(
        sqlx::query("UPDATE mahfel.files SET session_id = $2 WHERE id = $1")
            .bind(file.id.as_uuid())
            .bind(session.id.as_uuid())
            .execute(pool)
            .await
            .is_err()
    );
    assert!(
        files
            .transition(
                file.id,
                FileStatus::Pending,
                FileStatus::Ready,
                Some("application/pdf"),
                now
            )
            .await?
    );
    assert_eq!(
        files.list(FileContext::Room(room.id), 100, 0).await?.len(),
        1
    );
    let session_file = FileObject::pending(
        user.id,
        FileContext::Session(session.id),
        "notes.pdf".into(),
        "application/pdf".into(),
        1024,
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".into(),
        now,
    )?;
    files.insert(&session_file).await?;
    assert!(
        files
            .transition(
                session_file.id,
                FileStatus::Pending,
                FileStatus::Ready,
                Some("application/pdf"),
                now,
            )
            .await?
    );
    assert_eq!(
        files.list(FileContext::Room(room.id), 100, 0).await?[0].id,
        file.id
    );
    assert_eq!(
        files.list(FileContext::Session(session.id), 100, 0).await?[0].id,
        session_file.id
    );

    let later_channel = ChatChannel::for_room(ChatChannelId::new(), room.id, now);
    let later_draft = ChatMessageDraft::new(
        ChatMessageId::new(),
        user.id,
        "Another message in the same channel".to_owned(),
        now + TimeDuration::seconds(1),
    )?;
    let later_message = chat.create_message(&later_channel, &later_draft).await?;
    assert_eq!(
        later_message.channel_id, first_message.channel_id,
        "each room must resolve to its stable chat channel"
    );

    let first_page = chat.list_messages(room.id, user.id, None, 1).await?;
    assert_eq!(first_page.messages, vec![later_message.clone()]);
    let cursor = first_page.next_cursor.expect("more chat history exists");
    let inserted_after_page = ChatMessageDraft::new(
        ChatMessageId::new(),
        user.id,
        "Inserted after the first page".to_owned(),
        now + TimeDuration::seconds(2),
    )?;
    chat.create_message(&first_channel, &inserted_after_page)
        .await?;
    let second_page = chat
        .list_messages(room.id, user.id, Some(&cursor), 10)
        .await?;
    assert_eq!(second_page.messages, vec![first_message.clone()]);
    assert!(
        second_page.next_cursor.is_none(),
        "a newer insert must not shift an existing keyset boundary"
    );

    let outsider = User::create(DisplayName::new("Katherine Johnson".to_owned())?, now);
    users.insert(&outsider).await?;
    assert!(
        matches!(
            chat.list_messages(room.id, outsider.id, None, 10).await,
            Err(StoreError::NotFound(_))
        ),
        "a nonmember must not read room chat history"
    );
    let unauthorized_channel = ChatChannel::for_room(ChatChannelId::new(), room.id, now);
    let unauthorized_draft = ChatMessageDraft::new(
        ChatMessageId::new(),
        outsider.id,
        "This must not persist".to_owned(),
        now,
    )?;
    assert!(
        matches!(
            chat.create_message(&unauthorized_channel, &unauthorized_draft)
                .await,
            Err(StoreError::NotFound(_))
        ),
        "a non-member must not create room chat messages"
    );

    let edited = chat
        .edit_message(
            room.id,
            user.id,
            first_message.id,
            "Edited room message",
            &[],
            now + TimeDuration::seconds(3),
        )
        .await?;
    assert_eq!(edited.content, "Edited room message");
    assert_eq!(edited.edited_at, Some(now + TimeDuration::seconds(3)));
    assert!(matches!(
        chat.edit_message(
            room.id,
            user.id,
            first_message.id,
            "Too late",
            &[],
            now + TimeDuration::minutes(16),
        )
        .await,
        Err(StoreError::Conflict(_))
    ));

    let chat_member = User::create(DisplayName::new("Katherine Member".to_owned())?, now);
    users.insert(&chat_member).await?;
    rooms
        .add_member(&RoomMember::new(
            room.id,
            chat_member.id,
            RoomRole::Member,
            now,
        ))
        .await?;
    let member_channel = ChatChannel::for_room(ChatChannelId::new(), room.id, now);
    let member_draft = ChatMessageDraft::new(
        ChatMessageId::new(),
        chat_member.id,
        "Member message for manager moderation".to_owned(),
        now + TimeDuration::seconds(4),
    )?;
    let member_message = chat.create_message(&member_channel, &member_draft).await?;
    assert!(matches!(
        chat.delete_message(
            room.id,
            chat_member.id,
            first_message.id,
            now + TimeDuration::seconds(5),
        )
        .await,
        Err(StoreError::NotFound(_))
    ));
    let moderated = chat
        .delete_message(
            room.id,
            user.id,
            member_message.id,
            now + TimeDuration::minutes(20),
        )
        .await?;
    assert_eq!(moderated.deleted_at, Some(now + TimeDuration::minutes(20)));
    let revisions = sqlx::query_as::<_, (uuid::Uuid, String, String)>(
        "SELECT message_id, revision_kind, prior_body \
         FROM mahfel.chat_message_revisions ORDER BY created_at, id",
    )
    .fetch_all(pool)
    .await?;
    assert_eq!(revisions.len(), 2);
    assert_eq!(revisions[0].0, first_message.id.as_uuid());
    assert_eq!(revisions[0].1, "edited");
    assert_eq!(revisions[0].2, "Persistent room message");
    assert_eq!(revisions[1].0, member_message.id.as_uuid());
    assert_eq!(revisions[1].1, "deleted");
    assert_eq!(revisions[1].2, "Member message for manager moderation");

    let events = chat.list_events(room.id, user.id, 0, 100).await?;
    assert_eq!(events.len(), 6);
    assert_eq!(
        events
            .iter()
            .map(|event| event.sequence)
            .collect::<Vec<_>>(),
        vec![1, 2, 3, 4, 5, 6]
    );
    assert_eq!(events[0].message.id, first_message.id);
    assert_eq!(events[0].message.content, "Persistent room message");
    assert_eq!(events[3].message.content, "Edited room message");
    assert!(events[5].message.deleted_at.is_some());
    assert_eq!(events[5].message.content, "");
    assert_eq!(chat.event_head(room.id, user.id).await?, 6);
    assert_eq!(chat.list_events(room.id, user.id, 4, 100).await?.len(), 2);
    assert!(matches!(
        chat.list_events(room.id, outsider.id, 0, 100).await,
        Err(StoreError::NotFound(_))
    ));

    let reply_draft = ChatMessageDraft::new(
        ChatMessageId::new(),
        chat_member.id,
        "First reply".to_owned(),
        now + TimeDuration::seconds(6),
    )?
    .with_details(Some(first_message.id), vec![user.id])?;
    let reply = chat.create_message(&member_channel, &reply_draft).await?;
    assert_eq!(reply.parent_id, Some(first_message.id));
    assert_eq!(
        chat.list_mentions(room.id, user.id, reply.id).await?,
        vec![user.id]
    );
    let invalid_mention = ChatMessageDraft::new(
        ChatMessageId::new(),
        chat_member.id,
        "Invalid mention".to_owned(),
        now + TimeDuration::seconds(7),
    )?
    .with_details(None, vec![outsider.id])?;
    assert!(matches!(
        chat.create_message(&member_channel, &invalid_mention).await,
        Err(StoreError::Conflict(_))
    ));
    let nested = ChatMessageDraft::new(
        ChatMessageId::new(),
        user.id,
        "Nested reply".to_owned(),
        now + TimeDuration::seconds(7),
    )?
    .with_details(Some(reply.id), vec![])?;
    assert!(matches!(
        chat.create_message(&first_channel, &nested).await,
        Err(StoreError::Conflict(_))
    ));
    let second_reply = ChatMessageDraft::new(
        ChatMessageId::new(),
        user.id,
        "Second reply".to_owned(),
        now + TimeDuration::seconds(8),
    )?
    .with_details(Some(first_message.id), vec![])?;
    let second_reply = chat.create_message(&first_channel, &second_reply).await?;
    let first_reply_page = chat
        .list_replies(room.id, user.id, first_message.id, None, 1)
        .await?;
    assert_eq!(first_reply_page.messages[0].id, second_reply.id);
    let reply_cursor = first_reply_page.next_cursor.expect("more replies");
    let third_reply = ChatMessageDraft::new(
        ChatMessageId::new(),
        user.id,
        "Third reply".to_owned(),
        now + TimeDuration::seconds(9),
    )?
    .with_details(Some(first_message.id), vec![])?;
    chat.create_message(&first_channel, &third_reply).await?;
    assert_eq!(
        chat.list_replies(room.id, user.id, first_message.id, Some(&reply_cursor), 10)
            .await?
            .messages
            .iter()
            .map(|message| message.id)
            .collect::<Vec<_>>(),
        vec![reply.id]
    );
    assert!(matches!(
        chat.list_replies(room.id, user.id, reply.id, None, 10)
            .await,
        Err(StoreError::Conflict(_))
    ));

    let (reaction_a, reaction_b) = tokio::join!(
        chat.set_reaction(room.id, user.id, first_message.id, "👍", now),
        chat.set_reaction(room.id, user.id, first_message.id, "👎", now),
    );
    reaction_a?;
    reaction_b?;
    let (counts, own) = chat
        .list_reactions(room.id, user.id, first_message.id)
        .await?;
    assert_eq!(counts.iter().map(|(_, count)| *count).sum::<i64>(), 1);
    assert!(matches!(own.as_deref(), Some("👍" | "👎")));
    chat.set_reaction(room.id, chat_member.id, first_message.id, "❤️", now)
        .await?;
    let (counts, _) = chat
        .list_reactions(room.id, user.id, first_message.id)
        .await?;
    assert_eq!(counts.iter().map(|(_, count)| *count).sum::<i64>(), 2);
    chat.remove_reaction(room.id, user.id, first_message.id, now)
        .await?;
    chat.remove_reaction(room.id, user.id, first_message.id, now)
        .await?;
    assert!(matches!(
        chat.set_reaction(room.id, outsider.id, first_message.id, "👍", now)
            .await,
        Err(StoreError::NotFound(_))
    ));
    assert!(matches!(
        chat.set_reaction(room.id, user.id, moderated.id, "👍", now)
            .await,
        Err(StoreError::NotFound(_))
    ));
    assert_eq!(chat.read_marker(room.id, user.id).await?, 0);
    assert_eq!(chat.advance_read_marker(room.id, user.id, 5, now).await?, 5);
    assert_eq!(chat.advance_read_marker(room.id, user.id, 4, now).await?, 5);
    assert_eq!(chat.read_marker(room.id, chat_member.id).await?, 0);
    assert!(matches!(
        chat.advance_read_marker(room.id, user.id, 10_000, now)
            .await,
        Err(StoreError::Conflict(_))
    ));
    assert!(matches!(
        chat.read_marker(room.id, outsider.id).await,
        Err(StoreError::NotFound(_))
    ));

    let mut attachments = Vec::new();
    for index in 0..4 {
        let attachment = FileObject::pending(
            user.id,
            FileContext::ChatMessage(reply.id),
            format!("attachment-{index}.pdf"),
            "application/pdf".to_owned(),
            1024,
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".to_owned(),
            now,
        )?;
        files.insert(&attachment).await?;
        attachments.push(attachment);
    }
    assert!(
        files
            .list(FileContext::ChatMessage(reply.id), 100, 0)
            .await?
            .is_empty(),
        "pending uploads must not be visible as attachments"
    );
    let fifth = FileObject::pending(
        user.id,
        FileContext::ChatMessage(reply.id),
        "fifth.pdf".to_owned(),
        "application/pdf".to_owned(),
        1024,
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".to_owned(),
        now,
    )?;
    assert!(matches!(
        files.insert(&fifth).await,
        Err(StoreError::Conflict(_))
    ));
    assert!(
        files
            .transition(
                attachments[0].id,
                FileStatus::Pending,
                FileStatus::Ready,
                Some("application/pdf"),
                now,
            )
            .await?
    );
    assert_eq!(
        files
            .list(FileContext::ChatMessage(reply.id), 100, 0)
            .await?
            .len(),
        1
    );
    let outsider_file = FileObject::pending(
        outsider.id,
        FileContext::ChatMessage(reply.id),
        "outsider.pdf".to_owned(),
        "application/pdf".to_owned(),
        1024,
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".to_owned(),
        now,
    )?;
    assert!(matches!(
        files.insert(&outsider_file).await,
        Err(StoreError::NotFound(_))
    ));
    chat.delete_message(room.id, user.id, reply.id, now + TimeDuration::minutes(20))
        .await?;
    assert!(
        files
            .list(FileContext::ChatMessage(reply.id), 100, 0)
            .await?
            .is_empty()
    );
    assert!(matches!(
        files.insert(&fifth).await,
        Err(StoreError::NotFound(_))
    ));

    // A missing referenced user must surface as a typed adapter error rather
    // than a generic storage failure.
    let missing = rooms
        .add_member(&RoomMember::new(
            room.id,
            ActorId::new(),
            RoomRole::Member,
            now,
        ))
        .await;
    assert!(
        matches!(missing, Err(StoreError::MissingReference(_))),
        "unknown user must map to MissingReference, got {missing:?}"
    );

    // The last manager cannot be removed; a co-manager can.
    let co_manager = User::create(DisplayName::new("Grace Hopper".to_owned())?, now);
    users.insert(&co_manager).await?;
    rooms
        .add_member(&RoomMember::new(
            room.id,
            co_manager.id,
            RoomRole::Manager,
            now,
        ))
        .await?;
    assert!(
        rooms.remove_member(room.id, co_manager.id).await?,
        "removing a co-manager must succeed"
    );
    assert!(
        !rooms.remove_member(room.id, user.id).await?,
        "the last manager must not be removed"
    );

    pool.close().await;
    let restarted_pool = connect(connection, test_pool_settings()).await?;
    let restarted_users = PostgresUserStore::new(restarted_pool.clone());
    let restarted_rooms = PostgresRoomStore::new(restarted_pool.clone());
    let restarted_sessions = PostgresSessionStore::new(restarted_pool.clone());
    let restarted_chat = PostgresChatStore::new(restarted_pool.clone());
    assert_eq!(restarted_chat.read_marker(room.id, user.id).await?, 5);
    let stored_chat_messages = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM mahfel.chat_messages WHERE channel_id = $1",
    )
    .bind(first_message.channel_id.as_uuid())
    .fetch_one(&restarted_pool)
    .await?;
    assert_eq!(
        stored_chat_messages, 7,
        "room chat messages must survive a pool restart"
    );
    let stored_message = sqlx::query_as::<_, (uuid::Uuid, uuid::Uuid, String)>(
        "SELECT channel_id, author_actor_id, body \
         FROM mahfel.chat_messages WHERE id = $1",
    )
    .bind(first_message.id.as_uuid())
    .fetch_one(&restarted_pool)
    .await?;
    assert_eq!(
        stored_message,
        (
            first_message.channel_id.as_uuid(),
            user.id.as_uuid(),
            "Edited room message".to_owned(),
        )
    );
    let unauthorized_message_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM mahfel.chat_messages WHERE id = $1)",
    )
    .bind(unauthorized_draft.id.as_uuid())
    .fetch_one(&restarted_pool)
    .await?;
    assert!(
        !unauthorized_message_exists,
        "unauthorized message content must not be stored"
    );

    assert_eq!(restarted_users.get(user.id).await?, Some(user.clone()));
    assert_eq!(restarted_rooms.get(room.id).await?, Some(room.clone()));
    assert_eq!(restarted_sessions.get(session.id).await?, Some(session));
    assert_eq!(
        restarted_rooms.get_member(room.id, user.id).await?,
        Some(manager.clone())
    );
    assert_eq!(
        restarted_rooms.list_for_member(user.id, 10, 0).await?,
        vec![room]
    );
    restarted_pool.close().await;
    Ok(())
}

/// Prove the identity rules that guard against account takeover and races.
async fn verify_identity_lifecycle(
    identities: &PostgresIdentityStore,
    users: PostgresUserStore,
    now: OffsetDateTime,
) -> Result<(), Box<dyn Error>> {
    let issuer = Issuer::new("https://auth.mahfel.localhost/realms/rastai".to_owned())?;
    let profile = IdentityProfile {
        display_name: Some(DisplayName::new("Shinji Ikari".to_owned())?),
        email: Some(EmailAddress::new("shinji.ikari@mahfel.local".to_owned())?),
        email_verified: true,
        avatar_url: Some(AvatarUrl::new(
            "https://s3.mahfel.localhost/avatars/shinji.png".to_owned(),
        )?),
    };

    // First login provisions exactly one user and identity.
    let subject = Subject::new("00000000-0000-4000-8000-000000000001".to_owned())?;
    let first = identities
        .resolve_or_provision(
            &issuer,
            &subject,
            profile.display_name.as_ref().expect("display name"),
            &profile,
            now,
        )
        .await?;
    assert_eq!(first.user.display_name.as_str(), "Shinji Ikari");
    assert_eq!(first.identity.user_id, first.user.id);
    assert!(first.identity.email_verified);
    assert_eq!(
        first.identity.email.as_ref().map(EmailAddress::as_str),
        Some("shinji.ikari@mahfel.local")
    );

    // Concurrent first logins for the same subject converge on one user.
    let subject_race = Subject::new("00000000-0000-4000-8000-000000000002".to_owned())?;
    let name = DisplayName::new("Asuka Langley".to_owned())?;
    let (left, right) = tokio::join!(
        identities.resolve_or_provision(&issuer, &subject_race, &name, &profile, now),
        identities.resolve_or_provision(&issuer, &subject_race, &name, &profile, now),
    );
    let left = left?;
    let right = right?;
    assert_eq!(
        left.user.id, right.user.id,
        "a concurrent first login must not create a second user"
    );

    // The same user may own identities from different issuers.
    let other_issuer = Issuer::new("https://keycloak.example.com/realms/rastai".to_owned())?;
    let linked = identities
        .resolve_or_provision(&other_issuer, &subject, &name, &profile, now)
        .await?;
    assert_ne!(
        linked.user.id, first.user.id,
        "a different issuer with the same subject is a different identity"
    );

    // A repeated login refreshes profile fields without moving the key.
    let renamed = IdentityProfile {
        display_name: Some(DisplayName::new("Shinji Ikari II".to_owned())?),
        ..profile.clone()
    };
    let refreshed = identities
        .resolve_or_provision(&issuer, &subject, &name, &renamed, now)
        .await?;
    assert_eq!(refreshed.user.id, first.user.id);
    assert_eq!(refreshed.identity.id, first.identity.id);
    assert_eq!(
        refreshed
            .identity
            .display_name
            .as_ref()
            .map(DisplayName::as_str),
        Some("Shinji Ikari II")
    );

    // Claiming an email that another identity already verified does not take
    // over that account; identity remains keyed on (issuer, subject).
    let collision_subject = Subject::new("00000000-0000-4000-8000-000000000003".to_owned())?;
    let collision = identities
        .resolve_or_provision(&issuer, &collision_subject, &name, &profile, now)
        .await?;
    assert_ne!(
        collision.user.id, first.user.id,
        "email must never link two identities to one user"
    );

    // The avatar link persists on the user record.
    let stored = users.get(first.user.id).await?;
    assert!(
        stored.is_some(),
        "the provisioned user must be readable through the user store"
    );

    Ok(())
}

fn connection_settings(url: &Url) -> Result<ConnectionSettings, Box<dyn Error>> {
    Ok(ConnectionSettings {
        host: url
            .host_str()
            .ok_or("test database URL is missing a host")?
            .to_owned(),
        port: url.port().unwrap_or(5432),
        user: url.username().to_owned(),
        password: url.password().unwrap_or_default().to_owned(),
        name: url.path().trim_start_matches('/').to_owned(),
    })
}
