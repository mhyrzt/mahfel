use anyhow::{Context, Result};
use mahfel_domain::{ActorId, Session};
use serde::Deserialize;

use crate::seed::{assets, files::SeedSource, identity, rooms};

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum SeedFile {
    DevelopmentPrincipal {
        id: String,
        username: String,
        email: String,
        display_name: String,
    },
    DevelopmentUsers {
        users: Vec<identity::DevelopmentUser>,
    },
    Rooms {
        rooms: Vec<rooms::DevelopmentRoom>,
    },
    Sessions {
        sessions: Vec<rooms::StandaloneSession>,
    },
    Files {
        files: Vec<assets::AssetSpec>,
    },
}

pub(super) enum Fixture {
    DevelopmentPrincipal(identity::ValidatedPrincipal),
    DevelopmentUsers(Vec<identity::ValidatedUser>),
    Rooms(Vec<rooms::ValidatedRoom>),
    Sessions(Vec<Session>),
    Files(Vec<assets::SeedAsset>),
}

impl Fixture {
    pub(super) fn needs_issuer(&self) -> bool {
        matches!(self, Self::DevelopmentUsers(_))
    }
}

pub(super) fn parse_seeds<S: Into<SeedSource>>(sources: Vec<S>) -> Result<Vec<(String, Fixture)>> {
    sources
        .into_iter()
        .map(|source| {
            let SeedSource {
                name,
                content,
                base_dir,
            } = source.into();
            let fixture = (|| -> Result<Fixture> {
                let seed: SeedFile = toml::from_str(&content).context("parse TOML")?;
                Ok(match seed {
                    SeedFile::DevelopmentPrincipal {
                        id,
                        username,
                        email,
                        display_name,
                    } => Fixture::DevelopmentPrincipal(identity::validate_principal(
                        id,
                        username,
                        email,
                        display_name,
                    )?),
                    SeedFile::DevelopmentUsers { users } => {
                        Fixture::DevelopmentUsers(identity::validate_users(users)?)
                    }
                    SeedFile::Rooms { rooms } => Fixture::Rooms(rooms::validate_rooms(rooms)?),
                    SeedFile::Sessions { sessions } => {
                        Fixture::Sessions(rooms::validate_sessions(sessions)?)
                    }
                    SeedFile::Files { files } => {
                        Fixture::Files(assets::validate_assets(files, base_dir.as_deref())?)
                    }
                })
            })()
            .with_context(|| format!("validate seed {name}"))?;
            Ok((name, fixture))
        })
        .collect()
}

pub(super) fn parse_actor_id(id: &str) -> Result<ActorId> {
    Ok(ActorId::from_uuid(
        uuid::Uuid::parse_str(id).context("validate fixture actor id")?,
    ))
}

#[cfg(test)]
mod tests {
    use super::{Fixture, parse_seeds};
    use crate::seed::files;

    #[test]
    fn embedded_fixtures_parse_in_filename_order() {
        let seeds = parse_seeds(files::embedded_seeds()).expect("embedded fixtures are valid");
        assert_eq!(seeds.len(), 3);
        assert_eq!(seeds[0].0, "0001_development_principal.toml");
        assert_eq!(seeds[1].0, "0002_development_users.toml");
        assert_eq!(seeds[2].0, "0003_development_rooms.toml");
        assert!(matches!(seeds[0].1, Fixture::DevelopmentPrincipal(_)));
        assert!(matches!(seeds[1].1, Fixture::DevelopmentUsers(_)));
        assert!(matches!(seeds[2].1, Fixture::Rooms(_)));
    }

    #[test]
    fn rejects_unexpected_fields_and_invalid_values() {
        let extra = "kind = 'development_principal'\nid = '00000000-0000-0000-0000-000000000001'\nusername = 'dev'\nemail = 'dev@mahfel.local'\ndisplay_name = 'Dev'\nstatement = 'DELETE FROM mahfel.users'\n";
        assert!(parse_seeds(vec![("extra.toml".into(), extra.into())]).is_err());
        let invalid = "kind = 'development_users'\n[[users]]\nid = 'bad'\nusername = 'dev'\nemail = 'dev@mahfel.local'\ndisplay_name = 'Dev'\npassword = 'password'\n";
        assert!(parse_seeds(vec![("invalid.toml".into(), invalid.into())]).is_err());
        let duplicate_member = "kind = 'rooms'\n[[rooms]]\nid = '00000000-0000-4000-8000-000000000101'\ntitle = 'Room'\nmanager_id = '00000000-0000-4000-8000-000000000004'\nvisibility = 'invite_only'\nmembers = ['00000000-0000-4000-8000-000000000004']\n";
        assert!(parse_seeds(vec![("room.toml".into(), duplicate_member.into())]).is_err());
        let standalone = "kind = 'sessions'\n[[sessions]]\nid = '00000000-0000-4000-8000-000000000201'\nroom_id = '00000000-0000-4000-8000-000000000101'\ntitle = 'Lesson'\n";
        assert!(matches!(
            parse_seeds(vec![("session.toml".into(), standalone.into())])
                .expect("standalone session parses")[0]
                .1,
            Fixture::Sessions(_)
        ));
    }
}
