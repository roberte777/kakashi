CREATE TABLE IF NOT EXISTS launchables (
  id TEXT PRIMARY KEY
  last_launched_at DATETIME
);

CREATE TABLE IF NOT EXISTS launches (
  id            INTEGER PRIMARY KEY,
  launchable_id TEXT NOT NULL REFERENCES launchables(id),
  launched_at   DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
