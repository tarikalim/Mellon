//! `mellon mcp`: MCP server over stdio (newline-delimited JSON-RPC 2.0), no auth.
//! Tools let agents use env values without being shown them. The `mellon` skill
//! (skills/mellon) is served through the Skills extension.

use std::io::{self, BufRead as _, Write as _};
use std::process::Command;

use serde_json::{Value, json};
use sha2::{Digest as _, Sha256};

use crate::config::Config;
use crate::store::{self, Entry};

const PROTOCOL: &str = "2025-06-18";
const SUPPORTED: &[&str] = &["2025-11-25", "2025-06-18", "2025-03-26", "2024-11-05"];
const SKILLS: &str = "io.modelcontextprotocol/skills";

/// The skill's files: (path inside the skill, text).
const SKILL_FILES: &[(&str, &str)] = &[
    ("SKILL.md", include_str!("../skills/mellon/SKILL.md")),
    (
        "references/recipes.md",
        include_str!("../skills/mellon/references/recipes.md"),
    ),
];

const INSTRUCTIONS: &str = "Mellon indexes every KEY=VALUE in the .env files on this computer. Never read a .env file \
yourself. Find a key with list_keys (repo:KEY, never values), then run a command with it set as an environment \
variable with run_with_env; values that show up in the output are masked. The `mellon` skill explains both.";

pub fn run() -> io::Result<()> {
    let mut out = io::stdout().lock();
    for line in io::stdin().lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        if let Some(resp) = handle(&line) {
            writeln!(out, "{resp}")?;
            out.flush()?;
        }
    }
    Ok(())
}

fn rpc_error(id: Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

/// One JSON-RPC message in, the response out (None for notifications).
pub fn handle(line: &str) -> Option<Value> {
    let msg: Value = match serde_json::from_str(line) {
        Ok(v) => v,
        Err(e) => return Some(rpc_error(Value::Null, -32700, &format!("parse error: {e}"))),
    };
    let id = msg.get("id").cloned();
    let Some(method) = msg.get("method").and_then(Value::as_str) else {
        return match id {
            Some(id) if msg.get("result").is_none() && msg.get("error").is_none() => {
                Some(rpc_error(id, -32600, "invalid request"))
            }
            _ => None,
        };
    };
    let id = id?;
    let params = msg.get("params").cloned().unwrap_or_else(|| json!({}));
    let capabilities =
        json!({ "tools": { "listChanged": false }, "resources": {}, "extensions": { SKILLS: {} } });
    let cache = |mut v: Value| {
        v["resultType"] = json!("complete");
        v["ttlMs"] = json!(300_000);
        v["cacheScope"] = json!("public");
        v
    };
    let result = match method {
        "initialize" => {
            let asked = params["protocolVersion"].as_str();
            let version = asked.filter(|v| SUPPORTED.contains(v)).unwrap_or(PROTOCOL);
            Ok(json!({
                "protocolVersion": version,
                "capabilities": capabilities,
                "serverInfo": { "name": "mellon", "version": env!("CARGO_PKG_VERSION") },
                "instructions": INSTRUCTIONS
            }))
        }
        "server/discover" => Ok(
            json!({ "capabilities": capabilities, "serverInfo": { "name": "mellon", "version": env!("CARGO_PKG_VERSION") }, "instructions": INSTRUCTIONS }),
        ),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({ "tools": tools() })),
        "tools/call" => {
            let name = params["name"].as_str().unwrap_or_default();
            if tools()
                .as_array()
                .is_some_and(|t| t.iter().any(|t| t["name"] == name))
            {
                let (text, is_error) = match call(&params) {
                    Ok(t) => (t, false),
                    Err(t) => (t, true),
                };
                Ok(json!({ "content": [{ "type": "text", "text": text }], "isError": is_error }))
            } else {
                Err((-32602, format!("unknown tool: {name}")))
            }
        }
        "skills/list" => Ok(cache(json!({ "skills": [skill_entry()] }))),
        "skills/get" => match params["uri"].as_str() {
            Some(uri) if uri == skill_uri("SKILL.md") => {
                Ok(cache(json!({ "skill": skill_entry() })))
            }
            _ => Err((-32602, "unknown skill".into())),
        },
        "resources/list" => Ok(
            json!({ "resources": SKILL_FILES.iter().map(|(path, text)| json!({
            "uri": skill_uri(path), "name": format!("mellon/{path}"), "mimeType": "text/markdown", "size": text.len()
        })).collect::<Vec<_>>() }),
        ),
        "resources/read" => match SKILL_FILES
            .iter()
            .find(|(path, _)| params["uri"].as_str() == Some(&skill_uri(path)))
        {
            Some((path, text)) => Ok(cache(
                json!({ "contents": [{ "uri": skill_uri(path), "mimeType": "text/markdown", "text": text }] }),
            )),
            None => Err((-32602, "unknown resource".into())),
        },
        _ => Err((-32601, format!("method not found: {method}"))),
    };
    Some(match result {
        Ok(r) => json!({ "jsonrpc": "2.0", "id": id, "result": r }),
        Err((code, m)) => rpc_error(id, code, &m),
    })
}

