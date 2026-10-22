// SPDX-License-Identifier: MIT OR Apache-2.0

use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::cli::Error;

/// How a Git LFS pointer file begins: what a checkout made without git-lfs holds in place of the
/// constitution.
const LFS_POINTER: &[u8] = b"version https://git-lfs.github.com/spec/v1";

#[derive(Clone, Debug)]
pub(crate) struct Context {
    root: Arc<PathBuf>,
}

impl Context {
    pub(crate) fn discover() -> Result<Self, Error> {
        Self::discover_from(std::env::current_dir()?)
    }

    fn discover_from(mut current: PathBuf) -> Result<Self, Error> {
        loop {
            let constitution = current.join("book-1/source/constitution.nibli");
            if current.join("verify.sh").is_file() && constitution.is_file() {
                refuse_lfs_pointer(&constitution)?;
                return Ok(Self {
                    root: Arc::new(current),
                });
            }
            if !current.pop() {
                return Err(Error::usage(
                    "run from the rights-nobody-has-to-earn repository",
                ));
            }
        }
    }

    pub(crate) fn root(&self) -> &Path {
        self.root.as_path()
    }

    pub(crate) fn path(&self, relative: impl AsRef<Path>) -> PathBuf {
        self.root.join(relative)
    }

    pub(crate) fn read(&self, relative: impl AsRef<Path>) -> Result<String, Error> {
        Ok(std::fs::read_to_string(self.path(relative))?)
    }

    #[cfg(test)]
    pub(crate) fn from_test_root(root: PathBuf) -> Self {
        Self {
            root: Arc::new(root),
        }
    }
}

/// A checkout made without git-lfs holds a 133-byte pointer where the constitution should be. Say
/// so, rather than let the parser report the pointer as broken nibli.
fn refuse_lfs_pointer(constitution: &Path) -> Result<(), Error> {
    let mut start = Vec::new();
    std::fs::File::open(constitution)?
        .take(LFS_POINTER.len() as u64)
        .read_to_end(&mut start)?;
    if start == LFS_POINTER {
        return Err(Error::usage(format!(
            "{} is a Git LFS pointer, not the constitution: this checkout was made without \
             git-lfs. Install git-lfs and run `git lfs pull`, or see README.md (Verify) for the \
             fallback.",
            constitution.display()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::Context;

    fn checkout(constitution: &[u8]) -> tempfile::TempDir {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("verify.sh"), "#!/bin/sh\n").unwrap();
        std::fs::create_dir_all(root.path().join("book-1/source")).unwrap();
        std::fs::write(
            root.path().join("book-1/source/constitution.nibli"),
            constitution,
        )
        .unwrap();
        root
    }

    #[test]
    fn a_checkout_without_git_lfs_is_named_not_parsed() {
        let pointer = checkout(
            b"version https://git-lfs.github.com/spec/v1\n\
              oid sha256:73fbc8c86df7ac0d4174367001a18d77723e30cde6a5ce40ecb4038136a26ecb\n\
              size 57745277\n",
        );
        let error = Context::discover_from(pointer.path().join("book-1"))
            .unwrap_err()
            .to_string();
        assert!(error.contains("Git LFS pointer"), "{error}");
        assert!(error.contains("git-lfs"), "{error}");

        let complete = checkout(b"person(Ada).\n");
        let found = Context::discover_from(complete.path().join("book-1")).unwrap();
        assert_eq!(found.root(), complete.path());
    }
}
