// SPDX-License-Identifier: MIT OR Apache-2.0

//! Deterministic, bounded execution for independent verification jobs.
//!
//! Jobs may finish in any order, but successful results are returned in the
//! caller's original order. The coordinator starts no new work after the first
//! reported failure, raises a shared cancellation token, and joins every worker
//! it started before returning. Workers that own child processes should either
//! poll [`CancellationToken::is_cancelled`] or register a best-effort cleanup
//! hook with [`CancellationToken::on_cancel`].

use std::collections::BTreeMap;
use std::io;
use std::panic::{self, AssertUnwindSafe};
use std::process::{Child, ExitStatus};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Weak, mpsc};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// A clonable, one-way signal shared by the scheduler and every active job.
#[derive(Clone, Debug)]
pub(crate) struct CancellationToken {
    state: Arc<CancellationState>,
}

#[derive(Default)]
struct CancellationState {
    cancelled: AtomicBool,
    next_hook_id: AtomicU64,
    hooks: Mutex<BTreeMap<u64, CancelHook>>,
}

type CancelHook = Box<dyn FnOnce() + Send + 'static>;

impl std::fmt::Debug for CancellationState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CancellationState")
            .field("cancelled", &self.cancelled.load(Ordering::Acquire))
            .finish_non_exhaustive()
    }
}

impl CancellationToken {
    pub(crate) fn new() -> Self {
        Self {
            state: Arc::new(CancellationState::default()),
        }
    }

    pub(crate) fn is_cancelled(&self) -> bool {
        self.state.cancelled.load(Ordering::Acquire)
    }

    /// Raise the signal once and run every registered hook.
    ///
    /// Hooks are removed before they run, so a hook may register another hook
    /// without deadlocking. A panicking cleanup hook cannot prevent the other
    /// hooks from running or unwind through the scheduler.
    pub(crate) fn cancel(&self) -> bool {
        if self.state.cancelled.swap(true, Ordering::AcqRel) {
            return false;
        }

        let hooks = {
            let mut registered = self
                .state
                .hooks
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            std::mem::take(&mut *registered)
        };
        for (_, hook) in hooks {
            let _ = panic::catch_unwind(AssertUnwindSafe(hook));
        }
        true
    }

    /// Register cleanup to run if another job fails.
    ///
    /// Dropping the returned guard before cancellation unregisters the hook.
    /// If cancellation has already happened, the hook runs synchronously and
    /// the returned guard is inert.
    pub(crate) fn on_cancel(&self, hook: impl FnOnce() + Send + 'static) -> CancellationHookGuard {
        let id = self.state.next_hook_id.fetch_add(1, Ordering::Relaxed);
        let mut hook = Some(Box::new(hook) as CancelHook);
        let mut registered = self
            .state
            .hooks
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if self.state.cancelled.load(Ordering::Acquire) {
            drop(registered);
            if let Some(hook) = hook.take() {
                let _ = panic::catch_unwind(AssertUnwindSafe(hook));
            }
            return CancellationHookGuard::inactive();
        }
        registered.insert(id, hook.take().expect("cancellation hook is present"));
        CancellationHookGuard {
            state: Arc::downgrade(&self.state),
            id: Some(id),
        }
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

/// Unregisters one cancellation hook when its protected resource is finished.
#[derive(Debug)]
pub(crate) struct CancellationHookGuard {
    state: Weak<CancellationState>,
    id: Option<u64>,
}

impl CancellationHookGuard {
    fn inactive() -> Self {
        Self {
            state: Weak::new(),
            id: None,
        }
    }

    /// Keep the hook registered after this guard is dropped.
    #[allow(dead_code)]
    pub(crate) fn disarm(mut self) {
        self.id = None;
    }
}

impl Drop for CancellationHookGuard {
    fn drop(&mut self) {
        let (Some(state), Some(id)) = (self.state.upgrade(), self.id.take()) else {
            return;
        };
        let mut hooks = state
            .hooks
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        hooks.remove(&id);
    }
}

/// Why bounded execution stopped.
#[derive(Debug)]
pub(crate) enum ScheduleError<E> {
    InvalidWorkerCount,
    JobFailed { index: usize, source: E },
    WorkerPanicked { index: usize, message: String },
    CoordinatorLostWorker { active_indices: Vec<usize> },
}

impl<E: std::fmt::Display> std::fmt::Display for ScheduleError<E> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidWorkerCount => formatter.write_str("worker count must be positive"),
            Self::JobFailed { index, source } => {
                write!(formatter, "job {index} failed: {source}")
            }
            Self::WorkerPanicked { index, message } => {
                write!(formatter, "job {index} panicked: {message}")
            }
            Self::CoordinatorLostWorker { active_indices } => write!(
                formatter,
                "scheduler lost its completion channel with active jobs {active_indices:?}",
            ),
        }
    }
}

