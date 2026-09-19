use thiserror::Error;

/// A stored status string that does not map to a known variant.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
#[error("unsupported {kind} status `{value}`")]
pub struct InvalidStatus {
    kind: &'static str,
    value: String,
}

impl InvalidStatus {
    pub(crate) fn new(kind: &'static str, value: String) -> Self {
        Self { kind, value }
    }
}

/// Give a status enum its stable storage name: `as_str` plus `TryFrom<&str>`.
///
/// Storage names are part of the database contract and must not change.
macro_rules! status_conversion {
    ($type:ty, $kind:literal, {$($variant:ident => $value:literal),+ $(,)?}) => {
        impl $type {
            #[must_use]
            pub const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $value),+
                }
            }
        }

        impl TryFrom<&str> for $type {
            type Error = $crate::status::InvalidStatus;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                match value {
                    $($value => Ok(Self::$variant)),+,
                    _ => Err($crate::status::InvalidStatus::new($kind, value.to_owned())),
                }
            }
        }
    };
}

pub(crate) use status_conversion;
