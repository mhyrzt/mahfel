use super::error::RtcError;
use super::producer::ProducerMap;
use super::room::RoomMap;
use super::transport::TransportMap;
use mahfel_domain::{ActorId, RoomId};
use mediasoup::prelude::{Consumer, ConsumerOptions, RtpCapabilities, Transport};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

#[allow(dead_code)]
pub(super) struct ConsumerEntry {
    pub(super) consumer: Consumer,
    pub(super) room_id: RoomId,
    pub(super) transport_id: String,
    pub(super) producer_id: String,
    pub(super) participant_id: ActorId,
}

pub(super) type ConsumerMap = Arc<RwLock<HashMap<String, ConsumerEntry>>>;

#[allow(clippy::too_many_arguments)]
pub(super) async fn create_consumer(
    rooms: &RoomMap,
    transports: &TransportMap,
    producers: &ProducerMap,
    consumers: &ConsumerMap,
    room_id: RoomId,
    transport_id: &str,
    participant_id: ActorId,
    producer_id: &str,
    rtp_capabilities_json: &str,
) -> Result<(String, String, bool), RtcError> {
    let rtp_capabilities: RtpCapabilities = serde_json::from_str(rtp_capabilities_json)
        .map_err(|e| RtcError::RtpCapabilities(e.to_string()))?;

    let router = {
        let map = rooms.read().await;
        let entry = map
            .get(&room_id)
            .ok_or(RtcError::RoomNotFound { room_id })?;
        entry.router.clone()
    };

    {
        let map = producers.read().await;
        let entry = map
            .get(producer_id)
            .ok_or_else(|| RtcError::ProducerNotFound {
                producer_id: producer_id.to_owned(),
            })?;
        if entry.room_id != room_id {
            return Err(RtcError::ProducerNotFound {
                producer_id: producer_id.to_owned(),
            });
        }
        let producer_id_typed: mediasoup::prelude::ProducerId =
            producer_id
                .parse()
                .map_err(|_| RtcError::ProducerNotFound {
                    producer_id: producer_id.to_owned(),
                })?;
        if !router.can_consume(&producer_id_typed, &rtp_capabilities) {
            return Err(RtcError::Consumer(
                "cannot consume with given RTP capabilities".to_owned(),
            ));
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

    let producer_id_typed: mediasoup::prelude::ProducerId =
        producer_id
            .parse()
            .map_err(|_| RtcError::ProducerNotFound {
                producer_id: producer_id.to_owned(),
            })?;
    let mut options = ConsumerOptions::new(producer_id_typed, rtp_capabilities);
    options.paused = true;

    let consumer = transport
        .consume(options)
        .await
        .map_err(|e| RtcError::Consumer(e.to_string()))?;

    let rtp_parameters_json = serde_json::to_string(consumer.rtp_parameters())
        .map_err(|e| RtcError::Consumer(e.to_string()))?;
    let paused = consumer.paused();
    let consumer_id = consumer.id().to_string();

    let transport_map = transports.read().await;
    if !transport_map
        .get(transport_id)
        .is_some_and(|entry| entry.room_id == room_id && entry.participant_id == participant_id)
    {
        return Err(RtcError::TransportNotFound {
            transport_id: transport_id.to_owned(),
        });
    }
    let producer_map = producers.read().await;
    if !producer_map
        .get(producer_id)
        .is_some_and(|entry| entry.room_id == room_id)
        || consumer.closed()
    {
        return Err(RtcError::ProducerNotFound {
            producer_id: producer_id.to_owned(),
        });
    }
    consumers.write().await.insert(
        consumer_id.clone(),
        ConsumerEntry {
            consumer,
            room_id,
            transport_id: transport_id.to_owned(),
            producer_id: producer_id.to_owned(),
            participant_id,
        },
    );

    Ok((consumer_id, rtp_parameters_json, paused))
}

pub(super) async fn pause_consumer(
    consumers: &ConsumerMap,
    room_id: RoomId,
    consumer_id: &str,
) -> Result<(), RtcError> {
    let map = consumers.read().await;
    let entry = map
        .get(consumer_id)
        .ok_or_else(|| RtcError::ConsumerNotFound {
            consumer_id: consumer_id.to_owned(),
        })?;
    if entry.room_id != room_id {
        return Err(RtcError::ConsumerNotFound {
            consumer_id: consumer_id.to_owned(),
        });
    }
    let consumer = entry.consumer.clone();
    drop(map);
    consumer
        .pause()
        .await
        .map_err(|e| RtcError::Consumer(e.to_string()))?;
    Ok(())
}

pub(super) async fn resume_consumer(
    consumers: &ConsumerMap,
    room_id: RoomId,
    consumer_id: &str,
) -> Result<(), RtcError> {
    let map = consumers.read().await;
    let entry = map
        .get(consumer_id)
        .ok_or_else(|| RtcError::ConsumerNotFound {
            consumer_id: consumer_id.to_owned(),
        })?;
    if entry.room_id != room_id {
        return Err(RtcError::ConsumerNotFound {
            consumer_id: consumer_id.to_owned(),
        });
    }
    let consumer = entry.consumer.clone();
    drop(map);
    consumer
        .resume()
        .await
        .map_err(|e| RtcError::Consumer(e.to_string()))?;
    Ok(())
}
