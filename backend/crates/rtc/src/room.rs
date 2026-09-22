use super::codecs::media_codecs;
use super::error::RtcError;
use super::speaker::SpeakerHub;
use mahfel_domain::RoomId;
use mediasoup::{
    audio_level_observer::{AudioLevelObserver, AudioLevelObserverOptions},
    prelude::Router,
    router::RouterOptions,
    worker::Worker,
};
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
    pub(super) audio_observer: AudioLevelObserver,
    pub(super) worker_index: usize,
}

pub(super) type RoomMap = Arc<RwLock<HashMap<RoomId, RouterEntry>>>;

pub(super) async fn create_room(
    rooms: &RoomMap,
    workers: &[Worker],
    next_worker: &AtomicUsize,
    speakers: &SpeakerHub,
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
    let mut options = AudioLevelObserverOptions::default();
    options.max_entries = std::num::NonZeroU16::new(8)
        .ok_or_else(|| RtcError::Observer("invalid speaker limit".to_owned()))?;
    options.threshold = -80;
    options.interval = 200;
    let audio_observer = router
        .create_audio_level_observer(options)
        .await
        .map_err(|error| RtcError::Observer(error.to_string()))?;
    speakers.add_room(room_id);
    let volumes_hub = speakers.clone();
    audio_observer
        .on_volumes(move |volumes| {
            let samples: Vec<_> = volumes
                .iter()
                .map(|volume| (volume.producer.id().to_string(), volume.volume))
                .collect();
            volumes_hub.observe(room_id, &samples);
        })
        .detach();
    let silence_hub = speakers.clone();
    audio_observer
        .on_silence(move || silence_hub.silence(room_id))
        .detach();
    map.insert(
        room_id,
        RouterEntry {
            router,
            audio_observer,
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
    speakers: &SpeakerHub,
) -> bool {
    let removed = {
        let mut map = rooms.write().await;
        map.remove(&room_id)
    };
    if removed.is_some() {
        transports.write().await.retain(|_, v| v.room_id != room_id);
        producers.write().await.retain(|_, v| v.room_id != room_id);
        consumers.write().await.retain(|_, v| v.room_id != room_id);
        speakers.remove_room(room_id);
        true
    } else {
        false
    }
}

pub(super) async fn clear_room_media(
    rooms: &RoomMap,
    room_id: RoomId,
    transports: &super::transport::TransportMap,
    producers: &super::producer::ProducerMap,
    consumers: &super::consumer::ConsumerMap,
    speakers: &SpeakerHub,
) {
    if !rooms.read().await.contains_key(&room_id) {
        return;
    }
    transports
        .write()
        .await
        .retain(|_, value| value.room_id != room_id);
    producers
        .write()
        .await
        .retain(|_, value| value.room_id != room_id);
    consumers
        .write()
        .await
        .retain(|_, value| value.room_id != room_id);
    speakers.clear_room(room_id);
}

#[cfg(test)]
mod tests {
    use super::*;
    use mediasoup::rtp_observer::RtpObserver;
    use mediasoup::worker::WorkerSettings;
    use mediasoup::worker_manager::WorkerManager;

    #[tokio::test]
    async fn native_worker_creates_and_closes_audio_observer_with_room() {
        let manager = WorkerManager::new();
        let worker = manager
            .create_worker(WorkerSettings::default())
            .await
            .expect("worker");
        let rooms: RoomMap = Arc::new(RwLock::new(HashMap::new()));
        let transports = Arc::new(RwLock::new(HashMap::new()));
        let producers = Arc::new(RwLock::new(HashMap::new()));
        let consumers = Arc::new(RwLock::new(HashMap::new()));
        let speakers = SpeakerHub::new();
        let room_id = RoomId::new();
        assert!(
            create_room(&rooms, &[worker], &AtomicUsize::new(0), &speakers, room_id)
                .await
                .expect("create room")
        );
        assert!(
            !rooms
                .read()
                .await
                .get(&room_id)
                .expect("router")
                .audio_observer
                .closed()
        );
        let (mut receiver, _) = speakers.subscribe_with_snapshots();
        clear_room_media(
            &rooms,
            room_id,
            &transports,
            &producers,
            &consumers,
            &speakers,
        )
        .await;
        assert!(rooms.read().await.contains_key(&room_id));
        assert!(
            receiver
                .try_recv()
                .expect("session media reset")
                .actor_ids
                .is_empty()
        );
        assert!(
            delete_room(
                &rooms,
                room_id,
                &transports,
                &producers,
                &consumers,
                &speakers
            )
            .await
        );
        assert_eq!(
            receiver.try_recv().expect("room clear").actor_ids,
            Vec::<mahfel_domain::ActorId>::new()
        );
    }
}
