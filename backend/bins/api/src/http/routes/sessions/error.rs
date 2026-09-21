use crate::http::error::api_error_from;
use mahfel_application::sessions::{
    CancelSessionError, EndSessionError, GetSessionError, ListSessionsError, ScheduleSessionError,
    StartSessionError,
};

api_error_from!(
    ScheduleSessionError,
    ListSessionsError,
    GetSessionError,
    StartSessionError,
    EndSessionError,
    CancelSessionError,
);
