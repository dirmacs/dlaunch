//! `dlaunch clean` — remove session files with no active tmux window.

use anyhow::Result;
use crate::{session, tmux};
use std::fs;

pub fn run() -> Result<()> {
    let sessions = session::all_sessions();

    if sessions.is_empty() {
        println!("No session files to clean up.");
        return Ok(());
    }

    println!("Scanning {} session file(s)...", sessions.len());
    let mut removed = 0;

    for (project, path, _id) in &sessions {
        let tmux_name = format!("claude-{}", project);
        if tmux::has_session(&tmux_name) {
            println!("  Keeping  {} (tmux session is live)", project);
        } else {
            println!("  Removing {} — {}", project, path.display());
            fs::remove_file(path)?;
            removed += 1;
        }
    }

    println!();
    if removed == 0 {
        println!("Nothing to remove — all sessions have an active tmux window.");
    } else {
        println!("Removed {} orphaned session file(s).", removed);
    }

    Ok(())
}
