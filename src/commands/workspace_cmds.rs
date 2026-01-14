// Copyright (c) 2024-2026 Nervosys LLC
// SPDX-License-Identifier: Apache-2.0
//! Workspace listing commands

use anyhow::Result;
use colored::Colorize;
use tabled::{settings::Style, Table, Tabled};

use crate::models::Workspace;
use crate::storage::read_empty_window_sessions;
use crate::workspace::discover_workspaces;

#[derive(Tabled)]
struct WorkspaceRow {
    #[tabled(rename = "Hash")]
    hash: String,
    #[tabled(rename = "Project Path")]
    project_path: String,
    #[tabled(rename = "Sessions")]
    sessions: String,
    #[tabled(rename = "Has Chats")]
    has_chats: String,
}

#[derive(Tabled)]
struct SessionRow {
    #[tabled(rename = "Project Path")]
    project_path: String,
    #[tabled(rename = "Session File")]
    session_file: String,
    #[tabled(rename = "Last Modified")]
    last_modified: String,
    #[tabled(rename = "Messages")]
    messages: usize,
}

/// List all VS Code workspaces
pub fn list_workspaces() -> Result<()> {
    let workspaces = discover_workspaces()?;

    if workspaces.is_empty() {
        println!("{} No workspaces found.", "[!]".yellow());
        return Ok(());
    }

    let rows: Vec<WorkspaceRow> = workspaces
        .iter()
        .map(|ws| WorkspaceRow {
            hash: format!(
                "{}",
                format!("{}...", &ws.hash[..12.min(ws.hash.len())]).cyan()
            ),
            project_path: ws
                .project_path
                .clone()
                .unwrap_or_else(|| "(none)".to_string()),
            sessions: format!("{}", ws.chat_session_count.to_string().green()),
            has_chats: if ws.has_chat_sessions {
                format!("{}", "Yes".green())
            } else {
                format!("{}", "No".red())
            },
        })
        .collect();

    let table = Table::new(rows).with(Style::ascii_rounded()).to_string();
    println!("{}", table);
    println!(
        "\n{} Total workspaces: {}",
        "[=]".blue(),
        workspaces.len().to_string().yellow()
    );

    // Show empty window sessions count (ALL SESSIONS)
    if let Ok(empty_count) = crate::storage::count_empty_window_sessions() {
        if empty_count > 0 {
            println!(
                "{} Empty window sessions (ALL SESSIONS): {}",
                "[i]".cyan(),
                empty_count.to_string().yellow()
            );
        }
    }

    Ok(())
}

/// List all chat sessions
pub fn list_sessions(project_path: Option<&str>) -> Result<()> {
    let workspaces = discover_workspaces()?;

    let filtered_workspaces: Vec<&Workspace> = if let Some(path) = project_path {
        let normalized = crate::workspace::normalize_path(path);
        workspaces
            .iter()
            .filter(|ws| {
                ws.project_path
                    .as_ref()
                    .map(|p| crate::workspace::normalize_path(p) == normalized)
                    .unwrap_or(false)
            })
            .collect()
    } else {
        workspaces.iter().collect()
    };

    let mut rows: Vec<SessionRow> = Vec::new();

    // Add empty window sessions (ALL SESSIONS) if no specific project filter
    if project_path.is_none() {
        if let Ok(empty_sessions) = read_empty_window_sessions() {
            for session in empty_sessions {
                let modified = chrono::DateTime::from_timestamp_millis(session.last_message_date)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                let session_id = session.session_id.as_deref().unwrap_or("unknown");
                rows.push(SessionRow {
                    project_path: "(ALL SESSIONS)".to_string(),
                    session_file: format!("{}.json", session_id),
                    last_modified: modified,
                    messages: session.request_count(),
                });
            }
        }
    }

    for ws in filtered_workspaces {
        if !ws.has_chat_sessions {
            continue;
        }

        let sessions = crate::workspace::get_chat_sessions_from_workspace(&ws.workspace_path)?;

        for session_with_path in sessions {
            let modified = session_with_path
                .path
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .map(|t| {
                    let datetime: chrono::DateTime<chrono::Utc> = t.into();
                    datetime.format("%Y-%m-%d %H:%M").to_string()
                })
                .unwrap_or_else(|| "unknown".to_string());

            rows.push(SessionRow {
                project_path: ws
                    .project_path
                    .clone()
                    .unwrap_or_else(|| "(none)".to_string()),
                session_file: session_with_path
                    .path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "unknown".to_string()),
                last_modified: modified,
                messages: session_with_path.session.request_count(),
            });
        }
    }

    if rows.is_empty() {
        println!("{} No chat sessions found.", "[!]".yellow());
        return Ok(());
    }

    let table = Table::new(&rows).with(Style::ascii_rounded()).to_string();

    println!("{}", table.dimmed());
    println!(
        "\n{} Total sessions: {}",
        "[=]".blue(),
        rows.len().to_string().yellow()
    );

    Ok(())
}

