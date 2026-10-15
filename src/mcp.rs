//! `envhub mcp`: an MCP server on stdio that lets agents use values without being shown them.

use std::io::{self, BufRead, Write};
use std::process::Command;

use serde_json::{Value, json};

use crate::config::Config;
use crate::store::{self, Entry};

pub fn serve() -> io::Result<()> {
    let stdin = io::stdin();
    let mut out = io::stdout().lock();
    for line in stdin.lock().lines() {
        let Ok(req) = serde_json::from_str::<Value>(&line?) else {
            continue;
        };
        // Notifications carry no id and get no reply.
        let Some(id) = req.get("id").cloned() else {
            continue;
        };
        let params = req.get("params").cloned().unwrap_or(Value::Null);
        let reply = match req["method"].as_str().unwrap_or("") {
            "initialize" => json!({"result": {
                "protocolVersion": params["protocolVersion"].as_str().unwrap_or("2025-06-18"),
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "envhub", "version": env!("CARGO_PKG_VERSION")},
            }}),
            "ping" => json!({"result": {}}),
            "tools/list" => json!({"result": {"tools": tools()}}),
            "tools/call" => {
                let (text, is_error) = match call(&params) {
                    Ok(t) => (t, false),
                    Err(t) => (t, true),
                };
                json!({"result": {"content": [{"type": "text", "text": text}], "isError": is_error}})
            }
            m => json!({"error": {"code": -32601, "message": format!("unknown method {m}")}}),
        };
        let mut reply = reply;
        reply["jsonrpc"] = json!("2.0");
        reply["id"] = id;
        writeln!(out, "{reply}")?;
        out.flush()?;
    }
    Ok(())
}

fn tools() -> Value {
    json!([
        {
            "name": "list_keys",
            "description": "List env keys found in the .env files on this machine, as `repo:KEY`. Values are never shown. Terms in `query` are ANDed over repo and key.",
            "inputSchema": {"type": "object", "properties": {"query": {"type": "string"}}},
        },
        {
            "name": "run_with_env",
            "description": "Run a shell command with the given keys set as environment variables, without showing their values. Reference them as $KEY in the command. A key is `KEY` or `repo:KEY` when several repos hold different values. Values are masked in the output.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "command": {"type": "string"},
                    "keys": {"type": "array", "items": {"type": "string"}},
                    "cwd": {"type": "string"},
                },
                "required": ["command", "keys"],
            },
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
            let mut cmd = Command::new("sh");
            cmd.arg("-c")
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
}
