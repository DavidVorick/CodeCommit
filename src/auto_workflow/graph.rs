use crate::app_error::AppError;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct ModuleNode {
    pub path: PathBuf,
    pub dependencies: Vec<PathBuf>,
    pub level: usize,
}

pub fn build_dependency_graph(
    root: &Path,
    modules: &[PathBuf],
) -> Result<Vec<ModuleNode>, AppError> {
    let module_set: HashSet<&PathBuf> = modules.iter().collect();
    let mut nodes: HashMap<PathBuf, ModuleNode> = HashMap::new();

    // 1. Parse dependencies for all modules
    for module_spec in modules {
        let module_dir = module_spec.parent().unwrap_or(Path::new(".")).to_path_buf();
        let raw_deps = parse_dependencies(&module_dir)?;

        // Validate dependencies exist in the module list and resolve them relative to root
        let mut resolved_deps = Vec::new();
        for dep in raw_deps {
            // Dependencies are relative to the project root
            let resolved = root.join(&dep);
            let dep_spec = resolved.join("UserSpecification.md");

            // Only add if the dependency is a valid module in the project
            if module_set.contains(&dep_spec) {
                resolved_deps.push(resolved);
            }
        }

        nodes.insert(
            module_dir.clone(),
            ModuleNode {
                path: module_dir,
                dependencies: resolved_deps,
                level: 0,
            },
        );
    }

    // 2. Calculate levels (Topological Sort / Depth)
    // Level 0: No dependencies
    // Level N: Max(dep levels) + 1
    // Detect cycles

    let mut visited = HashSet::new();
    let mut stack = HashSet::new();
    let keys: Vec<PathBuf> = nodes.keys().cloned().collect();

    for key in keys {
        calculate_level(&key, &mut nodes, &mut visited, &mut stack)?;
    }

    Ok(nodes.into_values().collect())
}

fn parse_dependencies(module_dir: &Path) -> Result<Vec<PathBuf>, AppError> {
    let dep_file = module_dir.join("ModuleDependencies.md");
    if !dep_file.exists() {
        return Err(AppError::Config(format!(
            "Module {} is missing ModuleDependencies.md",
            module_dir.display()
        )));
    }

    let content = fs::read_to_string(&dep_file).map_err(|e| {
        AppError::FileUpdate(format!(
            "Failed to read dependencies for {}: {}",
            module_dir.display(),
            e
        ))
    })?;

    let mut deps = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        deps.push(PathBuf::from(trimmed));
    }
    Ok(deps)
}

fn calculate_level(
    current: &PathBuf,
    nodes: &mut HashMap<PathBuf, ModuleNode>,
    visited: &mut HashSet<PathBuf>,
    stack: &mut HashSet<PathBuf>,
) -> Result<usize, AppError> {
    if stack.contains(current) {
        return Err(AppError::Config(format!(
            "Dependency cycle detected involving {}",
            current.display()
        )));
    }
    if visited.contains(current) {
        return Ok(nodes.get(current).map(|n| n.level).unwrap_or(0));
    }

    stack.insert(current.clone());

    let mut max_dep_level = 0;
    // Clone dependencies to avoid borrow checker issues with `nodes`
    let dependencies = nodes
        .get(current)
        .map(|n| n.dependencies.clone())
        .unwrap_or_default();

    let mut has_deps = false;

    for dep in dependencies {
        if nodes.contains_key(&dep) {
            has_deps = true;
            let level = calculate_level(&dep, nodes, visited, stack)?;
            if level >= max_dep_level {
                max_dep_level = level;
            }
        }
    }

    let my_level = if has_deps { max_dep_level + 1 } else { 0 };

    if let Some(node) = nodes.get_mut(current) {
        node.level = my_level;
    }

    stack.remove(current);
    visited.insert(current.clone());

    Ok(my_level)
}
