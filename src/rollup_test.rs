use crate::rollup::build_rollup_for_base_dir;
use std::fs;
use std::io::Write;
use tempfile::TempDir;

struct TestEnv {
    dir: TempDir,
}

impl TestEnv {
    fn new() -> Self {
        Self {
            dir: tempfile::tempdir().unwrap(),
        }
    }

    fn base(&self) -> &std::path::Path {
        self.dir.path()
    }

    fn write(&self, rel: &str, content: &str) {
        let path = self.base().join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let mut f = fs::File::create(path).unwrap();
        write!(f, "{}", content).unwrap();
    }
}

#[test]
fn rollup_includes_all_non_ignored_files_and_respects_protections() {
    let env = TestEnv::new();

    // .gitignore: ignore agent-config directory, target dir, and *.log
    env.write(
        ".gitignore",
        "/agent-config\n/target/\n*.log\n",
    );

    // Included files
    env.write("Cargo.toml", "[package]\nname = \"x\"\n");
    env.write("src/main.rs", "fn main() {}\n");

    // Ignored by .gitignore
    env.write("target/debug/app", "binary\n");
    env.write("debug.log", "should-be-ignored\n");

    // Protected directories
    env.write("agent-config/secret.txt", "nope\n");
    env.write("app-data/secret.json", "{\"nope\":true}\n");

    let rollup = build_rollup_for_base_dir(env.base(), false).unwrap();

    assert!(rollup.contains("--- Cargo.toml ---\n[package]\nname = \"x\"\n\n"));
    assert!(rollup.contains("--- src/main.rs ---\nfn main() {}\n\n"));

    assert!(!rollup.contains("target/debug/app"));
    assert!(!rollup.contains("--- debug.log ---"));
    assert!(!rollup.contains("agent-config/secret.txt"));
    assert!(!rollup.contains("app-data/secret.json"));
}

#[test]
fn rollup_skips_non_utf8_files_safely() {
    let env = TestEnv::new();
    env.write(".gitignore", "");

    // Write a binary file (invalid UTF-8 sequence)
    let bin_path = env.base().join("bin.dat");
    {
        let mut f = fs::File::create(&bin_path).unwrap();
        let bytes = [0xff, 0xfe, 0xfd, 0x00, 0x01];
        f.write_all(&bytes).unwrap();
    }

    env.write("src/lib.rs", "pub fn f() {}\n");

    let rollup = build_rollup_for_base_dir(env.base(), false).unwrap();
    assert!(rollup.contains("--- src/lib.rs ---\npub fn f() {}\n\n"));
    // Ensure binary file doesn't cause inclusion or crash
    assert!(!rollup.contains("--- bin.dat ---"));
}

#[test]
fn light_roll_excludes_cargo_lock() {
    let env = TestEnv::new();
    env.write(".gitignore", "");
    env.write("Cargo.toml", "[package]\n");
    env.write("Cargo.lock", "lock file content\n");
    env.write("src/main.rs", "fn main() {}\n");

    // Regular rollup should include Cargo.lock
    let rollup = build_rollup_for_base_dir(env.base(), false).unwrap();
    assert!(rollup.contains("--- Cargo.lock ---"));

    // Light roll should exclude Cargo.lock
    let light_rollup = build_rollup_for_base_dir(env.base(), true).unwrap();
    assert!(!light_rollup.contains("--- Cargo.lock ---"));
    assert!(light_rollup.contains("--- Cargo.toml ---"));
    assert!(light_rollup.contains("--- src/main.rs ---"));
}