impl<E: std::fmt::Debug + std::fmt::Display> std::error::Error for ScheduleError<E> {}

enum WorkerMessage<T, E> {
    Finished { index: usize, result: Result<T, E> },
    Panicked { index: usize, message: String },
}

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_owned()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "non-string panic payload".to_owned()
    }
}

fn start_job<J, T, E, F>(
    index: usize,
    job: J,
    run: Arc<F>,
    cancellation: CancellationToken,
    sender: mpsc::Sender<WorkerMessage<T, E>>,
) -> JoinHandle<()>
where
    J: Send + 'static,
    T: Send + 'static,
    E: Send + 'static,
    F: Fn(usize, J, CancellationToken) -> Result<T, E> + Send + Sync + 'static,
{
    thread::spawn(move || {
        let outcome = panic::catch_unwind(AssertUnwindSafe(|| run(index, job, cancellation)));
        let message = match outcome {
            Ok(result) => WorkerMessage::Finished { index, result },
            Err(payload) => WorkerMessage::Panicked {
                index,
                message: panic_message(payload),
            },
        };
        // The coordinator owns a sender until all workers have been joined. If
        // it is already gone, there is no caller left to consume this result.
        let _ = sender.send(message);
    })
}

/// Run independent jobs under one sliding worker bound.
///
/// At most `workers` jobs are live. A successful completion opens exactly one
/// slot for the next canonical job. The first reported error cancels the shared
/// token and prevents any later job from starting. Every already-started worker
/// is joined before the error is returned.
pub(crate) fn run_bounded<J, T, E, F>(
    jobs: impl IntoIterator<Item = J>,
    workers: usize,
    run: F,
) -> Result<Vec<T>, ScheduleError<E>>
where
    J: Send + 'static,
    T: Send + 'static,
    E: Send + 'static,
    F: Fn(usize, J, CancellationToken) -> Result<T, E> + Send + Sync + 'static,
{
    if workers == 0 {
        return Err(ScheduleError::InvalidWorkerCount);
    }

    let mut jobs: Vec<Option<J>> = jobs.into_iter().map(Some).collect();
    if jobs.is_empty() {
        return Ok(Vec::new());
    }

    let total = jobs.len();
    let run = Arc::new(run);
    let cancellation = CancellationToken::new();
    let (sender, receiver) = mpsc::channel();
    let mut active: BTreeMap<usize, JoinHandle<()>> = BTreeMap::new();
    let mut results: Vec<Option<T>> = std::iter::repeat_with(|| None).take(total).collect();
    let mut next = 0;

    while next < total && active.len() < workers {
        let job = jobs[next].take().expect("unstarted job is present");
        active.insert(
            next,
            start_job(
                next,
                job,
                Arc::clone(&run),
                cancellation.clone(),
                sender.clone(),
            ),
        );
        next += 1;
    }

    let mut first_error = None;
    while !active.is_empty() {
        let message = match receiver.recv() {
            Ok(message) => message,
            Err(_) => {
                cancellation.cancel();
                first_error = Some(ScheduleError::CoordinatorLostWorker {
                    active_indices: active.keys().copied().collect(),
                });
                break;
            }
        };

        let index = match &message {
            WorkerMessage::Finished { index, .. } | WorkerMessage::Panicked { index, .. } => *index,
        };
        if let Some(handle) = active.remove(&index) {
            // The worker catches its task panic, so a panic here can only be in
            // message delivery or thread teardown. Treat it as the same job's
            // failure instead of leaving an unjoined worker behind.
            if let Err(payload) = handle.join() {
                cancellation.cancel();
                if first_error.is_none() {
                    first_error = Some(ScheduleError::WorkerPanicked {
                        index,
                        message: panic_message(payload),
                    });
                }
            }
        }

        match message {
            WorkerMessage::Finished {
                index,
                result: Ok(value),
            } => results[index] = Some(value),
            WorkerMessage::Finished {
                index,
                result: Err(source),
            } => {
                cancellation.cancel();
                if first_error.is_none() {
                    first_error = Some(ScheduleError::JobFailed { index, source });
                }
            }
            WorkerMessage::Panicked { index, message } => {
                cancellation.cancel();
                if first_error.is_none() {
                    first_error = Some(ScheduleError::WorkerPanicked { index, message });
                }
            }
        }

        if first_error.is_none() && next < total {
            let job = jobs[next].take().expect("unstarted job is present");
            active.insert(
                next,
                start_job(
                    next,
                    job,
                    Arc::clone(&run),
                    cancellation.clone(),
                    sender.clone(),
                ),
            );
            next += 1;
        }
    }

    if first_error.is_some() {
        cancellation.cancel();
        // Active jobs are expected to observe the token or their registered
        // cleanup hook. Join all of them so no child-owning worker is orphaned.
        for (_, handle) in active {
            let _ = handle.join();
        }
        return Err(first_error.expect("failure was checked"));
    }

    Ok(results
        .into_iter()
        .enumerate()
        .map(|(index, result)| result.unwrap_or_else(|| panic!("job {index} returned no result")))
        .collect())
}

