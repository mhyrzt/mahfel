use super::error::RtcError;
use super::room::RoomMap;
use mahfel_domain::{ActorId, RoomId};
use mediasoup::prelude::{
    DtlsParameters, IceCandidate, IceParameters, Transport, WebRtcServer, WebRtcTransport,
    WebRtcTransportOptions, WebRtcTransportRemoteParameters,
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

#[allow(dead_code)]
pub(super) struct TransportEntry {
    pub(super) transport: WebRtcTransport,
    pub(super) room_id: RoomId,
    pub(super) participant_id: ActorId,
}

pub(super) type TransportMap = Arc<RwLock<HashMap<String, TransportEntry>>>;

pub(super) async fn create_transport(
    rooms: &RoomMap,
    servers: &[WebRtcServer],
    transports: &TransportMap,
    room_id: RoomId,
    participant_id: ActorId,
) -> Result<(String, String, String, String), RtcError> {
    let (router, worker_index) = {
        let map = rooms.read().await;
        let entry = map
            .get(&room_id)
            .ok_or(RtcError::RoomNotFound { room_id })?;
        (entry.router.clone(), entry.worker_index)
    };
    let server = servers[worker_index].clone();
    let transport = router
        .create_webrtc_transport(WebRtcTransportOptions::new_with_server(server))
        .await
        .map_err(|e| RtcError::Transport(e.to_string()))?;

    let ice_parameters: &IceParameters = transport.ice_parameters();
    let ice_candidates: &Vec<IceCandidate> = transport.ice_candidates();
    let dtls_parameters: DtlsParameters = transport.dtls_parameters();

    let ice_parameters_json =
        serde_json::to_string(ice_parameters).map_err(|e| RtcError::Transport(e.to_string()))?;
    let ice_candidates_json =
        serde_json::to_string(ice_candidates).map_err(|e| RtcError::Transport(e.to_string()))?;
    let dtls_parameters_json =
        serde_json::to_string(&dtls_parameters).map_err(|e| RtcError::Transport(e.to_string()))?;

    let transport_id = transport.id().to_string();
    transports.write().await.insert(
        transport_id.clone(),
        TransportEntry {
            transport,
            room_id,
            participant_id,
        },
    );

    Ok((
        transport_id,
        ice_parameters_json,
        ice_candidates_json,
        dtls_parameters_json,
    ))
}

pub(super) async fn connect_transport(
    transports: &TransportMap,
    room_id: RoomId,
    transport_id: &str,
    dtls_parameters_json: &str,
) -> Result<(), RtcError> {
    let dtls_parameters: DtlsParameters =
        serde_json::from_str(dtls_parameters_json).map_err(|e| RtcError::Dtls(e.to_string()))?;
    let map = transports.read().await;
    let entry = map
        .get(transport_id)
        .ok_or_else(|| RtcError::TransportNotFound {
            transport_id: transport_id.to_owned(),
        })?;
    if entry.room_id != room_id {
        return Err(RtcError::TransportNotFound {
            transport_id: transport_id.to_owned(),
        });
    }
    entry
        .transport
        .connect(WebRtcTransportRemoteParameters { dtls_parameters })
        .await
        .map_err(|e| RtcError::Connect(e.to_string()))?;
    Ok(())
}
