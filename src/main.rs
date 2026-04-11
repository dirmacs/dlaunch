//! dlaunch — Persistent Claude CLI session manager.
//!
//! Maintains a separate Claude session per project directory, so you never lose
//! context between terminal sessions. Supports optional tmux integration for
//! sessions that survive terminal crashes and restarts.
//!
//! ## Usage
//!
//! ```bash
//! # Start or resume the Claude session for the current directory
//! dlaunch
//!
//! # Same, but inside a tmux window (survives crashes/reboots)
//! dlaunch --tmux
//!
//! # Save the session ID Claude printed on startup
//! dlaunch save sess-abc123def456
//!
//! # List all project sessions
//! dlaunch list
//!
//! # Remove session files whose tmux window is no longer running
//! dlaunch clean
//! ```

mod cmd_clean;
mod cmd_list;
mod session;
mod tmux;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::env;
use std::io::{self, Write as IoWrite};
use std::process;

#[derive(Parser)]
#[command(
    name = "dlaunch",
    version,
    about = "Persistent Claude CLI session manager — per-project sessions with optional tmux"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Run inside tmux — session survives terminal crashes and system restarts
    #[arg(long)]
    tmux: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// List all project sessions and their tmux status
    List,

    /// Remove session files that have no active tmux window
    Clean,

    /// Save a session ID for the current project directory
    Save {
        /// Session ID printed by Claude on startup (format: sess-<alphanumeric>)
        id: String,
    },

    /// Show or delete the session for the current project
    Session {
        #[command(subcommand)]
        action: SessionAction,
    },
}

#[derive(Subcommand)]
enum SessionAction {
    /// Show the stored session ID for the current project
    Show,
    /// Delete the stored session for the current project
    Delete,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::List) => cmd_list::run(),
        Some(Commands::Clean) => cmd_clean::run(),
        Some(Commands::Save { id }) => {
            let project = current_project()?;
            session::save(&project, &id)?;
            println!("Session saved for project '{}': {}", project, id);
            Ok(())
        }
        Some(Commands::Session { action }) => match action {
            SessionAction::Show => {
                let project = current_project()?;
                match session::get_session_id(&project) {
                    Some(id) => println!("{}", id),
                    None => {
                        eprintln!("No session saved for project '{}'.", project);
                        process::exit(1);
                    }
                }
                Ok(())
            }
            SessionAction::Delete => {
                let project = current_project()?;
                session::delete(&project)?;
                println!("Session deleted for project '{}'.", project);
                Ok(())
            }
        },
        None => launch(cli.tmux),
    }
}

// ── helpers ──────────────────────────────────────────────────────────────────

/// Returns the base name of the current working directory.
fn current_project() -> Result<String> {
    let cwd = env::current_dir()?;
    let name = cwd
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "default".to_string());
    Ok(name)
}

/// Returns true when `name` is found on PATH.
pub fn command_exists(name: &str) -> bool {
    process::Command::new("sh")
        .args(["-c", &format!("command -v {} >/dev/null 2>&1", name)])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

// ── launch ────────────────────────────────────────────────────────────────────

fn launch(use_tmux: bool) -> Result<()> {
    // Require Claude CLI
    if !command_exists("claude") {
        eprintln!("Error: claude CLI not found.");
        eprintln!("Install it from: https://docs.anthropic.com/en/docs/claude-code");
        process::exit(1);
    }

    // First-run security acknowledgement
    if !session::is_warning_acknowledged() {
        eprintln!("WARNING: dlaunch invokes Claude with --dangerously-skip-permissions.");
        eprintln!("Claude will have full read/write access to your file system without");
        eprintln!("confirmation prompts. Only use this in trusted project directories.");
        eprintln!();
        eprint!("Continue? (y/N): ");
        io::stderr().flush()?;

        let mut answer = String::new();
        io::stdin().read_line(&mut answer)?;
        if !matches!(answer.trim(), "y" | "Y") {
            eprintln!("Aborted.");
            process::exit(1);
        }
        session::acknowledge_warning()?;
        eprintln!();
    }

    let project = current_project()?;

    // Auto-install tmux when requested but missing
    if use_tmux && !tmux::is_available() {
        println!("tmux not found — attempting installation...");
        tmux::install()?;
    }

    match session::get_session_id(&project) {
        Some(id) => resume_session(&project, &id, use_tmux),
        None => new_session(&project, use_tmux),
    }
}

fn resume_session(project: &str, id: &str, use_tmux: bool) -> Result<()> {
    let tmux_name = format!("claude-{}", project);

    if use_tmux {
        println!("Resuming Claude session '{}' in tmux window '{}'.", id, tmux_name);
        let cmd = format!(
            "claude code --resume {} --dangerously-skip-permissions",
            id
        );
        tmux::launch(&tmux_name, &cmd)
    } else {
        println!("Resuming Claude session: {}", id);
        exec_claude(&["code", "--resume", id, "--dangerously-skip-permissions"])
    }
}

fn new_session(project: &str, use_tmux: bool) -> Result<()> {
    let session_file = session::session_file_path(project);
    let tmux_name = format!("claude-{}", project);

    println!("No session saved for project '{}'.", project);
    println!("Starting a new Claude session...");
    println!();
    println!("Once Claude starts, find the session ID in its output and save it:");
    println!("  dlaunch save sess-<id>");
    println!("  # or: echo \"sess-<id>\" > {}", session_file.display());
    println!();

    if use_tmux {
        tmux::launch(
            &tmux_name,
            "claude code --dangerously-skip-permissions",
        )
    } else {
        exec_claude(&["code", "--dangerously-skip-permissions"])
    }
}

/// Replace the current process with `claude <args>` (Unix) or wait for it (other).
fn exec_claude(args: &[&str]) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let err = process::Command::new("claude").args(args).exec();
        return Err(anyhow::anyhow!("Failed to exec claude: {}", err));
    }
    #[cfg(not(unix))]
    {
        let status = process::Command::new("claude").args(args).status()?;
        process::exit(status.code().unwrap_or(1));
    }
}