/// Find workspaces by search pattern
pub fn find_workspaces(pattern: &str) -> Result<()> {
    let workspaces = discover_workspaces()?;

    // Resolve "." to current directory name
    let pattern = if pattern == "." {
        std::env::current_dir()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .unwrap_or_else(|| pattern.to_string())
    } else {
        pattern.to_string()
    };
    let pattern_lower = pattern.to_lowercase();

    let matching: Vec<&Workspace> = workspaces
        .iter()
        .filter(|ws| {
            ws.project_path
                .as_ref()
                .map(|p| p.to_lowercase().contains(&pattern_lower))
                .unwrap_or(false)
                || ws.hash.to_lowercase().contains(&pattern_lower)
        })
        .collect();

    if matching.is_empty() {
        println!(
            "{} No workspaces found matching '{}'",
            "[!]".yellow(),
            pattern.cyan()
        );
        return Ok(());
    }

    let rows: Vec<WorkspaceRow> = matching
        .iter()
        .map(|ws| WorkspaceRow {
            hash: format!(
                "{}",
                format!("{}...", &ws.hash[..12.min(ws.hash.len())]).cyan()
            ),
            project_path: ws
                .project_path
                .clone()
                .unwrap_or_else(|| "(none)".to_string()),
            sessions: format!("{}", ws.chat_session_count.to_string().green()),
            has_chats: if ws.has_chat_sessions {
                format!("{}", "Yes".green())
            } else {
                format!("{}", "No".red())
            },
        })
        .collect();

    let table = Table::new(rows).with(Style::ascii_rounded()).to_string();

    println!("{}", table);
    println!(
        "\n{} Found {} matching workspace(s)",
        "[=]".blue(),
        matching.len().to_string().yellow()
    );

    // Show session paths for each matching workspace
    for ws in &matching {
        if ws.has_chat_sessions {
            let project = ws.project_path.as_deref().unwrap_or("(none)");
            println!("\nSessions for {}:", project);

            if let Ok(sessions) =
                crate::workspace::get_chat_sessions_from_workspace(&ws.workspace_path)
            {
                for session_with_path in sessions {
                    println!("  {}", session_with_path.path.display());
                }
            }
        }
    }

    Ok(())
}

