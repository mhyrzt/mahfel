use crate::status::status_conversion;
use serde::{Deserialize, Serialize};

/// Whether the SFU carries a producer as audio or video.
///
/// This is the only media distinction the media plane makes. It is an
/// application-level value so the control plane and the private media client
/// can name a track kind without depending on the mediasoup implementation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaKind {
    Audio,
    Video,
}

status_conversion!(MediaKind, "media kind", {
    Audio => "audio",
    Video => "video",
});

/// What a producer is for, at the application level.
///
/// The SFU only distinguishes audio from video, and a screen share genuinely is
/// video, so `kind` alone cannot express whether a stream is a microphone, a
/// camera, or a screen. Moderation needs that distinction: stopping a screen
/// share must not stop the target's camera, and muting someone must not end
/// their video.
///
/// A source is an application-level intent, not an authorization decision. It
/// selects which producers an action applies to; whether a principal may
/// publish or share at all is decided by capability. A mislabelled source can
/// therefore narrow an action but never widen one.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProducerSource {
    /// Captured audio.
    Microphone,
    /// Captured video.
    Camera,
    /// Shared screen contents, which the SFU carries as video.
    Screen,
}

impl ProducerSource {
    /// Whether this source is carried as audio by the SFU.
    ///
    /// Used to keep a source consistent with the media kind it was sent with,
    /// so a caller cannot label an audio track as a screen share.
    #[must_use]
    pub const fn is_audio(self) -> bool {
        matches!(self, Self::Microphone)
    }
}

status_conversion!(ProducerSource, "producer source", {
    Microphone => "microphone",
    Camera => "camera",
    Screen => "screen",
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn media_kind_names_are_stable() {
        assert_eq!(MediaKind::Audio.as_str(), "audio");
        assert_eq!(MediaKind::Video.as_str(), "video");
        assert_eq!(
            MediaKind::try_from("video").expect("known kind"),
            MediaKind::Video
        );
        assert!(MediaKind::try_from("data").is_err());
    }

    #[test]
    fn storage_names_are_stable() {
        assert_eq!(ProducerSource::Microphone.as_str(), "microphone");
        assert_eq!(ProducerSource::Camera.as_str(), "camera");
        assert_eq!(ProducerSource::Screen.as_str(), "screen");
        assert_eq!(
            ProducerSource::try_from("screen").expect("known source"),
            ProducerSource::Screen
        );
        assert!(ProducerSource::try_from("display").is_err());
    }

    #[test]
    fn only_the_microphone_is_audio() {
        assert!(ProducerSource::Microphone.is_audio());
        assert!(!ProducerSource::Camera.is_audio());
        assert!(
            !ProducerSource::Screen.is_audio(),
            "a screen share is carried as video"
        );
    }
}
