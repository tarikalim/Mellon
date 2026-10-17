# Mellon

A native app for macOS, Linux and Windows that finds every `KEY=VALUE` in the `.env` files across your computer, so you can search them, copy one with a click and edit it in place, without opening ten repos to find the one secret someone just asked you for. Written in Rust with [GPUI](https://www.gpui.rs/). Values stay in your `.env` files exactly as they are; the app only reads and writes those files. Built for local development convenience, not for security.

The name is Tolkien's *mellon*, Elvish for friend: the word that opens the Doors of Durin, for those who know to say it.

## What it shows

- **Search**: filters by key or folder; terms are ANDed (`stripe key`). **Values** makes it look inside values too, so `localhost:5432` finds every key that points at that database.
- **Folders**: the sidebar lists every directory that has env files, with key counts. Click one to scope the list to it, or **All folders** to search everything.
- **Copy**: click a row to copy its value. Every row also has an eye that shows or hides just that value, a copy button, and an open button that opens the file in the default text editor (`open -t` on macOS, the system's default app elsewhere).
- **Reveal**: unmasks every value at once and resets the per-row eyes.
- **Commented**: also lists keys commented out in the files (`# KEY=VALUE`), dimmed with a `#`. Editing one keeps it commented.
- **Edit**: inline editing. Click a row, type, and press Enter or click away; only that one line of the file is rewritten. Off by default, so nothing changes by accident.
- **New key** (+): adds a key that belongs to no folder, to `~/.config/mellon/scratch.env`.
- **Rescan** (↻) reads the folders and the config again; the gear opens the config.

The default theme is the Zed theme *Islands iTerm*; the palette icon lists every theme in Zed's theme files.

## Getting data in

`~/.config/mellon/.env` (or `$XDG_CONFIG_HOME/mellon/.env`), an env file like any other:

```sh
MELLON_ROOTS=~/code:~/work/backend   # folders to search; unset means $HOME
MELLON_SKIP=dist:backup              # extra folder names to ignore
```

Both are optional and separated like `PATH`: `:`, or `;` on Windows. Mellon writes this file, commented out and ready to edit, the first time it runs. `node_modules`, `.git`, `Library`, `AppData`, `.local`, `target`, `.venv` and `.cache` are always skipped.

Coming from envhub: `~/.config/envhub` is moved here on first run, and its `ENVHUB_ROOTS` and `ENVHUB_SKIP` keys are still read.

### MCP

`mellon mcp` lets agents use a value without seeing it. Tool results are text.

| Tool | For |
|---|---|
| `list_keys` | `repo:KEY` for every key matching `query` (terms ANDed over repo and key), never values |
| `run_with_env` | Runs `sh -c command` (`cmd /C` on Windows) with the chosen keys set as environment variables (`KEY`, or `repo:KEY` when files disagree). Values that show up in the output come back as `***KEY***`; encoded forms are not masked |

The `mellon` skill ([skills/mellon](skills/mellon/SKILL.md)) explains both tools, what masking does not cover, and working recipes. It is served over the MCP Skills extension (`skills/list`, `skills/get`, `resources/read`).

## Install and run

### From a release

[Releases](../../releases) has a universal macOS app (`.dmg` or `.zip`, Apple Silicon and Intel), a Linux x86_64 `.tar.gz` and a Windows x86_64 `.zip`.

**macOS**

1. Open `Mellon-<version>-macos-universal.dmg` and drag **Mellon** to **Applications**.
2. The app is signed ad hoc, not notarized, so macOS blocks its first launch. Either open **System Settings → Privacy & Security** and choose **Open Anyway** next to the Mellon message, or remove the download's quarantine flag once:

   ```sh
   xattr -dr com.apple.quarantine /Applications/Mellon.app
   ```

**Linux**

1. Extract the archive: `tar -xzf Mellon-<version>-linux-x86_64.tar.gz`
2. Install the program, launcher and icon for your user:

   ```sh
   install -Dm755 Mellon-<version>/mellon ~/.local/bin/mellon
   install -Dm644 Mellon-<version>/mellon.desktop ~/.local/share/applications/mellon.desktop
   install -Dm644 Mellon-<version>/mellon.png ~/.local/share/icons/hicolor/512x512/apps/mellon.png
   ```

3. Open **Mellon** from the application menu. `~/.local/bin` must be on your `PATH`.

**Windows**

1. Extract the zip to a folder you keep, for example `%LOCALAPPDATA%\Programs\Mellon`.
2. Double-click `mellon.exe`. The program isn't signed: if Windows shows **Windows protected your PC**, click **More info**, then **Run anyway**.

**Optional, the command line and the MCP server for Claude Code** (on Linux the program is already `~/.local/bin/mellon`):

```sh
# macOS
ln -sf /Applications/Mellon.app/Contents/MacOS/mellon ~/.local/bin/mellon
claude mcp add --scope user mellon -- /Applications/Mellon.app/Contents/MacOS/mellon mcp
# Linux
claude mcp add --scope user mellon -- ~/.local/bin/mellon mcp
# Windows
claude mcp add --scope user mellon -- %LOCALAPPDATA%\Programs\Mellon\mellon.exe mcp
```

The window lists every env file under your home folder. Narrow it with `MELLON_ROOTS` (gear icon).

### From source

Rust stable; on Linux, also the packages `scripts/install-linux-deps.sh` installs.

```sh
cargo run --release
scripts/bundle.sh          # macOS: builds release and installs /Applications/Mellon.app in place
```

Then step 4 above. The script also removes a pre-rename `envhub.app`.

### Command line

The app, the CLI and the MCP server are one binary; `mellon` with no arguments opens the window.

| Command | What it does |
|---|---|
| `mellon` | Opens the window |
| `mellon mcp` | Runs the MCP server over stdio |

## Releases

Pushing a `v*` tag runs `.github/workflows/release.yml`: a universal macOS build (`.dmg` and `.zip`), a Linux `.tar.gz` and a Windows `.zip`, and a release with all of them.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Changes are listed in [CHANGELOG.md](CHANGELOG.md).

## Licence

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT),
at your option. Unless you state otherwise, any contribution you submit for inclusion in Mellon
is licensed the same way, without any additional terms or conditions.
