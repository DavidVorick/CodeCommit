use crate::app_error::AppError;
use crate::auto_workflow::prompts::{
    IMPLEMENTED_NO_CACHE, IMPLEMENTED_WITH_CACHE, RESPONSE_FORMAT_INSTRUCTIONS, SELF_CONSISTENT,
};
use crate::auto_workflow::types::Stage;
use ignore::WalkBuilder;
use std::fs;
use std::path::Path;

pub fn build_prompt(
    spec_path: &Path,
    stage: Stage,
    spec_content: &str,
) -> Result<String, AppError> {
    match stage {
        Stage::SelfConsistent => build_self_consistent_prompt(spec_content),
        Stage::Implemented => build_implemented_prompt(spec_path, spec_content),
    }
}

fn build_self_consistent_prompt(spec_content: &str) -> Result<String, AppError> {
    Ok(format!(
        "{}\n[response format instructions]\n{}\n[self consistent prompt]\n{}\n[top level UserSpecification.md]\n{}\n[target user specification]\n{}",
        "1. self-consistent",
        RESPONSE_FORMAT_INSTRUCTIONS,
        SELF_CONSISTENT,
        get_top_level_spec()?,
        spec_content
    ))
}

fn build_implemented_prompt(spec_path: &Path, spec_content: &str) -> Result<String, AppError> {
    // Determine if we have a cache to check against
    let root = Path::new(".");
    let module_dir = spec_path.parent().unwrap_or(root);
    let relative_module_dir = module_dir.strip_prefix(root).unwrap_or(module_dir);

    let cache_path = root
        .join("agent-state")
        .join("specifications")
        .join(relative_module_dir)
        .join(Stage::Implemented.as_str());

    let cached_content = if cache_path.exists() {
        Some(fs::read_to_string(&cache_path).map_err(|e| {
            AppError::FileUpdate(format!(
                "Failed to read cache {}: {}",
                cache_path.display(),
                e
            ))
        })?)
    } else {
        None
    };

    let codebase = build_codebase_context(module_dir)?;

    if let Some(cached) = cached_content {
        Ok(format!(
            "{}\n[response format instructions]\n{}\n[implementation-with-cache prompt]\n{}\n[cached target user specification]\n{}\n[target user specification]\n{}\n[top level UserSpecification.md]\n{}\n[codebase, including dependency files and top level UserSpecification]\n{}",
            "2. implemented - cached UserSpecification",
            RESPONSE_FORMAT_INSTRUCTIONS,
            IMPLEMENTED_WITH_CACHE,
            cached,
            spec_content,
            get_top_level_spec()?,
            codebase
        ))
    } else {
        Ok(format!(
            "{}\n[response format instructions]\n{}\n[implementation-no-cache prompt]\n{}\n[target user specification]\n{}\n[codebase, including dependency files and top level UserSpecification]\n{}",
            "2. implemented - no cached UserSpecification",
            RESPONSE_FORMAT_INSTRUCTIONS,
            IMPLEMENTED_NO_CACHE,
            spec_content,
            codebase
        ))
    }
}

fn get_top_level_spec() -> Result<String, AppError> {
    fs::read_to_string("UserSpecification.md").or_else(|_| Ok("".to_string()))
}

fn build_codebase_context(target_module_dir: &Path) -> Result<String, AppError> {
    // 1. Top level spec
    let mut context = String::new();
    let top_spec = get_top_level_spec()?;
    if !top_spec.is_empty() {
        context.push_str("--- UserSpecification.md ---\n");
        context.push_str(&top_spec);
        context.push_str("\n\n");
    }

    // 2. All files in target module
    let walker = WalkBuilder::new(target_module_dir)
        .hidden(false)
        .git_ignore(true)
        .max_depth(Some(1)) // Only files in this directory, not submodules (submodules have their own spec)
        .build();

    for result in walker {
        let entry = result.map_err(|e| AppError::Config(format!("Error walking module: {e}")))?;
        if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
            let path = entry.path();
            // Don't include submodule folders or files in them, but WalkBuilder with max_depth(1) handles logic.
            // But we need to ensure we don't pick up sub-directories.

            // Also exclude the UserSpecification.md itself as it is provided separately?
            // The prompt says "codebase... including dependency files and top level UserSpecification".
            // It says "every single file in the target module".
            // It's safer to include everything.

            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue, // Skip binary or unreadable
            };

            context.push_str(&format!("--- {} ---\n{}\n\n", path.display(), content));
        }
    }

    // 3. Dependencies specs and signatures
    // Parse dependencies
    let dep_file = target_module_dir.join("ModuleDependencies.md");
    if dep_file.exists() {
        let dep_content = fs::read_to_string(&dep_file).unwrap_or_default();
        for line in dep_content.lines() {
            let dep_path = line.trim();
            if dep_path.is_empty() || dep_path.starts_with('#') {
                continue;
            }

            let dep_dir = Path::new(dep_path);
            let spec_path = dep_dir.join("UserSpecification.md");
            if spec_path.exists() {
                let s = fs::read_to_string(&spec_path).unwrap_or_default();
                context.push_str(&format!("--- {} ---\n{}\n\n", spec_path.display(), s));
            }

            let sig_path = dep_dir.join("APISignatures.md");
            if sig_path.exists() {
                let s = fs::read_to_string(&sig_path).unwrap_or_default();
                context.push_str(&format!("--- {} ---\n{}\n\n", sig_path.display(), s));
            }
        }
    }

    Ok(context)
}
