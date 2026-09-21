//! Revocable invite links: a short, human-typable bearer capability (ADR 0021).
//!
//! The code is deliberately low-entropy and is a capability rather than a
//! secret, so it must never be logged or used as a metric label.

use crate::actor::ActorId;
use crate::ids::entity_id;
use crate::room::RoomId;
use crate::session::SessionId;
use crate::status::status_conversion;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::OffsetDateTime;

entity_id!(InviteLinkId);

const CODE_LETTERS: usize = 10;
const CODE_GROUPS: [usize; 3] = [3, 4, 3];
const LETTERS: u32 = 26;
const CODE_SPACE: u64 = 141_167_095_653_376;

pub const DEFAULT_EXPIRY_DAYS: i64 = 7;
pub const MAX_EXPIRY_DAYS: i64 = 30;

/// Why a link operation was rejected.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum InviteLinkError {
    #[error("invite code is not in the xxx-yyyy-zzz format")]
    InvalidCode,
    #[error("invite link expiry is out of the permitted range")]
    InvalidExpiry,
    #[error("invite link is already revoked")]
    AlreadyRevoked,
}

/// A normalized invite code. Construction is the only way in, so a stored or
/// compared code is always canonical.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct InviteCode([u8; CODE_LETTERS]);

impl InviteCode {
    /// Bytes are rejection-sampled so every letter keeps equal probability;
    /// `byte % 26` would bias the first 22 letters.
    #[must_use]
    pub fn generate() -> Self {
        let mut letters = [0_u8; CODE_LETTERS];
        for slot in &mut letters {
            loop {
                let byte = random_byte();
                if let Some(letter) = letter_from(byte) {
                    *slot = letter;
                    break;
                }
            }
        }
        Self(letters)
    }

    /// Normalizes mixed case and separators before checking shape, so
    /// `XXX YYYY ZZZ` and `xxx_yyyy_zzz` resolve to the same code.
    ///
    /// # Errors
    /// [`InviteLinkError::InvalidCode`] when the input is not ten letters.
    pub fn parse(raw: &str) -> Result<Self, InviteLinkError> {
        let mut letters = [0_u8; CODE_LETTERS];
        let mut len = 0;
        for character in raw.chars() {
            if is_separator(character) {
                continue;
            }
            if len == CODE_LETTERS {
                return Err(InviteLinkError::InvalidCode);
            }
            let lowered = character.to_ascii_lowercase();
            if !lowered.is_ascii_lowercase() {
                return Err(InviteLinkError::InvalidCode);
            }
            letters[len] = lowered as u8;
            len += 1;
        }
        if len != CODE_LETTERS {
            return Err(InviteLinkError::InvalidCode);
        }
        Ok(Self(letters))
    }

    /// The canonical code without separators, as stored.
    #[must_use]
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.0).unwrap_or_default()
    }

    /// The displayed form, `xxx-yyyy-zzz`.
    #[must_use]
    pub fn format(&self) -> String {
        let mut out = String::with_capacity(CODE_LETTERS + 2);
        let mut offset = 0;
        for (index, group) in CODE_GROUPS.iter().enumerate() {
            if index > 0 {
                out.push('-');
            }
            out.push_str(
                self.as_str()
                    .get(offset..offset + group)
                    .unwrap_or_default(),
            );
            offset += group;
        }
        out
    }

    #[must_use]
    pub const fn space() -> u64 {
        CODE_SPACE
    }
}

impl std::fmt::Display for InviteCode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.format())
    }
}

impl TryFrom<&str> for InviteCode {
    type Error = InviteLinkError;

    fn try_from(raw: &str) -> Result<Self, Self::Error> {
        Self::parse(raw)
    }
}

fn is_separator(character: char) -> bool {
    character.is_whitespace() || matches!(character, '-' | '_' | '.' | ':' | '/')
}

fn letter_from(byte: u8) -> Option<u8> {
    let unbiased = u32::from(byte);
    let floor = (u32::from(u8::MAX) + 1) / LETTERS * LETTERS;
    if unbiased >= floor {
        return None;
    }
    Some(b'a' + (unbiased % LETTERS) as u8)
}

/// One random byte from UUIDv4 entropy, so no second random source is added.
fn random_byte() -> u8 {
    uuid::Uuid::new_v4().as_bytes()[0]
}

/// What a link points at. Exactly one target, never both.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InviteTargetKind {
    Room,
    Session,
}

status_conversion!(InviteTargetKind, "invite target", {
    Room => "room",
    Session => "session",
});

