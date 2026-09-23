use crate::DatabaseError;
use crate::stores::{store_error, user::UserRow};
use mahfel_application::ports::{
    BoxFuture, IdentityProfile, IdentityStore, ProvisionedIdentity, StoreError,
};
use mahfel_domain::{
    ActorId, AvatarUrl, DisplayName, EmailAddress, Identity, IdentityId, Issuer, Subject, User,
    Username,
};
use sqlx::{FromRow, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone)]
pub struct PostgresIdentityStore {
    pool: PgPool,
}

impl PostgresIdentityStore {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Mirror a development fixture to its stable local user ID. This keeps
    /// local identity records attached to the same user if an in-memory
    /// development Keycloak is recreated and issues a new subject.
    pub async fn mirror_development_identity(
        &self,
        user_id: ActorId,
        issuer: &Issuer,
        subject: &Subject,
        display_name: &DisplayName,
        profile: &IdentityProfile,
        now: OffsetDateTime,
    ) -> Result<(), StoreError> {
        let email = profile.email.as_ref().ok_or_else(|| {
            StoreError::Unexpected("required identity email is missing".to_owned())
        })?;
        let user = User {
            id: user_id,
            display_name: display_name.clone(),
            email: email.clone(),
            username: profile.username.clone(),
            status: mahfel_domain::UserStatus::Active,
            avatar_url: profile.avatar_url.clone(),
            created_at: now,
            updated_at: now,
        };
        let identity = Identity::create(
            user_id,
            issuer.clone(),
            subject.clone(),
            profile.display_name.clone(),
            profile.email.clone(),
            profile.username.clone(),
            profile.email_verified,
            profile.avatar_url.clone(),
            now,
        );
        let mut transaction = self.pool.begin().await.map_err(store_error)?;

        sqlx::query(
            "INSERT INTO mahfel.actors (id, kind, created_at, updated_at) \
             VALUES ($1, 'user', $2, $2) ON CONFLICT (id) DO NOTHING",
        )
        .bind(user_id.as_uuid())
        .bind(now)
        .execute(&mut *transaction)
        .await
        .map_err(store_error)?;
        sqlx::query(
            "INSERT INTO mahfel.users (id, display_name, email, username, status, created_at, updated_at, avatar_url) \
             VALUES ($1, $2, $3, $4, 'active', $5, $5, $6) ON CONFLICT (id) DO NOTHING",
        )
        .bind(user.id.as_uuid())
        .bind(user.display_name.as_str())
        .bind(user.email.as_str())
        .bind(user.username.as_str())
        .bind(now)
        .bind(user.avatar_url.as_ref().map(AvatarUrl::as_str))
        .execute(&mut *transaction)
        .await
        .map_err(store_error)?;
        sqlx::query(INSERT_IDENTITY)
            .bind(identity.id.as_uuid())
            .bind(identity.user_id.as_uuid())
            .bind(identity.issuer.as_str())
            .bind(identity.subject.as_str())
            .bind(identity.email.as_ref().map(EmailAddress::as_str))
            .bind(identity.username.as_str())
            .bind(identity.email_verified)
            .bind(identity.display_name.as_ref().map(DisplayName::as_str))
            .bind(identity.avatar_url.as_ref().map(AvatarUrl::as_str))
            .bind(identity.created_at)
            .bind(identity.updated_at)
            .execute(&mut *transaction)
            .await
            .map_err(store_error)?;

        let stored_user_id: Uuid = sqlx::query_scalar(
            "SELECT user_id FROM mahfel.identities WHERE issuer = $1 AND subject = $2",
        )
        .bind(issuer.as_str())
        .bind(subject.as_str())
        .fetch_one(&mut *transaction)
        .await
        .map_err(store_error)?;
        if stored_user_id != user_id.as_uuid() {
            return Err(StoreError::Conflict(
                "development identity is already linked to another user".to_owned(),
            ));
        }

        transaction.commit().await.map_err(store_error)
    }
}

