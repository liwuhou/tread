use crossterm::event::{KeyCode, KeyEvent};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetKind {
    Web,
    Epub,
    Markdown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashboardEntry {
    pub id: String,
    pub kind: TargetKind,
    pub title: String,
    pub target: String,
    pub progress_percent: i16,
    pub updated_at: String,
    pub starred: bool,
    pub hidden: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashboardRow {
    pub entry_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathSuggestion {
    pub path: PathBuf,
    pub display: String,
    pub is_dir: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptTarget {
    Url(String),
    LocalPath(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptErrorKind {
    MissingPath,
    Io,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptError {
    pub kind: PromptErrorKind,
    pub recoverable: bool,
    pub message: String,
}

pub struct DashboardState {
    entries: Vec<DashboardEntry>,
    selected: usize,
}

impl DashboardState {
    pub fn from_history(entries: Vec<DashboardEntry>) -> Self {
        Self {
            entries,
            selected: 0,
        }
    }

    pub fn starred_rows(&self) -> Vec<DashboardRow> {
        self.visible_entries()
            .into_iter()
            .filter(|entry| entry.starred)
            .map(row_for)
            .collect()
    }

    pub fn recent_rows(&self) -> Vec<DashboardRow> {
        let mut entries: Vec<&DashboardEntry> = self
            .visible_entries()
            .into_iter()
            .filter(|entry| !entry.starred)
            .collect();
        entries.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        entries.into_iter().map(row_for).collect()
    }

    pub fn visible_rows(&self) -> Vec<DashboardRow> {
        let mut rows = self.starred_rows();
        rows.extend(self.recent_rows());
        rows
    }

    pub fn entry(&self, id: &str) -> Option<&DashboardEntry> {
        self.entries.iter().find(|entry| entry.id == id)
    }

    pub fn selected_entry_id(&self) -> Option<&str> {
        let row = self.visible_rows().get(self.selected)?.entry_id.clone();
        self.entries
            .iter()
            .find(|entry| entry.id == row)
            .map(|entry| entry.id.as_str())
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self.move_selection(1),
            KeyCode::Char('k') | KeyCode::Up => self.move_selection(-1),
            KeyCode::Char('s') => self.toggle_selected_star(),
            KeyCode::Char('r') => self.hide_selected(),
            _ => {}
        }
    }

    fn visible_entries(&self) -> Vec<&DashboardEntry> {
        self.entries.iter().filter(|entry| !entry.hidden).collect()
    }

    fn move_selection(&mut self, delta: isize) {
        let len = self.visible_rows().len();
        if len == 0 {
            self.selected = 0;
            return;
        }
        if delta > 0 {
            self.selected = (self.selected + delta as usize).min(len - 1);
        } else {
            self.selected = self.selected.saturating_sub(delta.unsigned_abs());
        }
    }

    fn toggle_selected_star(&mut self) {
        let Some(id) = self.selected_entry_id().map(str::to_string) else {
            return;
        };
        if let Some(entry) = self.entries.iter_mut().find(|entry| entry.id == id) {
            entry.starred = !entry.starred;
        }
        self.clamp_selection();
    }

    fn hide_selected(&mut self) {
        let Some(id) = self.selected_entry_id().map(str::to_string) else {
            return;
        };
        if let Some(entry) = self.entries.iter_mut().find(|entry| entry.id == id) {
            entry.hidden = true;
            entry.starred = false;
        }
        self.clamp_selection();
    }

    fn clamp_selection(&mut self) {
        let len = self.visible_rows().len();
        if len == 0 {
            self.selected = 0;
        } else if self.selected >= len {
            self.selected = len - 1;
        }
    }
}

fn row_for(entry: &DashboardEntry) -> DashboardRow {
    DashboardRow {
        entry_id: entry.id.clone(),
    }
}

pub fn parse_open_prompt(input: &str) -> PromptTarget {
    if input.starts_with("http://") || input.starts_with("https://") {
        PromptTarget::Url(input.to_string())
    } else {
        PromptTarget::LocalPath(expand_tilde(input))
    }
}

pub fn suggest_local_paths(input: &str) -> Result<Vec<PathSuggestion>, PromptError> {
    let expanded = expand_tilde(input);
    let (dir, prefix) = suggestion_dir_and_prefix(&expanded);

    if !dir.exists() {
        return Err(prompt_error(
            PromptErrorKind::MissingPath,
            format!("路径不存在: {}", dir.display()),
        ));
    }
    if !dir.is_dir() {
        return Err(prompt_error(
            PromptErrorKind::Io,
            format!("不是目录: {}", dir.display()),
        ));
    }

    let mut suggestions: Vec<PathSuggestion> = fs::read_dir(&dir)
        .map_err(|err| prompt_error(PromptErrorKind::Io, err.to_string()))?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let file_name = entry.file_name().to_string_lossy().to_string();
            if !file_name.starts_with(&prefix) {
                return None;
            }
            let path = entry.path();
            let is_dir = path.is_dir();
            let display = if is_dir {
                format!("{file_name}/")
            } else {
                file_name
            };
            Some(PathSuggestion {
                path,
                display,
                is_dir,
            })
        })
        .collect();

    suggestions.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.display.cmp(&b.display),
    });
    suggestions.truncate(50);
    Ok(suggestions)
}

fn suggestion_dir_and_prefix(path: &Path) -> (PathBuf, String) {
    if input_ends_with_separator(path) {
        (path.to_path_buf(), String::new())
    } else {
        let dir = path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        let prefix = path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_default();
        (dir, prefix)
    }
}

fn input_ends_with_separator(path: &Path) -> bool {
    let s = path.to_string_lossy();
    s.ends_with(std::path::MAIN_SEPARATOR) || s.ends_with('/')
}

fn expand_tilde(input: &str) -> PathBuf {
    if input == "~" {
        return dirs::home_dir().unwrap_or_else(|| PathBuf::from(input));
    }
    if let Some(rest) = input.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(input)
}

fn prompt_error(kind: PromptErrorKind, message: String) -> PromptError {
    PromptError {
        kind,
        recoverable: true,
        message,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyModifiers};
    use std::fs;
    use std::path::{Path, PathBuf};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn entry(
        id: &str,
        title: &str,
        target: &str,
        updated_at: &str,
        starred: bool,
        hidden: bool,
    ) -> DashboardEntry {
        DashboardEntry {
            id: id.to_string(),
            kind: TargetKind::Markdown,
            title: title.to_string(),
            target: target.to_string(),
            progress_percent: 0,
            updated_at: updated_at.to_string(),
            starred,
            hidden,
        }
    }

    fn row_ids(rows: &[DashboardRow]) -> Vec<String> {
        rows.iter().map(|row| row.entry_id.clone()).collect()
    }

    fn suggestion_labels(suggestions: &[PathSuggestion]) -> Vec<String> {
        suggestions
            .iter()
            .map(|suggestion| suggestion.display.clone())
            .collect()
    }

    fn temp_dir(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "tread_dashboard_test_{}_{}",
            name,
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn write_file(path: &Path) {
        fs::write(path, b"content").unwrap();
    }

    #[test]
    fn visible_rows_split_starred_and_recent_without_duplicates() {
        let entries = vec![
            entry(
                "starred",
                "Starred",
                "starred.md",
                "2026-07-09T00:00:01Z",
                true,
                false,
            ),
            entry(
                "recent",
                "Recent",
                "recent.md",
                "2026-07-09T00:00:02Z",
                false,
                false,
            ),
            entry(
                "hidden-starred",
                "Hidden",
                "hidden.md",
                "2026-07-09T00:00:03Z",
                true,
                true,
            ),
        ];

        let state = DashboardState::from_history(entries);

        assert_eq!(row_ids(&state.starred_rows()), vec!["starred"]);
        assert_eq!(row_ids(&state.recent_rows()), vec!["recent"]);
        let all_rows = row_ids(&state.visible_rows());
        assert_eq!(all_rows, vec!["starred", "recent"]);
        assert_eq!(
            all_rows
                .iter()
                .filter(|id| id.as_str() == "starred")
                .count(),
            1
        );
        assert!(!all_rows.iter().any(|id| id == "hidden-starred"));
    }

    #[test]
    fn recent_rows_sort_by_descending_updated_at() {
        let entries = vec![
            entry(
                "older",
                "Older",
                "older.md",
                "2026-07-08T00:00:00Z",
                false,
                false,
            ),
            entry(
                "newest",
                "Newest",
                "newest.md",
                "2026-07-10T00:00:00Z",
                false,
                false,
            ),
            entry(
                "middle",
                "Middle",
                "middle.md",
                "2026-07-09T00:00:00Z",
                false,
                false,
            ),
        ];

        let state = DashboardState::from_history(entries);

        assert_eq!(
            row_ids(&state.recent_rows()),
            vec!["newest", "middle", "older"]
        );
    }

    #[test]
    fn s_toggles_selected_entry_between_starred_and_recent() {
        let entries = vec![entry(
            "book",
            "Book",
            "book.md",
            "2026-07-09T00:00:00Z",
            false,
            false,
        )];
        let mut state = DashboardState::from_history(entries);

        state.handle_key(key(KeyCode::Char('s')));
        assert_eq!(row_ids(&state.starred_rows()), vec!["book"]);
        assert!(state.recent_rows().is_empty());

        state.handle_key(key(KeyCode::Char('s')));
        assert!(state.starred_rows().is_empty());
        assert_eq!(row_ids(&state.recent_rows()), vec!["book"]);
    }

    #[test]
    fn r_hides_selected_entry_and_clears_starred() {
        let entries = vec![entry(
            "favorite",
            "Favorite",
            "favorite.md",
            "2026-07-09T00:00:00Z",
            true,
            false,
        )];
        let mut state = DashboardState::from_history(entries);

        state.handle_key(key(KeyCode::Char('r')));

        let removed = state.entry("favorite").unwrap();
        assert!(removed.hidden);
        assert!(!removed.starred);
        assert!(state.visible_rows().is_empty());
    }

    #[test]
    fn j_k_and_arrows_move_selection_within_visible_rows() {
        let entries = vec![
            entry(
                "first",
                "First",
                "first.md",
                "2026-07-09T00:00:03Z",
                false,
                false,
            ),
            entry(
                "second",
                "Second",
                "second.md",
                "2026-07-09T00:00:02Z",
                false,
                false,
            ),
            entry(
                "third",
                "Third",
                "third.md",
                "2026-07-09T00:00:01Z",
                false,
                false,
            ),
        ];
        let mut state = DashboardState::from_history(entries);

        assert_eq!(state.selected_entry_id(), Some("first"));
        state.handle_key(key(KeyCode::Char('j')));
        assert_eq!(state.selected_entry_id(), Some("second"));
        state.handle_key(key(KeyCode::Down));
        assert_eq!(state.selected_entry_id(), Some("third"));
        state.handle_key(key(KeyCode::Down));
        assert_eq!(state.selected_entry_id(), Some("third"));
        state.handle_key(key(KeyCode::Char('k')));
        assert_eq!(state.selected_entry_id(), Some("second"));
        state.handle_key(key(KeyCode::Up));
        assert_eq!(state.selected_entry_id(), Some("first"));
        state.handle_key(key(KeyCode::Up));
        assert_eq!(state.selected_entry_id(), Some("first"));
    }

    #[test]
    fn open_prompt_detects_http_and_https_urls() {
        assert_eq!(
            parse_open_prompt("https://example.com/read"),
            PromptTarget::Url("https://example.com/read".to_string())
        );
        assert_eq!(
            parse_open_prompt("http://example.com/read"),
            PromptTarget::Url("http://example.com/read".to_string())
        );
        assert!(matches!(
            parse_open_prompt("ftp://example.com/read"),
            PromptTarget::LocalPath(_)
        ));
        assert!(matches!(
            parse_open_prompt("example.com/read"),
            PromptTarget::LocalPath(_)
        ));
    }

    #[test]
    fn local_path_suggestions_are_one_level_directory_first_sorted_and_limited() {
        let root = temp_dir("suggestions_sorted");
        fs::create_dir(root.join("beta_dir")).unwrap();
        fs::create_dir(root.join("alpha_dir")).unwrap();
        write_file(&root.join("zeta.md"));
        write_file(&root.join("alpha.md"));
        fs::create_dir(root.join("alpha_dir").join("nested_match")).unwrap();
        for i in 0..60 {
            write_file(&root.join(format!("file_{i:02}.md")));
        }

        let input = root.join("").to_string_lossy().to_string();
        let suggestions = suggest_local_paths(&input).unwrap();
        let labels = suggestion_labels(&suggestions);

        assert_eq!(labels.len(), 50);
        assert_eq!(labels[0], "alpha_dir/");
        assert_eq!(labels[1], "beta_dir/");
        assert_eq!(labels[2], "alpha.md");
        assert_eq!(labels[3], "file_00.md");
        assert!(!labels.iter().any(|label| label == "nested_match/"));
    }

    #[test]
    fn path_suggestions_match_prefix_in_current_directory_only() {
        let root = temp_dir("suggestions_prefix");
        fs::create_dir(root.join("bookshelf")).unwrap();
        fs::create_dir(root.join("other")).unwrap();
        write_file(&root.join("book.epub"));
        write_file(&root.join("notes.md"));
        write_file(&root.join("other").join("book-hidden.md"));

        let input = root.join("bo").to_string_lossy().to_string();
        let suggestions = suggest_local_paths(&input).unwrap();

        assert_eq!(
            suggestion_labels(&suggestions),
            vec!["bookshelf/", "book.epub"]
        );
    }

    #[test]
    fn missing_suggestion_path_returns_recoverable_prompt_error() {
        let missing = std::env::temp_dir().join(format!(
            "tread_dashboard_missing_{}_nope",
            std::process::id()
        ));

        let err = suggest_local_paths(&missing.join("prefix").to_string_lossy()).unwrap_err();

        assert!(matches!(err.kind, PromptErrorKind::MissingPath));
        assert!(err.recoverable);
    }

    #[test]
    fn unreadable_suggestion_directory_returns_recoverable_prompt_error() {
        let file_parent = temp_dir("suggestions_file_parent").join("not_a_directory");
        write_file(&file_parent);

        let err = suggest_local_paths(&file_parent.join("child").to_string_lossy()).unwrap_err();

        assert!(matches!(err.kind, PromptErrorKind::Io));
        assert!(err.recoverable);
    }
}
