#![forbid(unsafe_code)]

use serde::Deserialize;
use std::time::Duration;

const DEFAULT_URL: &str = "https://litellm.nas.iks-homelab.ovh";
const DEFAULT_TOP_K: u32 = 5;
const MAX_QUERY_CHARS: usize = 1000;
const MCP_SERVER: &str = "mem0";

/// Configuration for talking to a LiteLLM MCP gateway.
#[derive(Debug, Clone)]
pub struct Config {
    /// Base URL of the LiteLLM MCP gateway, without a trailing slash.
    pub url: String,
    /// API key sent in the `x-litellm-api-key` header as a bearer token.
    pub api_key: String,
    /// Maximum number of memories to request.
    pub top_k: u32,
}

impl Config {
    /// Read configuration from process environment variables.
    pub fn from_env() -> Result<Config, String> {
        Self::from_lookup(|key| std::env::var(key).ok())
    }

    /// Read configuration from a lookup function.
    pub fn from_lookup(get: impl Fn(&str) -> Option<String>) -> Result<Config, String> {
        let api_key = match get("MEM0_LITELLM_KEY") {
            Some(value) if !value.trim().is_empty() => value.trim().to_string(),
            _ => return Err("MEM0_LITELLM_KEY is not set".to_string()),
        };

        let url = Self::string_from_lookup(&get, "LITELLM_URL", DEFAULT_URL);
        let url = url.trim_end_matches('/').to_string();
        let url = if url.is_empty() {
            DEFAULT_URL.to_string()
        } else {
            url
        };

        let top_k = match get("MEM0_TOP_K") {
            Some(value) if !value.trim().is_empty() => {
                let value = value.trim();
                let parsed: u32 = value
                    .parse()
                    .map_err(|_| "MEM0_TOP_K must be a positive integer".to_string())?;
                if parsed == 0 {
                    return Err("MEM0_TOP_K must be a positive integer".to_string());
                }
                parsed
            }
            _ => DEFAULT_TOP_K,
        };

        Ok(Config {
            url,
            api_key,
            top_k,
        })
    }

    fn string_from_lookup(
        get: impl Fn(&str) -> Option<String>,
        key: &str,
        default: &str,
    ) -> String {
        match get(key) {
            Some(value) if !value.trim().is_empty() => value.trim().to_string(),
            _ => default.to_string(),
        }
    }
}

/// A single memory fact returned by the MCP gateway.
#[derive(Debug, Clone, Deserialize)]
pub struct Memory {
    /// The memory text.
    #[serde(default)]
    pub memory: String,
}

#[derive(Deserialize)]
struct ToolResult {
    #[serde(default)]
    results: Vec<Memory>,
}

/// Search memories via the LiteLLM MCP gateway.
pub fn search(cfg: &Config, query: &str) -> Result<Vec<Memory>, String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(3)))
        .build()
        .into();

    let url = format!("{}/mcp-rest/tools/call", cfg.url);
    let body = serde_json::json!({
        "server_id": MCP_SERVER,
        "name": "search_memories",
        "arguments": {
            "query": query,
            "limit": cfg.top_k
        }
    });

    let mut response = agent
        .post(&url)
        .header("x-litellm-api-key", format!("Bearer {}", cfg.api_key))
        .send_json(body)
        .map_err(|error| describe_error(cfg, error.to_string()))?;

    let parsed: serde_json::Value = response
        .body_mut()
        .read_json()
        .map_err(|error| describe_error(cfg, error.to_string()))?;

    parse_tool_result(&parsed).map_err(|error| describe_error(cfg, error))
}

fn parse_tool_result(body: &serde_json::Value) -> Result<Vec<Memory>, String> {
    let is_error = body
        .get("isError")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);

    if is_error {
        return Err(first_text_content(body)
            .map(|text| text.chars().take(300).collect())
            .unwrap_or_else(|| "empty MCP response".to_string()));
    }

    let text = first_text_content(body).ok_or_else(|| "empty MCP response".to_string())?;
    let tool_result: ToolResult = serde_json::from_str(&text).map_err(|error| error.to_string())?;

    Ok(tool_result.results)
}

fn first_text_content(body: &serde_json::Value) -> Option<String> {
    body.get("content")
        .and_then(serde_json::Value::as_array)?
        .iter()
        .find(|item| item.get("type").and_then(serde_json::Value::as_str) == Some("text"))
        .and_then(|item| item.get("text").and_then(serde_json::Value::as_str))
        .map(|text| text.to_string())
}

fn describe_error(cfg: &Config, message: String) -> String {
    if cfg.api_key.is_empty() {
        message
    } else {
        message.replace(&cfg.api_key, "[REDACTED]")
    }
}

/// Build a search query from an optional project name and prompt.
pub fn build_query(project: Option<&str>, prompt: &str) -> String {
    let prompt = prompt.trim();
    let query = match project {
        Some(project) if !project.trim().is_empty() => format!("{}: {}", project.trim(), prompt),
        _ => prompt.to_string(),
    };
    query.chars().take(MAX_QUERY_CHARS).collect()
}

/// Extract a project name from a working directory path.
pub fn project_from_cwd(cwd: &str) -> Option<String> {
    if cwd.is_empty() {
        return None;
    }

    std::path::Path::new(cwd)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| !name.is_empty())
}

