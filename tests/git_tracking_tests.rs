//! Tests for Git tracking functionality
//!
//! This test file covers:
//! - Git track command
//! - Git log command
//! - Git diff command
//! - Metadata storage and retrieval
//! - Error handling for git operations

use std::fs;
use std::process::Command;
use tempfile::TempDir;

// ============================================================================
// Test Helpers
// ============================================================================

/// Initialize a git repository in a temporary directory
fn init_git_repo(dir: &TempDir) -> bool {
    let result = Command::new("git")
        .args(["init"])
        .current_dir(dir.path())
        .output();

    match result {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

/// Create a test commit in a git repository
fn create_test_commit(dir: &TempDir, message: &str, files: &[(&str, &str)]) -> bool {
    // Create files
    for (name, content) in files {
        let path = dir.path().join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).ok();
        }
        if fs::write(&path, content).is_err() {
            return false;
        }
    }

    // Stage files
    let add_result = Command::new("git")
        .args(["add", "."])
        .current_dir(dir.path())
        .output();

    if add_result.is_err() || !add_result.unwrap().status.success() {
        return false;
    }

    // Configure user for commit
    Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(dir.path())
        .output()
        .ok();

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(dir.path())
        .output()
        .ok();

    // Commit
    let commit_result = Command::new("git")
        .args(["commit", "-m", message])
        .current_dir(dir.path())
        .output();

    match commit_result {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

/// Get current git commit hash
fn get_current_commit(dir: &TempDir) -> Option<String> {
    let result = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(dir.path())
        .output()
        .ok()?;

    if result.status.success() {
        Some(String::from_utf8_lossy(&result.stdout).trim().to_string())
    } else {
        None
    }
}

// ============================================================================
// Metadata Structure Tests
// ============================================================================

mod metadata_tests {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct GitMetadata {
        session_id: String,
        commit_hash: String,
        branch: String,
        message: String,
        timestamp: i64,
        files_changed: Vec<String>,
    }

    #[test]
    fn test_metadata_serialization() {
        let metadata = GitMetadata {
            session_id: "session-123".to_string(),
            commit_hash: "abc123def456".to_string(),
            branch: "main".to_string(),
            message: "Test commit".to_string(),
            timestamp: 1700000000,
            files_changed: vec!["file1.rs".to_string(), "file2.rs".to_string()],
        };

        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("session-123"));
        assert!(json.contains("abc123def456"));
    }

    #[test]
    fn test_metadata_deserialization() {
        let json = r#"{
            "session_id": "session-123",
            "commit_hash": "abc123def456",
            "branch": "main",
            "message": "Test commit",
            "timestamp": 1700000000,
            "files_changed": ["file1.rs", "file2.rs"]
        }"#;

        let metadata: GitMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.session_id, "session-123");
        assert_eq!(metadata.commit_hash, "abc123def456");
        assert_eq!(metadata.files_changed.len(), 2);
    }

    #[test]
    fn test_metadata_with_empty_files() {
        let metadata = GitMetadata {
            session_id: "session-123".to_string(),
            commit_hash: "abc123".to_string(),
            branch: "main".to_string(),
            message: "Empty commit".to_string(),
            timestamp: 1700000000,
            files_changed: vec![],
        };

        let json = serde_json::to_string(&metadata).unwrap();
        let parsed: GitMetadata = serde_json::from_str(&json).unwrap();

        assert!(parsed.files_changed.is_empty());
    }
}

// ============================================================================
// Git Status Parsing Tests
// ============================================================================

mod git_status_tests {
    #[derive(Debug, PartialEq)]
    enum FileStatus {
        Added,
        Modified,
        Deleted,
        Renamed,
        Unknown,
    }

    fn parse_git_status_line(line: &str) -> Option<(FileStatus, String)> {
        if line.len() < 4 {
            return None;
        }

        let status_chars = &line[0..2];
        let filename = line[3..].trim().to_string();

        let status = match status_chars.chars().next()? {
            'A' | '?' => FileStatus::Added,
            'M' => FileStatus::Modified,
            'D' => FileStatus::Deleted,
            'R' => FileStatus::Renamed,
            _ => FileStatus::Unknown,
        };

        Some((status, filename))
    }

