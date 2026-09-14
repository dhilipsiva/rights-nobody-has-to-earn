// SPDX-License-Identifier: MIT OR Apache-2.0

//! A fixed worker pool. Every started job is joined; later jobs are cancelled
//! when an earlier case fails. No persisted results or Git state are involved.

use std::collections::BTreeMap;
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
    run_grouped(jobs, workers, init, |_| None::<()>, execute)
}

/// Keep jobs with the same key on one worker, in their original relative order.
/// Unkeyed jobs remain independently scheduled. Results, failure priority and
/// cancellation use original job indices, never group indices: a late failure
/// in an early group must not suppress an earlier case in a different group.
pub(crate) fn run_grouped<'a, T, S, R, E, K>(
    jobs: &'a [T],
    workers: usize,
    init: impl Fn() -> S + Sync,
    key: impl Fn(&'a T) -> Option<K>,
    execute: impl Fn(&mut S, &T, &CancellationToken) -> Result<R, E> + Sync,
) -> Result<Vec<R>, E>
where
    T: Sync,
    R: Send,
    E: Send,
    K: Ord,
{
    // Groups and their members are both ordered by first occurrence. Building
    // the partition here guarantees that every input belongs to exactly one.
    let mut groups: Vec<Vec<usize>> = Vec::new();
    let mut keyed = BTreeMap::new();
    for (index, job) in jobs.iter().enumerate() {
        if let Some(key) = key(job) {
            let group = *keyed.entry(key).or_insert_with(|| {
                groups.push(Vec::new());
                groups.len() - 1
            });
            groups[group].push(index);
        } else {
            groups.push(vec![index]);
        }
    }
    let next = AtomicUsize::new(0);
    let first_failure = AtomicUsize::new(jobs.len());
    let tokens: Vec<_> = jobs.iter().map(|_| CancellationToken::new()).collect();
    let mut completed = thread::scope(|scope| {
        let handles: Vec<_> = (0..workers.min(groups.len()))
            .map(|_| {
                let (next, first_failure, tokens, init, execute, groups) =
                    (&next, &first_failure, &tokens, &init, &execute, &groups);
                scope.spawn(move || {
                    let mut state = init();
                    let mut results = Vec::new();
                    loop {
                        let group = next.fetch_add(1, Ordering::Relaxed);
                        let Some(indices) = groups.get(group) else {
                            break;
                        };
                        if indices[0] > first_failure.load(Ordering::Relaxed) {
                            break;
                        }
                        for &index in indices {
                            if index > first_failure.load(Ordering::Relaxed) {
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
    fn grouped_jobs_reuse_worker_state_and_return_every_result_in_input_order() {
        let jobs: Vec<_> = [Some(0), None, Some(1), Some(0), None, Some(1), Some(0)]
            .into_iter()
            .enumerate()
            .collect();
        for workers in 1..=4 {
            let preparations = AtomicUsize::new(0);
            let results = run_grouped(
                &jobs,
                workers,
                || None,
                |(_, key)| *key,
                |retained, (index, key), _| {
                    if key.is_some() && retained != key {
                        preparations.fetch_add(1, Ordering::Relaxed);
                        *retained = *key;
                    }
                    Ok::<_, ()>(*index)
                },
            )
            .unwrap();
            assert_eq!(results, (0..jobs.len()).collect::<Vec<_>>());
            assert_eq!(preparations.load(Ordering::Relaxed), 2);
        }
    }

    #[test]
    fn a_late_failure_in_an_early_group_does_not_suppress_earlier_cases() {
        for workers in 1..=4 {
            let result = run_grouped(
                &(0..8).collect::<Vec<_>>(),
                workers,
                || (),
                |index| (*index == 0 || *index == 5).then_some(()),
                |_, index, cancel| {
                    if *index <= 2 {
                        assert!(!cancel.is_cancelled());
                    }
                    if *index == 2 || *index == 5 {
                        Err(*index)
                    } else {
                        Ok(*index)
                    }
                },
            );
            assert_eq!(result, Err(2));
        }
    }

    #[test]
    fn grouped_cancellation_reaches_later_source_indices_in_earlier_groups() {
        for workers in 2..=4 {
            let (started, receive) = std::sync::mpsc::sync_channel(1);
            let receive = std::sync::Mutex::new(receive);
            let result = run_grouped(
                &(0..6).collect::<Vec<_>>(),
                workers,
                || (),
                |index| Some(*index == 0 || *index == 5),
                |_, index, cancel| match *index {
                    0 => Ok(0),
                    1 => {
                        receive
                            .lock()
                            .unwrap()
                            .recv_timeout(Duration::from_secs(3))
                            .unwrap();
                        Err(1)
                    }
                    5 => {
                        started.send(()).unwrap();
                        let deadline = std::time::Instant::now() + Duration::from_secs(3);
                        while !cancel.is_cancelled() {
                            assert!(
                                std::time::Instant::now() < deadline,
                                "later case was not cancelled"
                            );
                            thread::sleep(Duration::from_millis(1));
                        }
                        Err(5)
                    }
                    _ => panic!("case after the first failure was started"),
                },
            );
            assert_eq!(result, Err(1));
        }
    }

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
