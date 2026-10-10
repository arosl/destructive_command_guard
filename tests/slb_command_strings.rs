//! slb hands a command to the caller's shell: `slb run '<cmd>'`, `slb request
//! '<cmd>' --wait --execute` and `slb emergency-execute '<cmd>' --yes --ack
//! <sha256>`. Passed as one quoted word, the destructive command in them was
//! allowed. `slb execute <request-id>` runs a stored request dcg never sees,
//! and emergency-execute needs no approval, so both are a human's to run.
//! Review and status subcommands stay allowed.

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
fn slb_command_strings_and_overrides_are_denied() {
    for command in [
        "slb run 'git reset --hard' --reason x",
        "slb run \"rm -rf ./build\" --reason \"Clean build artifacts\"",
        "slb run --reason 'tidy up' 'git clean -fdx'",
        "slb -C /repo run 'git reset --hard'",
        "slb -sabc run 'git reset --hard'",
        "slb -jr why emergency-execute 'git reset --hard'",
        "slb -C/repo --session-id=s1 run 'git reset --hard'",
        "slb run -- 'git reset --hard'",
        "slb run 'git reset' ' --hard'",
        "slb request 'git reset --hard' --wait --execute",
        "slb --actor me request \"rm -rf ./build\" --reason x --wait --execute",
        "slb emergency-execute 'git reset --hard' -r x --yes --ack 0123",
        "slb emergency-execute 'true' -r x --yes --ack 0123",
        "slb execute abc123",
        "slb execute abc123 --session-id s1 --background",
        "/usr/local/bin/slb execute abc123",
        "cd /repo && slb execute abc123",
    ] {
        assert!(claude_hook_denies(command), "{command:?}");
    }
    for command in [
        "slb run 'git status' --reason check",
        "slb run --reason 'git reset --hard' 'ls -la'",
        "slb request 'ls -la' --wait",
        "slb pending",
        "slb approve abc123 --session-id s1",
        "slb status abc123",
        "slb show abc123",
        "slb review abc123",
        "slb check 'git reset --hard'",
        "echo slb execute abc123",
    ] {
        assert!(!claude_hook_denies(command), "{command:?}");
    }
}
