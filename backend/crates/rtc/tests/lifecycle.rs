use mahfel_domain::{ActorId, ProducerSource, RoomId};
use mahfel_rtc::MediaRuntime;
use mediasoup::prelude::MediaKind;
use std::{error::Error, net::UdpSocket};

#[tokio::test]
async fn participant_counts_and_consumer_cleanup_use_live_media_resources()
-> Result<(), Box<dyn Error>> {
    let port = UdpSocket::bind("127.0.0.1:0")?.local_addr()?.port();
    let media = MediaRuntime::start(1, port, port, "127.0.0.1", "127.0.0.1").await?;
    let room = RoomId::new();
    media.create_room(room).await?;
    let publisher = ActorId::new();
    let listener = ActorId::new();
    let (send, _, _, _, _) = media.create_transport(room, publisher).await?;
    let (receive, _, _, _, capabilities) = media.create_transport(room, listener).await?;
    let (second_receive, _, _, _, _) = media.create_transport(room, listener).await?;
    assert_eq!(
        media.participant_count().await,
        2,
        "receive-only actors count once"
    );
    let producer = media.create_producer(room, &send, publisher, MediaKind::Audio, ProducerSource::Microphone,
        r#"{"codecs":[{"mimeType":"audio/opus","payloadType":100,"clockRate":48000,"channels":2,"parameters":{},"rtcpFeedback":[]}],"headerExtensions":[],"encodings":[{"ssrc":12345}],"rtcp":{"cname":"lifecycle","reducedSize":true}}"#,
    ).await?;
    media
        .create_consumer(room, &receive, listener, &producer, &capabilities)
        .await?;
    media
        .create_consumer(room, &second_receive, listener, &producer, &capabilities)
        .await?;
    assert_eq!(media.consumer_count().await, 2);
    assert_eq!(media.close_participant(room, listener).await?, 0);
    assert_eq!(
        media.transport_count().await,
        1,
        "removal closes both devices of the target"
    );
    assert_eq!(
        media.consumer_count().await,
        0,
        "removing a listener removes owned consumers"
    );
    assert_eq!(
        media.producer_count().await,
        1,
        "another actor's producer survives"
    );
    let (receive, _, _, _, _) = media.create_transport(room, listener).await?;
    media
        .create_consumer(room, &receive, listener, &producer, &capabilities)
        .await?;
    assert_eq!(media.close_participant(room, publisher).await?, 1);
    assert_eq!(media.transport_count().await, 1);
    assert_eq!(
        media.close_participant(room, publisher).await?,
        0,
        "repeated removal is idempotent"
    );
    assert_eq!(
        media.consumer_count().await,
        0,
        "closed producers leave no dependent consumers"
    );
    let second_room = RoomId::new();
    media.create_room(second_room).await?;
    media.create_transport(second_room, listener).await?;
    assert_eq!(
        media.participant_count().await,
        2,
        "actors in different rooms count separately"
    );
    media.clear_room_media(room).await;
    assert_eq!(media.participant_count().await, 1);
    media.clear_room_media(second_room).await;
    assert_eq!(media.participant_count().await, 0);
    for _ in 0..12 {
        let racing_room = RoomId::new();
        media.create_room(racing_room).await?;
        let (creation, deletion) = tokio::join!(
            media.create_transport(racing_room, listener),
            media.delete_room(racing_room),
        );
        assert!(deletion);
        drop(creation);
        assert_eq!(media.transport_count().await, 0);
        assert_eq!(media.participant_count().await, 0);
    }
    Ok(())
}
