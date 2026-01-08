//! CLI Integration Tests
//!
//! Tests that verify the actual CLI binary works correctly.
//! These tests run the compiled `csm` binary and check output.

use assert_cmd::Command;
use predicates::prelude::*;

// =============================================================================
// Helper Functions
// =============================================================================

#[allow(deprecated)] // cargo_bin is still the standard way to test CLI binaries
fn csm_cmd() -> Command {
    Command::cargo_bin("chasm").unwrap()
}

// =============================================================================
// Basic CLI Tests
// =============================================================================

mod basic_cli {
    use super::*;

    #[test]
    fn test_help_flag() {
        csm_cmd()
            .arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("Manage and merge chat sessions"))
            .stdout(predicate::str::contains("Usage:"));
    }

    #[test]
    fn test_version_flag() {
        csm_cmd()
            .arg("--version")
            .assert()
            .success()
            .stdout(predicate::str::contains("csm"));
    }

    #[test]
    fn test_no_args_shows_help() {
        csm_cmd()
            .assert()
            .failure()
            .stderr(predicate::str::contains("Usage:"));
    }
}

// =============================================================================
// List Command Tests
// =============================================================================

mod list_commands {
    use super::*;

    #[test]
    fn test_list_workspaces_help() {
        csm_cmd()
            .args(["list", "workspaces", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("List all VS Code workspaces"));
    }

    #[test]
    fn test_list_sessions_help() {
        csm_cmd()
            .args(["list", "sessions", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("sessions"));
    }

    #[test]
    fn test_list_help() {
        csm_cmd()
            .args(["list", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("List"));
    }
}

// =============================================================================
// Provider Command Tests
// =============================================================================

mod provider_commands {
    use super::*;

    #[test]
    fn test_provider_list_help() {
        csm_cmd()
            .args(["provider", "list", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("providers"));
    }

    #[test]
    fn test_provider_list_runs() {
        csm_cmd()
            .args(["provider", "list"])
            .assert()
            .success()
            .stdout(predicate::str::contains("LLM Providers"));
    }

    #[test]
    fn test_provider_info_help() {
        csm_cmd()
            .args(["provider", "info", "--help"])
            .assert()
            .success();
    }

    #[test]
    fn test_provider_info_ollama() {
        csm_cmd()
            .args(["provider", "info", "ollama"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Ollama"));
    }

    #[test]
    fn test_provider_info_invalid() {
        csm_cmd()
            .args(["provider", "info", "invalid-provider"])
            .assert()
            .failure();
    }
}

// =============================================================================
// Detect Command Tests
// =============================================================================

mod detect_commands {
    use super::*;

    #[test]
    fn test_detect_help() {
        csm_cmd()
            .args(["detect", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Detect"));
    }

    #[test]
    fn test_detect_providers_help() {
        csm_cmd()
            .args(["detect", "providers", "--help"])
            .assert()
            .success();
    }

    #[test]
    fn test_detect_providers_runs() {
        csm_cmd()
            .args(["detect", "providers"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Detecting"));
    }
}

// =============================================================================
// Harvest Command Tests
// =============================================================================

mod harvest_commands {
    use super::*;

    #[test]
    fn test_harvest_help() {
        csm_cmd()
            .args(["harvest", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("harvest"));
    }

    #[test]
    fn test_harvest_scan_help() {
        csm_cmd()
            .args(["harvest", "scan", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Scan"))
            .stdout(predicate::str::contains("--web"))
            .stdout(predicate::str::contains("--timeout"));
    }

    #[test]
    fn test_harvest_scan_runs() {
        csm_cmd()
            .args(["harvest", "scan"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Scanning"))
            .stdout(predicate::str::contains("Summary"));
    }

    #[test]
    fn test_harvest_init_help() {
        csm_cmd()
            .args(["harvest", "init", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Initialize"));
    }

    #[test]
    fn test_harvest_status_help() {
        csm_cmd()
            .args(["harvest", "status", "--help"])
            .assert()
            .success();
    }

    #[test]
    fn test_harvest_run_help() {
        csm_cmd()
            .args(["harvest", "run", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Run"));
    }

    #[test]
    fn test_harvest_git_help() {
        csm_cmd()
            .args(["harvest", "git", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Git"));
    }
}

// =============================================================================
// Export/Import Command Tests
// =============================================================================

mod export_import_commands {
    use super::*;

    #[test]
    fn test_export_help() {
        csm_cmd()
            .args(["export", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Export"));
    }

    #[test]
    fn test_import_help() {
        csm_cmd()
            .args(["import", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Import"));
    }
}

// =============================================================================
// Find Command Tests
// =============================================================================

mod find_commands {
    use super::*;

    #[test]
    fn test_find_help() {
        csm_cmd()
            .args(["find", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Search"));
    }

    #[test]
    fn test_find_workspace_help() {
        csm_cmd()
            .args(["find", "workspace", "--help"])
            .assert()
            .success();
    }

    #[test]
    fn test_find_session_help() {
        csm_cmd()
            .args(["find", "session", "--help"])
            .assert()
            .success();
    }
}

// =============================================================================
// Merge Command Tests
// =============================================================================

mod merge_commands {
    use super::*;

    #[test]
    fn test_merge_help() {
        csm_cmd()
            .args(["merge", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Merge"));
    }

    #[test]
    fn test_merge_workspaces_help() {
        csm_cmd()
            .args(["merge", "workspaces", "--help"])
            .assert()
            .success();
    }

    #[test]
    fn test_merge_sessions_help() {
        csm_cmd()
            .args(["merge", "sessions", "--help"])
            .assert()
            .success();
    }
}

// =============================================================================
// Git Command Tests
// =============================================================================

mod git_commands {
    use super::*;

    #[test]
    fn test_git_help() {
        csm_cmd()
            .args(["git", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Git"));
    }

    #[test]
    fn test_git_init_help() {
        csm_cmd().args(["git", "init", "--help"]).assert().success();
    }

    #[test]
    fn test_git_snapshot_help() {
        csm_cmd()
            .args(["git", "snapshot", "--help"])
            .assert()
            .success();
    }

    #[test]
    fn test_git_status_help() {
        csm_cmd()
            .args(["git", "status", "--help"])
            .assert()
            .success();
    }
}

// =============================================================================
// TUI Command Tests
// =============================================================================

mod tui_commands {
    use super::*;

    #[test]
    fn test_run_help() {
        csm_cmd()
            .args(["run", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("TUI"));
    }
}

// =============================================================================
// Error Handling Tests
// =============================================================================

mod error_handling {
    use super::*;

    #[test]
    fn test_invalid_command() {
        csm_cmd()
            .arg("invalid-command-that-does-not-exist")
            .assert()
            .failure();
    }

    #[test]
    fn test_provider_info_missing_arg() {
        csm_cmd().args(["provider", "info"]).assert().failure();
    }
}
