//! RFC 3339 timestamp (de)serialization for transport DTOs.
//!
//! `time`'s default `serde` representation for `OffsetDateTime` is a tuple, and
//! its human-readable form uses a space separator, mandatory fractional
//! seconds, and offset seconds — not what a browser sends or expects. This
//! module pins the wire format to RFC 3339 so a timestamp in a request body and
//! a timestamp in a response are the same, conventional string.

use serde::{Deserialize, Deserializer, Serializer};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

/// Serialize an `OffsetDateTime` as an RFC 3339 string.
///
/// # Errors
/// Returns the serializer's error when the value cannot be formatted, which
/// only happens for an unrepresentable offset.
pub fn serialize<S>(value: &OffsetDateTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&value.format(&Rfc3339).map_err(serde::ser::Error::custom)?)
}

/// Deserialize an `OffsetDateTime` from an RFC 3339 string.
///
/// # Errors
/// Returns a deserialization error when the input is not a valid RFC 3339
/// timestamp.
pub fn deserialize<'de, D>(deserializer: D) -> Result<OffsetDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = String::deserialize(deserializer)?;
    OffsetDateTime::parse(&raw, &Rfc3339).map_err(serde::de::Error::custom)
}

/// RFC 3339 for an optional timestamp. Present-and-null is `None`.
pub mod option {
    use super::{Deserialize, Deserializer, OffsetDateTime, Rfc3339, Serializer};

    /// Serialize `Some` as RFC 3339 and `None` as JSON null.
    ///
    /// # Errors
    /// Returns the serializer's error when the value cannot be formatted.
    pub fn serialize<S>(value: &Option<OffsetDateTime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(value) => serializer
                .serialize_str(&value.format(&Rfc3339).map_err(serde::ser::Error::custom)?),
            None => serializer.serialize_none(),
        }
    }

    /// Deserialize JSON null as `None`, otherwise parse RFC 3339.
    ///
    /// # Errors
    /// Returns a deserialization error when the input is neither null nor a
    /// valid RFC 3339 timestamp.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<OffsetDateTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = Option::<String>::deserialize(deserializer)?;
        raw.map(|raw| OffsetDateTime::parse(&raw, &Rfc3339).map_err(serde::de::Error::custom))
            .transpose()
    }
}
