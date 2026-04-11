//! Session file management.
//!
//! Session state is stored in `~/.claude_session_<project>` — one file per
//! project directory name. The file contains a single line with the session ID
//! printed by Claude on startup (format: `sess-<alphanumeric>`).

use anyhow::Result;
use dirs::home_dir;
use std::fs;
use std::path::PathBuf;

// ── paths ─────────────────────────────────────────────────────────────────────

fn home() -> PathBuf {
    home_dir().unwrap_or_else(|| PathBuf::from("/tmp"))
}

/// `~/.claude_session_<project>`
pub fn session_file_path(project: &str) -> PathBuf {
    home().join(format!(".claude_session_{}", project))
}

/// `~/.dlaunch_warning_acknowledged`
fn warning_ack_path() -> PathBuf {
    home().join(".dlaunch_warning_acknowledged")
}

// ── warning acknowledgement ───────────────────────────────────────────────────

pub fn is_warning_acknowledged() -> bool {
    warning_ack_path().exists()
}

pub fn acknowledge_warning() -> Result<()> {
    fs::write(warning_ack_path(), "")?;
    Ok(())
}

// ── session CRUD ──────────────────────────────────────────────────────────────

/// Read the stored session ID for `project`, if valid.
pub fn get_session_id(project: &str) -> Option<String> {
    let raw = fs::read_to_string(session_file_path(project)).ok()?;
    let id = raw.trim().to_string();
    if id.is_empty() {
        eprintln!(
            "Warning: session file for '{}' is empty — run `dlaunch save <id>` to populate it.",
            project
        );
        return None;
    }
    if !validate_session_id(&id) {
        eprintln!(
            "Warning: session ID '{}' has an unexpected format (expected sess-<alphanumeric>).",
            id
        );
        eprintln!("Delete the file or run `dlaunch save <id>` to replace it.");
        return None;
    }
    Some(id)
}

/// Write `id` to `~/.claude_session_<project>`.
pub fn save(project: &str, id: &str) -> Result<()> {
    if !validate_session_id(id) {
        anyhow::bail!(
            "Invalid session ID '{}'. Expected format: sess-<alphanumeric>",
            id
        );
    }
    fs::write(session_file_path(project), format!("{}\n", id))?;
    Ok(())
}

/// Delete the session file for `project`.
pub fn delete(project: &str) -> Result<()> {
    let path = session_file_path(project);
    if path.exists() {
        fs::remove_file(&path)?;
    }
    Ok(())
}

/// Scan `~` for all `.claude_session_*` files.
/// Returns `(project_name, file_path, Option<session_id>)`.
pub fn all_sessions() -> Vec<(String, PathBuf, Option<String>)> {
    let mut sessions = Vec::new();
    let Ok(entries) = fs::read_dir(home()) else {
        return sessions;
    };
    for entry in entries.flatten() {
        let file_name = entry.file_name().to_string_lossy().to_string();
        if let Some(project) = file_name.strip_prefix(".claude_session_") {
            let path = entry.path();
            let id = fs::read_to_string(&path)
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty() && validate_session_id(s));
            sessions.push((project.to_string(), path, id));
        }
    }
    sessions.sort_by(|a, b| a.0.cmp(&b.0));
    sessions
}

// ── validation ────────────────────────────────────────────────────────────────

/// `sess-[a-zA-Z0-9]+`
pub fn validate_session_id(id: &str) -> bool {
    id.starts_with("sess-")
        && id.len() > 5
        && id[5..].chars().all(|c| c.is_ascii_alphanumeric())
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_session_ids() {
        assert!(validate_session_id("sess-abc123"));
        assert!(validate_session_id("sess-ABC123abc"));
        assert!(validate_session_id("sess-a"));
    }

    #[test]
    fn invalid_session_ids() {
        assert!(!validate_session_id("sess-"));          // empty suffix
        assert!(!validate_session_id("sess-abc-def"));   // hyphen in suffix
        assert!(!validate_session_id("session-abc123")); // wrong prefix
        assert!(!validate_session_id(""));               // empty
        assert!(!validate_session_id("abc123"));         // no prefix
        assert!(!validate_session_id("sess-abc 123"));   // space
    }

    #[test]
    fn session_file_path_format() {
        let path = session_file_path("myproject");
        let name = path.file_name().unwrap().to_str().unwrap();
        assert_eq!(name, ".claude_session_myproject");
    }
}
