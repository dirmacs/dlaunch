//! tmux integration — availability check, auto-install, session management.

use anyhow::{bail, Result};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

// ── availability ──────────────────────────────────────────────────────────────

pub fn is_available() -> bool {
    crate::command_exists("tmux")
}

/// Returns true when a tmux session named `name` is currently running.
pub fn has_session(name: &str) -> bool {
    Command::new("tmux")
        .args(["has-session", "-t", name])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

// ── installation ──────────────────────────────────────────────────────────────

/// Auto-install tmux via the platform package manager (macOS or Debian/Ubuntu).
pub fn install() -> Result<()> {
    if cfg!(target_os = "macos") {
        if crate::command_exists("brew") {
            println!("Installing tmux via Homebrew...");
            run("brew", &["install", "tmux"])?;
        } else {
            bail!(
                "Homebrew is not installed.\n\
                 Install it from https://brew.sh/ then run: brew install tmux"
            );
        }
    } else if std::path::Path::new("/etc/debian_version").exists() {
        println!("Installing tmux via apt...");
        run("sudo", &["apt-get", "install", "-y", "tmux"])?;
    } else {
        bail!(
            "Cannot auto-install tmux on this platform.\n\
             Please install it manually: https://github.com/tmux/tmux"
        );
    }
    Ok(())
}

// ── session launch ────────────────────────────────────────────────────────────

/// Attach to the tmux session `session_name` if it exists, or create it running
/// `cmd`.  Uses a temporary config file for mouse support and cursor fixes.
pub fn launch(session_name: &str, cmd: &str) -> Result<()> {
    let conf = write_tmp_config()?;
    let conf_str = conf.to_string_lossy().to_string();

    let status = Command::new("tmux")
        .args([
            "-f",
            &conf_str,
            "new-session",
            "-A",          // attach if session already exists
            "-s",
            session_name,
            cmd,
        ])
        .status()?;

    // Best-effort cleanup of the temp config
    fs::remove_file(&conf).ok();

    if !status.success() {
        bail!("tmux exited with a non-zero status");
    }
    Ok(())
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Write a minimal tmux config to a temp file and return its path.
fn write_tmp_config() -> Result<PathBuf> {
    let mut path = env::temp_dir();
    path.push(format!("dlaunch-tmux-{}.conf", std::process::id()));

    fs::write(
        &path,
        r#"# dlaunch temporary tmux config
set -g mouse on
set -ga terminal-overrides ',*:Ss=\E[%p1%d q:Se=\E[2 q'
set -ga terminal-overrides ',xterm*:Cr=\E]12;gray\007'
set -ga terminal-overrides ',screen*:Cr=\E]12;gray\007'
set -ga terminal-overrides ',tmux*:Cr=\E]12;gray\007'
set -ga terminal-overrides ',*:Cs=\E[4 q:Ce=\E[2 q'
set -as terminal-overrides ',*:Smulx=\E[4::%p1%dm'
set -g default-terminal "screen-256color"
set -ga terminal-features "*:RGB"
set -g focus-events on
"#,
    )?;

    Ok(path)
}

fn run(cmd: &str, args: &[&str]) -> Result<()> {
    let status = Command::new(cmd).args(args).status()?;
    if !status.success() {
        bail!("{} failed (exit code {:?})", cmd, status.code());
    }
    Ok(())
}
