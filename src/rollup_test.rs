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

    env.write(".gitignore", "/agent-config\n/target/\n*.log\n");

    env.write("Cargo.toml", "[package]\nname = \"x\"\n");
    env.write("src/main.rs", "fn main() {}\n");

    env.write("target/debug/app", "binary\n");
    env.write("debug.log", "should-be-ignored\n");

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

    let bin_path = env.base().join("bin.dat");
    {
        let mut f = fs::File::create(&bin_path).unwrap();
        let bytes = [0xff, 0xfe, 0xfd, 0x00, 0x01];
        f.write_all(&bytes).unwrap();
    }

    env.write("src/lib.rs", "pub fn f() {}\n");

    let rollup = build_rollup_for_base_dir(env.base(), false).unwrap();
    assert!(rollup.contains("--- src/lib.rs ---\npub fn f() {}\n\n"));
    assert!(!rollup.contains("--- bin.dat ---"));
}

#[test]
fn rollup_by_default_excludes_cargo_lock_and_rollup_full_includes_it() {
    let env = TestEnv::new();
    env.write(".gitignore", "");
    env.write("Cargo.toml", "[package]\n");
    env.write("Cargo.lock", "lock file content\n");
    env.write("src/main.rs", "fn main() {}\n");

    let default_rollup = build_rollup_for_base_dir(env.base(), false).unwrap();
    assert!(!default_rollup.contains("--- Cargo.lock ---"));

    let full_rollup = build_rollup_for_base_dir(env.base(), true).unwrap();
    assert!(full_rollup.contains("--- Cargo.lock ---"));
    assert!(full_rollup.contains("--- Cargo.toml ---"));
    assert!(full_rollup.contains("--- src/main.rs ---"));
}