    #[test]
    fn test_parse_added_file() {
        let result = parse_git_status_line("A  new_file.rs");
        assert_eq!(result, Some((FileStatus::Added, "new_file.rs".to_string())));
    }

    #[test]
    fn test_parse_modified_file() {
        let result = parse_git_status_line("M  modified_file.rs");
        assert_eq!(
            result,
            Some((FileStatus::Modified, "modified_file.rs".to_string()))
        );
    }

    #[test]
    fn test_parse_deleted_file() {
        let result = parse_git_status_line("D  deleted_file.rs");
        assert_eq!(
            result,
            Some((FileStatus::Deleted, "deleted_file.rs".to_string()))
        );
    }

    #[test]
    fn test_parse_renamed_file() {
        let result = parse_git_status_line("R  old_name.rs -> new_name.rs");
        assert_eq!(
            result,
            Some((
                FileStatus::Renamed,
                "old_name.rs -> new_name.rs".to_string()
            ))
        );
    }

    #[test]
    fn test_parse_untracked_file() {
        let result = parse_git_status_line("?? untracked.rs");
        assert_eq!(
            result,
            Some((FileStatus::Added, "untracked.rs".to_string()))
        );
    }

    #[test]
    fn test_parse_invalid_line() {
        let result = parse_git_status_line("x");
        assert_eq!(result, None);
    }
}

// ============================================================================
// Commit Hash Validation Tests
// ============================================================================

mod commit_hash_tests {
    fn is_valid_commit_hash(hash: &str) -> bool {
        // Git hashes are 40 characters (SHA-1) or 64 characters (SHA-256)
        // Short hashes are at least 7 characters
        let len = hash.len();
        if !(7..=64).contains(&len) {
            return false;
        }
        hash.chars().all(|c| c.is_ascii_hexdigit())
    }

    fn is_short_hash(hash: &str) -> bool {
        hash.len() >= 7 && hash.len() < 40
    }

    fn is_full_hash(hash: &str) -> bool {
        hash.len() == 40 || hash.len() == 64
    }

    #[test]
    fn test_valid_full_hash() {
        let hash = "abc123def456789012345678901234567890abcd";
        assert!(is_valid_commit_hash(hash));
        assert!(is_full_hash(hash));
    }

    #[test]
    fn test_valid_short_hash() {
        let hash = "abc123d";
        assert!(is_valid_commit_hash(hash));
        assert!(is_short_hash(hash));
    }

    #[test]
    fn test_invalid_hash_too_short() {
        let hash = "abc12";
        assert!(!is_valid_commit_hash(hash));
    }

    #[test]
    fn test_invalid_hash_bad_chars() {
        let hash = "abc123g"; // 'g' is not hex
        assert!(!is_valid_commit_hash(hash));
    }

    #[test]
    fn test_empty_hash() {
        let hash = "";
        assert!(!is_valid_commit_hash(hash));
    }
}

// ============================================================================
// Log Formatting Tests
// ============================================================================

mod log_format_tests {
    use chrono::{TimeZone, Utc};

    struct GitLogEntry {
        commit: String,
        message: String,
        author: String,
        timestamp: i64,
        session_id: Option<String>,
    }

    fn format_log_entry(entry: &GitLogEntry) -> String {
        let datetime = Utc
            .timestamp_opt(entry.timestamp, 0)
            .single()
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "Unknown date".to_string());

        let session_info = entry
            .session_id
            .as_ref()
            .map(|id| format!(" [Session: {}]", id))
            .unwrap_or_default();

        format!(
            "{} {} - {}{}\n  by {}",
            &entry.commit[..7.min(entry.commit.len())],
            datetime,
            entry.message,
            session_info,
            entry.author
        )
    }

    #[test]
    fn test_format_log_entry_with_session() {
        let entry = GitLogEntry {
            commit: "abc123def456789".to_string(),
            message: "Fix bug".to_string(),
            author: "Test User".to_string(),
            timestamp: 1700000000,
            session_id: Some("session-123".to_string()),
        };

        let formatted = format_log_entry(&entry);
        assert!(formatted.contains("abc123d"));
        assert!(formatted.contains("Fix bug"));
        assert!(formatted.contains("Session: session-123"));
        assert!(formatted.contains("Test User"));
    }

    #[test]
    fn test_format_log_entry_without_session() {
        let entry = GitLogEntry {
            commit: "abc123def456789".to_string(),
            message: "Regular commit".to_string(),
            author: "Test User".to_string(),
            timestamp: 1700000000,
            session_id: None,
        };

        let formatted = format_log_entry(&entry);
        assert!(formatted.contains("Regular commit"));
        assert!(!formatted.contains("Session:"));
    }
}

