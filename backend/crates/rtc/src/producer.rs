use super::error::RtcError;
use super::room::RoomMap;
use super::transport::TransportMap;
use mahfel_domain::{RoomId, UserId};
use mediasoup::prelude::{MediaKind, Producer, ProducerOptions, RtpParameters, Transport};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

#[allow(dead_code)]
pub(super) struct ProducerEntry {
    pub(super) producer: Producer,
    pub(super) room_id: RoomId,
    pub(super) transport_id: String,
    pub(super) participant_id: UserId,
}

pub(super) type ProducerMap = Arc<RwLock<HashMap<String, ProducerEntry>>>;

#[allow(clippy::too_many_arguments)]
pub(super) async fn create_producer(
    rooms: &RoomMap,
    transports: &TransportMap,
    producers: &ProducerMap,
    room_id: RoomId,
    transport_id: &str,
    participant_id: UserId,
    kind: MediaKind,
    rtp_parameters_json: &str,
) -> Result<String, RtcError> {
    let rtp_parameters: RtpParameters = serde_json::from_str(rtp_parameters_json)
        .map_err(|e| RtcError::RtpParameters(e.to_string()))?;

    {
        let map = rooms.read().await;
        if !map.contains_key(&room_id) {
            return Err(RtcError::RoomNotFound { room_id });
        }
    }

    let transport = {
        let map = transports.read().await;
        let entry = map
            .get(transport_id)
            .ok_or_else(|| RtcError::TransportNotFound {
                transport_id: transport_id.to_owned(),
            })?;
        if entry.room_id != room_id || entry.participant_id != participant_id {
            return Err(RtcError::TransportNotFound {
                transport_id: transport_id.to_owned(),
            });
        }
        entry.transport.clone()
    };

    let producer = transport
        .produce(ProducerOptions::new(kind, rtp_parameters))
        .await
        .map_err(|e| RtcError::Producer(e.to_string()))?;

    let producer_id = producer.id().to_string();
    producers.write().await.insert(
        producer_id.clone(),
        ProducerEntry {
            producer,
            room_id,
            transport_id: transport_id.to_owned(),
            participant_id,
        },
    );
    Ok(producer_id)
}
