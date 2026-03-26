use std::error::Error as StdError;

use async_trait::async_trait;
use frecency::{FrecencyDataProvider, FrecencyScorer, LaunchEvent};

use crate::db::DbConn;

// TODO: Consider fetching a list of scores, would help reducing number of sql queries
/// Tracks usage, used to inform score
#[async_trait]
pub trait UsageTracker {
    async fn record_launch(&mut self, id: &str) -> Result<(), Box<dyn StdError>>;
    async fn frecency_score(&self, id: &str) -> f64;
}

/// Used to perform Frecency operations
pub struct FrecencyTracker {
    db: DbConn,
}

impl FrecencyTracker {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &DbConn {
        &self.db
    }
}

#[async_trait]
impl UsageTracker for FrecencyTracker {
    async fn record_launch(&mut self, id: &str) -> Result<(), Box<dyn StdError>> {
        Ok(self.db.record_launch(id).await?)
    }

    async fn frecency_score(&self, id: &str) -> f64 {
        FrecencyScorer::score(self, id).await
    }
}

#[async_trait]
impl FrecencyDataProvider for FrecencyTracker {
    async fn get_launches(&self, id: &str) -> Vec<LaunchEvent> {
        match self.db.get_launches(id).await {
            Ok(timestamps) => timestamps
                .into_iter()
                .map(|timestamp| LaunchEvent { timestamp })
                .collect(),
            Err(e) => {
                eprintln!("Failed to get launches for {}: {}", id, e);
                Vec::new()
            }
        }
    }
}
