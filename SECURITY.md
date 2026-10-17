# Security policy

## Supported versions

Only the latest release gets fixes.

## Reporting a vulnerability

Please do not open a public issue. Report it privately through
[GitHub's private vulnerability reporting](https://github.com/tarikalim/Mellon/security/advisories/new),
or send a private message to [@tarikalim](https://github.com/tarikalim). Include the steps to
reproduce it and the version you used. You will get an answer within a week.

## Scope

Mellon is a local development convenience, not a secret store: it reads and writes your `.env`
files as they are, keeps its settings in `~/.config/mellon`, and serves MCP over stdio to the
client that starts it. It opens no network port and sends no data anywhere. `mellon mcp` masks
values that appear in a command's output, but not their encoded forms.
