//! Private active-speaker stream, reset frames, and bounded delivery.

use super::MediaControlServiceImpl;
use mahfel_rtc::SpeakerSnapshot;
use mahfel_rtc_protocol::media::WatchActiveSpeakersResponse;
use std::{pin::Pin, time::Duration};
use tokio::sync::{broadcast, mpsc};
use tokio_stream::{Stream, wrappers::ReceiverStream};
use tonic::{Response, Status};

pub(super) type SpeakerStream =
    Pin<Box<dyn Stream<Item = Result<WatchActiveSpeakersResponse, Status>> + Send + 'static>>;

pub(super) async fn watch_active_speakers(
    service: &MediaControlServiceImpl,
) -> Result<Response<SpeakerStream>, Status> {
    let hub = service.media.speaker_hub();
    let dropped = service.metrics.speaker_stream_dropped_updates.clone();
    let (mut updates, initial) = hub.subscribe_with_snapshots();
    let (sender, receiver) = mpsc::channel(64);
    tokio::spawn(async move {
        if !send_speaker_reset(&sender, initial).await {
            return;
        }
        loop {
            match updates.recv().await {
                Ok(snapshot) => {
                    if !send_speaker_frame(
                        &sender,
                        WatchActiveSpeakersResponse {
                            room_id: snapshot.room_id.to_string(),
                            actor_ids: snapshot.actor_ids.iter().map(ToString::to_string).collect(),
                            reset: false,
                        },
                    )
                    .await
                    {
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    dropped.inc_by(skipped as f64);
                    tracing::warn!(skipped, "active speaker stream lagged; resynchronizing");
                    let (replacement, snapshots) = hub.subscribe_with_snapshots();
                    updates = replacement;
                    if !send_speaker_reset(&sender, snapshots).await {
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    });
    Ok(Response::new(Box::pin(ReceiverStream::new(receiver))))
}

async fn send_speaker_reset(
    sender: &mpsc::Sender<Result<WatchActiveSpeakersResponse, Status>>,
    snapshots: Vec<SpeakerSnapshot>,
) -> bool {
    if !send_speaker_frame(
        sender,
        WatchActiveSpeakersResponse {
            room_id: String::new(),
            actor_ids: Vec::new(),
            reset: true,
        },
    )
    .await
    {
        return false;
    }
    for snapshot in snapshots {
        if !send_speaker_frame(
            sender,
            WatchActiveSpeakersResponse {
                room_id: snapshot.room_id.to_string(),
                actor_ids: snapshot.actor_ids.iter().map(ToString::to_string).collect(),
                reset: false,
            },
        )
        .await
        {
            return false;
        }
    }
    true
}

async fn send_speaker_frame(
    sender: &mpsc::Sender<Result<WatchActiveSpeakersResponse, Status>>,
    frame: WatchActiveSpeakersResponse,
) -> bool {
    tokio::time::timeout(Duration::from_secs(5), sender.send(Ok(frame)))
        .await
        .is_ok_and(|result| result.is_ok())
}