/// Find sessions by search pattern
#[allow(dead_code)]
pub fn find_sessions(pattern: &str, project_path: Option<&str>) -> Result<()> {
    let workspaces = discover_workspaces()?;
    let pattern_lower = pattern.to_lowercase();

    let filtered_workspaces: Vec<&Workspace> = if let Some(path) = project_path {
        let normalized = crate::workspace::normalize_path(path);
        workspaces
            .iter()
            .filter(|ws| {
                ws.project_path
                    .as_ref()
                    .map(|p| crate::workspace::normalize_path(p) == normalized)
                    .unwrap_or(false)
            })
            .collect()
    } else {
        workspaces.iter().collect()
    };

    let mut rows: Vec<SessionRow> = Vec::new();

    for ws in filtered_workspaces {
        if !ws.has_chat_sessions {
            continue;
        }

        let sessions = crate::workspace::get_chat_sessions_from_workspace(&ws.workspace_path)?;

        for session_with_path in sessions {
            // Check if session matches the pattern
            let session_id_matches = session_with_path
                .session
                .session_id
                .as_ref()
                .map(|id| id.to_lowercase().contains(&pattern_lower))
                .unwrap_or(false);
            let title_matches = session_with_path
                .session
                .title()
                .to_lowercase()
                .contains(&pattern_lower);
            let content_matches = session_with_path.session.requests.iter().any(|r| {
                r.message
                    .as_ref()
                    .map(|m| {
                        m.text
                            .as_ref()
                            .map(|t| t.to_lowercase().contains(&pattern_lower))
                            .unwrap_or(false)
                    })
                    .unwrap_or(false)
            });

            if !session_id_matches && !title_matches && !content_matches {
                continue;
            }

            let modified = session_with_path
                .path
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .map(|t| {
                    let datetime: chrono::DateTime<chrono::Utc> = t.into();
                    datetime.format("%Y-%m-%d %H:%M").to_string()
                })
                .unwrap_or_else(|| "unknown".to_string());

            rows.push(SessionRow {
                project_path: ws
                    .project_path
                    .clone()
                    .unwrap_or_else(|| "(none)".to_string()),
                session_file: session_with_path
                    .path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "unknown".to_string()),
                last_modified: modified,
                messages: session_with_path.session.request_count(),
            });
        }
    }

    if rows.is_empty() {
        println!("No sessions found matching '{}'", pattern);
        return Ok(());
    }

    let table = Table::new(&rows).with(Style::ascii_rounded()).to_string();

    println!("{}", table);
    println!(
        "\n{} Found {} matching session(s)",
        "[=]".blue(),
        rows.len().to_string().yellow()
    );

    Ok(())
}

