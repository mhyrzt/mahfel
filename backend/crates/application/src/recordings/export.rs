use crate::{
    jobs::{JobEnvelope, JobFailure},
    policy::ServerCapabilityResolver,
    ports::{
        Capability, CapabilityContext, CapabilityResolver, Principal, RecordingExportStore,
        RecordingExporter, RoomStore, SessionStore,
    },
    recordings::RecordingOperationError,
};
use mahfel_domain::{RecordingExport, SessionId};
use uuid::Uuid;

pub async fn get_export<S: RecordingExportStore + ?Sized>(
    store: &S,
    session: SessionId,
    recording: Uuid,
    principal: &Principal,
) -> Result<RecordingExport, RecordingOperationError> {
    if !principal
        .roles
        .iter()
        .any(|role| role.eq_ignore_ascii_case("admin"))
        && !store.can_download(session, principal.actor_id).await?
    {
        return Err(RecordingOperationError::NotFound);
    }
    store
        .get(session, recording)
        .await?
        .ok_or(RecordingOperationError::NotFound)
}

pub async fn retry_export<
    S: RecordingExportStore + ?Sized,
    R: RoomStore + ?Sized,
    T: SessionStore + ?Sized,
>(
    store: &S,
    rooms: &R,
    sessions: &T,
    session: SessionId,
    recording: Uuid,
    principal: &Principal,
) -> Result<RecordingExport, RecordingOperationError> {
    get_export(store, session, recording, principal).await?;
    let target = sessions
        .get(session)
        .await?
        .ok_or(RecordingOperationError::NotFound)?;
    let membership = rooms.get_member(target.room_id, principal.actor_id).await?;
    if !ServerCapabilityResolver
        .resolve(
            principal,
            CapabilityContext::in_room(membership.map(|member| member.role)),
        )
        .has(Capability::ManageRecording)
    {
        return Err(RecordingOperationError::Forbidden);
    }
    Ok(store.retry(session, recording).await?)
}

pub async fn export_recording<S: RecordingExportStore + ?Sized, E: RecordingExporter + ?Sized>(
    store: &S,
    exporter: &E,
    job: &JobEnvelope,
    token: Uuid,
) -> Result<(), JobFailure> {
    let Some(source) = store
        .begin(job, token)
        .await
        .map_err(|_| JobFailure::Retryable("export_store_unavailable"))?
    else {
        return Ok(());
    };
    let artifact = exporter.export(&source, token).await?;
    if !store
        .accept(job, token, &artifact)
        .await
        .map_err(|_| JobFailure::Retryable("export_accept_unavailable"))?
    {
        exporter.discard(&artifact).await?;
        return Err(JobFailure::Retryable("export_claim_lost"));
    }
    Ok(())
}
