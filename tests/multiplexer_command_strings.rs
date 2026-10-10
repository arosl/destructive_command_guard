//! A terminal multiplexer that runs a command string in a pane hands that
//! string to a shell: `herdr pane run <pane> '<cmd>'`, `herdr pane send-text
//! <pane> '<cmd>'` (typed into the pane's shell), `tmux new-window '<cmd>'`,
//! `tmux send-keys -t x '<cmd>' Enter`. Passed as one quoted word, the
//! destructive command in it was allowed, while the unquoted and `bash -c`
//! forms were denied. `herdr agent prompt` sends text to an agent, not to a
//! shell, and stays data.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn claude_hook_denies(shell_command: &str) -> bool {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(dir.path().join("home")).unwrap();
    std::fs::create_dir_all(dir.path().join("xdg")).unwrap();
    let config_path = dir.path().join("policy.toml");
    std::fs::write(&config_path, "[general]\ncolor = \"never\"\n").unwrap();
    let mut child = Command::new(PathBuf::from(env!("CARGO_BIN_EXE_dcg")))
        .env_clear()
        .env("HOME", dir.path().join("home"))
        .env("USERPROFILE", dir.path().join("home"))
        .env("XDG_CONFIG_HOME", dir.path().join("xdg"))
        .env("DCG_ALLOWLIST_SYSTEM_PATH", "")
        .env("DCG_CONFIG", &config_path)
        .env("DCG_HOOK_TIMEOUT_MS", "5000")
        .current_dir(dir.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn dcg");
    let payload = serde_json::json!({
        "hook_event_name": "PreToolUse",
        "tool_name": "Bash",
        "tool_input": { "command": shell_command },
        "cwd": dir.path(),
    });
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(format!("{payload}\n").as_bytes())
        .unwrap();
    let output = child.wait_with_output().expect("wait dcg");
    String::from_utf8_lossy(&output.stdout).contains("\"deny\"")
}

#[test]
fn herdr_pane_command_strings_are_judged() {
    for command in [
        "herdr pane run p1 'git reset --hard'",
        "herdr pane run p1 'rm -rf ./build'",
        "herdr pane run p1 \"git clean -fdx\"",
        "herdr pane run w1:t2:p3 'cd /repo && git reset --hard'",
        "herdr pane send-text p1 'git reset --hard'",
        "herdr pane send-text p1 $'rm -rf ./build\\n'",
        "herdr pane send-text p1 'rm -rf ./build\n'",
        "herdr --session acfs pane run p1 'git reset --hard'",
        "herdr --remote host pane send-text p1 'git reset --hard'",
        "/usr/local/bin/herdr pane run p1 'git reset --hard'",
        "sudo -u bob herdr pane run p1 'rm -rf ./build'",
        "herdr pane run p1 'git reset' --hard",
        "true && herdr pane run p1 'git reset --hard'",
    ] {
        assert!(claude_hook_denies(command), "{command:?}");
    }
    for command in [
        "herdr pane run p1 'git status'",
        "herdr pane send-text p1 'ls -la'",
        "herdr pane send-keys p1 Enter",
        "herdr pane read p1 --source recent-unwrapped",
        "herdr pane list",
        "herdr agent prompt jadewren 'never run git reset --hard here'",
        "herdr agent prompt jadewren \"don't rm -rf the tree\" --wait",
        "herdr pane rename p1 'git reset --hard'",
        "herdr tab create --workspace w1 --label 'rm -rf' --no-focus",
        "herdr pane send-text p1 'please check your Agent Mail inbox'",
        "echo herdr pane run p1 'git reset --hard'",
    ] {
        assert!(!claude_hook_denies(command), "{command:?}");
    }
}

#[test]
fn tmux_shell_command_strings_are_judged() {
    for command in [
        "tmux new-window 'git reset --hard'",
        "tmux new-window -n build -c /repo 'rm -rf ./build'",
        "tmux neww -d 'git reset --hard'",
        "tmux new-session -d -s x 'git reset --hard'",
        "tmux new -d -s x 'rm -rf ./build'",
        "tmux split-window -h -l 20 'git reset --hard'",
        "tmux splitw -t x:1 'git clean -fdx'",
        "tmux respawn-pane -k -t x 'git reset --hard'",
        "tmux respawn-window -k 'rm -rf ./build'",
        "tmux run-shell 'git reset --hard'",
        "tmux run -b 'rm -rf ./build'",
        "tmux display-popup -E 'git reset --hard'",
        "tmux send-keys -t x 'git reset --hard' Enter",
        "tmux send-keys -t x \"rm -rf ./build\" C-m",
        "tmux send -t x 'git reset --hard' Enter",
        "tmux -L sock new-window 'git reset --hard'",
        "tmux -S /tmp/sock send-keys -t x 'git reset --hard' Enter",
        "tmux new-window 'git status' \\; send-keys -t x 'git reset --hard' Enter",
        "tmux send-keys -t x 'git reset' ' --hard' Enter",
    ] {
        assert!(claude_hook_denies(command), "{command:?}");
    }
    for command in [
        "tmux new-window 'git status'",
        "tmux new-window -n 'rm -rf' htop",
        "tmux new-session -d -s 'git reset --hard'",
        "tmux send-keys -t x 'ls -la' Enter",
        "tmux send-keys -t 'rm -rf' Enter",
        "tmux list-panes -a",
        "tmux rename-window 'git reset --hard'",
        "tmux display-message 'rm -rf ./build'",
        "tmux capture-pane -p -t x",
        "echo tmux new-window 'git reset --hard'",
    ] {
        assert!(!claude_hook_denies(command), "{command:?}");
    }
}