/// Format memories as a context block for an agent.
pub fn format_context(memories: &[Memory]) -> String {
    if memories.is_empty() {
        return String::new();
    }

    let mut out =
        String::from("Central memory (mem0), facts that may be relevant to this request:\n");
    for memory in memories {
        let line = memory.memory.replace(['\n', '\r'], " ");
        out.push_str("- ");
        out.push_str(&line);
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_uses_defaults() {
        let cfg = Config::from_lookup(|key| match key {
            "MEM0_LITELLM_KEY" => Some("secret".to_string()),
            _ => None,
        })
        .unwrap();

        assert_eq!(cfg.url, "https://litellm.nas.iks-homelab.ovh");
        assert_eq!(cfg.api_key, "secret");
        assert_eq!(cfg.top_k, 5);
    }

    #[test]
    fn config_requires_api_key() {
        let err = Config::from_lookup(|_| None).unwrap_err();
        assert_eq!(err, "MEM0_LITELLM_KEY is not set");

        let err =
            Config::from_lookup(|key| (key == "MEM0_LITELLM_KEY").then(String::new)).unwrap_err();
        assert_eq!(err, "MEM0_LITELLM_KEY is not set");
    }

    #[test]
    fn config_rejects_bad_top_k() {
        for value in ["0", "-1", "abc"] {
            let err = Config::from_lookup(|key| match key {
                "MEM0_LITELLM_KEY" => Some("secret".to_string()),
                "MEM0_TOP_K" => Some(value.to_string()),
                _ => None,
            })
            .unwrap_err();
            assert_eq!(err, "MEM0_TOP_K must be a positive integer");
        }
    }

    #[test]
    fn config_trims_trailing_slash() {
        let cfg = Config::from_lookup(|key| match key {
            "MEM0_LITELLM_KEY" => Some("secret".to_string()),
            "LITELLM_URL" => Some("https://example.com/".to_string()),
            _ => None,
        })
        .unwrap();

        assert_eq!(cfg.url, "https://example.com");
    }

    #[test]
    fn parse_tool_result_parses_results() {
        let body = serde_json::json!({
            "content": [
                {
                    "type": "text",
                    "text": "{\"results\":[{\"memory\":\"one\"},{\"memory\":\"two\"}]}"
                }
            ],
            "isError": false
        });

        let memories = parse_tool_result(&body).unwrap();
        assert_eq!(memories.len(), 2);
        assert_eq!(memories[0].memory, "one");
        assert_eq!(memories[1].memory, "two");
    }

    #[test]
    fn parse_tool_result_empty_results() {
        let body = serde_json::json!({
            "content": [
                {
                    "type": "text",
                    "text": "{\"results\":[]}"
                }
            ],
            "isError": false
        });

        assert!(parse_tool_result(&body).unwrap().is_empty());
    }

    #[test]
    fn parse_tool_result_is_error() {
        let body = serde_json::json!({
            "content": [
                {
                    "type": "text",
                    "text": "boom"
                }
            ],
            "isError": true
        });

        assert_eq!(parse_tool_result(&body).unwrap_err(), "boom");
    }

    #[test]
    fn parse_tool_result_missing_content() {
        let body = serde_json::json!({
            "isError": false
        });

        assert_eq!(parse_tool_result(&body).unwrap_err(), "empty MCP response");
    }

    #[test]
    fn build_query_trims_and_prefixes() {
        assert_eq!(build_query(None, "  hello  "), "hello");
        assert_eq!(build_query(Some("proj"), " hello "), "proj: hello");
        assert_eq!(build_query(Some("  "), " hello "), "hello");
    }

    #[test]
    fn build_query_truncates_on_char_boundary() {
        let long = "é".repeat(1001);
        assert_eq!(build_query(None, &long).chars().count(), 1000);

        let mixed = format!("{}{}{}", "a".repeat(999), "é", "b");
        let query = build_query(None, &mixed);
        assert_eq!(query.chars().count(), 1000);
        assert!(query.ends_with('é'));

        let prefixed = format!("{}{}", "p".repeat(998), "é");
        let query = build_query(Some("x"), &prefixed);
        assert_eq!(query.chars().count(), 1000);
    }

    #[test]
    fn project_from_cwd_returns_last_component() {
        assert_eq!(
            project_from_cwd("/home/user/project"),
            Some("project".to_string())
        );
        assert_eq!(project_from_cwd("/home/user/"), Some("user".to_string()));
        assert_eq!(project_from_cwd(""), None);
        assert_eq!(project_from_cwd("/"), None);
    }

    #[test]
    fn format_context_handles_memories() {
        assert_eq!(format_context(&[]), "");

        let memories = vec![
            Memory {
                memory: "one".to_string(),
            },
            Memory {
                memory: "two".to_string(),
            },
        ];
        assert_eq!(
            format_context(&memories),
            "Central memory (mem0), facts that may be relevant to this request:\n- one\n- two\n"
        );

        let newline = vec![Memory {
            memory: "a\nb".to_string(),
        }];
        assert_eq!(
            format_context(&newline),
            "Central memory (mem0), facts that may be relevant to this request:\n- a b\n"
        );
    }
}
