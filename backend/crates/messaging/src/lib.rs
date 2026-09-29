//! AMQP transport for durable background jobs.

mod jobs;

pub use jobs::{
    ConfirmedPublisher, JobConsumer, JobDelivery, MessagingError, connect, declare_topology,
};

pub const JOB_EXCHANGE: &str = "mahfel.jobs.v1";
pub const FILE_QUEUE: &str = "mahfel.jobs.files.cleanup.v1";
pub const DEAD_EXCHANGE: &str = "mahfel.jobs.dead.v1";
pub const DEAD_QUEUE: &str = "mahfel.jobs.files.cleanup.v1.dead";
pub const FILE_ROUTING_KEY: &str = "files.cleanup_deleted_attachment.v1";

pub const RECORDING_QUEUE: &str = "mahfel.jobs.recordings.finalize.v1";
pub const RECORDING_DEAD_QUEUE: &str = "mahfel.jobs.recordings.finalize.v1.dead";
pub const RECORDING_ROUTING_KEY: &str = "recordings.finalize.v1";
