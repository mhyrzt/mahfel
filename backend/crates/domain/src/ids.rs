use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Define a UUID-backed entity identifier.
///
/// Each aggregate invokes this next to the entity that owns the identifier,
/// for example `entity_id!(UserId)` in `user.rs`. Paths are fully qualified so
/// the invocation site needs only the macro. Identifiers are value objects:
/// `Copy`, comparable, serialized transparently, and never constructed from
/// arbitrary strings.
macro_rules! entity_id {
    ($name:ident) => {
        #[derive(
            Clone, Copy, Debug, Eq, Hash, PartialEq, ::serde::Serialize, ::serde::Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(::uuid::Uuid);

        impl $name {
            #[must_use]
            pub fn new() -> Self {
                Self(::uuid::Uuid::new_v4())
            }

            #[must_use]
            pub const fn from_uuid(value: ::uuid::Uuid) -> Self {
                Self(value)
            }

            #[must_use]
            pub const fn as_uuid(self) -> ::uuid::Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, formatter: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                self.0.fmt(formatter)
            }
        }
    };
}

pub(crate) use entity_id;

/// Identifier of a media node. Unlike entity identifiers this is an opaque
/// string, because node identity comes from the deployment (host/instance),
/// not from a database row.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NodeId(String);

impl NodeId {
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
