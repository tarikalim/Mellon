# Contributing

Thanks for helping. Mellon is a small project, so the process is short.

## Build and run

You need macOS and stable Rust.

```sh
cargo run                  # the window, with a debug build
cargo test
scripts/bundle.sh          # release build installed as /Applications/Mellon.app
```

Point `XDG_CONFIG_HOME` at another folder, and `MELLON_ROOTS` in its `mellon/.env` at a test
folder, to try changes without touching your own files.

## Before a pull request

```sh
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
cargo deny check           # cargo install cargo-deny
```

The pull request check runs these, and also checks:

- **Branch name**: [Conventional Branch](https://conventionalbranch.org/): `feat/`, `fix/`,
  `hotfix/` or `chore/` + lower-case words joined by hyphens (`feat/copy-key`), or
  `release/<version>`. Documentation changes go on `chore/`. Rule: `scripts/check-branch-name.sh`.
- **Commit messages and the PR title**: [Conventional Commits](https://www.conventionalcommits.org/),
  `feat: …`, `fix: …`, `docs: …`, at most 100 characters in the subject. Rule: `committed.toml`.
  The body says why, with any measurement that led to the change.
- **No AI tool as co-author** (`Co-authored-by:` trailers).
- **No secrets** in the new commits (gitleaks).
- **Dependencies**: no known vulnerability (RustSec), only the licences in `deny.toml`.

Clippy and the tests run on macOS and are skipped while the pull request is a draft.

- One change per pull request, with a test when it changes behaviour.
- Code follows `cargo fmt`'s defaults. Keep code comments to a line or two; the reasoning goes in
  the commit message.
- Add a line under `Unreleased` in `CHANGELOG.md` for anything a user would notice.

## Where things are

| Path | What |
|---|---|
| `src/store.rs` | Finding env files, parsing lines, writing one line back |
| `src/config.rs` | Search roots, skip list, the config and scratch files |
| `src/ui.rs`, `src/theme.rs` | The window |
| `src/mcp.rs` | MCP server |

## Releases

The maintainer bumps the version in `Cargo.toml`, moves `Unreleased` in `CHANGELOG.md` under the
new version, and pushes a `v*` tag. The release workflow builds and publishes the app.
