use std::time::{Duration, SystemTime};

use async_trait::async_trait;

/// Represents a single launch event
#[derive(Debug, Clone)]
pub struct LaunchEvent {
    pub timestamp: SystemTime,
}

/// Data provider trait that the frecency algorithm needs
#[async_trait]
pub trait FrecencyDataProvider {
    /// Get all launch events for a given item ID
    async fn get_launches(&self, id: &str) -> Vec<LaunchEvent>;
}

pub struct FrecencyScorer;

impl FrecencyScorer {
    /// Calculate frecency score for an item based on its launch history
    ///
    /// Algorithm:
    /// - Recent launches are weighted more heavily
    /// - Score decays exponentially with time
    /// - Uses time buckets: last 4 hours, day, 3 days, week, month, 90 days
    pub async fn score<P: FrecencyDataProvider>(provider: &P, id: &str) -> f64 {
        let launches = provider.get_launches(id).await;

        if launches.is_empty() {
            return 0.0;
        }

        let now = SystemTime::now();
        let mut score = 0.0;

        for launch in launches {
            let Ok(elapsed) = now.duration_since(launch.timestamp) else {
                // currently skipping future timestamps (could happen if clock changes)
                continue;
            };

            score += Self::time_weight(elapsed);
        }

        score
    }

    /// Weight function based on how long ago the launch was
    /// More recent = higher weight
    fn time_weight(elapsed: Duration) -> f64 {
        let hours = elapsed.as_secs_f64() / 3600.0;

        match hours {
            h if h < 4.0 => 100.0,  // Last 4 hours
            h if h < 24.0 => 70.0,  // Last day
            h if h < 72.0 => 50.0,  // Last 3 days
            h if h < 168.0 => 30.0, // Last week
            h if h < 720.0 => 10.0, // Last month (30 days)
            _ => 5.0,               // Older (up to 90 days, then pruned)
        }
    }
}
