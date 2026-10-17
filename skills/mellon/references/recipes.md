# Recipes

Each one is a `run_with_env` call. The key names are examples; find the real ones with `list_keys`.

## A database

```json
{"command": "psql \"$DATABASE_URL\" -c 'select count(*) from users'", "keys": ["work/api:DATABASE_URL"]}
```

## An HTTP API with a token

```json
{"command": "curl -sS -H \"Authorization: Bearer $API_TOKEN\" https://api.example.com/v1/me", "keys": ["API_TOKEN"]}
```

## A CLI that reads its own variable

Many tools read a variable by name; set it and run the tool, nothing else:

```json
{"command": "gh api user --jq .login", "keys": ["GH_TOKEN"]}
```

## A variable under another name

The variable is named after the key. When a tool wants another name, map it in the command:

```json
{"command": "PGPASSWORD=\"$DB_PASS\" psql -h localhost -U app -d app -c 'select 1'", "keys": ["work/api:DB_PASS"]}
```

## A password piped to a program

```json
{"command": "printf '%s\\n' \"$SUDO_PASS\" | sudo -S true && echo ok", "keys": ["scratch:SUDO_PASS"]}
```

## Two keys from two projects

```json
{"command": "pg_dump \"$SOURCE_URL\" | psql \"$TARGET_URL\"", "keys": ["work/old:SOURCE_URL", "work/new:TARGET_URL"]}
```

## Not this

- `echo $TOKEN`, `env`, `printenv`, `set -x`: the value comes back masked only in its plain form.
- `cat .env`, `grep KEY .env`, `source .env`: the value lands in your context.
- `base64 <<< "$TOKEN"`, `jq -n --arg t "$TOKEN" '$t|@uri'`: encoded values are not masked.
