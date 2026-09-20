use mediasoup::prelude::{MimeTypeAudio, MimeTypeVideo, RtcpFeedback, RtpCodecCapability};
use std::num::{NonZeroU8, NonZeroU32};

/// Canonical router codecs for Mahfel classrooms.
///
/// Chosen from `supported_rtp_capabilities.rs` with stable payload types
/// 100-102 to avoid collisions with dynamic allocations.
#[must_use]
pub fn media_codecs() -> Vec<RtpCodecCapability> {
    let mut opus_params = mediasoup::prelude::RtpCodecParametersParameters::default();
    opus_params.insert("useinbandfec", 1_u32);
    opus_params.insert("usedtx", 1_u32);

    let mut h264_params = mediasoup::prelude::RtpCodecParametersParameters::default();
    h264_params.insert("packetization-mode", 1_u32);
    h264_params.insert("profile-level-id", "42e01f");
    h264_params.insert("level-asymmetry-allowed", 1_u32);

    vec![
        RtpCodecCapability::Audio {
            mime_type: MimeTypeAudio::Opus,
            preferred_payload_type: Some(100),
            clock_rate: NonZeroU32::new(48000).expect("non-zero"),
            channels: NonZeroU8::new(2).expect("non-zero"),
            parameters: opus_params,
            rtcp_feedback: vec![RtcpFeedback::TransportCc],
        },
        RtpCodecCapability::Video {
            mime_type: MimeTypeVideo::Vp8,
            preferred_payload_type: Some(101),
            clock_rate: NonZeroU32::new(90000).expect("non-zero"),
            parameters: mediasoup::prelude::RtpCodecParametersParameters::default(),
            rtcp_feedback: vec![
                RtcpFeedback::Nack,
                RtcpFeedback::NackPli,
                RtcpFeedback::CcmFir,
                RtcpFeedback::GoogRemb,
                RtcpFeedback::TransportCc,
            ],
        },
        RtpCodecCapability::Video {
            mime_type: MimeTypeVideo::H264,
            preferred_payload_type: Some(102),
            clock_rate: NonZeroU32::new(90000).expect("non-zero"),
            parameters: h264_params,
            rtcp_feedback: vec![
                RtcpFeedback::Nack,
                RtcpFeedback::NackPli,
                RtcpFeedback::CcmFir,
                RtcpFeedback::GoogRemb,
                RtcpFeedback::TransportCc,
            ],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codecs_are_stable() {
        let codecs = media_codecs();
        assert_eq!(codecs.len(), 3);
        assert_eq!(codecs[0].preferred_payload_type(), Some(100));
        assert_eq!(codecs[1].preferred_payload_type(), Some(101));
        assert_eq!(codecs[2].preferred_payload_type(), Some(102));
    }
}
