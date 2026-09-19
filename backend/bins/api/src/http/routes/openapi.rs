use utoipa::OpenApi;

/// REST contract generated from the transport handlers and schemas.
///
/// The WebSocket gateway is intentionally absent: it is not REST (ADR 0015).
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Mahfel API",
        version = env!("CARGO_PKG_VERSION"),
        description = "Control-plane REST API. Requests are authenticated with a bearer \
                       credential; the local development principal is configured server-side."
    ),
    paths(
        super::status::status,
        super::health::health,
        super::echo::list,
        super::echo::create,
        super::identity::me,
        super::rooms::create::create,
        super::rooms::list::list_rooms,
        super::rooms::get::get_room,
        super::rooms::join::join,
        super::rooms::archive::archive,
        super::rooms::members::add,
        super::rooms::members::remove,
        super::rooms::update::update,
    ),
    components(schemas(
        super::status::StatusResponse,
        super::health::HealthResponse,
        super::identity::IdentityResponse,
        super::echo::EchoPayload,
        crate::http::pagination::PaginationMeta,
        crate::http::pagination::Paginated<super::echo::EchoPayload>,
        crate::http::pagination::Paginated<super::rooms::dto::RoomResponse>,
        super::rooms::dto::CreateRoomRequest,
        super::rooms::dto::UpdateRoomRequest,
        super::rooms::dto::AddMemberRequest,
        super::rooms::dto::RoomResponse,
        super::rooms::dto::MemberResponse,
        super::rooms::dto::RoomVisibilityDto,
        super::rooms::dto::RoomRoleDto,
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
    "/v1/me",
    "/v1/rooms",
    "/v1/rooms/{room_id}",
    "/v1/rooms/{room_id}/join",
    "/v1/rooms/{room_id}/archive",
    "/v1/rooms/{room_id}/members",
    "/v1/rooms/{room_id}/members/{user_id}",
];