#[derive(FromRow)]
struct IdentityRow {
    id: Uuid,
    user_id: Uuid,
    issuer: String,
    subject: String,
    email: Option<String>,
    username: String,
    email_verified: bool,
    display_name: Option<String>,
    avatar_url: Option<String>,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

impl TryFrom<IdentityRow> for Identity {
    type Error = DatabaseError;

    fn try_from(row: IdentityRow) -> Result<Self, Self::Error> {
        let invalid = |field| DatabaseError::InvalidStoredValue(field);
        let email = row
            .email
            .map(|value| EmailAddress::new(value).map_err(|_| invalid("invalid identity email")))
            .transpose()?;
        let display_name = row
            .display_name
            .map(|value| {
                DisplayName::new(value).map_err(|_| invalid("invalid identity display name"))
            })
            .transpose()?;
        let avatar_url = row
            .avatar_url
            .map(|value| AvatarUrl::new(value).map_err(|_| invalid("invalid identity avatar")))
            .transpose()?;
        Ok(Self {
            id: IdentityId::from_uuid(row.id),
            user_id: ActorId::from_uuid(row.user_id),
            issuer: Issuer::new(row.issuer).map_err(|_| invalid("invalid identity issuer"))?,
            subject: Subject::new(row.subject).map_err(|_| invalid("invalid identity subject"))?,
            email,
            username: Username::new(row.username)
                .map_err(|_| invalid("invalid identity username"))?,
            email_verified: row.email_verified,
            display_name,
            avatar_url,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

/// Literal statements with bind parameters; the column list stays in one place.
const SELECT_IDENTITY: &str = "SELECT id, user_id, issuer, subject, email, username, email_verified, \
     display_name, avatar_url, created_at, updated_at \
     FROM mahfel.identities WHERE issuer = $1 AND subject = $2";

const INSERT_IDENTITY: &str = "INSERT INTO mahfel.identities \
     (id, user_id, issuer, subject, email, username, email_verified, display_name, avatar_url, created_at, updated_at) \
     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) \
     ON CONFLICT (issuer, subject) DO NOTHING";

impl IdentityStore for PostgresIdentityStore {
    fn resolve_or_provision<'a>(
        &'a self,
        issuer: &'a Issuer,
        subject: &'a Subject,
        display_name: &'a DisplayName,
        profile: &'a IdentityProfile,
        now: OffsetDateTime,
    ) -> BoxFuture<'a, Result<ProvisionedIdentity, StoreError>> {
        Box::pin(async move {
            if let Some(existing) = self.find_by_identity(issuer, subject).await? {
                return self.refresh(existing, profile, now).await;
            }

            // Provision in a transaction: if a concurrent first login inserts the
            // same (issuer, subject) first, the unique constraint rolls this
            // attempt back and the re-read adopts the winner.
            let mut transaction = self.pool.begin().await.map_err(store_error)?;
            let email = profile.email.clone().ok_or_else(|| {
                StoreError::Unexpected("required identity email is missing".to_owned())
            })?;
            let user = User::create(
                display_name.clone(),
                email.clone(),
                profile.username.clone(),
                now,
            );

            sqlx::query(
                "INSERT INTO mahfel.actors (id, kind, created_at, updated_at) \
                 VALUES ($1, 'user', $2, $2)",
            )
            .bind(user.id.as_uuid())
            .bind(user.created_at)
            .execute(&mut *transaction)
            .await
            .map_err(store_error)?;

            sqlx::query(
                "INSERT INTO mahfel.users (id, display_name, email, username, status, created_at, updated_at, avatar_url) \
                 VALUES ($1, $2, $3, $4, $5, $6, $6, $7)",
            )
            .bind(user.id.as_uuid())
            .bind(user.display_name.as_str())
            .bind(user.email.as_str())
            .bind(user.username.as_str())
            .bind(user.status.as_str())
            .bind(user.created_at)
            .bind(profile.avatar_url.as_ref().map(AvatarUrl::as_str))
            .execute(&mut *transaction)
            .await
            .map_err(store_error)?;

            let identity = Identity::create(
                user.id,
                issuer.clone(),
                subject.clone(),
                profile.display_name.clone(),
                profile.email.clone(),
                profile.username.clone(),
                profile.email_verified,
                profile.avatar_url.clone(),
                now,
            );

            let inserted = sqlx::query(INSERT_IDENTITY)
                .bind(identity.id.as_uuid())
                .bind(identity.user_id.as_uuid())
                .bind(identity.issuer.as_str())
                .bind(identity.subject.as_str())
                .bind(identity.email.as_ref().map(EmailAddress::as_str))
                .bind(identity.username.as_str())
                .bind(identity.email_verified)
                .bind(identity.display_name.as_ref().map(DisplayName::as_str))
                .bind(identity.avatar_url.as_ref().map(AvatarUrl::as_str))
                .bind(identity.created_at)
                .bind(identity.updated_at)
                .execute(&mut *transaction)
                .await
                .map_err(store_error)?;

            if inserted.rows_affected() == 0 {
                transaction.rollback().await.map_err(store_error)?;
                let existing = self
                    .find_by_identity(issuer, subject)
                    .await?
                    .ok_or_else(|| {
                        StoreError::Unexpected(
                            "identity vanished after a provisioning conflict".to_owned(),
                        )
                    })?;
                return self.refresh(existing, profile, now).await;
            }

            transaction.commit().await.map_err(store_error)?;
            Ok(ProvisionedIdentity { user, identity })
        })
    }

    fn find_by_identity<'a>(
        &'a self,
        issuer: &'a Issuer,
        subject: &'a Subject,
    ) -> BoxFuture<'a, Result<Option<ProvisionedIdentity>, StoreError>> {
        Box::pin(async move {
            let row = sqlx::query_as::<_, IdentityRow>(SELECT_IDENTITY)
                .bind(issuer.as_str())
                .bind(subject.as_str())
                .fetch_optional(&self.pool)
                .await
                .map_err(store_error)?;

            let Some(row) = row else {
                return Ok(None);
            };
            let identity = Identity::try_from(row).map_err(store_error)?;
            let user_row = sqlx::query_as::<_, UserRow>(
                "SELECT id, display_name, email, username, status, created_at, updated_at, avatar_url \
                 FROM mahfel.users WHERE id = $1",
            )
            .bind(identity.user_id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(store_error)?
            .ok_or_else(|| StoreError::MissingReference(format!("user {}", identity.user_id)))?;
            let user = User::try_from(user_row).map_err(store_error)?;
            Ok(Some(ProvisionedIdentity { user, identity }))
        })
    }
}

impl PostgresIdentityStore {
    /// Refresh stored provider profile fields for an existing identity.
    async fn refresh(
        &self,
        existing: ProvisionedIdentity,
        profile: &IdentityProfile,
        now: OffsetDateTime,
    ) -> Result<ProvisionedIdentity, StoreError> {
        let mut identity = existing.identity;
        identity.refresh_profile(
            profile.display_name.clone(),
            profile.email.clone(),
            profile.username.clone(),
            profile.email_verified,
            profile.avatar_url.clone(),
            now,
        );

        sqlx::query(
            "UPDATE mahfel.identities \
             SET email = $3, username = $4, email_verified = $5, display_name = $6, avatar_url = $7, updated_at = $8 \
             WHERE id = $1 AND user_id = $2",
        )
        .bind(identity.id.as_uuid())
        .bind(identity.user_id.as_uuid())
        .bind(identity.email.as_ref().map(EmailAddress::as_str))
        .bind(identity.username.as_str())
        .bind(identity.email_verified)
        .bind(identity.display_name.as_ref().map(DisplayName::as_str))
        .bind(identity.avatar_url.as_ref().map(AvatarUrl::as_str))
        .bind(identity.updated_at)
        .execute(&self.pool)
        .await
        .map_err(store_error)?;

        Ok(ProvisionedIdentity {
            user: existing.user,
            identity,
        })
    }
}
