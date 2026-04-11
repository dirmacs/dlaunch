# dlaunch — Agent Guidelines

## What This Project Is

dlaunch is a single-binary Rust CLI that gives every project directory its own persistent Claude session. It wraps `claude code --dangerously-skip-permissions` and stores session IDs in `~/.claude_session_<project>`. Optional tmux integration keeps sessions alive across terminal restarts.

## For Agents Working On This Codebase

### Before Making Changes

1. Run `cargo test` — all tests must pass
2. Run `cargo clippy` — no new warnings
3. Run `cargo build --release` — binary must compile cleanly

### Key Design Decisions

- **Single crate** — no workspace, no library split. All logic lives in `src/`
- **No async** — everything is synchronous `std::process::Command` calls
- **No regex dependency** — session ID validation is done with simple string checks
- **`exec` on Unix** — `dlaunch` replaces itself with `claude` so there's no wrapper process in the process tree
- **`dirs::home_dir()`** — never hardcode `~` or `/home/user`
- **Temp tmux config** — written to `$TMPDIR/dlaunch-tmux-<pid>.conf` and deleted after the tmux session exits

### What NOT To Do

- Don't add async/tokio — not needed for a launcher
- Don't add a config file — all state is in `~/.claude_session_*` files
- Don't hardcode `/home`, `/root`, or any user paths
- Don't skip the security warning acknowledgement flow
- Don't commit without running `cargo test`

### Subagent Patterns

- Research agents: read code, grep patterns, check tests
- Implementation: always done in main context
- Test after every change: `cargo test`
- Commit with: `git -c user.name="Abhijay Bharathan" -c user.email="abhijaybharathan@gmail.com" commit`