/// Optimized session search with filtering
///
/// This function is optimized for speed by:
/// 1. Filtering workspaces first (by name/path)
/// 2. Filtering by file modification date before reading content
/// 3. Only parsing JSON when needed
/// 4. Content search is opt-in (expensive)
/// 5. Parallel file scanning with rayon
pub fn find_sessions_filtered(
    pattern: &str,
    workspace_filter: Option<&str>,
    title_only: bool,
    search_content: bool,
    after: Option<&str>,
    before: Option<&str>,
    limit: usize,
) -> Result<()> {
    use chrono::{NaiveDate, Utc};
    use rayon::prelude::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let pattern_lower = pattern.to_lowercase();

    // Parse date filters upfront
    let after_date = after.and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
    let before_date = before.and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    // Get workspace storage path directly - avoid full discovery if filtering
    let storage_path = crate::workspace::get_workspace_storage_path()?;
    if !storage_path.exists() {
        println!("No workspaces found");
        return Ok(());
    }

    // Collect workspace directories with minimal I/O
    let ws_filter_lower = workspace_filter.map(|s| s.to_lowercase());

    let workspace_dirs: Vec<_> = std::fs::read_dir(&storage_path)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter_map(|entry| {
            let workspace_dir = entry.path();
            let workspace_json_path = workspace_dir.join("workspace.json");

            // Quick check: does chatSessions exist?
            let chat_sessions_dir = workspace_dir.join("chatSessions");
            if !chat_sessions_dir.exists() {
                return None;
            }

            // Parse workspace.json for project path (needed for filtering)
            let project_path =
                std::fs::read_to_string(&workspace_json_path)
                    .ok()
                    .and_then(|content| {
                        serde_json::from_str::<crate::models::WorkspaceJson>(&content)
                            .ok()
                            .and_then(|ws| {
                                ws.folder
                                    .map(|f| crate::workspace::decode_workspace_folder(&f))
                            })
                    });

            // Apply workspace filter early
            if let Some(ref filter) = ws_filter_lower {
                let hash = entry.file_name().to_string_lossy().to_lowercase();
                let path_matches = project_path
                    .as_ref()
                    .map(|p| p.to_lowercase().contains(filter))
                    .unwrap_or(false);
                if !hash.contains(filter) && !path_matches {
                    return None;
                }
            }

            let ws_name = project_path
                .as_ref()
                .and_then(|p| std::path::Path::new(p).file_name())
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| {
                    entry.file_name().to_string_lossy()[..8.min(entry.file_name().len())]
                        .to_string()
                });

            Some((chat_sessions_dir, ws_name))
        })
        .collect();

    if workspace_dirs.is_empty() {
        if let Some(ws) = workspace_filter {
            println!("No workspaces found matching '{}'", ws);
        } else {
            println!("No workspaces with chat sessions found");
        }
        return Ok(());
    }

    // Collect all session file paths
    let session_files: Vec<_> = workspace_dirs
        .iter()
        .flat_map(|(chat_dir, ws_name)| {
            std::fs::read_dir(chat_dir)
                .into_iter()
                .flatten()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path()
                        .extension()
                        .map(|ext| ext == "json")
                        .unwrap_or(false)
                })
                .map(|e| (e.path(), ws_name.clone()))
                .collect::<Vec<_>>()
        })
        .collect();

    let total_files = session_files.len();
    let scanned = AtomicUsize::new(0);
    let skipped_by_date = AtomicUsize::new(0);

    // Process files in parallel
    let mut results: Vec<_> = session_files
        .par_iter()
        .filter_map(|(path, ws_name)| {
            // Date filter using file metadata (very fast)
            if after_date.is_some() || before_date.is_some() {
                if let Ok(metadata) = path.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        let file_date: chrono::DateTime<Utc> = modified.into();
                        let file_naive = file_date.date_naive();

                        if let Some(after) = after_date {
                            if file_naive < after {
                                skipped_by_date.fetch_add(1, Ordering::Relaxed);
                                return None;
                            }
                        }
                        if let Some(before) = before_date {
                            if file_naive > before {
                                skipped_by_date.fetch_add(1, Ordering::Relaxed);
                                return None;
                            }
                        }
                    }
                }
            }

            scanned.fetch_add(1, Ordering::Relaxed);

            // Read file content once
            let content = match std::fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => return None,
            };

            // Extract title from content
            let title =
                extract_title_from_content(&content).unwrap_or_else(|| "Untitled".to_string());
            let title_lower = title.to_lowercase();

            // Check session ID from filename
            let session_id = path
                .file_stem()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let id_matches =
                !pattern_lower.is_empty() && session_id.to_lowercase().contains(&pattern_lower);

            // Check title match
            let title_matches = !pattern_lower.is_empty() && title_lower.contains(&pattern_lower);

            // Content search if requested
            let content_matches = if search_content
                && !title_only
                && !id_matches
                && !title_matches
                && !pattern_lower.is_empty()
            {
                content.to_lowercase().contains(&pattern_lower)
            } else {
                false
            };

            // Empty pattern matches everything (for listing)
            let matches =
                pattern_lower.is_empty() || id_matches || title_matches || content_matches;
            if !matches {
                return None;
            }

            let match_type = if pattern_lower.is_empty() {
                ""
            } else if id_matches {
                "ID"
            } else if title_matches {
                "title"
            } else {
                "content"
            };

            // Count messages from content (already loaded)
            let message_count = content.matches("\"message\":").count();

            // Get modification time
            let modified = path
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .map(|t| {
                    let datetime: chrono::DateTime<chrono::Utc> = t.into();
                    datetime.format("%Y-%m-%d %H:%M").to_string()
                })
                .unwrap_or_else(|| "unknown".to_string());

            Some((
                title,
                ws_name.clone(),
                modified,
                message_count,
                match_type.to_string(),
            ))
        })
        .collect();

    let scanned_count = scanned.load(Ordering::Relaxed);
    let skipped_count = skipped_by_date.load(Ordering::Relaxed);

    if results.is_empty() {
        println!("No sessions found matching '{}'", pattern);
        if skipped_count > 0 {
            println!("  ({} sessions skipped due to date filter)", skipped_count);
        }
        return Ok(());
    }

    // Sort by modification date (newest first)
    results.sort_by(|a, b| b.2.cmp(&a.2));

    // Apply limit
    results.truncate(limit);

    #[derive(Tabled)]
    struct SearchResultRow {
        #[tabled(rename = "Title")]
        title: String,
        #[tabled(rename = "Workspace")]
        workspace: String,
        #[tabled(rename = "Modified")]
        modified: String,
        #[tabled(rename = "Msgs")]
        messages: usize,
        #[tabled(rename = "Match")]
        match_type: String,
    }

    let rows: Vec<SearchResultRow> = results
        .into_iter()
        .map(
            |(title, workspace, modified, messages, match_type)| SearchResultRow {
                title: truncate_string(&title, 40),
                workspace: truncate_string(&workspace, 20),
                modified,
                messages,
                match_type,
            },
        )
        .collect();

    let table = Table::new(&rows).with(Style::ascii_rounded()).to_string();

    println!("{}", table);
    println!(
        "\nFound {} session(s) (scanned {} of {} files{})",
        rows.len(),
        scanned_count,
        total_files,
        if skipped_count > 0 {
            format!(", {} skipped by date", skipped_count)
        } else {
            String::new()
        }
    );
    if rows.len() >= limit {
        println!("  (results limited to {}; use --limit to show more)", limit);
    }

    Ok(())
}

