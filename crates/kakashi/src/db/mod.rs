use std::path::{Path, PathBuf};

use anyhow::Context;
use turso::{Builder, Connection};

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
