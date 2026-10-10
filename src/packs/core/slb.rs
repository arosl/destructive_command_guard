//! slb patterns - commands that run what an slb request holds.
//!
//! slb (simultaneous launch button) runs a command once enough reviewers
//! approve it. Two of its subcommands run a command that dcg never sees as a
//! command: `slb execute <request-id>` runs the stored request, and
//! `slb emergency-execute '<cmd>' --yes --ack <sha256>` runs one with no
//! approval and, with `--yes`, no human. An agent computes the ack itself, and
//! one process can open two slb sessions and approve its own request, so
//! either subcommand turns every block in dcg into advice. A human runs them in
//! their own terminal. `slb run`, `slb request` and `slb emergency-execute`
//! also hand their operand to a shell, and the heredoc tier judges that
//! operand as a command string.

use crate::destructive_pattern;
use crate::packs::{DestructivePattern, Pack, SafePattern};

/// Create the slb pack.
#[must_use]
pub fn create_pack() -> Pack {
    Pack {
        id: "core.slb".to_string(),
        name: "Core slb",
        description: "Keeps agents from running slb requests or emergency overrides, which \
                      run a command dcg cannot see",
        keywords: &["slb", "/slb"],
        safe_patterns: create_safe_patterns(),
        destructive_patterns: create_destructive_patterns(),
        keyword_matcher: None,
        safe_regex_set: None,
        safe_regex_set_is_complete: false,
    }
}

fn create_safe_patterns() -> Vec<SafePattern> {
    vec![]
}

fn create_destructive_patterns() -> Vec<DestructivePattern> {
    // The subcommand is the first operand. Global flags may come first, with
    // or without a value (`slb -C /repo execute`, `slb -j execute`).
    vec![
        destructive_pattern!(
            "slb-execute",
            r"(?:^|[^[:alnum:]_-])slb(?:\s+-[^\s=]+(?:=\S*)?(?:\s+[^\s-]\S*)?)*\s+['\x22]?execute['\x22]?(?:\s|$)",
            "slb execute runs a stored request that dcg never judged. A human runs it.",
            High,
            "slb execute <request-id> runs the command held in an approved slb request. dcg \
             sees only the request id, and slb's approvals don't prove that a second person \
             looked: one process can open two sessions and approve its own request.\n\n\
             Ask the human to run `slb execute <request-id>` in their own terminal, or run the \
             command itself, which dcg judges.",
            executables = ["slb"]
        ),
        destructive_pattern!(
            "slb-emergency-execute",
            r"(?:^|[^[:alnum:]_-])slb(?:\s+-[^\s=]+(?:=\S*)?(?:\s+[^\s-]\S*)?)*\s+['\x22]?emergency-execute['\x22]?(?:\s|$)",
            "slb emergency-execute is a human override that bypasses approval. A human runs it.",
            Critical,
            "slb emergency-execute runs a command with no approval, and with --yes --ack it \
             runs with no human at all; the ack is only the command's SHA-256, which anyone \
             can compute.\n\n\
             Ask the human to run it in their own terminal, or run the command itself, which \
             dcg judges.",
            executables = ["slb"]
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packs::test_helpers::*;

    #[test]
    fn execute_and_emergency_execute_are_denied() {
        let pack = create_pack();
        assert_blocks_with_pattern(&pack, "slb execute abc123", "slb-execute");
        assert_blocks_with_pattern(
            &pack,
            "slb execute abc123 --session-id s1 --background",
            "slb-execute",
        );
        assert_blocks_with_pattern(&pack, "slb -C /repo execute abc123", "slb-execute");
        assert_blocks_with_pattern(&pack, "slb -j execute abc123", "slb-execute");
        assert_blocks_with_pattern(&pack, "/usr/local/bin/slb execute abc123", "slb-execute");
        assert_blocks_with_pattern(
            &pack,
            "slb emergency-execute 'true' -r x --yes --ack 0123",
            "slb-emergency-execute",
        );
        assert_blocks_with_pattern(
            &pack,
            "slb --actor me emergency-execute -r x -- true",
            "slb-emergency-execute",
        );
    }

    #[test]
    fn review_and_status_subcommands_are_allowed() {
        let pack = create_pack();
        for command in [
            "slb pending",
            "slb approve abc123 --session-id s1",
            "slb status abc123",
            "slb show abc123",
            "slb review abc123",
            "slb history",
            "slb run 'git status' --reason check",
            "slb request 'ls -la'",
            "echo slb execute abc123",
        ] {
            assert_allows(&pack, command);
        }
    }
}