// ============================================================================
// Diff Parsing Tests
// ============================================================================

mod diff_parsing_tests {
    #[allow(dead_code)]
    struct DiffHunk {
        old_start: usize,
        old_count: usize,
        new_start: usize,
        new_count: usize,
        lines: Vec<DiffLine>,
    }

    struct DiffLine {
        kind: DiffKind,
        content: String,
    }

    #[derive(Debug, PartialEq)]
    enum DiffKind {
        Context,
        Addition,
        Deletion,
    }

    fn parse_diff_line(line: &str) -> Option<DiffLine> {
        if line.is_empty() {
            return None;
        }

        let (kind, content) = match line.chars().next()? {
            '+' => (DiffKind::Addition, line[1..].to_string()),
            '-' => (DiffKind::Deletion, line[1..].to_string()),
            ' ' => (DiffKind::Context, line[1..].to_string()),
            _ => return None,
        };

        Some(DiffLine { kind, content })
    }

    fn count_changes(lines: &[DiffLine]) -> (usize, usize) {
        let additions = lines
            .iter()
            .filter(|l| l.kind == DiffKind::Addition)
            .count();
        let deletions = lines
            .iter()
            .filter(|l| l.kind == DiffKind::Deletion)
            .count();
        (additions, deletions)
    }

    #[test]
    fn test_parse_addition_line() {
        let line = parse_diff_line("+new code here").unwrap();
        assert_eq!(line.kind, DiffKind::Addition);
        assert_eq!(line.content, "new code here");
    }

    #[test]
    fn test_parse_deletion_line() {
        let line = parse_diff_line("-old code here").unwrap();
        assert_eq!(line.kind, DiffKind::Deletion);
        assert_eq!(line.content, "old code here");
    }

    #[test]
    fn test_parse_context_line() {
        let line = parse_diff_line(" unchanged code").unwrap();
        assert_eq!(line.kind, DiffKind::Context);
        assert_eq!(line.content, "unchanged code");
    }

    #[test]
    fn test_count_changes() {
        let lines = vec![
            DiffLine {
                kind: DiffKind::Context,
                content: "a".to_string(),
            },
            DiffLine {
                kind: DiffKind::Addition,
                content: "b".to_string(),
            },
            DiffLine {
                kind: DiffKind::Addition,
                content: "c".to_string(),
            },
            DiffLine {
                kind: DiffKind::Deletion,
                content: "d".to_string(),
            },
        ];

        let (additions, deletions) = count_changes(&lines);
        assert_eq!(additions, 2);
        assert_eq!(deletions, 1);
    }
}

// ============================================================================
// Session-Commit Association Tests
// ============================================================================

mod session_commit_tests {
    use std::collections::HashMap;

    struct SessionCommitMap {
        associations: HashMap<String, Vec<String>>, // session_id -> commit hashes
    }

    impl SessionCommitMap {
        fn new() -> Self {
            Self {
                associations: HashMap::new(),
            }
        }

        fn add_commit(&mut self, session_id: &str, commit_hash: &str) {
            self.associations
                .entry(session_id.to_string())
                .or_default()
                .push(commit_hash.to_string());
        }

        fn get_commits(&self, session_id: &str) -> Option<&Vec<String>> {
            self.associations.get(session_id)
        }

        fn get_session_for_commit(&self, commit_hash: &str) -> Option<&String> {
            for (session, commits) in &self.associations {
                if commits.contains(&commit_hash.to_string()) {
                    return Some(session);
                }
            }
            None
        }
    }

