use super::graph;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn setup_project() -> TempDir {
    tempfile::Builder::new()
        .prefix("graph-test")
        .tempdir()
        .unwrap()
}

fn create_module(root: &Path, name: &str, deps: &[&str]) {
    let mod_dir = root.join("src").join(name);
    fs::create_dir_all(&mod_dir).unwrap();

    fs::write(mod_dir.join("UserSpecification.md"), "# Spec").unwrap();

    let mut dep_content = String::from("# Module Dependencies\n\n");
    for dep in deps {
        dep_content.push_str(&format!("src/{dep}\n"));
    }
    fs::write(mod_dir.join("ModuleDependencies.md"), dep_content).unwrap();
}

#[test]
fn test_dependency_graph_levels() {
    let temp = setup_project();
    let root = temp.path();

    // Structure:
    // A -> B -> C
    // D (no deps)

    create_module(root, "C", &[]);
    create_module(root, "B", &["C"]);
    create_module(root, "A", &["B"]);
    create_module(root, "D", &[]);

    let modules = vec![
        root.join("src/A/UserSpecification.md"),
        root.join("src/B/UserSpecification.md"),
        root.join("src/C/UserSpecification.md"),
        root.join("src/D/UserSpecification.md"),
    ];

    let nodes = graph::build_dependency_graph(root, &modules).expect("Build graph");

    let find_level = |name: &str| {
        nodes
            .iter()
            .find(|n| n.path.ends_with(format!("src/{name}")))
            .map(|n| n.level)
            .unwrap()
    };

    assert_eq!(find_level("C"), 0);
    assert_eq!(find_level("D"), 0);
    assert_eq!(find_level("B"), 1);
    assert_eq!(find_level("A"), 2);
}

#[test]
fn test_cycle_detection() {
    let temp = setup_project();
    let root = temp.path();

    // A -> B -> A
    create_module(root, "A", &["B"]);
    create_module(root, "B", &["A"]);

    let modules = vec![
        root.join("src/A/UserSpecification.md"),
        root.join("src/B/UserSpecification.md"),
    ];

    let result = graph::build_dependency_graph(root, &modules);
    assert!(result.is_err());
    let err = result.err().unwrap().to_string();
    assert!(err.contains("Dependency cycle"));
}
