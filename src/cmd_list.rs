//! `dlaunch list` — print all project sessions and their tmux status.

use anyhow::Result;
use crate::{session, tmux};

pub fn run() -> Result<()> {
    let sessions = session::all_sessions();

    if sessions.is_empty() {
        println!("No Claude sessions found.");
        println!("Run `dlaunch` in any project directory to start one.");
        return Ok(());
    }

    println!("Claude sessions:");
    for (project, _path, id) in &sessions {
        let tmux_name = format!("claude-{}", project);
        let tmux_tag = if tmux::has_session(&tmux_name) {
            "tmux: active"
        } else {
            "tmux: inactive"
        };

        match id {
            Some(id) => println!("  {:30} {} ({})", project, id, tmux_tag),
            None     => println!("  {:30} (no session ID)  ({})", project, tmux_tag),
        }
    }

    Ok(())
}