/// Cheap in-process behavioral check used by both quick and full verification.
/// It exercises the sliding bound, canonical output ordering, and fail-fast
/// cancellation without spawning an interpreter or shell harness.
pub(crate) fn self_test() -> Result<String, crate::cli::Error> {
    use std::sync::atomic::{AtomicUsize, Ordering};

    let active = Arc::new(AtomicUsize::new(0));
    let maximum = Arc::new(AtomicUsize::new(0));
    let active_jobs = Arc::clone(&active);
    let maximum_jobs = Arc::clone(&maximum);
    let ordered = run_bounded(0..12, 4, move |index, job, _| {
        let now = active_jobs.fetch_add(1, Ordering::SeqCst) + 1;
        maximum_jobs.fetch_max(now, Ordering::SeqCst);
        if job < 4 {
            thread::sleep(Duration::from_millis(3));
        }
        active_jobs.fetch_sub(1, Ordering::SeqCst);
        Ok::<_, &'static str>((index, job))
    })
    .map_err(|error| crate::cli::Error::new(format!("scheduler self-test failed: {error}")))?;
    if maximum.load(Ordering::SeqCst) > 4
        || active.load(Ordering::SeqCst) != 0
        || ordered
            .iter()
            .enumerate()
            .any(|(index, result)| *result != (index, index))
    {
        return Err(crate::cli::Error::new(
            "scheduler self-test violated its bound or canonical order",
        ));
    }

    let started = Arc::new(AtomicUsize::new(0));
    let started_jobs = Arc::clone(&started);
    let failure = run_bounded(0..8, 2, move |_, job, cancellation| {
        started_jobs.fetch_add(1, Ordering::SeqCst);
        if job == 1 {
            return Err("watched failure");
        }
        while !cancellation.is_cancelled() {
            thread::yield_now();
        }
        Ok(job)
    });
    if !matches!(
        failure,
        Err(ScheduleError::JobFailed {
            source: "watched failure",
            ..
        })
    ) || started.load(Ordering::SeqCst) > 2
    {
        return Err(crate::cli::Error::new(
            "scheduler self-test did not fail fast",
        ));
    }
    Ok("native bounded scheduler self-test passes".to_owned())
}

/// Result of waiting on a child with cooperative scheduler cancellation.
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

