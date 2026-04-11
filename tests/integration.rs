//! Integration tests for dlaunch CLI subcommands.
//!
//! Each test invokes the compiled `dlaunch` binary with an isolated `HOME`
//! directory so session files never bleed between tests.  The launch path
//! (`dlaunch` with no subcommand) is covered by the e2e suite; these tests
//! focus on the safe, non-interactive subcommands.

use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use tempfile::TempDir;

// ── helpers ───────────────────────────────────────────────────────────────────

/// Returns a fresh temp directory used as the isolated HOME for one test.
fn isolated_home() -> TempDir {
    tempfile::tempdir().expect("create temp HOME")
}

/// Runs `dlaunch <args>` with HOME pointing at `home`.
fn dlaunch(home: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("dlaunch").expect("dlaunch binary");
    cmd.env("HOME", home.path());
    cmd
}

/// Creates a project directory inside `home` and returns its path.
fn project_dir(home: &TempDir, name: &str) -> std::path::PathBuf {
    let dir = home.path().join(name);
    fs::create_dir_all(&dir).expect("create project dir");
    dir
}

/// Path of the session file for `project` inside `home`.
fn session_file(home: &TempDir, project: &str) -> std::path::PathBuf {
    home.path().join(format!(".claude_session_{}", project))
}

// ── version / help ────────────────────────────────────────────────────────────

#[test]
fn version() {
    let home = isolated_home();
    dlaunch(&home)
        .arg("--version")
        .assert()
        .success()
        .stdout(contains("0.1.0"));
}

#[test]
fn help() {
    let home = isolated_home();
    dlaunch(&home)
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("dlaunch"))
        .stdout(contains("tmux"));
}

// ── list ──────────────────────────────────────────────────────────────────────

#[test]
fn list_empty() {
    let home = isolated_home();
    dlaunch(&home)
        .arg("list")
        .assert()
        .success()
        .stdout(contains("No Claude sessions found"));
}

#[test]
fn list_with_sessions() {
    let home = isolated_home();
    // Manually write two session files
    fs::write(session_file(&home, "alpha"), "sess-aaa111\n").unwrap();
    fs::write(session_file(&home, "beta"), "sess-bbb222\n").unwrap();

    let output = dlaunch(&home).arg("list").assert().success();
    output
        .stdout(contains("alpha"))
        .stdout(contains("sess-aaa111"))
        .stdout(contains("beta"))
        .stdout(contains("sess-bbb222"));
}

// ── clean ─────────────────────────────────────────────────────────────────────

#[test]
fn clean_empty() {
    let home = isolated_home();
    dlaunch(&home)
        .arg("clean")
        .assert()
        .success()
        .stdout(contains("No session files"));
}

#[test]
fn clean_removes_orphans() {
    let home = isolated_home();
    // Write two session files; neither has an active tmux session
    let f1 = session_file(&home, "orphan1");
    let f2 = session_file(&home, "orphan2");
    fs::write(&f1, "sess-orph1\n").unwrap();
    fs::write(&f2, "sess-orph2\n").unwrap();

    dlaunch(&home).arg("clean").assert().success();

    assert!(!f1.exists(), "orphan1 session file should be removed");
    assert!(!f2.exists(), "orphan2 session file should be removed");
}

// ── save ──────────────────────────────────────────────────────────────────────

#[test]
fn save_valid() {
    let home = isolated_home();
    let proj = project_dir(&home, "myproject");

    dlaunch(&home)
        .current_dir(&proj)
        .args(["save", "sess-abc123"])
        .assert()
        .success();

    let content = fs::read_to_string(session_file(&home, "myproject")).unwrap();
    assert!(content.contains("sess-abc123"));
}

#[test]
fn save_invalid_no_prefix() {
    let home = isolated_home();
    dlaunch(&home)
        .args(["save", "notvalid"])
        .assert()
        .failure()
        .stderr(contains("Invalid"));
}

#[test]
fn save_invalid_empty_suffix() {
    let home = isolated_home();
    dlaunch(&home)
        .args(["save", "sess-"])
        .assert()
        .failure()
        .stderr(contains("Invalid"));
}

#[test]
fn save_invalid_hyphen_in_suffix() {
    let home = isolated_home();
    dlaunch(&home)
        .args(["save", "sess-abc-def"])
        .assert()
        .failure()
        .stderr(contains("Invalid"));
}

#[test]
fn save_overwrites_previous() {
    let home = isolated_home();
    let proj = project_dir(&home, "overwrite-proj");

    // First save
    dlaunch(&home)
        .current_dir(&proj)
        .args(["save", "sess-first11"])
        .assert()
        .success();

    // Overwrite with second ID
    dlaunch(&home)
        .current_dir(&proj)
        .args(["save", "sess-second22"])
        .assert()
        .success();

    // session show must return the second ID
    dlaunch(&home)
        .current_dir(&proj)
        .args(["session", "show"])
        .assert()
        .success()
        .stdout(contains("sess-second22"));
}

// ── save + list ───────────────────────────────────────────────────────────────

#[test]
fn save_then_list() {
    let home = isolated_home();
    let proj = project_dir(&home, "listproj");

    dlaunch(&home)
        .current_dir(&proj)
        .args(["save", "sess-listtest1"])
        .assert()
        .success();

    dlaunch(&home)
        .arg("list")
        .assert()
        .success()
        .stdout(contains("listproj"))
        .stdout(contains("sess-listtest1"));
}

// ── session show ──────────────────────────────────────────────────────────────

#[test]
fn session_show() {
    let home = isolated_home();
    let proj = project_dir(&home, "showproj");

    dlaunch(&home)
        .current_dir(&proj)
        .args(["save", "sess-show999"])
        .assert()
        .success();

    dlaunch(&home)
        .current_dir(&proj)
        .args(["session", "show"])
        .assert()
        .success()
        .stdout(contains("sess-show999"));
}

#[test]
fn session_show_missing() {
    let home = isolated_home();
    let proj = project_dir(&home, "noproj");

    dlaunch(&home)
        .current_dir(&proj)
        .args(["session", "show"])
        .assert()
        .failure()
        .stderr(contains("No session"));
}

// ── session delete ────────────────────────────────────────────────────────────

#[test]
fn session_delete() {
    let home = isolated_home();
    let proj = project_dir(&home, "deleteproj");

    // Save first
    dlaunch(&home)
        .current_dir(&proj)
        .args(["save", "sess-del123"])
        .assert()
        .success();

    // Delete
    dlaunch(&home)
        .current_dir(&proj)
        .args(["session", "delete"])
        .assert()
        .success()
        .stdout(contains("deleted"));

    // File must be gone
    assert!(
        !session_file(&home, "deleteproj").exists(),
        "session file should be removed after delete"
    );
}

#[test]
fn session_delete_idempotent() {
    let home = isolated_home();
    let proj = project_dir(&home, "noproj2");

    // Delete when no session file exists — should succeed silently
    dlaunch(&home)
        .current_dir(&proj)
        .args(["session", "delete"])
        .assert()
        .success();
}
