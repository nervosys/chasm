// Copyright (c) 2024-2026 Nervosys LLC
// SPDX-License-Identifier: Apache-2.0
//! Workspace listing commands

use anyhow::Result;
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
    sessions: usize,
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
        println!("No workspaces found.");
        return Ok(());
    }

    let rows: Vec<WorkspaceRow> = workspaces
        .iter()
        .map(|ws| WorkspaceRow {
            hash: format!("{}...", &ws.hash[..12.min(ws.hash.len())]),
            project_path: ws
                .project_path
                .clone()
                .unwrap_or_else(|| "(none)".to_string()),
            sessions: ws.chat_session_count,
            has_chats: if ws.has_chat_sessions {
                "Yes".to_string()
            } else {
                "No".to_string()
            },
        })
        .collect();

    let table = Table::new(rows).with(Style::ascii_rounded()).to_string();

    println!("{}", table);
    println!("\nTotal workspaces: {}", workspaces.len());

    // Show empty window sessions count (ALL SESSIONS)
    if let Ok(empty_count) = crate::storage::count_empty_window_sessions() {
        if empty_count > 0 {
            println!("Empty window sessions (ALL SESSIONS): {}", empty_count);
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
        println!("No chat sessions found.");
        return Ok(());
    }

    let table = Table::new(&rows).with(Style::ascii_rounded()).to_string();

    println!("{}", table);
    println!("\nTotal sessions: {}", rows.len());

    Ok(())
}

/// Find workspaces by search pattern
pub fn find_workspaces(pattern: &str) -> Result<()> {
    let workspaces = discover_workspaces()?;
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
        println!("No workspaces found matching '{}'", pattern);
        return Ok(());
    }

    let rows: Vec<WorkspaceRow> = matching
        .iter()
        .map(|ws| WorkspaceRow {
            hash: format!("{}...", &ws.hash[..12.min(ws.hash.len())]),
            project_path: ws
                .project_path
                .clone()
                .unwrap_or_else(|| "(none)".to_string()),
            sessions: ws.chat_session_count,
            has_chats: if ws.has_chat_sessions {
                "Yes".to_string()
            } else {
                "No".to_string()
            },
        })
        .collect();

    let table = Table::new(rows).with(Style::ascii_rounded()).to_string();

    println!("{}", table);
    println!("\nFound {} matching workspace(s)", matching.len());

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
    println!("\nFound {} matching session(s)", rows.len());

    Ok(())
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
