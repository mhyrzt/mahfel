use crate::http::error::api_error_from;
use mahfel_application::sessions::SessionOperationError;

api_error_from!(SessionOperationError);
