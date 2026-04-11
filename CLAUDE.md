# dlaunch

Persistent Claude CLI session manager. Rust CLI that wraps `claude code --dangerously-skip-permissions` with per-project session persistence and optional tmux support.

## Build

```bash
cargo build --release
cargo test
```

## Install

```bash
cargo install --path .
# or
cp target/release/dlaunch /usr/local/bin/
```

## CLI Commands

```bash
dlaunch                        # start or resume Claude session for current directory
dlaunch --tmux                 # same, inside a persistent tmux window
dlaunch save sess-<id>         # save the session ID Claude printed on startup
dlaunch list                   # list all project sessions + tmux status
dlaunch clean                  # remove session files with no active tmux window
dlaunch session show           # print stored session ID for current project
dlaunch session delete         # delete stored session for current project
```

## Architecture

- `src/main.rs` — CLI entry point, clap definitions, launch logic
- `src/session.rs` — session file CRUD (`~/.claude_session_<project>`), validation
- `src/tmux.rs` — tmux availability, auto-install, session launch
- `src/cmd_list.rs` — `dlaunch list` implementation
- `src/cmd_clean.rs` — `dlaunch clean` implementation

## Session files

Stored at `~/.claude_session_<project>` where `project` is the `basename` of `$PWD`.  
Format: single line, `sess-[a-zA-Z0-9]+`.  
Warning acknowledgement: `~/.dlaunch_warning_acknowledged` (empty marker file).

## Conventions

- Git author: `Abhijay Bharathan <abhijaybharathan@gmail.com>`
- No hardcoded paths — always use `dirs::home_dir()`
- `--dangerously-skip-permissions` is intentional and documented
- Run `cargo test` before every commit
