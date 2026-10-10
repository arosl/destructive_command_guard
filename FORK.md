# This fork

`arosl/destructive_command_guard` is a fork of [Dicklesworthstone/destructive_command_guard](https://github.com/Dicklesworthstone/destructive_command_guard), kept for the [ACFS fork](https://github.com/arosl/agentic_coding_flywheel_setup). That fork runs its coding agents in [herdr](https://herdr.dev) panes, and they use `slb`, so dcg has to judge the commands those two tools run.

The fork follows upstream closely. It carries only the changes below, merges upstream regularly, and adds no features of its own. Every change is listed here with its reason, and a change that upstream takes is dropped at the next merge.

## Changes from upstream

On `main`, after upstream's `main`:

| Commit | Change | Why |
|---|---|---|
| `018a4519` | `herdr pane run`, `herdr pane send-text`, tmux's shell-command subcommands and the keys of `tmux send-keys` are command-string runners. `herdr agent prompt`, which sends text to an agent, stays data. | These hand their string to a shell, so a destructive command passed as one quoted word was allowed, while its unquoted and `bash -c` forms were denied. |
| `4c00c5b1` | `slb run`, `slb request --wait --execute` and `slb emergency-execute` are command-string runners. A new always-on `core.slb` pack denies `slb execute <id>` and `emergency-execute --yes --ack`. | These run the command in the caller's shell. `slb execute` runs a stored request that dcg never sees, and the `--ack` path needs no human. |
| `1e147c7a` | `install.sh`, `install.ps1`, `dcg update` (`src/update.rs`, `src/cli.rs`) and `Cargo.toml` point at `arosl`, and the version is 0.15.4. | The fork's installer and `dcg update` must fetch the fork's releases and check their cosign identity. Otherwise `dcg update` replaces the fork's binary with upstream's. |
| `6ffe22d1` | `dist.yml` installs `nightly-2026-08-31`, the toolchain that `rust-toolchain.toml` pins. | With the older nightly, the musl and aarch64-windows builds failed (E0463), and the release job was skipped. Upstream's `main` has the same mismatch, so this can go upstream's way at a merge. |

Disabled in this repository's Actions settings: `release-automation` (it tags `v<Cargo version>` on every push to `main`), and `Notify ACFS checksum monitor` and `Notify ACFS of Installer Change` (they dispatch to upstream ACFS).

## Releases

| Release | Branch | Contents |
|---|---|---|
| v0.15.4 | `release-v0.15.4` | upstream's v0.15.3, plus the four changes above, cherry-picked |

A release is tagged by hand, never by `release-automation`. Its version must be one that upstream hasn't released and that sorts above every earlier fork release; `dcg update` compares them. `dist` builds it: `gh workflow run dist.yml --ref <tag>`.

## Merging upstream

1. `git fetch upstream`, then `git merge upstream/main` into `main`. Never rebase `main`.
2. Resolve each conflict so that the fork keeps exactly the changes in the table. Drop a change that upstream now has, and update the table.
3. Run the `dist` test job's checks locally: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --lib`, plus the multiplexer and slb tests (`cargo test --test multiplexer_command_strings --test slb_command_strings`).
4. For a release: make a `release-v<version>` branch from upstream's release tag, cherry-pick the fork's changes, bump the version, tag it and run `dist`.
5. In ACFS, `checksums.yaml` pins this fork's `install.sh`. After any change to it, run ACFS's canonical checksum refresh (its `AGENTS.md`, "Verified Installer Checksum Discipline").