/// The single room or session a link points at. A closed enum rather than a
/// nullable pair, so "exactly one target" cannot be violated in memory.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "id")]
pub enum InviteTarget {
    Room(RoomId),
    Session(SessionId),
}

impl InviteTarget {
    #[must_use]
    pub const fn kind(self) -> InviteTargetKind {
        match self {
            Self::Room(_) => InviteTargetKind::Room,
            Self::Session(_) => InviteTargetKind::Session,
        }
    }

    #[must_use]
    pub const fn id(self) -> uuid::Uuid {
        match self {
            Self::Room(id) => id.as_uuid(),
            Self::Session(id) => id.as_uuid(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InviteLinkStatus {
    Active,
    Revoked,
    Expired,
}

status_conversion!(InviteLinkStatus, "invite link status", {
    Active => "active",
    Revoked => "revoked",
    Expired => "expired",
});

/// A revocable, human-typable entry capability for one target.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct InviteLink {
    pub id: InviteLinkId,
    pub code: InviteCode,
    pub target: InviteTarget,
    pub status: InviteLinkStatus,
    pub expires_at: OffsetDateTime,
    pub created_by: ActorId,
    pub created_at: OffsetDateTime,
    pub revoked_at: Option<OffsetDateTime>,
}

impl InviteLink {
    /// # Errors
    /// [`InviteLinkError::InvalidExpiry`] when the lifetime is not positive or
    /// exceeds [`MAX_EXPIRY_DAYS`].
    pub fn issue(
        code: InviteCode,
        target: InviteTarget,
        created_by: ActorId,
        expiry_days: i64,
        now: OffsetDateTime,
    ) -> Result<Self, InviteLinkError> {
        if expiry_days <= 0 || expiry_days > MAX_EXPIRY_DAYS {
            return Err(InviteLinkError::InvalidExpiry);
        }
        Ok(Self {
            id: InviteLinkId::new(),
            code,
            target,
            status: InviteLinkStatus::Active,
            expires_at: now + time::Duration::days(expiry_days),
            created_by,
            created_at: now,
            revoked_at: None,
        })
    }

    /// Terminal: a repeat revoke is rejected rather than moving the time.
    ///
    /// # Errors
    /// [`InviteLinkError::AlreadyRevoked`] on a repeat.
    pub fn revoke(&mut self, now: OffsetDateTime) -> Result<(), InviteLinkError> {
        if self.status == InviteLinkStatus::Revoked {
            return Err(InviteLinkError::AlreadyRevoked);
        }
        self.status = InviteLinkStatus::Revoked;
        self.revoked_at = Some(now);
        Ok(())
    }

    /// Expiry is decided by the clock rather than a stored flag, so there is no
    /// sweep to run and no window where a stale status is trusted.
    #[must_use]
    pub fn is_redeemable_at(&self, now: OffsetDateTime) -> bool {
        self.status == InviteLinkStatus::Active && now < self.expires_at
    }

    #[must_use]
    pub fn is_expired_at(&self, now: OffsetDateTime) -> bool {
        now >= self.expires_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn actor() -> ActorId {
        ActorId::new()
    }

    #[test]
    fn generated_codes_have_the_canonical_shape() {
        for _ in 0..64 {
            let code = InviteCode::generate();
            let formatted = code.format();
            assert_eq!(formatted.len(), 12, "3 + 4 + 3 letters plus two dashes");
            let parts: Vec<&str> = formatted.split('-').collect();
            assert_eq!(parts.len(), 3);
            assert_eq!((parts[0].len(), parts[1].len(), parts[2].len()), (3, 4, 3));
            assert!(
                code.as_str()
                    .chars()
                    .all(|c| c.is_ascii_lowercase() && c.is_ascii_alphabetic()),
                "codes are lowercase a-z only: {formatted}"
            );
        }
    }

    #[test]
    fn generation_is_not_obviously_repeating() {
        let codes: Vec<String> = (0..64).map(|_| InviteCode::generate().format()).collect();
        let mut unique = codes.clone();
        unique.sort();
        unique.dedup();
        assert!(unique.len() > 60, "generated codes should be near-unique");
    }

    #[test]
    fn parsing_normalizes_case_and_separators() {
        let canonical = InviteCode::parse("abc-defg-hij").expect("canonical");
        for variant in [
            "ABCDEFGHIJ",
            "abc defg hij",
            "ABC DEFG HIJ",
            "abc_defg_hij",
            "abc.defg.hij",
            "  abc-defg-hij  ",
        ] {
            assert_eq!(
                InviteCode::parse(variant).expect(variant),
                canonical,
                "{variant} must normalize to the canonical code"
            );
        }
        assert_eq!(canonical.as_str(), "abcdefghij");
        assert_eq!(canonical.format(), "abc-defg-hij");
    }

    #[test]
    fn parsing_rejects_non_codes() {
        for invalid in ["", "abc", "abcdefgh", "abcdefghijk", "abcdefghi1"] {
            assert_eq!(
                InviteCode::parse(invalid),
                Err(InviteLinkError::InvalidCode),
                "{invalid} must be rejected"
            );
        }
    }

    #[test]
    fn parsing_is_idempotent_and_canonical() {
        let code = InviteCode::generate();
        assert_eq!(InviteCode::parse(code.as_str()).expect("plain"), code);
        assert_eq!(InviteCode::parse(&code.format()).expect("formatted"), code);
        assert_eq!(
            InviteCode::parse(&code.format().to_uppercase()).expect("uppercase"),
            code
        );
    }

    #[test]
    fn link_issue_bounds_the_expiry() {
        let now = OffsetDateTime::now_utc();
        for days in [0, -1, MAX_EXPIRY_DAYS + 1] {
            assert_eq!(
                InviteLink::issue(
                    InviteCode::generate(),
                    InviteTarget::Room(RoomId::new()),
                    actor(),
                    days,
                    now
                ),
                Err(InviteLinkError::InvalidExpiry),
                "{days} days must be out of range"
            );
        }
        for days in [1, DEFAULT_EXPIRY_DAYS, MAX_EXPIRY_DAYS] {
            let link = InviteLink::issue(
                InviteCode::generate(),
                InviteTarget::Room(RoomId::new()),
                actor(),
                days,
                now,
            )
            .expect("in range");
            assert_eq!(link.expires_at, now + time::Duration::days(days));
        }
    }

    #[test]
    fn redeemability_rejects_revoked_and_expired() {
        let now = OffsetDateTime::now_utc();
        let mut link = InviteLink::issue(
            InviteCode::generate(),
            InviteTarget::Session(SessionId::new()),
            actor(),
            DEFAULT_EXPIRY_DAYS,
            now,
        )
        .expect("issue");
        assert!(link.is_redeemable_at(now));
        assert!(link.is_redeemable_at(link.expires_at - time::Duration::seconds(1)));
        assert!(!link.is_redeemable_at(link.expires_at));
        assert!(!link.is_redeemable_at(link.expires_at + time::Duration::days(1)));
        assert!(link.is_expired_at(link.expires_at));

        link.revoke(now).expect("revoke");
        assert!(!link.is_redeemable_at(now));
    }

    #[test]
    fn revoke_is_state_checked_and_terminal() {
        let now = OffsetDateTime::now_utc();
        let mut link = InviteLink::issue(
            InviteCode::generate(),
            InviteTarget::Room(RoomId::new()),
            actor(),
            1,
            now,
        )
        .expect("issue");
        link.revoke(now).expect("first revoke");
        assert_eq!(link.status, InviteLinkStatus::Revoked);
        assert_eq!(link.revoked_at, Some(now));
        assert_eq!(
            link.revoke(now + time::Duration::hours(1)),
            Err(InviteLinkError::AlreadyRevoked)
        );
        assert_eq!(
            link.revoked_at,
            Some(now),
            "a repeat must not move the time"
        );
    }

    #[test]
    fn target_kind_matches_the_variant() {
        let room = InviteTarget::Room(RoomId::new());
        let session = InviteTarget::Session(SessionId::new());
        assert_eq!(room.kind(), InviteTargetKind::Room);
        assert_eq!(session.kind(), InviteTargetKind::Session);
        assert_eq!(
            room.id(),
            match room {
                InviteTarget::Room(id) => id.as_uuid(),
                InviteTarget::Session(_) => unreachable!(),
            }
        );
    }

    #[test]
    fn storage_names_are_stable() {
        assert_eq!(InviteLinkStatus::Active.as_str(), "active");
        assert_eq!(InviteLinkStatus::Revoked.as_str(), "revoked");
        assert_eq!(InviteLinkStatus::Expired.as_str(), "expired");
        assert_eq!(InviteTargetKind::Room.as_str(), "room");
        assert_eq!(InviteTargetKind::Session.as_str(), "session");
        assert!(InviteLinkStatus::try_from("open").is_err());
    }

    #[test]
    fn code_space_is_documented() {
        assert_eq!(InviteCode::space(), 26_u64.pow(10));
    }
}