/// Extract title from full JSON content (more reliable than header-only)
fn extract_title_from_content(content: &str) -> Option<String> {
    // Look for "customTitle" first (user-set title)
    if let Some(start) = content.find("\"customTitle\"") {
        if let Some(colon) = content[start..].find(':') {
            let after_colon = &content[start + colon + 1..];
            let trimmed = after_colon.trim_start();
            if let Some(stripped) = trimmed.strip_prefix('"') {
                if let Some(end) = stripped.find('"') {
                    let title = &stripped[..end];
                    if !title.is_empty() && title != "null" {
                        return Some(title.to_string());
                    }
                }
            }
        }
    }

    // Fall back to first request's message text
    if let Some(start) = content.find("\"text\"") {
        if let Some(colon) = content[start..].find(':') {
            let after_colon = &content[start + colon + 1..];
            let trimmed = after_colon.trim_start();
            if let Some(stripped) = trimmed.strip_prefix('"') {
                if let Some(end) = stripped.find('"') {
                    let title = &stripped[..end];
                    if !title.is_empty() && title.len() < 100 {
                        return Some(title.to_string());
                    }
                }
            }
        }
    }

    None
}

/// Fast title extraction from JSON header
#[allow(dead_code)]
fn extract_title_fast(header: &str) -> Option<String> {
    extract_title_from_content(header)
}

