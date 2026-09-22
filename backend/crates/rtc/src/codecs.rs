use mediasoup::prelude::{MimeTypeAudio, MimeTypeVideo, RtcpFeedback, RtpCodecCapability};
use std::num::{NonZeroU8, NonZeroU32};

const OPUS_PAYLOAD_TYPE: u8 = 100;
const VP8_PAYLOAD_TYPE: u8 = 101;
// Mediasoup assigns 102 to VP8 RTX, so H.264 must use the next free type.
const H264_PAYLOAD_TYPE: u8 = 103;

/// Canonical router codecs for Mahfel classrooms.
///
/// Chosen from `supported_rtp_capabilities.rs` with stable payload types.
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
            preferred_payload_type: Some(OPUS_PAYLOAD_TYPE),
            clock_rate: NonZeroU32::new(48000).expect("non-zero"),
            channels: NonZeroU8::new(2).expect("non-zero"),
            parameters: opus_params,
            rtcp_feedback: vec![RtcpFeedback::TransportCc],
        },
        RtpCodecCapability::Video {
            mime_type: MimeTypeVideo::Vp8,
            preferred_payload_type: Some(VP8_PAYLOAD_TYPE),
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
            preferred_payload_type: Some(H264_PAYLOAD_TYPE),
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
        assert_eq!(
            (OPUS_PAYLOAD_TYPE, VP8_PAYLOAD_TYPE, H264_PAYLOAD_TYPE),
            (100, 101, 103)
        );
        let codecs = media_codecs();
        assert_eq!(codecs.len(), 3);
        assert_eq!(codecs[0].preferred_payload_type(), Some(OPUS_PAYLOAD_TYPE));
        assert_eq!(codecs[1].preferred_payload_type(), Some(VP8_PAYLOAD_TYPE));
        assert_eq!(codecs[2].preferred_payload_type(), Some(H264_PAYLOAD_TYPE));
    }
}
