#![forbid(unsafe_code)]

use mem0_recall::{Config, build_query, format_context, project_from_cwd, search};
use serde::Deserialize;
use std::io::Read;
use std::process::ExitCode;

#[derive(Deserialize)]
struct HookInput {
    prompt: Option<String>,
    cwd: Option<String>,
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let result = match args.get(1).map(String::as_str) {
        Some("hook") => hook_query().and_then(|query| recall(&query)),
        Some("search") => recall(args[2..].join(" ").trim()),
        _ => {
            eprintln!("usage: mem0-recall hook | mem0-recall search <query...>");
            return ExitCode::from(2);
        }
    };
    // A hook must never break the agent: errors go to stderr, the exit code stays 0.
    if let Err(error) = result {
        eprintln!("mem0-recall: {error}");
    }
    ExitCode::SUCCESS
}

/// Reads the Claude Code hook JSON from stdin and builds the search query ("" to skip).
fn hook_query() -> Result<String, String> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .map_err(|e| e.to_string())?;
    let hook: HookInput = serde_json::from_str(&input).map_err(|e| e.to_string())?;
    let prompt = hook.prompt.unwrap_or_default();
    if prompt.trim().is_empty() {
        return Ok(String::new());
    }
    let project = hook.cwd.as_deref().and_then(project_from_cwd);
    Ok(build_query(project.as_deref(), &prompt))
}

fn recall(query: &str) -> Result<(), String> {
    if query.is_empty() {
        return Ok(());
    }
    let memories = search(&Config::from_env()?, query)?;
    print!("{}", format_context(&memories));
    Ok(())
}
