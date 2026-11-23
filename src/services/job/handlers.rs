use serde::{Deserialize, Serialize};
use tracing::info;

use crate::dto::JobDto;
use crate::run::AppState;

pub(super) const LOGGER_JOB_TYPE: &str = "logger_job_handler";

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LoggerJobPayload {
    pub message: String,
}

async fn logger_job_handler(message: &str) {
    info!("{}", message);
}

pub(super) async fn dispatch(state: &AppState, job: &JobDto) -> std::result::Result<(), String> {
    match job.job_type.as_str() {
        LOGGER_JOB_TYPE => {
            let payload: LoggerJobPayload = serde_json::from_str(&job.payload)
                .map_err(|_| "Invalid logger job payload".to_string())?;
            logger_job_handler(&payload.message).await;
            Ok(())
        }
        _ => Err("Unknown job type".to_string()),
    }
}
