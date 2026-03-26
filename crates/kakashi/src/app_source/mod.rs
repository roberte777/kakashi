use std::{collections::HashMap, error::Error, path::PathBuf, str::FromStr as _};

use freedesktop_file_parser::{DesktopFile, parse};

/// Generic entry format that supports multiple sources
#[derive(Debug, Clone)]
pub struct Entry {
    /// Unique identifier for this entry
    pub id: String,
    /// Human readable name to display
    pub name: String,
    /// Optional icon
    pub icon: Option<Icon>,
    /// The action to perform when launched
    pub action: LaunchAction,
    /// Source this entry came from
    pub source: SourceType,
}

#[derive(Debug, Clone)]
pub enum Icon {
    // TODO: check out freedesktop_icons to see if we can use that to get icon
    // from theme
    /// Icon name from theme
    Name(String),
    /// Absolute path to icon file
    Path(PathBuf),
}

#[derive(Debug, Clone)]
pub enum LaunchAction {
    /// Execute a command
    Command { exec: String, terminal: bool },
}

#[derive(Debug, Clone)]
pub enum SourceType {
    DesktopFile,
}

// TODO: Impl should use nucleo_matcher, should take many entries instead of 1
// Provides a score for a user query
pub trait Matcher {
    fn score(&self, query: &str, entry: &Entry) -> f64;
}

// Launches an application
pub trait Launcher {
    fn launch(&self, entry: &Entry) -> Result<(), Box<dyn Error>>;
}

// Sources the applications
pub trait AppSource {
    fn scan(&self) -> Vec<Entry>;
}

// Places to search for .desktop files, in order of least priority to highest
pub const SEARCH_LOCATIONS: [&str; 5] = [
    "/var/lib/flatpak/exports/share/applications",
    "~/.local/share/flatpak/exports/share/applications",
    "/usr/share/applications",
    "/usr/local/share/applications",
    "~/.local/share/applications",
];

pub struct DesktopEntry {
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
fn parse_paths(potential_entries: Vec<PotentialDesktopEntry>) -> Vec<DesktopEntry> {
    let mut entries = Vec::default();

    for entry in potential_entries {
        let Ok(content) = std::fs::read_to_string(&entry.path) else {
            continue;
        };

        let Ok(desktop_file) = parse(&content) else {
            continue;
        };
        let entry = DesktopEntry {
            id: entry.id,
            file: desktop_file,
        };
        entries.push(entry);
    }
    entries
}

/// Parses common linux desktop search locations in priority order. Returns the
/// parsed [`DesktopFile`] objects.
pub fn create_entries() -> Vec<DesktopEntry> {
    let files = desktop_files();
    parse_paths(files)
}
