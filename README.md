<h1 align="center">dlaunch</h1>

<p align="center">
  Persistent Claude CLI session manager — one session per project, zero context loss.
</p>

<p align="center">
  <a href="https://crates.io/crates/dlaunch"><img src="https://img.shields.io/crates/v/dlaunch.svg" alt="crates.io"></a>
  <a href="https://github.com/dirmacs/dlaunch/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
  <img src="https://img.shields.io/badge/rust-stable-orange.svg" alt="Rust">
</p>

---

**dlaunch** is a lightweight Rust CLI that wraps `claude code --dangerously-skip-permissions` with per-project session persistence. Every directory gets its own Claude session stored in `~/.claude_session_<project>`. Optionally run inside tmux so sessions survive terminal crashes and reboots.

A Rust rewrite of [claunch](https://github.com/0xkaz/claunch), built on the DIRMACS toolchain alongside `dstack`, `dwasm`, and friends.

## Install

```bash
cargo install dlaunch
```

```bash
# From source
git clone https://github.com/dirmacs/dlaunch && cd dlaunch
cargo install --path .
```

## Usage

```bash
# Start or resume the Claude session for the current directory
dlaunch

# Same, but inside a persistent tmux window
dlaunch --tmux

# After Claude starts, save the session ID it prints
dlaunch save sess-abc123def456

# List all sessions across all projects
dlaunch list

# Remove session files whose tmux window is gone
dlaunch clean

# Inspect or delete the current project's session
dlaunch session show
dlaunch session delete
```

## How it works

| Scenario | Behaviour |
|---|---|
| No session file exists | Starts a fresh `claude code --dangerously-skip-permissions` |
| Session file found (`sess-…`) | Resumes with `claude code --resume <id> --dangerously-skip-permissions` |
| `--tmux` flag | Wraps the above in `tmux new-session -A -s claude-<project>` |
| tmux not installed | Auto-installs via `brew` (macOS) or `apt` (Debian/Ubuntu) |

Session IDs are stored in `~/.claude_session_<project>` — one plain-text file per directory name.

## First run

On the first invocation dlaunch shows a one-time security notice about `--dangerously-skip-permissions`. Acknowledging it writes `~/.dlaunch_warning_acknowledged` so the prompt never repeats.

## Comparison with claunch

| | claunch | dlaunch |
|---|---|---|
| Language | bash | Rust |
| Explicit `save` command | no (manual echo) | `dlaunch save sess-…` |
| Session inspection | no | `dlaunch session show` |
| Session deletion | no | `dlaunch session delete` |
| Install | curl pipe bash | `cargo install dlaunch` |
| Validation errors | silent | descriptive |

## Built by

[DIRMACS](https://dirmacs.com) — see also `dstack`, `dwasm`, `ares`, `eruka`.
