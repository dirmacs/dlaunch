//! End-to-end tests for the dlaunch launch flow.
//!
//! These tests exercise the full `dlaunch` execution path — checking that the
//! correct arguments are forwarded to the underlying `claude` binary.  A
//! `mock_claude` binary (built alongside dlaunch) is copied into a temp bin
//! directory and placed first on PATH so no real Claude CLI is needed.
//!
//! All tests pre-create `HOME/.dlaunch_warning_acknowledged` to skip the
//! interactive security prompt.

use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

// ── helpers ───────────────────────────────────────────────────────────────────

/// Resolves the compiled `mock_claude[.exe]` binary from the cargo build output.
fn mock_claude_bin() -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let exe = if cfg!(windows) { "mock_claude.exe" } else { "mock_claude" };
    // Prefer debug (default for `cargo test`), fall back to release.
    let debug = manifest.join("target").join("debug").join(exe);
    if debug.exists() {
        return debug;
    }
    manifest.join("target").join("release").join(exe)
}

/// Copy `mock_claude` into `bin_dir` as `claude[.exe]` and make it executable.
fn install_mock_claude(bin_dir: &Path) {
    let src = mock_claude_bin();
    let exe = if cfg!(windows) { "claude.exe" } else { "claude" };
    let dst = bin_dir.join(exe);
    fs::copy(&src, &dst).unwrap_or_else(|e| {
        panic!(
            "Failed to copy mock_claude ({}) to {}: {}",
            src.display(),
            dst.display(),
            e
        )
    });

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&dst, fs::Permissions::from_mode(0o755))
            .expect("chmod mock claude");
    }
}

/// Builds a PATH string with `prepend` in front of the existing PATH.
fn prepend_path(prepend: &Path) -> String {
    let sep = if cfg!(windows) { ";" } else { ":" };
    let existing = std::env::var("PATH").unwrap_or_default();
    format!("{}{}{}", prepend.display(), sep, existing)
}

struct E2EFixture {
    home: TempDir,
    bin_dir: TempDir,
    project: PathBuf,
}

impl E2EFixture {
    fn new(project_name: &str) -> Self {
        let home = tempfile::tempdir().expect("temp home");
        let bin_dir = tempfile::tempdir().expect("temp bin dir");

        // Pre-acknowledge the security warning
        fs::write(home.path().join(".dlaunch_warning_acknowledged"), "")
            .expect("write warning ack");

        // Create the project directory (so the project name is deterministic)
        let project = home.path().join(project_name);
        fs::create_dir_all(&project).expect("create project dir");

        // Install mock claude as "claude" on PATH
        install_mock_claude(bin_dir.path());

        Self { home, bin_dir, project }
    }

    /// Returns a `dlaunch` Command pre-configured with the fixture's HOME, PATH,
    /// and current directory.
    fn cmd(&self) -> Command {
        let mut cmd = Command::cargo_bin("dlaunch").expect("dlaunch binary");
        cmd.env("HOME", self.home.path())
            .env("PATH", prepend_path(self.bin_dir.path()))
            .current_dir(&self.project);
        cmd
    }

    /// Writes a session file so the resume path is exercised.
    fn write_session(&self, id: &str) {
        let project_name = self.project.file_name().unwrap().to_str().unwrap();
        let session_file = self.home.path().join(format!(".claude_session_{}", project_name));
        fs::write(&session_file, format!("{}\n", id)).expect("write session file");
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

/// When no session file exists, dlaunch should start a new session and invoke
/// mock_claude with just `code --dangerously-skip-permissions`.
#[test]
fn new_session_runs_mock_claude() {
    let fix = E2EFixture::new("new-session-proj");

    fix.cmd()
        .assert()
        .success()
        .stdout(contains("No session saved for project"))
        .stdout(contains("mock-claude: code --dangerously-skip-permissions"));
}

/// When a session file exists, dlaunch should resume it by passing `--resume
/// <id>` to mock_claude.
#[test]
fn resume_session_runs_mock_claude() {
    let fix = E2EFixture::new("resume-proj");
    fix.write_session("sess-testid123");

    fix.cmd()
        .assert()
        .success()
        .stdout(contains("Resuming Claude session: sess-testid123"))
        .stdout(contains("mock-claude: code --resume sess-testid123"));
}

/// The new-session path should print the `dlaunch save sess-` hint.
#[test]
fn new_session_shows_save_hint() {
    let fix = E2EFixture::new("hint-proj");

    fix.cmd()
        .assert()
        .success()
        .stdout(contains("dlaunch save sess-"));
}

/// The resume path must always forward `--dangerously-skip-permissions`.
#[test]
fn resume_passes_dangerously_skip_permissions() {
    let fix = E2EFixture::new("flags-proj");
    fix.write_session("sess-abc999");

    fix.cmd()
        .assert()
        .success()
        .stdout(contains("--dangerously-skip-permissions"));
}
