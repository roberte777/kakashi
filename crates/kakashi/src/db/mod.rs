use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Context;
use turso::{params, Builder, Connection};

pub struct DbConn {
    inner: Connection,
}

impl DbConn {
    pub async fn new() -> anyhow::Result<Self> {
        let db_path = db_path();
        setup_db_folder(&db_path).context("Should be able to make the database folder")?;
        let db = Builder::new_local(
            db_path
                .to_str()
                .context("Should be able to make a string out of the database path")?,
        )
        .build()
        .await?;
        let conn = db.connect()?;
        let mut conn = Self { inner: conn };
        conn.init_schema().await?;
        conn.prune().await?;
        Ok(conn)
    }

    /// Puts the schema in the db
    async fn init_schema(&self) -> anyhow::Result<()> {
        self.inner
            .execute_batch(include_str!("../../schema.sql"))
            .await?;
        Ok(())
    }

    /// Prunes entries older than 90 days 1 out of 100 times this is ran.
    ///
    /// In order to avoid hitting the database every time we use the application,
    /// randomly prune 1 out of 100 times. This may be a bit overkill :/
    async fn prune(&mut self) -> anyhow::Result<()> {
        if rand::random::<f32>() < 0.01 {
            self.inner
                .execute(
                    "DELETE FROM launches WHERE launched_at < datetime('now', '-90 days')",
                    (),
                )
                .await?;
        }
        Ok(())
    }

    /// Record a launch event for an item
    pub async fn record_launch(&self, id: &str) -> anyhow::Result<()> {
        // Insert or update the launchables table
        self.inner
            .execute(
                "INSERT INTO launchables (id, last_launched_at)
                 VALUES (?, CURRENT_TIMESTAMP)
                 ON CONFLICT(id) DO UPDATE SET last_launched_at = CURRENT_TIMESTAMP",
                params!(id),
            )
            .await?;

        // Insert into launches table
        self.inner
            .execute(
                "INSERT INTO launches (launchable_id, launched_at) VALUES (?, CURRENT_TIMESTAMP)",
                params!(id),
            )
            .await?;

        Ok(())
    }

    /// Get all launch timestamps for a given item ID
    pub async fn get_launches(&self, id: &str) -> anyhow::Result<Vec<SystemTime>> {
        let mut stmt = self
            .inner
            .prepare("SELECT launched_at FROM launches WHERE launchable_id = ? ORDER BY launched_at DESC")
            .await?;

        let mut rows = stmt.query(params!(id)).await?;

        let mut launches = Vec::new();
        while let Some(row) = rows.next().await? {
            let timestamp_str: String = row.get(0)?;
            if let Ok(timestamp) = parse_sqlite_timestamp(&timestamp_str) {
                launches.push(timestamp);
            }
        }

        Ok(launches)
    }
}

/// Parse SQLite datetime string to SystemTime
fn parse_sqlite_timestamp(s: &str) -> anyhow::Result<SystemTime> {
    // SQLite CURRENT_TIMESTAMP format: "YYYY-MM-DD HH:MM:SS"
    let dt = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
        .context("Failed to parse timestamp")?;

    let seconds = dt.and_utc().timestamp();
    let nanos = dt.and_utc().timestamp_subsec_nanos();

    Ok(UNIX_EPOCH + Duration::from_secs(seconds as u64) + Duration::from_nanos(nanos as u64))
}

fn db_path() -> PathBuf {
    let data_dir = std::env::var("XDG_DATA_HOME").unwrap_or_else(|_| {
        format!(
            "{}/.local/share",
            std::env::var("HOME").expect("HOME not set")
        )
    });

    std::path::PathBuf::from(data_dir)
        .join("kakashi")
        .join("frecency.db")
}

fn setup_db_folder<P>(db_path: P) -> anyhow::Result<()>
where
    P: AsRef<Path>,
{
    std::fs::create_dir_all(db_path.as_ref().parent().unwrap())?;
    Ok(())
}
