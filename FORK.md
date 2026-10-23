# This fork

`arosl/destructive_command_guard` is a fork of [Dicklesworthstone/destructive_command_guard](https://github.com/Dicklesworthstone/destructive_command_guard), kept for the [ACFS fork](https://github.com/arosl/agentic_coding_flywheel_setup). That fork runs its coding agents in [herdr](https://herdr.dev) panes, and they use `slb`, so dcg has to judge the commands those two tools run.

The fork follows upstream closely. It carries only the changes below, merges upstream regularly, and adds no features of its own. Every change is listed here with its reason, and a change that upstream takes is dropped at the next merge.

## Changes from upstream

On `main`, after upstream's `main` (last merged at `786b5aed`, 2026-10-23):

| Commit | Change | Why |
|---|---|---|
| `018a4519` | `herdr pane run`, `herdr pane send-text`, tmux's shell-command subcommands and the keys of `tmux send-keys` are command-string runners. `herdr agent prompt`, which sends text to an agent, stays data. | These hand their string to a shell, so a destructive command passed as one quoted word was allowed, while its unquoted and `bash -c` forms were denied. |
| `4c00c5b1` | `slb run`, `slb request --wait --execute` and `slb emergency-execute` are command-string runners. A new always-on `core.slb` pack denies `slb execute <id>` and `emergency-execute --yes --ack`. | These run the command in the caller's shell. `slb execute` runs a stored request that dcg never sees, and the `--ack` path needs no human. |
| `1e147c7a` | `install.sh`, `install.ps1`, `dcg update` (`src/update.rs`, `src/cli.rs`) and `Cargo.toml` point at `arosl`, and the version is 0.15.4. | The fork's installer and `dcg update` must fetch the fork's releases and check their cosign identity. Otherwise `dcg update` replaces the fork's binary with upstream's. |
| `6ffe22d1` | `dist.yml` installs `nightly-2026-08-31`, the toolchain that `rust-toolchain.toml` pins. | With the older nightly, the musl and aarch64-windows builds failed (E0463), and the release job was skipped. Upstream's `main` still installs `nightly-2026-08-25` (at `786b5aed`); drop this once it installs the pinned nightly. |
| `2e854df0` | `.beads/` is no longer tracked, except its `.gitignore`, which ignores everything. The files stay on disk, and the fork's own work is tracked in this checkout's br. It is not part of a release. | ACFS's agents take reports on dcg in this repo (the operator, 2026-10-14), and br writes the local user name and checkout path into every bead, while this fork is public. |

Disabled in this repository's Actions settings: `release-automation` (it tags `v<Cargo version>` on every push to `main`), and `Notify ACFS checksum monitor` and `Notify ACFS of Installer Change` (they dispatch to upstream ACFS).

## Releases

| Release | Branch | Contents |
|---|---|---|
| v0.15.4 | `release-v0.15.4` | upstream's v0.15.3, plus the four changes above, cherry-picked |

A release is tagged by hand, never by `release-automation`. Its version must be one that upstream hasn't released and that sorts above every earlier fork release; `dcg update` compares them. `dist` builds it: `gh workflow run dist.yml --ref <tag>`.

## Merging upstream

Merge in a worktree, never in the checkout where br runs: when upstream changes its bead files, the merge writes them over the local bead queue.

1. `git fetch upstream`, then `git worktree add --detach <scratch>/dcg-sync main`, and in it `git merge --no-commit upstream/main`. Never rebase `main`.
2. Resolve each conflict so that the fork keeps exactly the changes in the table. Drop a change that upstream now has, and update the table. `CHANGELOG.md` conflicts at the top of `Unreleased`: keep both sides' entries, the fork's first. If the merge brought bead files in, run `git rm --cached -r --ignore-unmatch .beads ':!.beads/.gitignore'`.
3. Run the `dist` test job's checks locally: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --lib`, plus the multiplexer and slb tests (`cargo test --test multiplexer_command_strings --test slb_command_strings`). Then commit the merge in the worktree. In the main checkout, run `git merge --ff-only <merge commit>`, and remove the worktree.
4. For a release: make a `release-v<version>` branch from upstream's release tag, cherry-pick the fork's changes, bump the version, tag it and run `dist`.
5. ACFS pins this fork's `install.sh` at a commit, with its SHA-256 (`checksums.yaml`, `scripts/lib/security.sh`), so a push doesn't break ACFS installs, and ACFS keeps the old installer until it moves the pin. When a merge changes `install.sh`, review it for new download hosts and changes to checksum or signature checks, and send the review and the pushed SHA to an ACFS agent, who moves the pin with ACFS's canonical checksum refresh (its `AGENTS.md`, "Verified Installer Checksum Discipline").
