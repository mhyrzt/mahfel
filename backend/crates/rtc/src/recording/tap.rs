use mahfel_domain::{ActorId, ProducerSource};
use mediasoup::plain_transport::PlainTransportRemoteParameters;
use mediasoup::prelude::{
    Consumer, ConsumerOptions, PlainTransport, PlainTransportOptions, Router, RtpCapabilities,
    Transport,
};

use crate::{
    RtcError,
    recording::{CaptureClock, CapturePipeline},
};

pub struct RecordingTap {
    pub producer_id: String,
    pub actor_id: ActorId,
    pub source: ProducerSource,
    pub pipeline: CapturePipeline,
    consumer: Consumer,
    _transport: PlainTransport,
}

impl RecordingTap {
    pub async fn create(
        router: &Router,
        producer_id: &str,
        actor_id: ActorId,
        source: ProducerSource,
        directory: &std::path::Path,
        clock: &CaptureClock,
    ) -> Result<Self, RtcError> {
        let listen = mediasoup::prelude::ListenInfo {
            protocol: mediasoup::prelude::Protocol::Udp,
            ip: std::net::Ipv4Addr::LOCALHOST.into(),
            announced_address: None,
            expose_internal_ip: false,
            port: None,
            port_range: None,
            flags: None,
            send_buffer_size: Some(262_144),
            recv_buffer_size: Some(262_144),
        };
        let mut options = PlainTransportOptions::new(listen);
        options.rtcp_mux = false;
        let transport = router
            .create_plain_transport(options)
            .await
            .map_err(|error| RtcError::Recording(error.to_string()))?;
        let capabilities: RtpCapabilities = serde_json::from_value(
            serde_json::to_value(router.rtp_capabilities())
                .map_err(|error| RtcError::Recording(error.to_string()))?,
        )
        .map_err(|error| RtcError::Recording(error.to_string()))?;
        let mut options = ConsumerOptions::new(
            producer_id
                .parse()
                .map_err(|_| RtcError::Recording("invalid producer id".into()))?,
            capabilities,
        );
        options.paused = true;
        if !source.is_audio() {
            options.preferred_layers = Some(mediasoup::consumer::ConsumerLayers {
                spatial_layer: 0,
                temporal_layer: None,
            });
        }
        let consumer = transport
            .consume(options)
            .await
            .map_err(|error| RtcError::Recording(error.to_string()))?;
        let parameters = serde_json::to_value(consumer.rtp_parameters())
            .map_err(|error| RtcError::Recording(error.to_string()))?;
        let directory = directory.to_owned();
        let clock = clock.clone();
        let pipeline = tokio::task::spawn_blocking(move || {
            CapturePipeline::start(&parameters, &directory, &clock)
        })
        .await
        .map_err(|error| RtcError::Recording(error.to_string()))??;
        let rtcp_port = transport
            .rtcp_tuple()
            .ok_or_else(|| RtcError::Recording("missing RTCP tuple".into()))?
            .local_port();
        pipeline.set_rtcp_feedback_port(rtcp_port)?;
        transport
            .connect(PlainTransportRemoteParameters {
                ip: Some(std::net::Ipv4Addr::LOCALHOST.into()),
                port: Some(pipeline.rtp_port()?),
                rtcp_port: Some(pipeline.rtcp_port()?),
                srtp_parameters: None,
            })
            .await
            .map_err(|error| RtcError::Recording(error.to_string()))?;
        consumer
            .resume()
            .await
            .map_err(|error| RtcError::Recording(error.to_string()))?;
        if !source.is_audio() {
            consumer
                .request_key_frame()
                .await
                .map_err(|error| RtcError::Recording(error.to_string()))?;
        }
        Ok(Self {
            producer_id: producer_id.to_owned(),
            actor_id,
            source,
            pipeline,
            consumer,
            _transport: transport,
        })
    }

    pub async fn request_key_frame(&self) -> Result<(), RtcError> {
        if !self.source.is_audio() && !self.consumer.closed() && !self.consumer.producer_paused() {
            self.consumer
                .request_key_frame()
                .await
                .map_err(|error| RtcError::Recording(error.to_string()))?;
        }
        Ok(())
    }

    pub fn closed(&self) -> bool {
        self.consumer.closed()
    }

    pub async fn seal(&self) -> Result<(), RtcError> {
        if !self.consumer.closed() {
            self.consumer
                .pause()
                .await
                .map_err(|error| RtcError::Recording(error.to_string()))?;
        }
        self.pipeline.seal()
    }
}
