use utoipa::OpenApi;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::openapi::{Content, Ref, RefOr, path::Operation};

struct AuthScheme;

impl utoipa::Modify for AuthScheme {
    fn modify(&self, doc: &mut utoipa::openapi::OpenApi) {
        doc.components
            .get_or_insert_with(Default::default)
            .add_security_scheme(
            "bearerAuth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .description(Some(
                        "Keycloak access token, or a Mahfel service principal key where supported",
                    ))
                    .build(),
            ),
        );
        for path in doc.paths.paths.values_mut() {
            for operation in [
                &mut path.get,
                &mut path.put,
                &mut path.post,
                &mut path.delete,
                &mut path.options,
                &mut path.head,
                &mut path.patch,
                &mut path.trace,
            ]
            .into_iter()
            .flatten()
            {
                document_error_responses(operation);
            }
        }
    }
}

fn document_error_responses(operation: &mut Operation) {
    for (status, response) in &mut operation.responses.responses {
        if !status.starts_with('4') && !status.starts_with('5') {
            continue;
        }
        let RefOr::T(response) = response else {
            continue;
        };
        if response.content.is_empty() {
            response.content.insert(
                "application/json".to_owned(),
                Content::new(Some(Ref::from_schema_name("ErrorBody"))),
            );
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Mahfel API",
        version = env!("CARGO_PKG_VERSION"),
        description = r#"Control-plane REST API.
Protected routes require an Authorization: Bearer token. User requests use Keycloak access tokens. Supported service-principal routes also accept Mahfel keys. Provisioning requires a Keycloak access token."#
    ),
    modifiers(&AuthScheme),
    security(("bearerAuth" = [])),
    tags(
        (name = "files", description = "Context-authorized file metadata and direct S3 transfers")
    ),
    paths(
        // System and health endpoints
        super::status::status,
        super::health::health,
        super::echo::list,
        super::echo::create,
        crate::http::ws::tickets::create_ticket,
        // Files
        super::files::upload::initiate,
        super::files::upload::complete,
        super::files::download::download,
        super::files::library::list,
        super::files::library::remove,
        super::collaboration::handlers::documents,
        super::collaboration::handlers::register_image,
        super::collaboration::handlers::events,
        super::collaboration::handlers::set_grant,
        super::collaboration::handlers::grants,
        super::collaboration::handlers::update_viewport,
        super::collaboration::presentation::update_presentation,
        super::collaboration::presentation::get_presentation,
        // Identity and service principals
        super::identity::provision,
        super::identity::me,
        super::admin::ping,
        super::admin::service_principals::create,
        super::admin::service_principals::list,
        super::admin::service_principals::rotate,
        super::admin::service_principals::revoke,
        super::rooms::create::create,
        super::rooms::list::list_rooms,
        super::rooms::get::get_room,
        super::rooms::membership::own_membership,
        super::rooms::members::list,
        super::files::library::link_to_session,
        super::rooms::lobby::get_lobby,
        super::rooms::lobby::admit_principal,
        super::rooms::lobby::reject_principal,
        super::invite::status::request_status,
        super::rooms::chat::list_messages,
        super::rooms::chat::create_message,
        super::rooms::chat::edit_message,
        super::rooms::chat::delete_message,
        super::rooms::chat_interactions::list_replies,
        super::rooms::chat_interactions::create_reply,
        super::rooms::chat_interactions::list_mentions,
        super::rooms::chat_interactions::get_reactions,
        super::rooms::chat_interactions::set_reaction,
        super::rooms::chat_interactions::remove_reaction,
        super::rooms::chat_read_marker::get_marker,
        super::rooms::chat_read_marker::advance_marker,
        super::rooms::join::join,
        super::rooms::archive::archive,
        super::rooms::members::add,
        super::rooms::members::change_role,
        super::rooms::members::remove,
        super::rooms::update::update,
        super::sessions::schedule::schedule,
        super::sessions::details::update,
        super::sessions::list::list_sessions,
        super::sessions::get::get_session,
        super::sessions::roster::roster,
        super::sessions::capabilities::get_capabilities,
        super::participation::session::session_summary,
        super::participation::participant::participant_summary,
        super::participation::correct::correct,
        super::invite::create::create_for_room,
        super::invite::create::create_for_session,
        super::invite::list::list_for_room,
        super::invite::list::list_for_session,
        super::invite::redeem::redeem,
        super::invite::manage::revoke,
        super::invite::manage::rotate,
        super::rooms::admission::get_admission,
        super::rooms::admission::set_admission,
        // Session access and lifecycle
        super::sessions::access::get_access,
        super::sessions::access::set_access,
        super::sessions::access::get_lobby,
        super::sessions::access::own_admission,
        super::sessions::access::admit_principal,
        super::sessions::access::reject_principal,
        super::sessions::start::start,
        super::sessions::end::end,
        super::sessions::cancel::cancel,
        crate::http::turn::turn_credentials,
    ),
    components(schemas(
        crate::http::error::ErrorBody,
        crate::http::error::ErrorDetail,
        super::status::StatusResponse,
        super::health::HealthResponse,
        super::identity::ProfileResponse,
        super::admin::AdminResponse,
        super::admin::service_principals::dto::CreateServicePrincipalRequest,
        super::admin::service_principals::dto::ServicePrincipalResponse,
        super::admin::service_principals::dto::CreatedServicePrincipalResponse,
        super::admin::service_principals::dto::RotatedServicePrincipalKeyResponse,
        super::echo::EchoPayload,
        super::files::FileContextKind,
        super::files::FileContextInput,
        super::files::InitiateUploadRequest,
        super::files::FileResponse,
        super::files::SignedFileResponse,
        super::files::SignedHeader,
        super::files::FileListResponse,
        super::collaboration::CollaborationDocumentsResponse,
        super::collaboration::CollaborationEventsResponse,
        super::collaboration::EventResponse,
        super::collaboration::EditGrantRequest,
        super::collaboration::RegisterImageRequest,
        super::collaboration::ParticipantGrantsResponse,
        super::collaboration::ViewportRequest,
        super::collaboration::PresentationRequest,
        super::collaboration::PresentationStateResponse,
        crate::http::pagination::PaginationMeta,
        crate::http::pagination::Paginated<super::echo::EchoPayload>,
        crate::http::pagination::Paginated<super::rooms::dto::RoomResponse>,
        super::rooms::dto::CreateRoomRequest,
        super::rooms::dto::UpdateRoomRequest,
        super::rooms::dto::AddMemberRequest,
        super::rooms::dto::RoomResponse,
        super::rooms::dto::MemberResponse,
        super::rooms::dto::MemberListResponse,
        super::files::library::LinkRoomFileRequest,
        super::rooms::dto::OwnMembershipResponse,
        super::rooms::lobby::RoomLobbyResponse,
        super::rooms::lobby::RoomLobbyEntryResponse,
        super::invite::status::InviteRequestStatusResponse,
        super::rooms::dto::RoomVisibilityDto,
        super::rooms::dto::RoomRoleDto,
        super::rooms::chat::CreateChatMessageRequest,
        super::rooms::chat::ChatMessageResponse,
        super::rooms::chat::ChatMessageHistoryResponse,
        super::rooms::chat::EditChatMessageRequest,
        super::rooms::chat_interactions::ChatMentionsResponse,
        super::rooms::chat_interactions::SetReactionRequest,
        super::rooms::chat_interactions::ReactionCountResponse,
        super::rooms::chat_interactions::ReactionsResponse,
        super::rooms::chat_read_marker::AdvanceReadMarkerRequest,
        super::rooms::chat_read_marker::ReadMarkerResponse,
        super::sessions::dto::CreateSessionRequest,
        super::sessions::dto::UpdateSessionDetailsRequest,
        super::sessions::dto::SessionResponse,
        super::sessions::roster::RosterParticipantResponse,
        super::sessions::dto::SessionCapabilitiesResponse,
        super::participation::dto::CorrectionReasonDto,
        super::participation::dto::CorrectIntervalRequest,
        super::participation::dto::IntervalResponse,
        super::participation::dto::ParticipantParticipationResponse,
        super::participation::dto::ParticipantSummaryResponse,
        super::participation::dto::SessionParticipationResponse,
        super::invite::dto::InviteTargetKindDto,
        super::invite::dto::CreateRoomInviteLinkRequest,
        super::invite::dto::CreateSessionInviteLinkRequest,
        super::invite::dto::RotateInviteLinkRequest,
        super::invite::dto::InviteLinkResponse,
        super::invite::dto::InviteLinkListResponse,
        super::invite::redeem::RedeemInviteLinkRequest,
        super::invite::redeem::RedeemInviteLinkResponse,
        super::sessions::access::OwnAdmissionResponse,
        super::rooms::dto::AdmissionPolicyDto,
        super::sessions::dto::SessionVisibilityDto,
        super::rooms::admission::SetAdmissionRequest,
        super::rooms::admission::AdmissionPolicyResponse,
        super::sessions::access::SetSessionAccessRequest,
        super::sessions::access::SessionAccessResponse,
        super::sessions::access::LobbyEntryResponse,
        super::sessions::access::LobbyResponse,
        super::sessions::access::AdmittedResponse,
        super::sessions::dto::SessionStatusDto,
        crate::http::turn::TurnCredentialsResponse,
    ))
)]
pub(crate) struct ApiDoc;