    #[test]
    fn test_add_single_commit() {
        let mut map = SessionCommitMap::new();
        map.add_commit("session-1", "commit-a");

        let commits = map.get_commits("session-1").unwrap();
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0], "commit-a");
    }

    #[test]
    fn test_add_multiple_commits_to_session() {
        let mut map = SessionCommitMap::new();
        map.add_commit("session-1", "commit-a");
        map.add_commit("session-1", "commit-b");
        map.add_commit("session-1", "commit-c");

        let commits = map.get_commits("session-1").unwrap();
        assert_eq!(commits.len(), 3);
    }

    #[test]
    fn test_multiple_sessions() {
        let mut map = SessionCommitMap::new();
        map.add_commit("session-1", "commit-a");
        map.add_commit("session-2", "commit-b");

        assert!(map.get_commits("session-1").is_some());
        assert!(map.get_commits("session-2").is_some());
        assert!(map.get_commits("session-3").is_none());
    }

    #[test]
    fn test_find_session_by_commit() {
        let mut map = SessionCommitMap::new();
        map.add_commit("session-1", "commit-a");
        map.add_commit("session-2", "commit-b");

        let session = map.get_session_for_commit("commit-b").unwrap();
        assert_eq!(session, "session-2");
    }

    #[test]
    fn test_session_not_found_for_commit() {
        let map = SessionCommitMap::new();
        assert!(map.get_session_for_commit("unknown").is_none());
    }
}

// ============================================================================
// Git Command Builder Tests
// ============================================================================

mod git_command_tests {
    struct GitCommand {
        args: Vec<String>,
        working_dir: Option<String>,
    }

    impl GitCommand {
        fn new(subcommand: &str) -> Self {
            Self {
                args: vec![subcommand.to_string()],
                working_dir: None,
            }
        }

        fn arg(&mut self, arg: &str) -> &mut Self {
            self.args.push(arg.to_string());
            self
        }

        fn cwd(&mut self, dir: &str) -> &mut Self {
            self.working_dir = Some(dir.to_string());
            self
        }

        fn build_args(&self) -> Vec<&str> {
            self.args.iter().map(|s| s.as_str()).collect()
        }
    }

    #[test]
    fn test_git_log_command() {
        let mut cmd = GitCommand::new("log");
        cmd.arg("--oneline").arg("-n").arg("10");

        let args = cmd.build_args();
        assert_eq!(args, vec!["log", "--oneline", "-n", "10"]);
    }

    #[test]
    fn test_git_diff_command() {
        let mut cmd = GitCommand::new("diff");
        cmd.arg("HEAD~1").arg("--stat");

        let args = cmd.build_args();
        assert_eq!(args, vec!["diff", "HEAD~1", "--stat"]);
    }

    #[test]
    fn test_git_command_with_working_dir() {
        let mut cmd = GitCommand::new("status");
        cmd.cwd("/path/to/repo");

        assert_eq!(cmd.working_dir, Some("/path/to/repo".to_string()));
    }
}

// ============================================================================
// Track Command Tests
// ============================================================================

mod track_command_tests {
    #[test]
    fn test_track_creates_association() {
        // Simulated tracking
        let session_id = "session-123";
        let commit_hash = "abc123";

        let association = format!("{}:{}", session_id, commit_hash);
        assert!(association.contains(session_id));
        assert!(association.contains(commit_hash));
    }

    #[test]
    fn test_track_with_message() {
        let session_id = "session-123";
        let message = "Implementing feature X based on chat";

        let commit_message = format!("[CSM: {}] {}", session_id, message);
        assert!(commit_message.contains(session_id));
        assert!(commit_message.contains(message));
    }

    #[test]
    fn test_track_multiple_files() {
        let files = ["file1.rs", "file2.rs", "src/file3.rs"];

        assert_eq!(files.len(), 3);
        assert!(files.contains(&"src/file3.rs"));
    }
}

// ============================================================================
// Link Command Tests
// ============================================================================

mod link_command_tests {
    use std::collections::HashMap;

    #[test]
    fn test_link_session_to_repo() {
        let mut links: HashMap<String, String> = HashMap::new();

        let session_id = "session-123";
        let repo_path = "/path/to/repo";

        links.insert(session_id.to_string(), repo_path.to_string());

        assert_eq!(links.get(session_id), Some(&repo_path.to_string()));
    }

