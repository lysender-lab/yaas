use std::time::Duration;

use snafu::ResultExt;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{info, warn};

use crate::{Result, error::CronSnafu, run::AppState};

const CLEANUP_INTERVAL: Duration = Duration::from_secs(6 * 60 * 60);
const JOB_RETENTION_SECONDS: i64 = 7 * 24 * 60 * 60;

pub async fn start(state: AppState) -> Result<JobScheduler> {
    let scheduler = JobScheduler::new().await.context(CronSnafu)?;
    let cleanup_state = state.clone();

    // Test job to demonstrate cron job capability
    let test_job = Job::new_repeated_async(Duration::from_mins(1), move |_id, _scheduler| {
        Box::pin(log_cron_handler(state.clone()))
    })
    .context(CronSnafu)?;
    scheduler.add(test_job).await.context(CronSnafu)?;

    let cleanup_job = Job::new_repeated_async(CLEANUP_INTERVAL, move |_id, _scheduler| {
        Box::pin(cleanup_finished_jobs_handler(cleanup_state.clone()))
    })
    .context(CronSnafu)?;
    scheduler.add(cleanup_job).await.context(CronSnafu)?;

    scheduler.start().await.context(CronSnafu)?;
    Ok(scheduler)
}

async fn log_cron_handler(_state: AppState) {
    info!("hello from cron");
}

fn cleanup_cutoff(now: i64) -> i64 {
    now.saturating_sub(JOB_RETENTION_SECONDS)
}

async fn cleanup_finished_jobs_handler(state: AppState) {
    let cutoff = cleanup_cutoff(chrono::Utc::now().timestamp());
    match state.db.jobs.delete_finished_before(cutoff).await {
        Ok(deleted) => info!(deleted, "Finished job cleanup completed"),
        Err(error) => warn!(%error, "Unable to clean up finished jobs"),
    }
}

pub async fn shutdown(mut scheduler: JobScheduler) -> Result<()> {
    scheduler.shutdown().await.context(CronSnafu)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::NewJobDto;
    use crate::test::TestCtx;
    use turso::Value;

    #[test]
    fn cleanup_cutoff_is_seven_days_before_now() {
        assert_eq!(cleanup_cutoff(1_000_000), 1_000_000 - 7 * 24 * 60 * 60);
        assert_eq!(cleanup_cutoff(i64::MIN), i64::MIN);
    }

    #[tokio::test]
    async fn cleanup_handler_removes_only_old_finished_jobs() {
        let ctx = TestCtx::new("cron-cleanup").await.unwrap();
        let repo = &ctx.state.db.jobs;
        let now = chrono::Utc::now().timestamp();
        let old = repo
            .enqueue(NewJobDto {
                job_type: "test".into(),
                payload: "{}".into(),
                max_attempts: 1,
                available_at: now,
            })
            .await
            .unwrap();
        let pending = repo
            .enqueue(NewJobDto {
                job_type: "test".into(),
                payload: "{}".into(),
                max_attempts: 1,
                available_at: now,
            })
            .await
            .unwrap();
        ctx.execute_sql(
            "UPDATE jobs SET status = 'completed', updated_at = :updated_at WHERE id = :id",
            vec![
                (
                    ":updated_at".into(),
                    Value::Integer(cleanup_cutoff(now) - 10),
                ),
                (":id".into(), Value::Text(old.id.clone())),
            ],
        )
        .await
        .unwrap();
        cleanup_finished_jobs_handler(ctx.state.clone()).await;
        assert!(repo.get(&old.id).await.unwrap().is_none());
        assert!(repo.get(&pending.id).await.unwrap().is_some());
    }
}
