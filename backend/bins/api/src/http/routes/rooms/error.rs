use crate::http::error::api_error_from;
use mahfel_application::{admission::AdmissionOperationError, rooms::RoomOperationError};

api_error_from!(RoomOperationError, AdmissionOperationError);
