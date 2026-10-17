# Mellon

A native macOS app that finds every `KEY=VALUE` in the `.env` files across your Mac, so you can
search them, copy one with a click and edit it in place, without opening ten repos to find the
one secret someone just asked you for. Written in Rust with [GPUI](https://www.gpui.rs/).

The name is the password to the Doors of Durin: *speak, friend, and enter*. Mellon is Sindarin
for "friend".

Built for local development convenience, not for security: values stay in your `.env` files
exactly as they are, and the app only reads and writes those files.

## Use

- **Search** filters by key or folder; terms are ANDed (`stripe key`).
- **Folders** on the left lists every directory that has env files, with key counts. Click one to
  scope the list to it, or **All folders** to search everything.
- **Click a row** to copy its value. Each row also has a copy button and an open button, which
  opens the file in the Mac's default
  text editor (`open -t`).
- **Reveal** unmasks every value; the eye on a row shows or hides just that one. **Values**
  makes the search look inside values too, not only keys and folders.
- **Commented** also lists keys commented out in the files (`# KEY=VALUE`), shown dimmed with a
  `#`. Editing one keeps it commented.
- **Edit** turns on inline editing: click a row, type, and press Enter or click away. Only that
  one line of the file is rewritten. Off by default, so nothing changes by accident.
- **+** adds a key that belongs to no folder; it goes to `~/.config/mellon/scratch.env`.
- **↻** rescans the folders and reloads the config; the gear opens the config the same way.
- **Theme** (palette icon) lists every theme in Zed's theme files; the default is *Islands
  iTerm*, as in Ithildin and Redbook. The choice is kept in `~/.config/mellon/theme`.

## Config

`~/.config/mellon/.env` (or `$XDG_CONFIG_HOME/mellon/.env`), an env file like any other:

```sh
MELLON_ROOTS=~/code:~/work/backend   # folders to search; unset means $HOME
MELLON_SKIP=dist:backup              # extra folder names to ignore
```

Both are optional and `:`-separated like `PATH`. Mellon writes this file, commented out and ready
to edit, the first time it runs. A pre-rename `~/.config/envhub` folder is moved here on first
run, and its `ENVHUB_ROOTS` and `ENVHUB_SKIP` keys are still read.

## MCP for agents

`mellon mcp` is an MCP server on stdio. Agents can list keys and run a command with chosen keys
set as environment variables, without the values ever showing in their context:

```sh
claude mcp add --scope user mellon -- /Applications/Mellon.app/Contents/MacOS/mellon mcp
```

| Tool | What it does |
|---|---|
| `list_keys(query?)` | Lists `repo:KEY`, never values |
| `run_with_env(command, keys, cwd?)` | Runs `sh -c command` with the keys set (`KEY`, or `repo:KEY` when repos disagree). Values that show up in the output are masked as `***KEY***`; encoded forms are not |

## Install and run

### From a release

```sh
scripts/install-release.sh      # latest release into /Applications, quarantine flag cleared
```

Or by hand: download `Mellon-<version>-macos-universal.dmg` from [Releases](../../releases), drag
**Mellon** to **Applications**, then clear the quarantine flag once, because the app is signed ad
hoc, not notarized:

```sh
xattr -dr com.apple.quarantine /Applications/Mellon.app
```

### From source

Rust stable on macOS:

```sh
scripts/bundle.sh          # builds release and installs /Applications/Mellon.app in place
```

It also removes a pre-rename `envhub.app`.

### Command line

The app and the MCP server are one binary.

| Command | What it does |
|---|---|
| `mellon` | Opens the window |
| `mellon mcp` | Runs the MCP server over stdio |

## Releases

Pushing a `v*` tag runs `.github/workflows/release.yml`: tests, a universal build, and a release
with the `.dmg` and `.zip`.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Changes are listed in [CHANGELOG.md](CHANGELOG.md).

## Licence

[MIT](LICENSE).
