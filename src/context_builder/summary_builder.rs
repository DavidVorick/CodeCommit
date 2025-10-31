use crate::app_error::AppError;
use ignore::WalkBuilder;
use path_clean::PathClean;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::path_filter::PathFilter;

fn read_and_format_file(path: &Path) -> Result<String, AppError> {
    let content = fs::read_to_string(path).map_err(AppError::Io)?;
    Ok(format!("--- {} ---\n{}\n\n", path.display(), content))
}

fn to_relative_string(path: &Path) -> String {
    let cleaned = path.clean();
    if cleaned.is_absolute() {
        if let Ok(cwd) = std::env::current_dir() {
            if let Ok(stripped) = cleaned.strip_prefix(&cwd) {
                return stripped.to_string_lossy().to_string();
            }
        }
    }
    cleaned.to_string_lossy().to_string()
}

pub(crate) fn build_summary() -> Result<String, AppError> {
    let mut summary = String::new();

    summary.push_str("=== Project Root ===\n\n");

    let root_files_to_include = [
        ".gitignore",
        "build.sh",
        "Cargo.toml",
        "LLMInstructions.md",
        "UserSpecification.md",
    ];

    let filter = PathFilter::new()?;

    for file_path in root_files_to_include {
        let path = Path::new(file_path);
        if path.exists() {
            match filter.validate(path) {
                Ok(()) => {
                    summary.push_str(&read_and_format_file(path)?);
                }
                Err(AppError::FileUpdate(msg)) => {
                    return Err(AppError::Config(format!(
                        "Mandatory file '{}' is ignored by .gitignore or is invalid: {}",
                        path.display(),
                        msg
                    )));
                }
                Err(e) => {
                    return Err(AppError::Config(format!(
                        "Mandatory file '{}' is invalid: {}",
                        path.display(),
                        e
                    )));
                }
            }
        }
    }

    let mut top_level_filenames: Vec<String> = Vec::new();
    let mut src_top_level_filenames: Vec<String> = Vec::new();
    let mut modules: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();

    for result in WalkBuilder::new("./")
        .max_depth(Some(1))
        .git_ignore(true)
        .parents(true)
        .ignore(false)
        .git_global(false)
        .build()
    {
        let entry =
            result.map_err(|e| AppError::Config(format!("Error walking directory: {e}")))?;
        if entry.depth() == 1 && entry.file_type().is_some_and(|ft| ft.is_file()) {
            let rel = to_relative_string(&entry.into_path());
            top_level_filenames.push(rel);
        }
    }
    top_level_filenames.sort();

    let src_path = Path::new("src");
    if src_path.is_dir() {
        for result in WalkBuilder::new(src_path)
            .max_depth(Some(2))
            .git_ignore(true)
            .parents(true)
            .ignore(false)
            .git_global(false)
            .build()
        {
            let entry =
                result.map_err(|e| AppError::Config(format!("Error walking directory: {e}")))?;
            let path = entry.path().clean();
            if !path.is_file() {
                continue;
            }

            if entry.depth() == 1 {
                let rel = to_relative_string(&path);
                src_top_level_filenames.push(rel);
            } else if entry.depth() == 2 {
                let module_name = path
                    .parent()
                    .unwrap()
                    .strip_prefix(src_path)
                    .unwrap()
                    .to_string_lossy()
                    .into_owned();
                modules.entry(module_name).or_default().push(path);
            }
        }
    }
    src_top_level_filenames.sort();

    summary.push_str("--- FILENAMES ---\n");
    for name in top_level_filenames {
        summary.push_str(&name);
        summary.push('\n');
    }
    for name in src_top_level_filenames {
        summary.push_str(&name);
        summary.push('\n');
    }
    summary.push_str("--- END FILENAMES ---\n\n");

    for (module_name, mut files) in modules {
        let module_path = src_path.join(&module_name);
        summary.push_str(&format!("=== {} ===\n\n", module_path.display()));

        let internal_deps_path = module_path.join("InternalDependencies.md");
        if internal_deps_path.exists() {
            summary.push_str(&read_and_format_file(&internal_deps_path)?);
        }

        let public_api_path = module_path.join("PublicAPI.md");
        if public_api_path.exists() {
            summary.push_str(&read_and_format_file(&public_api_path)?);
        }

        summary.push_str("--- FILENAMES ---\n");
        files.sort();
        for file in files {
            summary.push_str(&format!("{}\n", to_relative_string(&file)));
        }
        summary.push_str("--- END FILENAMES ---\n\n");
    }

    Ok(summary)
}
