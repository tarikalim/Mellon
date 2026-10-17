# Changelog

All notable changes to Mellon. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and versions follow [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- An eye button on every row shows or hides just that value; Reveal still switches them all.

## [0.3.0] - 2026-10-17

### Changed

- Renamed from envhub to Mellon: app, binary, MCP server name, `~/.config/mellon` and the
  `MELLON_ROOTS` / `MELLON_SKIP` keys. The old config folder is moved on first run and the old
  keys are still read; `scripts/bundle.sh` removes the old `envhub.app`.
- The window is rebuilt with GPUI (gpui-kit) in the Islands iTerm colours, as Ithildin and
  Redbook: a folder sidebar, one row per key with key, folder and value columns, a copy and an
  open button on every row, and toggles in the header.
- Click anywhere on a row to copy its value; the copy button shows a check for a moment.
- New app icon, a silver M with a star on a graphite book, drawn from `assets/icon.svg`.
- Open and the config button open the file in the Mac's default text editor (`open -t`): an app
  opened from Finder has no `$EDITOR`.
- Releases are a universal macOS `.dmg` and `.zip`; there is no Linux build any more.

### Removed

- The second "filter these results" box: the search already narrows within the chosen folder.

### Added

- Theme menu: every theme in Zed's theme files, Islands iTerm the default, as in Ithildin.
- Contributing guide, code of conduct, security policy, changelog, issue and pull request
  templates, Dependabot, and a pull request check: branch name, commit messages and PR title, no
  AI co-author or attribution (`Claude-Session:` trailers, "Generated with …" in the PR body),
  gitleaks, `cargo fmt`, `cargo-deny`, clippy and tests.

## [0.2.0]

### Added

- `envhub mcp`: an MCP server that lets agents use env values without seeing them.

[Unreleased]: https://github.com/tarikalim/Mellon/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/tarikalim/Mellon/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/tarikalim/Mellon/releases/tag/v0.2.0