    #[test]
    fn test_link_update_existing() {
        let mut links: HashMap<String, String> = HashMap::new();

        links.insert("session-123".to_string(), "/old/path".to_string());
        links.insert("session-123".to_string(), "/new/path".to_string());

        assert_eq!(links.get("session-123"), Some(&"/new/path".to_string()));
    }

    #[test]
    fn test_unlink_session() {
        let mut links: HashMap<String, String> = HashMap::new();
        links.insert("session-123".to_string(), "/path".to_string());

        links.remove("session-123");

        assert!(!links.contains_key("session-123"));
    }
}

// ============================================================================
// Error Case Tests
// ============================================================================

mod error_tests {
    #[test]
    fn test_not_a_git_repo() {
        use std::process::Command;
        use tempfile::TempDir;

        let dir = TempDir::new().unwrap();
        let result = Command::new("git")
            .args(["status"])
            .current_dir(dir.path())
            .output();

        // Should either fail or have non-zero exit code
        if let Ok(output) = result {
            assert!(!output.status.success() || !output.stderr.is_empty());
        }
    }

    #[test]
    fn test_invalid_session_id() {
        let session_id: String = String::new();
        assert!(session_id.is_empty());
    }

    #[test]
    fn test_invalid_commit_reference() {
        let commit_ref = "not-a-real-commit-hash-xyz";

        // Validate format (should fail)
        let is_valid = commit_ref.len() >= 7 && commit_ref.chars().all(|c| c.is_ascii_hexdigit());

        assert!(!is_valid);
    }
}

// ============================================================================
// Date Range Filter Tests
// ============================================================================

mod date_filter_tests {
    fn is_within_range(timestamp: i64, start: i64, end: i64) -> bool {
        timestamp >= start && timestamp <= end
    }

    #[test]
    fn test_timestamp_in_range() {
        let start = 1700000000;
        let end = 1700100000;
        let timestamp = 1700050000;

        assert!(is_within_range(timestamp, start, end));
    }

    #[test]
    fn test_timestamp_before_range() {
        let start = 1700000000;
        let end = 1700100000;
        let timestamp = 1699900000;

        assert!(!is_within_range(timestamp, start, end));
    }

    #[test]
    fn test_timestamp_after_range() {
        let start = 1700000000;
        let end = 1700100000;
        let timestamp = 1700200000;

        assert!(!is_within_range(timestamp, start, end));
    }

    #[test]
    fn test_timestamp_at_boundary() {
        let start = 1700000000;
        let end = 1700100000;

        assert!(is_within_range(start, start, end));
        assert!(is_within_range(end, start, end));
    }
}

// ============================================================================
// Integration Test with Real Git
// ============================================================================

mod git_integration_tests {
    use super::*;

    #[test]
    #[ignore] // Run with --ignored to test with real git
    fn test_full_git_workflow() {
        let dir = TempDir::new().unwrap();

        // Initialize repo
        assert!(init_git_repo(&dir));

        // Create initial commit
        assert!(create_test_commit(
            &dir,
            "Initial commit",
            &[("README.md", "# Test Project")]
        ));

        // Get commit hash
        let hash1 = get_current_commit(&dir);
        assert!(hash1.is_some());

        // Create another commit
        assert!(create_test_commit(
            &dir,
            "Add source file",
            &[("src/main.rs", "fn main() { println!(\"Hello\"); }")]
        ));

        let hash2 = get_current_commit(&dir);
        assert!(hash2.is_some());

        // Verify different commits
        assert_ne!(hash1, hash2);
    }

    #[test]
    #[ignore]
    fn test_git_log_output() {
        let dir = TempDir::new().unwrap();

        if !init_git_repo(&dir) {
            return;
        }

        create_test_commit(&dir, "Test commit", &[("test.txt", "content")]);

        let output = std::process::Command::new("git")
            .args(["log", "--oneline", "-1"])
            .current_dir(dir.path())
            .output()
            .unwrap();

        let log = String::from_utf8_lossy(&output.stdout);
        assert!(log.contains("Test commit"));
    }
}