/// Truncate string to max length with ellipsis
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Show workspace details
pub fn show_workspace(workspace: &str) -> Result<()> {
    use colored::Colorize;

    let workspaces = discover_workspaces()?;
    let workspace_lower = workspace.to_lowercase();

    // Find workspace by name or hash
    let matching: Vec<&Workspace> = workspaces
        .iter()
        .filter(|ws| {
            ws.hash.to_lowercase().contains(&workspace_lower)
                || ws
                    .project_path
                    .as_ref()
                    .map(|p| p.to_lowercase().contains(&workspace_lower))
                    .unwrap_or(false)
        })
        .collect();

    if matching.is_empty() {
        println!(
            "{} No workspace found matching '{}'",
            "!".yellow(),
            workspace
        );
        return Ok(());
    }

    for ws in matching {
        println!("\n{}", "=".repeat(60).bright_blue());
        println!("{}", "Workspace Details".bright_blue().bold());
        println!("{}", "=".repeat(60).bright_blue());

        println!("{}: {}", "Hash".bright_white().bold(), ws.hash);
        println!(
            "{}: {}",
            "Path".bright_white().bold(),
            ws.project_path.as_ref().unwrap_or(&"(none)".to_string())
        );
        println!(
            "{}: {}",
            "Has Sessions".bright_white().bold(),
            if ws.has_chat_sessions {
                "Yes".green()
            } else {
                "No".red()
            }
        );
        println!(
            "{}: {}",
            "Workspace Path".bright_white().bold(),
            ws.workspace_path.display()
        );

        if ws.has_chat_sessions {
            let sessions = crate::workspace::get_chat_sessions_from_workspace(&ws.workspace_path)?;
            println!(
                "{}: {}",
                "Session Count".bright_white().bold(),
                sessions.len()
            );

            if !sessions.is_empty() {
                println!("\n{}", "Sessions:".bright_yellow());
                for (i, s) in sessions.iter().enumerate() {
                    let title = s.session.title();
                    let msg_count = s.session.request_count();
                    println!(
                        "  {}. {} ({} messages)",
                        i + 1,
                        title.bright_cyan(),
                        msg_count
                    );
                }
            }
        }
    }

    Ok(())
}

/// Show session details
pub fn show_session(session_id: &str, project_path: Option<&str>) -> Result<()> {
    use colored::Colorize;

    let workspaces = discover_workspaces()?;
    let session_id_lower = session_id.to_lowercase();

    let filtered_workspaces: Vec<&Workspace> = if let Some(path) = project_path {
        let normalized = crate::workspace::normalize_path(path);
        workspaces
            .iter()
            .filter(|ws| {
                ws.project_path
                    .as_ref()
                    .map(|p| crate::workspace::normalize_path(p) == normalized)
                    .unwrap_or(false)
            })
            .collect()
    } else {
        workspaces.iter().collect()
    };

    for ws in filtered_workspaces {
        if !ws.has_chat_sessions {
            continue;
        }

        let sessions = crate::workspace::get_chat_sessions_from_workspace(&ws.workspace_path)?;

        for s in sessions {
            let filename = s
                .path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            let matches = s
                .session
                .session_id
                .as_ref()
                .map(|id| id.to_lowercase().contains(&session_id_lower))
                .unwrap_or(false)
                || filename.to_lowercase().contains(&session_id_lower);

            if matches {
                println!("\n{}", "=".repeat(60).bright_blue());
                println!("{}", "Session Details".bright_blue().bold());
                println!("{}", "=".repeat(60).bright_blue());

                println!(
                    "{}: {}",
                    "Title".bright_white().bold(),
                    s.session.title().bright_cyan()
                );
                println!("{}: {}", "File".bright_white().bold(), filename);
                println!(
                    "{}: {}",
                    "Session ID".bright_white().bold(),
                    s.session
                        .session_id
                        .as_ref()
                        .unwrap_or(&"(none)".to_string())
                );
                println!(
                    "{}: {}",
                    "Messages".bright_white().bold(),
                    s.session.request_count()
                );
                println!(
                    "{}: {}",
                    "Workspace".bright_white().bold(),
                    ws.project_path.as_ref().unwrap_or(&"(none)".to_string())
                );

                // Show first few messages as preview
                println!("\n{}", "Preview:".bright_yellow());
                for (i, req) in s.session.requests.iter().take(3).enumerate() {
                    if let Some(msg) = &req.message {
                        if let Some(text) = &msg.text {
                            let preview: String = text.chars().take(100).collect();
                            let truncated = if text.len() > 100 { "..." } else { "" };
                            println!("  {}. {}{}", i + 1, preview.dimmed(), truncated);
                        }
                    }
                }

                return Ok(());
            }
        }
    }

    println!(
        "{} No session found matching '{}'",
        "!".yellow(),
        session_id
    );
    Ok(())
}
