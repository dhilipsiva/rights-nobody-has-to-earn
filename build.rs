// SPDX-License-Identifier: MIT OR Apache-2.0

//! Embed exact verifier and Nibli dependency inputs in `rights-verify`.

use std::env;
use std::ffi::OsStr;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use sha2::{Digest, Sha256};

const NIBLI_CRATES: &[&str] = &[
    "nibli-engine",
    "nibli-kr",
    "nibli-lexicon",
    "nibli-protocol",
    "nibli-reason",
    "nibli-render",
    "nibli-semantics",
    "nibli-session",
    "nibli-store",
    "nibli-types",
];

fn main() {
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest directory"));
    let verifier_inputs = verifier_inputs(&root);
    println!(
        "cargo:rustc-env=RIGHTS_VERIFY_COMPILED_INPUTS_SHA256={}",
        input_digest(&root, &verifier_inputs)
    );
    watch_inputs(&root, &verifier_inputs);

    let nibli = root.join("../nibli");
    let nibli_inputs = nibli_dependency_inputs(&nibli);
    println!(
        "cargo:rustc-env=RIGHTS_VERIFY_COMPILED_NIBLI_COMMIT={}",
        String::from_utf8(git(&nibli, ["rev-parse", "HEAD"]))
            .expect("Nibli commit is UTF-8")
            .trim()
    );
    println!(
        "cargo:rustc-env=RIGHTS_VERIFY_COMPILED_NIBLI_INPUTS_SHA256={}",
        input_digest(&nibli, &nibli_inputs)
    );
    watch_inputs(&nibli, &nibli_inputs);
    watch_nibli_git_identity(&nibli);
}

fn verifier_inputs(root: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for name in ["Cargo.toml", "Cargo.lock", "build.rs"] {
        add_if_file(root, Path::new(name), &mut paths);
    }
    collect_files(root, Path::new("src"), &mut paths);
    sort_paths(&mut paths);
    paths
}

fn nibli_dependency_inputs(root: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for name in ["Cargo.toml", "Cargo.lock"] {
        add_if_file(root, Path::new(name), &mut paths);
    }
    for crate_name in NIBLI_CRATES {
        let crate_root = Path::new(crate_name);
        add_if_file(root, &crate_root.join("Cargo.toml"), &mut paths);
        add_if_file(root, &crate_root.join("build.rs"), &mut paths);
        let source = crate_root.join("src");
        if root.join(&source).is_dir() {
            collect_files(root, &source, &mut paths);
        }
    }
    sort_paths(&mut paths);
    paths
}

fn add_if_file(root: &Path, relative: &Path, output: &mut Vec<PathBuf>) {
    let path = root.join(relative);
    if path.is_file() || path.is_symlink() {
        output.push(relative.to_path_buf());
    }
}

fn collect_files(root: &Path, relative_directory: &Path, output: &mut Vec<PathBuf>) {
    let directory = root.join(relative_directory);
    let mut entries = fs::read_dir(&directory)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", directory.display()))
        .map(|entry| entry.expect("read build input entry").path())
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        left.as_os_str()
            .as_bytes()
            .cmp(right.as_os_str().as_bytes())
    });
    for path in entries {
        let metadata = fs::symlink_metadata(&path)
            .unwrap_or_else(|error| panic!("cannot inspect {}: {error}", path.display()));
        let relative = path
            .strip_prefix(root)
            .expect("build input below source root")
            .to_path_buf();
        if metadata.file_type().is_dir() {
            collect_files(root, &relative, output);
        } else if metadata.file_type().is_file() || metadata.file_type().is_symlink() {
            output.push(relative);
        } else {
            panic!("unsupported build input: {}", path.display());
        }
    }
}

fn sort_paths(paths: &mut [PathBuf]) {
    paths.sort_by(|left, right| {
        left.as_os_str()
            .as_bytes()
            .cmp(right.as_os_str().as_bytes())
    });
}

fn input_digest(root: &Path, paths: &[PathBuf]) -> String {
    let mut digest = Sha256::new();
    for relative in paths {
        let path = root.join(relative);
        let metadata = fs::symlink_metadata(&path)
            .unwrap_or_else(|error| panic!("cannot inspect {}: {error}", path.display()));
        let body = if metadata.file_type().is_symlink() {
            fs::read_link(&path)
                .expect("read build-input symlink")
                .as_os_str()
                .as_bytes()
                .to_vec()
        } else {
            fs::read(&path)
                .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()))
        };
        bind(&mut digest, relative.as_os_str().as_bytes());
        bind(&mut digest, &body);
    }
    format!("{:x}", digest.finalize())
}

fn watch_inputs(root: &Path, paths: &[PathBuf]) {
    for relative in paths {
        println!("cargo:rerun-if-changed={}", root.join(relative).display());
    }
}

fn watch_nibli_git_identity(root: &Path) {
    let git_directory = PathBuf::from(
        String::from_utf8(git(root, ["rev-parse", "--absolute-git-dir"]))
            .expect("Nibli Git directory is UTF-8")
            .trim(),
    );
    for name in ["HEAD", "index", "packed-refs"] {
        let path = git_directory.join(name);
        if path.exists() {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
    if let Some(reference) = git_optional(root, ["symbolic-ref", "-q", "HEAD"]) {
        let path = git_directory.join(
            String::from_utf8(reference)
                .expect("Nibli Git reference is UTF-8")
                .trim(),
        );
        if path.exists() {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
}

fn git<I, S>(root: &Path, arguments: I) -> Vec<u8>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    git_optional(root, arguments).unwrap_or_else(|| {
        panic!(
            "Git provenance command failed for Nibli at {}",
            root.display()
        )
    })
}

fn git_optional<I, S>(root: &Path, arguments: I) -> Option<Vec<u8>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(arguments)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap_or_else(|error| panic!("cannot execute Git for {}: {error}", root.display()));
    output.status.success().then_some(output.stdout)
}

fn bind(digest: &mut Sha256, value: &[u8]) {
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value);
}
