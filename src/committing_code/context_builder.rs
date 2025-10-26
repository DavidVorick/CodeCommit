use crate::app_error::AppError;
use ignore::WalkBuilder;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

fn read_and_format_file(path: &Path) -> Result<String, AppError> {
    let content = fs::read_to_string(path).map_err(AppError::Io)?;
    Ok(format!("--- {} ---\n{}\n\n", path.display(), content))
}

pub(crate) fn build_codebase_summary() -> Result<String, AppError> {
    let mut summary = String::new();

    let root_files_to_include = [
        ".gitignore",
        "build.sh",
        "Cargo.toml",
        "LLMInstructions.md",
        "UserSpecification.md",
    ];
    for file_path in root_files_to_include {
        let path = Path::new(file_path);
        if path.exists() {
            summary.push_str(&read_and_format_file(path)?);
        }
    }

    let mut top_level_filenames = Vec::new();
    let mut src_top_level_filenames = Vec::new();
    let mut modules: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();

    for result in WalkBuilder::new("./").max_depth(Some(1)).build() {
        let entry =
            result.map_err(|e| AppError::Config(format!("Error walking directory: {e}")))?;
        if entry.depth() == 1 && entry.file_type().is_some_and(|ft| ft.is_file()) {
            top_level_filenames.push(entry.into_path());
        }
    }
    top_level_filenames.sort();

    let src_path = Path::new("src");
    if src_path.is_dir() {
        for result in WalkBuilder::new(src_path).max_depth(Some(2)).build() {
            let entry =
                result.map_err(|e| AppError::Config(format!("Error walking directory: {e}")))?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            if entry.depth() == 1 {
                src_top_level_filenames.push(path.to_path_buf());
            } else if entry.depth() == 2 {
                let module_name = path
                    .parent()
                    .unwrap()
                    .strip_prefix(src_path)
                    .unwrap()
                    .to_string_lossy()
                    .into_owned();
                modules
                    .entry(module_name)
                    .or_default()
                    .push(path.to_path_buf());
            }
        }
    }
    src_top_level_filenames.sort();

    summary.push_str("--- FILENAMES ---\n");
    for path in top_level_filenames {
        summary.push_str(&format!("{}\n", path.display()));
    }
    for path in src_top_level_filenames {
        summary.push_str(&format!("{}\n", path.display()));
    }
    summary.push_str("--- END FILENAMES ---\n\n");

    for (module_name, mut files) in modules {
        let module_path = src_path.join(&module_name);
        let public_api_path = module_path.join("PublicAPI.md");
        let internal_deps_path = module_path.join("InternalDependencies.md");

        if public_api_path.exists() && internal_deps_path.exists() {
            summary.push_str(&read_and_format_file(&internal_deps_path)?);
            summary.push_str(&read_and_format_file(&public_api_path)?);

            summary.push_str("--- FILENAMES ---\n");
            files.sort();
            for file in files {
                summary.push_str(&format!("{}\n", file.display()));
            }
            summary.push_str("--- END FILENAMES ---\n\n");
        }
    }

    Ok(summary)
}
