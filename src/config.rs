//! Search roots and skip lists, read from `~/.config/mellon/.env`.

use std::io;
use std::path::{Path, PathBuf};

// Noise that is never worth walking into.
const BUILTIN_SKIP: &[&str] = &[
    "Library",
    ".Trash",
    "node_modules",
    ".git",
    ".venv",
    "venv",
    "target",
    ".cargo",
    ".rustup",
    ".npm",
    ".cache",
    "Applications",
    // Windows and Linux app data, as large as macOS's Library.
    "AppData",
    ".local",
    "snap",
];

pub const MAX_DEPTH: usize = 8;

/// `HOME`, or `USERPROFILE` on Windows.
pub fn home() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"))
}

pub fn config_dir() -> PathBuf {
    let base = match std::env::var_os("XDG_CONFIG_HOME") {
        Some(d) if !d.is_empty() => PathBuf::from(d),
        _ => home().join(".config"),
    };
    let dir = base.join("mellon");
    // One-time move from the pre-rename folder.
    let old = base.join("envhub");
    if !dir.exists() && old.is_dir() {
        let _ = std::fs::rename(&old, &dir);
    }
    dir
}

pub fn config_file() -> PathBuf {
    config_dir().join(".env")
}

/// Written to [`config_file`] on first run so there is something to edit.
const TEMPLATE: &str = include_str!("../example.env");

pub fn write_template_if_missing() -> io::Result<PathBuf> {
    let path = config_file();
    if !path.exists() {
        std::fs::create_dir_all(config_dir())?;
        std::fs::write(&path, TEMPLATE)?;
    }
    Ok(path)
}

pub const ROOTS_KEY: &str = "MELLON_ROOTS";
pub const SKIP_KEY: &str = "MELLON_SKIP";

/// Holds keys that belong to no repo.
pub fn scratch_file() -> PathBuf {
    config_dir().join("scratch.env")
}

pub const SCRATCH_REPO: &str = "scratch";

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Config {
    /// Directories to scan; empty means the home directory.
    pub roots: Vec<PathBuf>,
    /// Extra directory names to skip.
    pub skip: Vec<String>,
}

impl Config {
    /// Falls back to the defaults when the file is missing or unreadable.
    pub fn load() -> Self {
        let _ = write_template_if_missing();
        let mut cfg = std::fs::read_to_string(config_file())
            .map(|text| Self::parse(&text))
            .unwrap_or_default();
        if cfg.roots.is_empty() {
            cfg.roots.push(home());
        }
        cfg
    }

    /// Reads `MELLON_ROOTS` and `MELLON_SKIP` (or the pre-rename `ENVHUB_*`) out of an env
    /// file. Both are colon-separated, like `PATH`.
    pub fn parse(text: &str) -> Self {
        let mut cfg = Self::default();
        for line in text.lines() {
            let Some((key, value, false)) = crate::store::parse_line(line) else {
                continue;
            };
            match key.as_str() {
                ROOTS_KEY | "ENVHUB_ROOTS" => cfg.roots = split_list(&value).map(expand).collect(),
                SKIP_KEY | "ENVHUB_SKIP" => {
                    cfg.skip = split_list(&value).map(str::to_owned).collect()
                }
                _ => {}
            }
        }
        cfg
    }
}

/// Separated like `PATH`: `:`, or `;` on Windows, where paths hold `C:`.
fn split_list(value: &str) -> impl Iterator<Item = &str> {
    value
        .split(if cfg!(windows) { ';' } else { ':' })
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

pub fn is_skipped(dir_name: &str, extra: &[String]) -> bool {
    BUILTIN_SKIP.contains(&dir_name) || extra.iter().any(|s| s == dir_name)
}

pub fn expand(path: &str) -> PathBuf {
    match path.strip_prefix("~/") {
        Some(rest) => home().join(rest),
        None if path == "~" => home(),
        None => PathBuf::from(path),
    }
}

/// `/Users/me/code/api` -> `code/api`.
pub fn display_path(path: &Path) -> String {
    path.strip_prefix(home())
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_roots_and_skips() {
        let sep = if cfg!(windows) { ';' } else { ':' };
        let cfg = Config::parse(&format!(
            "# comment\nMELLON_ROOTS=~/code {sep} /tmp/x\nMELLON_SKIP=dist{sep}backup\n"
        ));
        assert_eq!(
            cfg.roots,
            vec![home().join("code"), PathBuf::from("/tmp/x")]
        );
        assert_eq!(cfg.skip, vec!["dist".to_owned(), "backup".to_owned()]);
    }

    #[test]
    fn ignores_comments_unknown_keys_and_empty_values() {
        assert_eq!(
            Config::parse("# ENVHUB_ROOTS=~/nope\nOTHER=x\nENVHUB_ROOTS=\n"),
            Config::default()
        );
    }

    #[test]
    fn skips_builtin_and_configured_names() {
        let cfg = Config::parse("ENVHUB_SKIP=dist");
        assert!(is_skipped("node_modules", &cfg.skip));
        assert!(is_skipped("dist", &cfg.skip));
        assert!(!is_skipped("src", &cfg.skip));
    }

    #[test]
    fn expands_tilde_only_at_the_start() {
        assert_eq!(expand("~/code"), home().join("code"));
        assert_eq!(expand("~"), home());
        assert_eq!(expand("/etc/~"), PathBuf::from("/etc/~"));
    }
}
