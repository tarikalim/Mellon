# EnvHub

A small desktop app that finds every `KEY=VALUE` in the `.env` files scattered across your
machine, so you can search them, copy one to the clipboard, and edit it in place — without
opening ten repos to find the one secret someone just asked you for.

Built for local development convenience, not for security: values are stored in your `.env`
files exactly as they already are, and the app only reads and writes those files.

## Install

envhub is not notarized, so macOS quarantines it however you download it and refuses to
open it ("damaged"). The install script fetches the latest release, puts it in
`/Applications` and clears that flag:

```sh
./tools/install-release.sh          
```

Or do it by hand: download `envhub-macos-arm64.tar.gz` from
[Releases](../../releases), then

```sh
tar -xzf envhub-macos-arm64.tar.gz -C /Applications
xattr -dr com.apple.quarantine /Applications/envhub.app
```

On Linux, `envhub-linux-x86_64.tar.gz` holds a single binary — untar it onto your `PATH`.

Or build it yourself. Requires Rust, and macOS for the app bundle.

```sh
git clone <this repo> && cd envhub
cargo run --release          # just run it
./tools/install.sh           # or install it as ~/Applications/envhub.app
```

`install.sh` builds the release binary, copies it into the bundle with its icon, and leaves
you an app you can launch from Spotlight.

## Use

- Type in the search box to filter by key or repo; terms are ANDed (`stripe key`).
- Click a key or a value to copy the value. `copy` does the same; `open` opens the file in `$EDITOR`.
- Above the results there is a second box that narrows whatever is currently listed — handy
  when a single service has a hundred keys. It clears itself when you switch repos.
- The left panel lists every directory that has env files, with key counts. Click one to
  scope the list to it, or `all repos` to search everything.
- `reveal values` unmasks values; `search values` also searches inside them.
- `commented` also lists the keys that are commented out in the files (`# KEY=VALUE`), shown
  dimmed and prefixed with `#`. Editing one keeps it commented.
- `edit mode` turns on inline editing: click a value, type, and click away — only that one
  line of the file is rewritten. Off by default so nothing is changed by accident.
- `+ new` adds a key that belongs to no repo; it goes to `~/.config/envhub/scratch.env`.

## Config

`~/.config/envhub/.env` (or `$XDG_CONFIG_HOME/envhub/.env`) — an env file like any other:

```sh
ENVHUB_ROOTS=~/code:~/work/backend   # folders to search; unset means $HOME
ENVHUB_SKIP=dist:backup              # extra folder names to ignore
```

Both are optional and `:`-separated like `PATH`. envhub writes this file, commented out and
ready to edit, the first time it runs; the `config` button in the toolbar opens it in
`$EDITOR`, and `↻ rescan` reloads it.

## MCP for agents

`envhub mcp` is an MCP server on stdio. Agents can list keys and run a command with chosen keys
set as environment variables, without the values ever showing in their context:

```sh
claude mcp add -s user envhub -- /Applications/envhub.app/Contents/MacOS/envhub mcp
```

- `list_keys(query?)` lists `repo:KEY`, never values.
- `run_with_env(command, keys, cwd?)` runs `sh -c command` with the keys set (`KEY`, or `repo:KEY`
  when repos disagree). Values that show up in the output are masked as `***KEY***`; encoded forms are not.

## Layout

```
src/main.rs     window setup
src/app.rs      the egui interface
src/config.rs   search roots and skip lists
src/store.rs    parsing, scanning and writing .env files
src/mcp.rs      the `envhub mcp` server for agents
tools/          icon generator, macOS install script
```

`cargo test` covers the parsing, the config and the write path; `cargo clippy --all-targets`
and `cargo fmt --check` run in CI.

## Shell alternative

If you only want the search-and-copy half, a shell function with ripgrep and fzf covers it:

```zsh
envs() {
  rg --hidden --no-ignore --no-messages -n --no-heading \
     -g '.env*' -g '!*.example' -g '!**/node_modules/**' -g '!**/.git/**' -g '!Library/**' \
     '^[A-Za-z_][A-Za-z0-9_]*=' "$HOME" \
  | sed "s|^$HOME/||" \
  | fzf --delimiter='=' --nth=1 --preview 'echo {2..}' --preview-window=down:3:wrap \
  | cut -d= -f2- | tr -d '\n' | pbcopy
}
```

## License

MIT
