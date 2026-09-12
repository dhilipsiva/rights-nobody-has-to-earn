// SPDX-License-Identifier: MIT OR Apache-2.0

//! A fixed worker pool. Every started job is joined; later jobs are cancelled
//! when an earlier case fails. No persisted results or Git state are involved.

use std::io;
use std::process::{Child, ExitStatus};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

#[derive(Clone, Debug, Default)]
pub(crate) struct CancellationToken(Arc<AtomicBool>);

impl CancellationToken {
    pub(crate) fn new() -> Self {
        Self::default()
    }
    pub(crate) fn cancel(&self) -> bool {
        !self.0.swap(true, Ordering::Relaxed)
    }
    pub(crate) fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
    pub(crate) fn flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.0)
    }
}

pub(crate) fn configured_workers() -> Result<usize, crate::cli::Error> {
    match std::env::var("RIGHTS_VERIFY_JOBS") {
        Ok(value) => value
            .parse::<usize>()
            .ok()
            .filter(|value| (1..=4).contains(value))
            .ok_or_else(|| {
                crate::cli::Error::usage("RIGHTS_VERIFY_JOBS must be an integer from 1 through 4")
            }),
        Err(std::env::VarError::NotPresent) => Ok(thread::available_parallelism()
            .map_or(1, usize::from)
            .min(4)),
        Err(error) => Err(crate::cli::Error::usage(error.to_string())),
    }
}

pub(crate) fn run<T, S, R, E>(
    jobs: &[T],
    workers: usize,
    init: impl Fn() -> S + Sync,
    execute: impl Fn(&mut S, &T, &CancellationToken) -> Result<R, E> + Sync,
) -> Result<Vec<R>, E>
where
    T: Sync,
    R: Send,
    E: Send,
{
    let next = AtomicUsize::new(0);
    let first_failure = AtomicUsize::new(jobs.len());
    let tokens: Vec<_> = jobs.iter().map(|_| CancellationToken::new()).collect();
    let mut completed = thread::scope(|scope| {
        let handles: Vec<_> = (0..workers.min(jobs.len()))
            .map(|_| {
                let (next, first_failure, tokens, init, execute) =
                    (&next, &first_failure, &tokens, &init, &execute);
                scope.spawn(move || {
                    let mut state = init();
                    let mut results = Vec::new();
                    loop {
                        let index = next.fetch_add(1, Ordering::Relaxed);
                        if index >= jobs.len() || index > first_failure.load(Ordering::Relaxed) {
                            break;
                        }
                        let result = execute(&mut state, &jobs[index], &tokens[index]);
                        if result.is_err() {
                            first_failure.fetch_min(index, Ordering::Relaxed);
                            for token in &tokens[index + 1..] {
                                token.cancel();
                            }
                        }
                        results.push((index, result));
                    }
                    results
                })
            })
            .collect();
        // The scope joins remaining threads even if a worker panics.
        handles
            .into_iter()
            .flat_map(|handle| handle.join().expect("pin worker panicked"))
            .collect::<Vec<_>>()
    });
    completed.sort_by_key(|(index, _)| *index);
    completed.into_iter().map(|(_, result)| result).collect()
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum ChildWait {
    Exited(ExitStatus),
    Cancelled(ExitStatus),
}

impl ChildWait {
    pub(crate) fn status(self) -> ExitStatus {
        match self {
            Self::Exited(status) | Self::Cancelled(status) => status,
        }
    }
    pub(crate) fn was_cancelled(self) -> bool {
        matches!(self, Self::Cancelled(_))
    }
}

pub(crate) fn wait_for_child(
    child: &mut Child,
    cancellation: &CancellationToken,
    poll: Duration,
) -> io::Result<ChildWait> {
    wait(child, cancellation, poll, false)
}

#[cfg(unix)]
pub(crate) fn wait_for_child_group(
    child: &mut Child,
    cancellation: &CancellationToken,
    poll: Duration,
) -> io::Result<ChildWait> {
    wait(child, cancellation, poll, true)
}

fn wait(
    child: &mut Child,
    cancellation: &CancellationToken,
    poll: Duration,
    group: bool,
) -> io::Result<ChildWait> {
    loop {
        if cancellation.is_cancelled() {
            if group {
                terminate_group(child.id())?;
            } else if let Err(error) = child.kill() {
                if error.kind() != io::ErrorKind::InvalidInput {
                    return Err(error);
                }
            }
            return Ok(ChildWait::Cancelled(child.wait()?));
        }
        if let Some(status) = child.try_wait()? {
            if group {
                terminate_group(child.id())?;
            }
            return Ok(ChildWait::Exited(status));
        }
        thread::sleep(poll);
    }
}

#[cfg(unix)]
fn terminate_group(child: u32) -> io::Result<()> {
    let id = i32::try_from(child).map_err(|_| io::Error::other("invalid child process group"))?;
    // SAFETY: this is the group of a child spawned with process_group(0).
    if unsafe { libc::kill(-id, libc::SIGKILL) } != 0 {
        let error = io::Error::last_os_error();
        if error.raw_os_error() != Some(libc::ESRCH) {
            return Err(error);
        }
    }
    Ok(())
}

#[cfg(not(unix))]
fn terminate_group(_: u32) -> io::Result<()> {
    unreachable!("Unix groups only")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn results_and_failure_are_in_input_order() {
        for workers in 1..=4 {
            assert_eq!(
                run(
                    &(0..12).collect::<Vec<_>>(),
                    workers,
                    || (),
                    |_, i, _| Ok::<_, usize>(i * 2)
                )
                .unwrap(),
                (0..12).map(|i| i * 2).collect::<Vec<_>>()
            );
            assert_eq!(
                run(
                    &(0..12).collect::<Vec<_>>(),
                    workers,
                    || (),
                    |_, i, _| {
                        if *i == 2 {
                            thread::sleep(Duration::from_millis(10));
                        }
                        if *i == 2 || *i == 3 { Err(*i) } else { Ok(*i) }
                    }
                ),
                Err(2)
            );
        }
    }
}
