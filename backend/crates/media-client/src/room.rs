use super::gateway::GrpcMediaGateway;
use mahfel_application::ports::{MediaGateway, MediaGatewayError};
use mahfel_domain::RoomId;
use mahfel_rtc_protocol::media::{CreateRoomRequest, DeleteRoomRequest};

impl MediaGateway for GrpcMediaGateway {
    async fn create_room(&self, room: RoomId) -> Result<(), MediaGatewayError> {
        let mut client = self.client.clone();
        let request = CreateRoomRequest {
            room_id: room.to_string(),
        };
        client
            .create_room(request)
            .await
            .map(|_| ())
            .map_err(|e| MediaGatewayError::transport(e.to_string()))
    }

    async fn close_room(&self, room: RoomId) -> Result<(), MediaGatewayError> {
        let mut client = self.client.clone();
        let request = DeleteRoomRequest {
            room_id: room.to_string(),
        };
        client
            .delete_room(request)
            .await
            .map(|_| ())
            .map_err(|e| MediaGatewayError::transport(e.to_string()))
    }
}