/// Poll, terminate, and reap a child owned by one scheduled job.
///
/// This uses only `std`, so it terminates the immediate child rather than an OS
/// process group. Callers that create a process group may register a stronger
/// platform-specific hook with [`CancellationToken::on_cancel`].
pub(crate) fn wait_for_child(
    child: &mut Child,
    cancellation: &CancellationToken,
    poll_interval: Duration,
) -> io::Result<ChildWait> {
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(ChildWait::Exited(status));
        }
        if cancellation.is_cancelled() {
            // A child can exit between try_wait and kill. Ignore InvalidInput,
            // which is how some platforms report an already-exited process,
            // then always wait to release the process-table entry.
            if let Err(error) = child.kill()
                && error.kind() != io::ErrorKind::InvalidInput
            {
                return Err(error);
            }
            return child.wait().map(ChildWait::Cancelled);
        }
        thread::sleep(poll_interval.max(Duration::from_millis(1)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verifier_self_test_passes() {
        assert_eq!(
            self_test().expect("scheduler verifier self-test"),
            "native bounded scheduler self-test passes"
        );
    }
    use std::sync::Barrier;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Instant;

    #[test]
    fn results_are_returned_in_canonical_job_order() {
        let delays = [35_u64, 5, 20, 1];
        let results = run_bounded(delays, 4, |index, delay, _| {
            thread::sleep(Duration::from_millis(delay));
            Ok::<_, &'static str>(format!("job-{index}"))
        })
        .expect("jobs pass");

        assert_eq!(results, ["job-0", "job-1", "job-2", "job-3"]);
    }

    #[test]
    fn worker_bound_is_never_exceeded() {
        let active = Arc::new(AtomicUsize::new(0));
        let maximum = Arc::new(AtomicUsize::new(0));
        let gate = Arc::new(Barrier::new(3));
        let active_for_jobs = Arc::clone(&active);
        let maximum_for_jobs = Arc::clone(&maximum);
        let gate_for_jobs = Arc::clone(&gate);

        let results = run_bounded(0..12, 3, move |_, job, _| {
            let now = active_for_jobs.fetch_add(1, Ordering::SeqCst) + 1;
            maximum_for_jobs.fetch_max(now, Ordering::SeqCst);
            if job < 3 {
                gate_for_jobs.wait();
            }
            thread::sleep(Duration::from_millis(3));
            active_for_jobs.fetch_sub(1, Ordering::SeqCst);
            Ok::<_, &'static str>(job)
        })
        .expect("jobs pass");

        assert_eq!(results, (0..12).collect::<Vec<_>>());
        assert_eq!(maximum.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn first_failure_cancels_active_work_and_starts_nothing_else() {
        let started = Arc::new(Mutex::new(Vec::new()));
        let observed_cancel = Arc::new(AtomicBool::new(false));
        let started_for_jobs = Arc::clone(&started);
        let observed_for_jobs = Arc::clone(&observed_cancel);

        let error = run_bounded(0..8, 2, move |_, job, cancellation| {
            started_for_jobs.lock().expect("started lock").push(job);
            if job == 1 {
                thread::sleep(Duration::from_millis(15));
                return Err("watched failure");
            }
            while !cancellation.is_cancelled() {
                thread::sleep(Duration::from_millis(1));
            }
            observed_for_jobs.store(true, Ordering::SeqCst);
            Ok(job)
        })
        .expect_err("one job fails");

        assert!(matches!(
            error,
            ScheduleError::JobFailed {
                index: 1,
                source: "watched failure"
            }
        ));
        let mut started = started.lock().expect("started lock").clone();
        started.sort_unstable();
        assert_eq!(started, [0, 1]);
        assert!(observed_cancel.load(Ordering::SeqCst));
    }

    #[test]
    fn cancellation_hooks_run_once_and_can_be_unregistered() {
        let token = CancellationToken::new();
        let calls = Arc::new(AtomicUsize::new(0));

        let retained_calls = Arc::clone(&calls);
        let _retained = token.on_cancel(move || {
            retained_calls.fetch_add(1, Ordering::SeqCst);
        });
        let removed_calls = Arc::clone(&calls);
        let removed = token.on_cancel(move || {
            removed_calls.fetch_add(100, Ordering::SeqCst);
        });
        drop(removed);

        assert!(token.cancel());
        assert!(!token.cancel());
        assert_eq!(calls.load(Ordering::SeqCst), 1);

        let late_calls = Arc::clone(&calls);
        let _late = token.on_cancel(move || {
            late_calls.fetch_add(1, Ordering::SeqCst);
        });
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn worker_panics_are_reported_and_cancel_the_run() {
        let error = run_bounded(0..2, 2, |_, job, cancellation| {
            if job == 0 {
                panic!("controlled panic");
            }
            while !cancellation.is_cancelled() {
                thread::yield_now();
            }
            Ok::<_, &'static str>(job)
        })
        .expect_err("panic fails the schedule");

        assert!(matches!(
            error,
            ScheduleError::WorkerPanicked { index: 0, message }
                if message == "controlled panic"
        ));
    }

    #[cfg(unix)]
    #[test]
    fn cancelled_child_is_terminated_and_reaped() {
        let token = CancellationToken::new();
        let token_for_thread = token.clone();
        let canceller = thread::spawn(move || {
            thread::sleep(Duration::from_millis(20));
            token_for_thread.cancel();
        });
        let mut child = std::process::Command::new("sleep")
            .arg("10")
            .spawn()
            .expect("spawn sleep");
        let started = Instant::now();

        let outcome = wait_for_child(&mut child, &token, Duration::from_millis(2))
            .expect("cancel and reap child");
        canceller.join().expect("join canceller");

        assert!(outcome.was_cancelled());
        assert!(started.elapsed() < Duration::from_secs(2));
        assert!(child.try_wait().expect("child remains queryable").is_some());
    }

    #[cfg(unix)]
    #[test]
    fn normally_exited_child_is_reaped_without_cancellation() {
        let token = CancellationToken::new();
        let mut child = std::process::Command::new("sh")
            .args(["-c", "exit 7"])
            .spawn()
            .expect("spawn child");

        let outcome =
            wait_for_child(&mut child, &token, Duration::from_millis(2)).expect("wait for child");

        assert!(!outcome.was_cancelled());
        assert_eq!(outcome.status().code(), Some(7));
    }
}
