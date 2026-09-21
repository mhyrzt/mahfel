use crate::http::error::api_error_from;
use mahfel_application::rooms::RoomOperationError;

api_error_from!(RoomOperationError);
