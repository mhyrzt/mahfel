use super::codecs::media_codecs;
use super::error::RtcError;
use mahfel_domain::RoomId;
use mediasoup::{prelude::Router, router::RouterOptions, worker::Worker};
use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
use tokio::sync::RwLock;

#[allow(dead_code)]
pub(super) struct RouterEntry {
    pub(super) router: Router,
    pub(super) worker_index: usize,
}

pub(super) type RoomMap = Arc<RwLock<HashMap<RoomId, RouterEntry>>>;

pub(super) async fn create_room(
    rooms: &RoomMap,
    workers: &[Worker],
    next_worker: &AtomicUsize,
    room_id: RoomId,
) -> Result<bool, RtcError> {
    if rooms.read().await.contains_key(&room_id) {
        return Ok(false);
    }
    let mut map = rooms.write().await;
    if map.contains_key(&room_id) {
        return Ok(false);
    }
    let idx = next_worker.fetch_add(1, Ordering::Relaxed) % workers.len();
    let router = workers[idx]
        .create_router(RouterOptions::new(media_codecs()))
        .await?;
    map.insert(
        room_id,
        RouterEntry {
            router,
            worker_index: idx,
        },
    );
    Ok(true)
}

pub(super) async fn delete_room(
    rooms: &RoomMap,
    room_id: RoomId,
    transports: &super::transport::TransportMap,
    producers: &super::producer::ProducerMap,
    consumers: &super::consumer::ConsumerMap,
) -> bool {
    let removed = {
        let mut map = rooms.write().await;
        map.remove(&room_id)
    };
    if removed.is_some() {
        transports.write().await.retain(|_, v| v.room_id != room_id);
        producers.write().await.retain(|_, v| v.room_id != room_id);
        consumers.write().await.retain(|_, v| v.room_id != room_id);
        true
    } else {
        false
    }
}
