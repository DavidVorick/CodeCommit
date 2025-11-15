use crate::app_error::AppError;
use crate::cli::CliArgs;
use crate::logger::Logger;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use ignore::WalkBuilder;
use path_clean::PathClean;
use std::fs;
use std::path::{Component, Path, PathBuf};

pub async fn run(_logger: &Logger, cli_args: CliArgs) -> Result<(), AppError> {
    let rollup = build_rollup_for_base_dir(Path::new("."), cli_args.light_roll)?;
    let out_dir = Path::new("agent-config");
    fs::create_dir_all(out_dir)?;
    let out_path = out_dir.join("codebase.txt");
    fs::write(&out_path, rollup)?;
    println!("Codebase rollup saved to {}", out_path.display());
    Ok(())
}

fn build_gitignore_matcher(base_dir: &Path) -> Result<Gitignore, AppError> {
    let mut builder = GitignoreBuilder::new(base_dir);
    builder.add(base_dir.join(".gitignore"));
    builder
        .build()
        .map_err(|e| AppError::Config(format!("Failed to build .gitignore matcher: {e}")))
}

fn to_relative_string(base_dir: &Path, path: &Path) -> String {
    let cleaned = path.clean();
    if let Ok(stripped) = cleaned.strip_prefix(base_dir) {
        stripped.to_string_lossy().to_string()
    } else {
        cleaned.to_string_lossy().to_string()
    }
}

fn validate_path(path: &Path, base_dir: &Path, ignore: &Gitignore) -> Result<bool, AppError> {
    for c in path.components() {
        match c {
            Component::RootDir => {
                return Err(AppError::FileUpdate(
                    "Absolute paths are not allowed.".to_string(),
                ))
            }
            Component::ParentDir => {
                return Err(AppError::FileUpdate(
                    "Path traversal ('..') is not allowed.".to_string(),
                ))
            }
            _ => {}
        }
    }

    let rel = if let Ok(s) = path.clean().strip_prefix(base_dir) {
        s.to_path_buf()
    } else {
        path.clean()
    };

    if rel.components().any(|c| c.as_os_str() == ".git") {
        return Ok(false);
    }

    let allowed_agent_query = Path::new("agent-config").join("query.txt");
    let is_allowed_agent_query = rel == allowed_agent_query;

    if let Some(Component::Normal(first)) = rel.components().next() {
        if let Some(name) = first.to_str() {
            if matches!(name, "agent-config" | "app-data") && !is_allowed_agent_query {
                return Ok(false);
            }
        }
    }

    if !is_allowed_agent_query {
        if let ignore::Match::Ignore(_) = ignore.matched_path_or_any_parents(&rel, false) {
            return Ok(false);
        }
    }

    Ok(true)
}

pub(crate) fn build_rollup_for_base_dir(
    base_dir: &Path,
    light_roll: bool,
) -> Result<String, AppError> {
    let matcher = build_gitignore_matcher(base_dir)?;
    let mut files: Vec<PathBuf> = Vec::new();

    for result in WalkBuilder::new(base_dir)
        .follow_links(false)
        .git_ignore(false)
        .git_global(false)
        .parents(true)
        .ignore(false)
        .build()
    {
        let entry =
            result.map_err(|e| AppError::Config(format!("Error walking directory: {e}")))?;
        let path = entry.path().to_path_buf();
        if entry.file_type().is_some_and(|ft| ft.is_file())
            && validate_path(&path, base_dir, &matcher)?
        {
            if light_roll && path.file_name().and_then(|s| s.to_str()) == Some("Cargo.lock") {
                continue;
            }
            files.push(path);
        }
    }

    files.sort_by(|a, b| {
        let ra = to_relative_string(base_dir, a);
        let rb = to_relative_string(base_dir, b);
        ra.cmp(&rb)
    });

    let mut out = String::new();
    for file in files {
        let rel = to_relative_string(base_dir, &file);
        match fs::read_to_string(&file) {
            Ok(content) => {
                out.push_str(&format!("--- {rel} ---\n"));
                out.push_str(&content);
                if !content.ends_with('\n') {
                    out.push('\n');
                }
                out.push('\n');
            }
            Err(e) if e.kind() == std::io::ErrorKind::InvalidData => {
                // Skip non-UTF-8 files safely
            }
            Err(e) => {
                return Err(AppError::FileUpdate(format!(
                    "Failed to read file for rollup {rel}: {e}"
                )));
            }
        }
    }

    Ok(out)
}
