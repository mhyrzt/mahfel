use crate::metrics::MediaMetrics;
use crate::validation::saturating_u32;
use mahfel_domain::NodeId;
use mahfel_rtc::MediaRuntime;
use mahfel_rtc_protocol::media::{
    GetNodeInfoRequest, GetNodeInfoResponse, GetNodeStatsRequest, GetNodeStatsResponse,
    WatchActiveSpeakersRequest,
};
use tonic::{Request, Response, Status};

use crate::validation::saturating_u64;

mod speaker;

#[derive(Clone)]
pub struct MediaControlServiceImpl {
    pub recordings: crate::recording::Recordings,
    pub node_id: NodeId,
    pub worker_count: u32,
    pub rtc_ports: Vec<u32>,
    pub media: std::sync::Arc<MediaRuntime>,
    pub metrics: MediaMetrics,
}

#[tonic::async_trait]
impl mahfel_rtc_protocol::media::media_control_service_server::MediaControlService
    for MediaControlServiceImpl
{
    type WatchActiveSpeakersStream = speaker::SpeakerStream;
    async fn prepare_recording(
        &self,
        request: Request<mahfel_rtc_protocol::media::PrepareRecordingRequest>,
    ) -> Result<Response<mahfel_rtc_protocol::media::PrepareRecordingResponse>, Status> {
        self.recordings
            .prepare(&request.into_inner().room_id)
            .await?;
        Ok(Response::new(
            mahfel_rtc_protocol::media::PrepareRecordingResponse {},
        ))
    }
    async fn reconcile_recording(
        &self,
        request: Request<mahfel_rtc_protocol::media::ReconcileRecordingRequest>,
    ) -> Result<Response<mahfel_rtc_protocol::media::ReconcileRecordingResponse>, Status> {
        self.recordings
            .reconcile(request.into_inner())
            .await
            .map(Response::new)
    }

    async fn watch_active_speakers(
        &self,
        _request: Request<WatchActiveSpeakersRequest>,
    ) -> Result<Response<Self::WatchActiveSpeakersStream>, Status> {
        speaker::watch_active_speakers(self).await
    }

    async fn get_node_info(
        &self,
        _request: Request<GetNodeInfoRequest>,
    ) -> Result<Response<GetNodeInfoResponse>, Status> {
        Ok(Response::new(GetNodeInfoResponse {
            node_id: self.node_id.to_string(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            worker_count: self.worker_count,
            rtc_ports: self.rtc_ports.clone(),
        }))
    }

    async fn get_node_stats(
        &self,
        _request: Request<GetNodeStatsRequest>,
    ) -> Result<Response<GetNodeStatsResponse>, Status> {
        Ok(Response::new(GetNodeStatsResponse {
            active_workers: saturating_u32(self.media.active_workers()),
            active_rooms: saturating_u64(self.media.room_count().await),
            participants: 0,
            transports: saturating_u64(self.media.transport_count().await),
            producers: saturating_u64(self.media.producer_count().await),
            consumers: saturating_u64(self.media.consumer_count().await),
        }))
    }

    async fn create_room(
        &self,
        request: Request<mahfel_rtc_protocol::media::CreateRoomRequest>,
    ) -> Result<Response<mahfel_rtc_protocol::media::CreateRoomResponse>, Status> {
        crate::handlers::room::create_room(&self.media, request).await
    }

    async fn delete_room(
        &self,
        request: Request<mahfel_rtc_protocol::media::DeleteRoomRequest>,
    ) -> Result<Response<mahfel_rtc_protocol::media::DeleteRoomResponse>, Status> {
        self.recordings.seal_room(&request.get_ref().room_id).await;
        crate::handlers::room::delete_room(&self.media, request).await
    }

    async fn clear_room_media(
        &self,
        request: Request<mahfel_rtc_protocol::media::ClearRoomMediaRequest>,
    ) -> Result<Response<mahfel_rtc_protocol::media::ClearRoomMediaResponse>, Status> {
        self.recordings.seal_room(&request.get_ref().room_id).await;
        crate::handlers::room::clear_room_media(&self.media, request).await
    }

    async fn create_web_rtc_transport(
        &self,
        request: Request<mahfel_rtc_protocol::media::CreateWebRtcTransportRequest>,
    ) -> Result<Response<mahfel_rtc_protocol::media::CreateWebRtcTransportResponse>, Status> {
        crate::handlers::transport::create_web_rtc_transport(&self.media, request).await
    }

    async fn connect_web_rtc_transport(
        &self,
        request: Request<mahfel_rtc_protocol::media::ConnectWebRtcTransportRequest>,
    ) -> Result<Response<mahfel_rtc_protocol::media::ConnectWebRtcTransportResponse>, Status> {
        crate::handlers::transport::connect_web_rtc_transport(&self.media, request).await
    }

    async fn close_web_rtc_transport(
        &self,
        request: Request<mahfel_rtc_protocol::media::CloseWebRtcTransportRequest>,
    ) -> Result<Response<mahfel_rtc_protocol::media::CloseWebRtcTransportResponse>, Status> {
        crate::handlers::transport::close_web_rtc_transport(&self.media, request).await
    }

    async fn produce(
        &self,
        request: Request<mahfel_rtc_protocol::media::ProduceRequest>,
    ) -> Result<Response<mahfel_rtc_protocol::media::ProduceResponse>, Status> {
        crate::handlers::producer::produce(&self.media, request).await
    }

    async fn list_producers(
        &self,
        request: Request<mahfel_rtc_protocol::media::ListProducersRequest>,
    ) -> Result<Response<mahfel_rtc_protocol::media::ListProducersResponse>, Status> {
        crate::handlers::producer::list_producers(&self.media, request).await
    }

    async fn pause_producer(
        &self,
        request: Request<mahfel_rtc_protocol::media::PauseProducerRequest>,
    ) -> Result<Response<mahfel_rtc_protocol::media::PauseProducerResponse>, Status> {
        crate::handlers::producer::pause_producer(&self.media, request).await
    }

    async fn resume_producer(
        &self,
        request: Request<mahfel_rtc_protocol::media::ResumeProducerRequest>,
    ) -> Result<Response<mahfel_rtc_protocol::media::ResumeProducerResponse>, Status> {
        crate::handlers::producer::resume_producer(&self.media, request).await
    }

    async fn close_participant_source(
        &self,
        request: Request<mahfel_rtc_protocol::media::CloseParticipantSourceRequest>,
    ) -> Result<Response<mahfel_rtc_protocol::media::CloseParticipantSourceResponse>, Status> {
        crate::handlers::producer::close_participant_source(&self.media, request).await
    }

    async fn pause_participant_source(
        &self,
        request: Request<mahfel_rtc_protocol::media::PauseParticipantSourceRequest>,
    ) -> Result<Response<mahfel_rtc_protocol::media::PauseParticipantSourceResponse>, Status> {
        crate::handlers::producer::pause_participant_source(&self.media, request).await
    }

    async fn resume_participant_source(
        &self,
        request: Request<mahfel_rtc_protocol::media::ResumeParticipantSourceRequest>,
    ) -> Result<Response<mahfel_rtc_protocol::media::ResumeParticipantSourceResponse>, Status> {
        crate::handlers::producer::resume_participant_source(&self.media, request).await
    }

    async fn consume(
        &self,
        request: Request<mahfel_rtc_protocol::media::ConsumeRequest>,
    ) -> Result<Response<mahfel_rtc_protocol::media::ConsumeResponse>, Status> {
        crate::handlers::consumer::consume(&self.media, request).await
    }

    async fn pause_consumer(
        &self,
        request: Request<mahfel_rtc_protocol::media::PauseConsumerRequest>,
    ) -> Result<Response<mahfel_rtc_protocol::media::PauseConsumerResponse>, Status> {
        crate::handlers::consumer::pause_consumer(&self.media, request).await
    }

    async fn resume_consumer(
        &self,
        request: Request<mahfel_rtc_protocol::media::ResumeConsumerRequest>,
    ) -> Result<Response<mahfel_rtc_protocol::media::ResumeConsumerResponse>, Status> {
        crate::handlers::consumer::resume_consumer(&self.media, request).await
    }

    async fn close_participant(
        &self,
        request: Request<mahfel_rtc_protocol::media::CloseParticipantRequest>,
    ) -> Result<Response<mahfel_rtc_protocol::media::CloseParticipantResponse>, Status> {
        crate::handlers::producer::close_participant(&self.media, request).await
    }
}