/// REST path templates owned by the router, used by the coverage test to keep
/// the generated document and the route table in sync.
#[cfg(test)]
pub(crate) const REST_PATH_TEMPLATES: &[&str] = &[
    "/v1/status",
    "/v1/health",
    "/v1/echo",
    "/v1/files",
    "/v1/files/uploads",
    "/v1/files/{file_id}/complete",
    "/v1/files/{file_id}/download",
    "/v1/files/{file_id}",
    "/v1/sessions/{session_id}/collaboration",
    "/v1/sessions/{session_id}/collaboration/{kind}/images",
    "/v1/sessions/{session_id}/collaboration/events",
    "/v1/sessions/{session_id}/collaboration/grants",
    "/v1/sessions/{session_id}/collaboration/grants/{actor_id}",
    "/v1/sessions/{session_id}/collaboration/viewport",
    "/v1/sessions/{session_id}/collaboration/presentation",
    "/v1/auth/provision",
    "/v1/auth/me",
    "/v1/admin/ping",
    "/v1/admin/service-principals",
    "/v1/admin/service-principals/{id}",
    "/v1/admin/service-principals/{id}/rotate",
    "/v1/rooms",
    "/v1/rooms/{room_id}",
    "/v1/rooms/{room_id}/messages",
    "/v1/rooms/{room_id}/messages/{message_id}",
    "/v1/rooms/{room_id}/join",
    "/v1/rooms/{room_id}/archive",
    "/v1/rooms/{room_id}/members",
    "/v1/rooms/{room_id}/members/{user_id}",
    "/v1/rooms/{room_id}/sessions",
    "/v1/sessions/{session_id}",
    "/v1/sessions/{session_id}/capabilities",
    "/v1/sessions/{session_id}/roster",
    "/v1/sessions/{session_id}/start",
    "/v1/sessions/{session_id}/end",
    "/v1/sessions/{session_id}/cancel",
    "/v1/sessions/{session_id}/participation",
    "/v1/sessions/{session_id}/participants/{participant_id}/participation",
    "/v1/participation/intervals/{interval_id}/correct",
    "/v1/rooms/{room_id}/invite-links",
    "/v1/sessions/{session_id}/invite-links",
    "/v1/invite-links/redeem",
    "/v1/sessions/{session_id}/admission",
    "/v1/invite-links/{link_id}/revoke",
    "/v1/invite-links/{link_id}/rotate",
    "/v1/rooms/{room_id}/admission",
    "/v1/sessions/{session_id}/access",
    "/v1/sessions/{session_id}/lobby",
    "/v1/sessions/{session_id}/lobby/{actor_id}/admit",
    "/v1/sessions/{session_id}/lobby/{actor_id}/reject",
    "/v1/turn/credentials",
];