fn skill_uri(path: &str) -> String {
    format!("skill://mellon/{path}")
}

/// The skills/list entry: frontmatter and a manifest of every file with its digest and size.
fn skill_entry() -> Value {
    let skill = SKILL_FILES[0].1;
    let mut frontmatter = serde_json::Map::new();
    for line in skill.split("---").nth(1).unwrap_or_default().lines() {
        if let Some((k, v)) = line.split_once(':') {
            frontmatter.insert(k.trim().into(), json!(v.trim()));
        }
    }
    json!({
        "uri": skill_uri("SKILL.md"),
        "frontmatter": frontmatter,
        "resources": SKILL_FILES.iter().map(|(path, text)| json!({
            "uri": skill_uri(path),
            "digest": sha256(text),
            "size": text.len()
        })).collect::<Vec<_>>()
    })
}

/// `sha256:<hex>` of a file's bytes, as the Skills manifest wants it.
fn sha256(text: &str) -> String {
    let hex: String = Sha256::digest(text.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    format!("sha256:{hex}")
}

fn tools() -> Value {
    json!([
        {
            "name": "list_keys",
            "description": "List env keys found in the .env files on this machine, as `repo:KEY`. Values are never shown. Terms in `query` are ANDed over repo and key.",
            "inputSchema": {"type": "object", "properties": {"query": {"type": "string"}}, "additionalProperties": false},
            "annotations": {"readOnlyHint": true, "destructiveHint": false, "openWorldHint": false},
        },
        {
            "name": "run_with_env",
            "description": "Run a shell command with the given keys set as environment variables, without showing their values. Reference them as $KEY in the command (%KEY% on Windows, where it runs under cmd /C). A key is `KEY` or `repo:KEY` when several repos hold different values. Values are masked in the output.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "command": {"type": "string"},
                    "keys": {"type": "array", "items": {"type": "string"}},
                    "cwd": {"type": "string"},
                },
                "required": ["command", "keys"],
                "additionalProperties": false,
            },
            "annotations": {"readOnlyHint": false, "destructiveHint": true, "openWorldHint": true},
        },
    ])
}

fn call(params: &Value) -> Result<String, String> {
    let args = &params["arguments"];
    let entries: Vec<Entry> = store::scan(&Config::load())
        .into_iter()
        .filter(|e| !e.commented)
        .collect();
    match params["name"].as_str().unwrap_or("") {
        "list_keys" => {
            let terms: Vec<String> = args["query"]
                .as_str()
                .unwrap_or("")
                .split_whitespace()
                .map(str::to_lowercase)
                .collect();
            let lines: Vec<String> = entries
                .iter()
                .filter(|e| e.matches(&terms, false))
                .map(|e| format!("{}:{}", e.repo, e.key))
                .collect();
            Ok(if lines.is_empty() {
                "no keys match".into()
            } else {
                lines.join("\n")
            })
        }
        "run_with_env" => {
            let command = args["command"].as_str().ok_or("`command` is required")?;
            let mut env = Vec::new();
            for spec in args["keys"].as_array().ok_or("`keys` is required")? {
                let spec = spec.as_str().ok_or("keys must be strings")?;
                env.push(resolve(&entries, spec)?);
            }
            let (shell, flag) = if cfg!(windows) {
                ("cmd", "/C")
            } else {
                ("sh", "-c")
            };
            let mut cmd = Command::new(shell);
            cmd.arg(flag)
                .arg(command)
                .envs(env.iter().map(|(k, v)| (k, v)));
            if let Some(cwd) = args["cwd"].as_str() {
                cmd.current_dir(crate::config::expand(cwd));
            }
            let output = cmd.output().map_err(|e| format!("could not run: {e}"))?;
            let text = format!(
                "exit {}\n{}{}",
                output
                    .status
                    .code()
                    .map_or("signal".into(), |c| c.to_string()),
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
            );
            Ok(mask(text, &env))
        }
        n => Err(format!("unknown tool {n}")),
    }
}

