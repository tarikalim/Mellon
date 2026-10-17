---
name: mellon
description: Use a value from a local .env file (a token, a password, a database URL) without reading it. Use it whenever a command needs a secret or a setting that lives in a .env file on this Mac.
---

# Mellon

Mellon indexes every `KEY=VALUE` in the `.env` files on this Mac. Through MCP an agent can find a
key and run a command with it set, but never sees the value: values stay in the files, and any
value that shows up in the command's output comes back as `***KEY***`.

Never read a `.env` file yourself (`cat`, `grep`, `source`, an editor, a script that opens it).
That puts the secret in your context and in the transcript. Go through these tools instead.

## Finding a key

`list_keys(query?)` lists `repo:KEY`, never values. `repo` is the folder of the env file,
relative to the home folder (`work/api`), or `scratch` for keys that belong to no folder.
Terms in `query` are ANDed over repo and key, case-insensitive: `api database`, `stripe key`.

## Using it

`run_with_env(command, keys, cwd?)` runs `sh -c command` with the keys set as environment
variables. Refer to them as `$KEY` in the command:

```json
{"command": "psql \"$DATABASE_URL\" -c 'select 1'", "keys": ["work/api:DATABASE_URL"], "cwd": "~/work/api"}
```

- A key is `KEY`, or `repo:KEY` when several files hold different values for it. The error lists
  the choices; pick the one for the project you work in.
- The variable is always named `KEY`, even when you passed `repo:KEY`.
- `cwd` accepts `~/`. The command runs once, with no terminal; a prompt for input hangs it.
- The result is `exit <code>` and then stdout and stderr together.

## What masking does not cover

Only the plain value is masked. An encoded or transformed value passes through: base64, URL
encoding, a JSON-escaped string, a hash, part of the value. Do not print, echo, `env`, `set -x`,
encode or log the variables; use them only as arguments, headers or connection strings.

More examples: [references/recipes.md](references/recipes.md).
