// SPDX-License-Identifier: MIT OR Apache-2.0

use std::ffi::OsStr;
use std::path::Path;
use std::process::{Command, Output};

use crate::cli::Error;

pub(crate) fn output<I, S>(root: &Path, program: &str, args: I) -> Result<Output, Error>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Ok(Command::new(program)
        .args(args)
        .current_dir(root)
        .output()?)
}

pub(crate) fn checked_stdout<I, S>(root: &Path, program: &str, args: I) -> Result<String, Error>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let result = output(root, program, args)?;
    if !result.status.success() {
        return Err(Error::with_exit_code(
            format!(
                "{program} exited {}: {}",
                result.status,
                String::from_utf8_lossy(&result.stderr).trim()
            ),
            result.status.code().unwrap_or(2).clamp(1, u8::MAX as i32) as u8,
        ));
    }
    String::from_utf8(result.stdout).map_err(|error| Error::new(error.to_string()))
}
