use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TargetKind {
    Web,
    Epub,
    Markdown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ReadingPosition {
    Web { scroll: usize },
    Epub { chapter: usize, scroll: usize },
    Markdown { scroll: usize },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub kind: TargetKind,
    pub title: String,
    pub target: String,
    pub position: ReadingPosition,
    pub progress_percent: i16,
    pub updated_at: String,
    pub starred: bool,
    pub hidden: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct HistoryFile {
    pub entries: Vec<HistoryEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetIdentity {
    pub id: String,
    pub identity_source: String,
    pub display_target: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HistoryTarget {
    Web { url: String, title: String },
    Epub { path: PathBuf, title: String },
    Markdown { path: PathBuf, title: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistorySession {
    pub entry_id: String,
    pub opened_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryRecovery {
    pub recoverable: bool,
    pub backup_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedHistory {
    pub history: HistoryFile,
    pub recovery: Option<HistoryRecovery>,
}

#[derive(Debug, Clone)]
pub struct HistoryStore {
    path: PathBuf,
}

impl TargetIdentity {
    pub fn for_web_url(url: &str) -> Self {
        Self {
            id: format!("web:{}", crate::web::web_cache_key(url)),
            identity_source: url.to_string(),
            display_target: url.to_string(),
        }
    }

    pub fn for_local_path(path: &Path, kind: TargetKind, cwd: &Path) -> Result<Self> {
        let resolved = if path.is_absolute() {
            path.to_path_buf()
        } else {
            cwd.join(path)
        };
        let identity_path = resolved.canonicalize().unwrap_or(resolved);
        Ok(Self {
            id: Self::local_id(kind, &identity_path),
            identity_source: identity_path.to_string_lossy().to_string(),
            display_target: path.to_string_lossy().to_string(),
        })
    }

    pub fn local_id(kind: TargetKind, path: &Path) -> String {
        let prefix = match kind {
            TargetKind::Web => "web",
            TargetKind::Epub => "epub",
            TargetKind::Markdown => "markdown",
        };
        let hash = Sha256::digest(path.to_string_lossy().as_bytes());
        format!("{}:{}", prefix, &format!("{:x}", hash)[..16])
    }
}

impl HistoryTarget {
    fn kind(&self) -> TargetKind {
        match self {
            Self::Web { .. } => TargetKind::Web,
            Self::Epub { .. } => TargetKind::Epub,
            Self::Markdown { .. } => TargetKind::Markdown,
        }
    }

    fn title(&self) -> &str {
        match self {
            Self::Web { title, .. } | Self::Epub { title, .. } | Self::Markdown { title, .. } => {
                title
            }
        }
    }

    fn identity(&self) -> Result<TargetIdentity> {
        match self {
            Self::Web { url, .. } => Ok(TargetIdentity::for_web_url(url)),
            Self::Epub { path, .. } => TargetIdentity::for_local_path(
                path,
                TargetKind::Epub,
                &std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            ),
            Self::Markdown { path, .. } => TargetIdentity::for_local_path(
                path,
                TargetKind::Markdown,
                &std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            ),
        }
    }
}

impl HistoryFile {
    pub fn entry(&self, id: &str) -> Option<&HistoryEntry> {
        self.entries.iter().find(|entry| entry.id == id)
    }

    pub fn entry_mut(&mut self, id: &str) -> Option<&mut HistoryEntry> {
        self.entries.iter_mut().find(|entry| entry.id == id)
    }

    pub fn record_successful_open(
        &mut self,
        target: HistoryTarget,
        position: ReadingPosition,
        progress_percent: i16,
        updated_at: &str,
    ) -> HistorySession {
        let identity = target
            .identity()
            .expect("target identity should be derivable");
        let id = identity.id;
        let percent = clamp_percent(progress_percent);
        let kind = target.kind();
        let title = target.title().to_string();
        let stored_target = match kind {
            TargetKind::Web => identity.display_target,
            TargetKind::Epub | TargetKind::Markdown => identity.identity_source,
        };

        if let Some(entry) = self.entry_mut(&id) {
            let was_hidden = entry.hidden;
            entry.kind = kind;
            entry.title = title;
            entry.target = stored_target;
            entry.position = position;
            entry.progress_percent = percent;
            entry.updated_at = updated_at.to_string();
            entry.hidden = false;
            if was_hidden {
                entry.starred = false;
            }
        } else {
            self.entries.push(HistoryEntry {
                id: id.clone(),
                kind,
                title,
                target: stored_target,
                position,
                progress_percent: percent,
                updated_at: updated_at.to_string(),
                starred: false,
                hidden: false,
            });
        }

        HistorySession {
            entry_id: id,
            opened_at: updated_at.to_string(),
        }
    }

    pub fn update_progress_from_session(
        &mut self,
        session: &HistorySession,
        position: ReadingPosition,
        progress_percent: i16,
        updated_at: &str,
    ) -> Result<()> {
        if let Some(entry) = self.entry_mut(&session.entry_id) {
            entry.position = position;
            entry.progress_percent = clamp_percent(progress_percent);
            entry.updated_at = updated_at.to_string();
        }
        Ok(())
    }

    pub fn set_starred(&mut self, id: &str, starred: bool, updated_at: &str) -> Result<()> {
        if let Some(entry) = self.entry_mut(id) {
            entry.starred = starred;
            entry.updated_at = updated_at.to_string();
        }
        Ok(())
    }

    pub fn set_hidden(&mut self, id: &str, hidden: bool, updated_at: &str) -> Result<()> {
        if let Some(entry) = self.entry_mut(id) {
            entry.hidden = hidden;
            entry.updated_at = updated_at.to_string();
        }
        Ok(())
    }
}

impl HistoryStore {
    pub fn at(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn default() -> Self {
        let path = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".tread")
            .join("history.json");
        Self::at(path)
    }

    pub fn load(&self) -> Result<LoadedHistory> {
        if !self.path.exists() {
            return Ok(LoadedHistory {
                history: HistoryFile::default(),
                recovery: None,
            });
        }

        let content = fs::read_to_string(&self.path)?;
        match serde_json::from_str::<HistoryFile>(&content) {
            Ok(history) => Ok(LoadedHistory {
                history,
                recovery: None,
            }),
            Err(_) => {
                let backup_path = self.backup_path();
                fs::rename(&self.path, &backup_path)?;
                Ok(LoadedHistory {
                    history: HistoryFile::default(),
                    recovery: Some(HistoryRecovery {
                        recoverable: true,
                        backup_path,
                    }),
                })
            }
        }
    }

    pub fn save(&self, history: &HistoryFile) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let tmp = self
            .path
            .with_extension(format!("json.tmp.{}", std::process::id()));
        fs::write(&tmp, serde_json::to_string_pretty(history)?)?;
        fs::rename(&tmp, &self.path)?;
        Ok(())
    }

    pub fn upsert_open(
        &self,
        target: HistoryTarget,
        position: ReadingPosition,
        progress_percent: i16,
        updated_at: &str,
    ) -> Result<HistorySession> {
        let mut loaded = self.load()?.history;
        let session = loaded.record_successful_open(target, position, progress_percent, updated_at);
        self.save(&loaded)?;
        Ok(session)
    }

    pub fn update_progress(
        &self,
        session: &HistorySession,
        position: ReadingPosition,
        progress_percent: i16,
        updated_at: &str,
    ) -> Result<()> {
        let mut loaded = self.load()?.history;
        loaded.update_progress_from_session(session, position, progress_percent, updated_at)?;
        self.save(&loaded)
    }

    fn backup_path(&self) -> PathBuf {
        let stamp = timestamp_compact();
        self.path
            .with_file_name(format!("history.json.bak.{stamp}"))
    }
}

pub fn now_timestamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    secs.to_string()
}

fn timestamp_compact() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("{}-{}", millis, std::process::id())
}

fn clamp_percent(value: i16) -> i16 {
    value.clamp(0, 100)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};

    static CTR: AtomicUsize = AtomicUsize::new(0);

    fn temp_dir(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "tread_history_test_{}_{}_{}",
            name,
            std::process::id(),
            CTR.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn history_path(root: &Path) -> PathBuf {
        root.join(".tread").join("history.json")
    }

    fn web_target(url: &str, title: &str) -> HistoryTarget {
        HistoryTarget::Web {
            url: url.to_string(),
            title: title.to_string(),
        }
    }

    fn entry(id: String, kind: TargetKind, title: &str, target: &str) -> HistoryEntry {
        HistoryEntry {
            id,
            kind,
            title: title.to_string(),
            target: target.to_string(),
            position: ReadingPosition::Web { scroll: 0 },
            progress_percent: 0,
            updated_at: "2026-07-09T00:00:00Z".to_string(),
            starred: false,
            hidden: false,
        }
    }

    #[test]
    fn missing_history_file_loads_empty_without_recovery_warning() {
        let root = temp_dir("missing");
        let store = HistoryStore::at(history_path(&root));

        let loaded = store.load().unwrap();

        assert!(loaded.history.entries.is_empty());
        assert!(loaded.recovery.is_none());
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn malformed_history_file_is_backed_up_and_loads_recoverable_empty_history() {
        let root = temp_dir("malformed");
        let path = history_path(&root);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, b"{ this is not valid history json").unwrap();

        let loaded = HistoryStore::at(path.clone()).load().unwrap();

        assert!(loaded.history.entries.is_empty());
        let recovery = loaded
            .recovery
            .expect("malformed file should produce a recovery warning");
        assert!(recovery.recoverable);
        assert!(recovery.backup_path.exists());
        assert_eq!(
            recovery
                .backup_path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("history.json.bak."),
            true
        );
        assert_eq!(
            fs::read_to_string(recovery.backup_path).unwrap(),
            "{ this is not valid history json"
        );
        assert!(
            !path.exists(),
            "malformed history should be moved aside before replacement"
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn saving_history_atomically_leaves_parseable_json_at_history_path() {
        let root = temp_dir("save_parseable");
        let path = history_path(&root);
        let store = HistoryStore::at(path.clone());
        let url = "https://example.com/read";
        let id = TargetIdentity::for_web_url(url).id;
        let history = HistoryFile {
            entries: vec![entry(id, TargetKind::Web, "Readable Article", url)],
        };

        store.save(&history).unwrap();

        let contents = fs::read_to_string(&path).unwrap();
        let parsed: HistoryFile = serde_json::from_str(&contents).unwrap();
        assert_eq!(parsed.entries.len(), 1);
        assert_eq!(parsed.entries[0].title, "Readable Article");
        let leftovers: Vec<_> = fs::read_dir(path.parent().unwrap())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().contains(".tmp"))
            .collect();
        assert!(
            leftovers.is_empty(),
            "successful atomic save should not leave temp files"
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn web_identity_uses_same_url_string_key_as_web_progress_and_cache() {
        let url = "https://example.com/read?chapter=1#frag";
        let identity = TargetIdentity::for_web_url(url);

        assert_eq!(
            identity.id,
            format!("web:{}", crate::web::web_cache_key(url))
        );
        assert_eq!(identity.identity_source, url);
        assert_eq!(identity.display_target, url);
    }

    #[test]
    fn reopening_same_web_url_updates_the_existing_history_entry() {
        let root = temp_dir("web_reopen");
        let mut history = HistoryFile {
            entries: Vec::new(),
        };
        let url = "https://example.com/same-url";

        history.record_successful_open(
            web_target(url, "Original Title"),
            ReadingPosition::Web { scroll: 12 },
            10,
            "2026-07-09T00:00:00Z",
        );
        history.record_successful_open(
            web_target(url, "Updated Title"),
            ReadingPosition::Web { scroll: 34 },
            20,
            "2026-07-09T00:01:00Z",
        );

        assert_eq!(history.entries.len(), 1);
        assert_eq!(history.entries[0].title, "Updated Title");
        assert_eq!(
            history.entries[0].position,
            ReadingPosition::Web { scroll: 34 }
        );
        assert_eq!(history.entries[0].progress_percent, 20);
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn local_file_identity_uses_canonical_absolute_path_for_existing_files() {
        let root = temp_dir("canonical");
        let books = root.join("books");
        fs::create_dir_all(&books).unwrap();
        let file = books.join("novel.md");
        fs::write(&file, "# Novel").unwrap();
        let relative_with_dot = books.join(".").join("novel.md");

        let identity =
            TargetIdentity::for_local_path(&relative_with_dot, TargetKind::Markdown, &root)
                .unwrap();
        let canonical = file.canonicalize().unwrap();

        assert_eq!(
            identity.id,
            TargetIdentity::local_id(TargetKind::Markdown, &canonical)
        );
        assert_eq!(identity.identity_source, canonical.to_string_lossy());
        assert_eq!(identity.display_target, relative_with_dot.to_string_lossy());
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn local_file_identity_falls_back_to_resolved_absolute_path_when_canonicalization_fails() {
        let root = temp_dir("fallback_absolute");
        let missing_relative = PathBuf::from("drafts/missing.md");
        let resolved_absolute = root.join(&missing_relative);

        let identity =
            TargetIdentity::for_local_path(&missing_relative, TargetKind::Markdown, &root).unwrap();

        assert_eq!(
            identity.id,
            TargetIdentity::local_id(TargetKind::Markdown, &resolved_absolute)
        );
        assert_eq!(
            identity.identity_source,
            resolved_absolute.to_string_lossy()
        );
        assert_eq!(identity.display_target, missing_relative.to_string_lossy());
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn reopening_hidden_target_makes_it_visible_and_unstarred() {
        let url = "https://example.com/hidden";
        let id = TargetIdentity::for_web_url(url).id;
        let mut history = HistoryFile {
            entries: vec![HistoryEntry {
                hidden: true,
                starred: true,
                ..entry(id, TargetKind::Web, "Hidden", url)
            }],
        };

        history.record_successful_open(
            web_target(url, "Hidden"),
            ReadingPosition::Web { scroll: 0 },
            0,
            "2026-07-09T00:02:00Z",
        );

        assert_eq!(history.entries.len(), 1);
        assert!(!history.entries[0].hidden);
        assert!(!history.entries[0].starred);
    }

    #[test]
    fn reopening_starred_target_preserves_visible_star_but_clears_hidden_star() {
        let visible_url = "https://example.com/visible-starred";
        let hidden_url = "https://example.com/hidden-starred";
        let visible_id = TargetIdentity::for_web_url(visible_url).id;
        let hidden_id = TargetIdentity::for_web_url(hidden_url).id;
        let mut history = HistoryFile {
            entries: vec![
                HistoryEntry {
                    hidden: false,
                    starred: true,
                    ..entry(visible_id.clone(), TargetKind::Web, "Visible", visible_url)
                },
                HistoryEntry {
                    hidden: true,
                    starred: true,
                    ..entry(hidden_id.clone(), TargetKind::Web, "Hidden", hidden_url)
                },
            ],
        };

        history.record_successful_open(
            web_target(visible_url, "Visible reopened"),
            ReadingPosition::Web { scroll: 10 },
            10,
            "2026-07-09T00:02:00Z",
        );
        history.record_successful_open(
            web_target(hidden_url, "Hidden reopened"),
            ReadingPosition::Web { scroll: 20 },
            20,
            "2026-07-09T00:03:00Z",
        );

        let visible = history.entry(&visible_id).unwrap();
        assert!(!visible.hidden);
        assert!(visible.starred);

        let hidden = history.entry(&hidden_id).unwrap();
        assert!(!hidden.hidden);
        assert!(!hidden.starred);
    }

    #[test]
    fn progress_percent_is_clamped_before_storage() {
        let mut history = HistoryFile {
            entries: Vec::new(),
        };

        history.record_successful_open(
            web_target("https://example.com/negative", "Negative"),
            ReadingPosition::Web { scroll: 0 },
            -25,
            "2026-07-09T00:00:00Z",
        );
        history.record_successful_open(
            web_target("https://example.com/in-range", "In Range"),
            ReadingPosition::Web { scroll: 50 },
            42,
            "2026-07-09T00:00:01Z",
        );
        history.record_successful_open(
            web_target("https://example.com/too-high", "Too High"),
            ReadingPosition::Web { scroll: 100 },
            150,
            "2026-07-09T00:00:02Z",
        );

        let percents: Vec<i16> = history
            .entries
            .iter()
            .map(|entry| entry.progress_percent)
            .collect();
        assert_eq!(percents, vec![0, 42, 100]);
    }

    #[test]
    fn progress_update_preserves_newer_starred_and_hidden_changes() {
        let url = "https://example.com/concurrent";
        let mut history = HistoryFile {
            entries: Vec::new(),
        };
        let session = history.record_successful_open(
            web_target(url, "Concurrent"),
            ReadingPosition::Web { scroll: 10 },
            10,
            "2026-07-09T00:00:00Z",
        );

        history
            .set_starred(&session.entry_id, true, "2026-07-09T00:01:00Z")
            .unwrap();
        history
            .set_hidden(&session.entry_id, true, "2026-07-09T00:02:00Z")
            .unwrap();
        history
            .update_progress_from_session(
                &session,
                ReadingPosition::Web { scroll: 99 },
                90,
                "2026-07-09T00:03:00Z",
            )
            .unwrap();

        let entry = history.entry(&session.entry_id).unwrap();
        assert_eq!(entry.position, ReadingPosition::Web { scroll: 99 });
        assert_eq!(entry.progress_percent, 90);
        assert!(
            entry.starred,
            "newer dashboard star change must survive reader exit"
        );
        assert!(
            entry.hidden,
            "newer dashboard remove change must survive reader exit"
        );
    }

    #[test]
    fn epub_and_markdown_file_identities_do_not_collide_for_same_path() {
        let root = temp_dir("kind_prefix");
        let file = root.join("book.epub");
        fs::write(&file, b"not a real epub; identity only").unwrap();

        let epub = TargetIdentity::for_local_path(&file, TargetKind::Epub, &root).unwrap();
        let markdown = TargetIdentity::for_local_path(&file, TargetKind::Markdown, &root).unwrap();

        assert_ne!(epub.id, markdown.id);
        assert!(epub.id.starts_with("epub:"));
        assert!(markdown.id.starts_with("markdown:"));
        fs::remove_dir_all(root).ok();
    }
}
