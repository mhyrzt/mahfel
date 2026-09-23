use crate::http::error::api_error_from;
use mahfel_application::{
    admission::AdmissionOperationError, chat::ChatOperationError, rooms::RoomOperationError,
};

api_error_from!(
    RoomOperationError,
    AdmissionOperationError,
    ChatOperationError
);
