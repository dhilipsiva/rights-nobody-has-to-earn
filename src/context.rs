// SPDX-License-Identifier: MIT OR Apache-2.0

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::cli::Error;

#[derive(Clone, Debug)]
pub(crate) struct Context {
    root: Arc<PathBuf>,
}

impl Context {
    pub(crate) fn discover() -> Result<Self, Error> {
        let mut current = std::env::current_dir()?;
        loop {
            if current.join("verify.sh").is_file()
                && current.join("new-book-plans/constitution.nibli").is_file()
            {
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
