use std::{collections::HashMap, error::Error, path::PathBuf, str::FromStr as _};

use freedesktop_file_parser::{DesktopFile, parse};

// Sources the applications
pub trait AppSource {
    fn scan(&self) -> Vec<DesktopFile>;
}

// Provides a score for a user query
pub trait Matcher {
    fn score(&self, query: &str, entry: &DesktopFile) -> f64;
}

// Launches an application
pub trait Launcher {
    fn launch(&self, entry: &DesktopFile) -> Result<(), Box<dyn Error>>;
}

// Tracks usage, used to inform score
pub trait UsageTracker {
    fn record_launch(&mut self, id: &str);
    fn frecency_score(&self, id: &str) -> f64;
}

// Places to search for .desktop files, in order of least priority to highest
pub const SEARCH_LOCATIONS: [&str; 5] = [
    "/var/lib/flatpak/exports/share/applications",
    "~/.local/share/flatpak/exports/share/applications",
    "/usr/share/applications",
    "/usr/local/share/applications",
    "~/.local/share/applications",
];

pub struct Entry {
    id: String,
    file: DesktopFile,
}

pub struct PotentialDesktopEntry {
    id: String,
    path: PathBuf,
}

fn desktop_files() -> Vec<PotentialDesktopEntry> {
    let mut seen = HashMap::<String, PathBuf>::new();

    for location in SEARCH_LOCATIONS {
        let dir = PathBuf::from_str(location).unwrap();
        if !dir.is_dir() {
            continue;
        }

        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };

        for file in entries.flatten() {
            let path = file.path();

            // if not a file or the extension is not desktop
            if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("desktop") {
                continue;
            }

            let Some(id) = path.file_stem().and_then(|n| n.to_str()).map(String::from) else {
                continue;
            };

            // higher priority directories are later in the list, so this
            // naturally overwrites any earlier entry with the same filename.
            seen.insert(id, path);
        }
    }

    seen.into_iter()
        .map(|(k, v)| PotentialDesktopEntry { id: k, path: v })
        .collect()
}

// TODO: Replace the Entries returned here with our own type
fn parse_paths(potential_entries: Vec<PotentialDesktopEntry>) -> Vec<Entry> {
    let mut entries = Vec::default();

    for entry in potential_entries {
        let Ok(content) = std::fs::read_to_string(&entry.path) else {
            continue;
        };

        let Ok(desktop_file) = parse(&content) else {
            continue;
        };
        let entry = Entry {
            id: entry.id,
            file: desktop_file,
        };
        entries.push(entry);
    }
    entries
}

/// Parses common linux desktop search locations in priority order. Returns the
/// parsed [`DesktopFile`] objects.
pub fn create_entries() -> Vec<Entry> {
    let files = desktop_files();
    parse_paths(files)
}