/// `KEY` or `repo:KEY` → the key and its one value.
fn resolve(entries: &[Entry], spec: &str) -> Result<(String, String), String> {
    let (repo, key) = match spec.rsplit_once(':') {
        Some((r, k)) => (Some(r), k),
        None => (None, spec),
    };
    let hits: Vec<&Entry> = entries
        .iter()
        .filter(|e| e.key == key && repo.is_none_or(|r| e.repo == r))
        .collect();
    let Some(first) = hits.first() else {
        return Err(format!("no key {spec}"));
    };
    if hits.iter().any(|e| e.value != first.value) {
        let repos: Vec<String> = hits.iter().map(|e| format!("{}:{key}", e.repo)).collect();
        return Err(format!(
            "{key} has different values; pick one of:\n{}",
            repos.join("\n")
        ));
    }
    Ok((key.to_string(), first.value.clone()))
}

// ponytail: plain-text masking only; an encoded value (base64 etc.) passes through.
fn mask(mut text: String, env: &[(String, String)]) -> String {
    let mut env: Vec<_> = env.iter().filter(|(_, v)| v.len() >= 3).collect();
    env.sort_by_key(|(_, v)| std::cmp::Reverse(v.len()));
    for (k, v) in env {
        text = text.replace(v.as_str(), &format!("***{k}***"));
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn e(repo: &str, key: &str, value: &str) -> Entry {
        Entry::new(
            repo.into(),
            PathBuf::new(),
            0,
            key.into(),
            value.into(),
            false,
        )
    }

    #[test]
    fn resolves_and_masks() {
        let entries = [
            e("a", "TOKEN", "s3cret"),
            e("b", "TOKEN", "other"),
            e("c", "URL", "x"),
        ];
        assert!(resolve(&entries, "TOKEN").unwrap_err().contains("a:TOKEN"));
        assert_eq!(resolve(&entries, "a:TOKEN").unwrap().1, "s3cret");
        assert!(resolve(&entries, "NOPE").is_err());
        let env = [("TOKEN".to_string(), "s3cret".to_string())];
        assert_eq!(mask("got s3cret".into(), &env), "got ***TOKEN***");
    }

    fn rpc(method: &str, params: Value) -> Value {
        let req = json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params });
        handle(&req.to_string()).unwrap()
    }

    #[test]
    fn skill_is_served_with_a_matching_manifest() {
        let init = rpc("initialize", json!({ "protocolVersion": "2025-06-18" }));
        assert!(init["result"]["capabilities"]["extensions"][SKILLS].is_object());
        let list = rpc("skills/list", json!({}));
        let skill = &list["result"]["skills"][0];
        assert_eq!(skill["frontmatter"]["name"], "mellon");
        for file in skill["resources"].as_array().unwrap() {
            let read = rpc("resources/read", json!({ "uri": file["uri"] }));
            let text = read["result"]["contents"][0]["text"].as_str().unwrap();
            assert_eq!(file["size"], text.len());
            assert_eq!(file["digest"], sha256(text));
        }
        assert_eq!(
            rpc("skills/get", json!({ "uri": "skill://nope/SKILL.md" }))["error"]["code"],
            -32602
        );
        // A known vector, so the manifest's digest is not only checked against itself.
        assert_eq!(
            sha256("hello"),
            "sha256:2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn protocol_errors() {
        assert_eq!(handle("not json").unwrap()["error"]["code"], -32700);
        assert_eq!(rpc("nope", json!({}))["error"]["code"], -32601);
        assert_eq!(
            rpc("tools/call", json!({ "name": "nope" }))["error"]["code"],
            -32602
        );
        assert!(handle(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#).is_none());
    }
}
