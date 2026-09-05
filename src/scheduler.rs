// SPDX-License-Identifier: MIT OR Apache-2.0

//! Deterministic, bounded execution for independent verification jobs.
//!
//! Jobs may finish in any order, but successful results are returned in the
//! caller's original order. The coordinator starts no new work after the first
//! reported failure, cancels speculative work later than the current canonical
//! failure, and joins every worker it started before returning. Lower canonical
//! jobs settle so failure selection matches a one-worker run. Workers that own
//! child processes should either poll [`CancellationToken::is_cancelled`] or
//! register a best-effort cleanup hook with [`CancellationToken::on_cancel`].

use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::panic::{self, AssertUnwindSafe};
use std::process::{Child, ExitStatus};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock, Weak, mpsc};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

pub(crate) const MAX_WORKERS: usize = 4;
const CONTROL_POLL_INTERVAL: Duration = Duration::from_millis(5);
const RETIRED_WORKER_SETTINGS: [&str; 4] = [
    "STATE_FORM_MAX_PARALLEL",
    "TEMPORAL_ASSURANCE_JOBS",
    "AMENDMENT_AUDIT_JOBS",
    "RED_TEAM_JOBS",
];

fn unreported_deadline_expired(started: Instant, reported: &AtomicBool, timeout: Duration) -> bool {
    started.elapsed() >= timeout && !reported.load(Ordering::Acquire)
}

/// Reserve one heavyweight execution lane for the independent side-family
/// chain at capacities three and four. Smaller configurations keep the live
/// critical path at their entire capacity.
pub(crate) fn live_worker_allocation(workers: usize) -> usize {
    if workers <= 2 { workers } else { workers - 1 }
}

fn resolve_worker_configuration(
    configured: Option<&str>,
    retired: &[&str],
    available: usize,
) -> Result<usize, String> {
    if !retired.is_empty() {
        return Err(format!(
            "retired worker setting(s) {} are not accepted; use RIGHTS_VERIFY_JOBS",
            retired.join(", ")
        ));
    }
    match configured {
        Some(value) => value
            .parse::<usize>()
            .ok()
            .filter(|value| (1..=MAX_WORKERS).contains(value))
            .ok_or_else(|| {
                format!("RIGHTS_VERIFY_JOBS must be an integer from 1 through {MAX_WORKERS}")
            }),
        None => Ok(available.max(1).min(MAX_WORKERS)),
    }
}

fn cached_worker_configuration(
    cache: &OnceLock<Result<usize, String>>,
    resolve: impl FnOnce() -> Result<usize, String>,
) -> Result<usize, String> {
    cache.get_or_init(resolve).clone()
}

fn resolve_process_worker_configuration() -> Result<usize, String> {
    let retired = RETIRED_WORKER_SETTINGS
        .into_iter()
        .filter(|name| std::env::var_os(name).is_some())
        .collect::<Vec<_>>();
    if !retired.is_empty() {
        return resolve_worker_configuration(None, &retired, 1);
    }
    let configured = match std::env::var("RIGHTS_VERIFY_JOBS") {
        Ok(value) => Some(value),
        Err(std::env::VarError::NotPresent) => None,
        Err(error) => return Err(format!("cannot read RIGHTS_VERIFY_JOBS: {error}")),
    };
    let available = std::thread::available_parallelism().map_or(1, usize::from);
    resolve_worker_configuration(configured.as_deref(), &[], available)
}

/// Resolve the one process-wide heavyweight execution-lane capacity. Family
/// coordinator wrappers are joinable control threads and are separately
/// bounded; explicit values outside the supported equivalence range fail
/// closed instead of silently oversubscribing the host.
pub(crate) fn configured_workers() -> Result<usize, crate::cli::Error> {
    static CONFIGURATION: OnceLock<Result<usize, String>> = OnceLock::new();
    cached_worker_configuration(&CONFIGURATION, resolve_process_worker_configuration)
        .map_err(crate::cli::Error::usage)
}

/// A clonable, one-way signal shared by the scheduler and every active job.
#[derive(Clone, Debug)]
pub(crate) struct CancellationToken {
    state: Arc<CancellationState>,
}

#[derive(Default)]
struct CancellationState {
    cancelled: Arc<AtomicBool>,
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

    /// Clone the exact flag for a cooperative in-process engine.
    pub(crate) fn flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.state.cancelled)
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
    JobTimedOut { index: usize, timeout: Duration },
    WorkerPanicked { index: usize, message: String },
    WorkerTeardownPanicked { worker: usize, message: String },
    CoordinatorLostWorker { active_indices: Vec<usize> },
    Cancelled,
}

impl<E: std::fmt::Display> std::fmt::Display for ScheduleError<E> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidWorkerCount => formatter.write_str("worker count must be positive"),
            Self::JobFailed { index, source } => {
                write!(formatter, "job {index} failed: {source}")
            }
            Self::JobTimedOut { index, timeout } => write!(
                formatter,
                "job {index} exceeded its {:?} cooperative timeout",
                timeout
            ),
            Self::WorkerPanicked { index, message } => {
                write!(formatter, "job {index} panicked: {message}")
            }
            Self::WorkerTeardownPanicked { worker, message } => {
                write!(
                    formatter,
                    "worker {worker} panicked during teardown: {message}"
                )
            }
            Self::CoordinatorLostWorker { active_indices } => write!(
                formatter,
                "scheduler lost its completion channel with active jobs {active_indices:?}",
            ),
            Self::Cancelled => formatter.write_str("scheduler was cancelled"),
        }
    }
}

impl<E: std::fmt::Debug + std::fmt::Display> std::error::Error for ScheduleError<E> {}

enum WorkerCommand<J> {
    Run {
        index: usize,
        job: J,
        cancellation: CancellationToken,
        started: Instant,
        reported: Arc<AtomicBool>,
    },
    Shutdown,
}

enum WorkerMessage<T, E> {
    Finished {
        worker: usize,
        index: usize,
        result: Result<T, E>,
        elapsed: Duration,
    },
    Panicked {
        worker: usize,
        index: usize,
        message: String,
        elapsed: Duration,
    },
    Cancelled {
        worker: usize,
        index: usize,
        elapsed: Duration,
    },
    Stopped {
        worker: usize,
    },
    ExternalCancellation,
}

struct ActiveWorkerJob {
    worker: usize,
    cancellation: CancellationToken,
    started: Instant,
    reported: Arc<AtomicBool>,
    _external_link: Option<CancellationHookGuard>,
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

fn start_worker<J, T, E, S, I, F>(
    worker: usize,
    initialise: Arc<I>,
    run: Arc<F>,
    receiver: mpsc::Receiver<WorkerCommand<J>>,
    sender: mpsc::Sender<WorkerMessage<T, E>>,
) -> JoinHandle<()>
where
    J: Send + 'static,
    T: Send + 'static,
    E: Send + 'static,
    I: Fn(usize) -> S + Send + Sync + 'static,
    F: Fn(usize, &mut S, J, CancellationToken) -> Result<T, E> + Send + Sync + 'static,
{
    thread::spawn(move || {
        let mut state = None;
        while let Ok(command) = receiver.recv() {
            let WorkerCommand::Run {
                index,
                job,
                cancellation,
                started,
                reported,
            } = command
            else {
                break;
            };
            if cancellation.is_cancelled() {
                reported.store(true, Ordering::Release);
                if sender
                    .send(WorkerMessage::Cancelled {
                        worker,
                        index,
                        elapsed: started.elapsed(),
                    })
                    .is_err()
                {
                    reported.store(false, Ordering::Release);
                    return;
                }
                continue;
            }
            let outcome = panic::catch_unwind(AssertUnwindSafe(|| {
                let state = state.get_or_insert_with(|| initialise(worker));
                run(index, state, job, cancellation)
            }));
            let message = match outcome {
                Ok(result) => WorkerMessage::Finished {
                    worker,
                    index,
                    result,
                    elapsed: started.elapsed(),
                },
                Err(payload) => WorkerMessage::Panicked {
                    worker,
                    index,
                    message: panic_message(payload),
                    elapsed: started.elapsed(),
                },
            };
            // If the coordinator has gone, there is no caller left to consume
            // this result and the command channel will close immediately.
            reported.store(true, Ordering::Release);
            if sender.send(message).is_err() {
                reported.store(false, Ordering::Release);
                return;
            }
        }
        let _ = sender.send(WorkerMessage::Stopped { worker });
    })
}

fn stop_workers<J>(
    commands: &[mpsc::Sender<WorkerCommand<J>>],
    handles: Vec<JoinHandle<()>>,
) -> Option<(usize, String)> {
    for sender in commands {
        let _ = sender.send(WorkerCommand::Shutdown);
    }
    let mut first_panic = None;
    for (worker, handle) in handles.into_iter().enumerate() {
        if let Err(payload) = handle.join()
            && first_panic.is_none()
        {
            first_panic = Some((worker, panic_message(payload)));
        }
    }
    first_panic
}

enum CanonicalFailure<E> {
    Failed(E),
    TimedOut(Duration),
    Panicked(String),
}

impl<E> CanonicalFailure<E> {
    fn into_schedule_error(self, index: usize) -> ScheduleError<E> {
        match self {
            Self::Failed(source) => ScheduleError::JobFailed { index, source },
            Self::TimedOut(timeout) => ScheduleError::JobTimedOut { index, timeout },
            Self::Panicked(message) => ScheduleError::WorkerPanicked { index, message },
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ScheduleOptions {
    external_cancellation: Option<CancellationToken>,
    timeout: Option<Duration>,
}

impl ScheduleOptions {
    pub(crate) fn cancelled_by(cancellation: CancellationToken) -> Self {
        Self {
            external_cancellation: Some(cancellation),
            timeout: None,
        }
    }

    pub(crate) fn timeout_after(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

/// Run independent jobs under one sliding worker bound and persistent
/// worker-local state.
///
/// At most `workers` jobs are live. A successful completion opens exactly one
/// slot for the next canonical job. Once any job reports failure, no new job is
/// dispatched. Already-started jobs with lower canonical indices are allowed to
/// settle, while later jobs are cancelled. This makes the selected failure the
/// same lowest canonical failure for every worker count. Every worker is joined
/// before the result is returned.
///
/// `initialise` runs lazily inside each worker on its first job. Its state never
/// crosses a thread boundary, so it may own a deliberately `!Send` engine.
pub(crate) fn run_bounded_with_state_controlled<J, T, E, S, I, F>(
    jobs: impl IntoIterator<Item = J>,
    workers: usize,
    options: ScheduleOptions,
    initialise: I,
    run: F,
) -> Result<Vec<T>, ScheduleError<E>>
where
    J: Send + 'static,
    T: Send + 'static,
    E: Send + 'static,
    I: Fn(usize) -> S + Send + Sync + 'static,
    F: Fn(usize, &mut S, J, CancellationToken) -> Result<T, E> + Send + Sync + 'static,
{
    if workers == 0 {
        return Err(ScheduleError::InvalidWorkerCount);
    }

    let mut jobs: Vec<Option<J>> = jobs.into_iter().map(Some).collect();
    if jobs.is_empty() {
        return Ok(Vec::new());
    }

    let total = jobs.len();
    let initialise = Arc::new(initialise);
    let run = Arc::new(run);
    let (result_sender, result_receiver) = mpsc::channel();
    let mut commands = Vec::with_capacity(workers);
    let mut handles = Vec::with_capacity(workers);
    for worker in 0..workers.min(total) {
        let (command_sender, command_receiver) = mpsc::channel();
        commands.push(command_sender);
        handles.push(start_worker(
            worker,
            Arc::clone(&initialise),
            Arc::clone(&run),
            command_receiver,
            result_sender.clone(),
        ));
    }
    let cancellation_hook = options.external_cancellation.as_ref().map(|external| {
        let wake = result_sender.clone();
        external.on_cancel(move || {
            let _ = wake.send(WorkerMessage::ExternalCancellation);
        })
    });
    // Only workers and the registered cancellation hook may keep this channel
    // alive. If all workers disappear, the coordinator observes either their
    // explicit stop message or channel disconnection rather than waiting for a
    // completion that cannot arrive.
    drop(result_sender);
    let mut idle = (0..commands.len()).collect::<BTreeSet<_>>();
    let mut active = BTreeMap::<usize, ActiveWorkerJob>::new();
    let mut results: Vec<Option<T>> = std::iter::repeat_with(|| None).take(total).collect();
    let mut next = 0;
    let mut timed_out = BTreeSet::new();
    let mut first_error: Option<(usize, CanonicalFailure<E>)> = None;

    let dispatch = |index: usize,
                    worker: usize,
                    jobs: &mut [Option<J>],
                    active: &mut BTreeMap<usize, ActiveWorkerJob>|
     -> Result<bool, ()> {
        let cancellation = CancellationToken::new();
        let reported = Arc::new(AtomicBool::new(false));
        let external_link = options.external_cancellation.as_ref().map(|external| {
            let scheduled = cancellation.clone();
            external.on_cancel(move || {
                scheduled.cancel();
            })
        });
        if cancellation.is_cancelled() {
            return Ok(false);
        }
        let started = Instant::now();
        commands[worker]
            .send(WorkerCommand::Run {
                index,
                job: jobs[index].take().expect("unstarted job is present"),
                cancellation: cancellation.clone(),
                started,
                reported: Arc::clone(&reported),
            })
            .map_err(|_| ())?;
        active.insert(
            index,
            ActiveWorkerJob {
                worker,
                cancellation,
                started,
                reported,
                _external_link: external_link,
            },
        );
        Ok(true)
    };

    let mut externally_cancelled = options
        .external_cancellation
        .as_ref()
        .is_some_and(CancellationToken::is_cancelled);
    loop {
        if options
            .external_cancellation
            .as_ref()
            .is_some_and(CancellationToken::is_cancelled)
        {
            externally_cancelled = true;
            for job in active.values() {
                job.cancellation.cancel();
            }
        }

        if active
            .iter()
            .any(|(_, job)| handles.get(job.worker).is_some_and(JoinHandle::is_finished))
        {
            for job in active.values() {
                job.cancellation.cancel();
            }
            let active_indices = active.keys().copied().collect();
            drop(cancellation_hook);
            let _ = stop_workers(&commands, handles);
            return Err(ScheduleError::CoordinatorLostWorker { active_indices });
        }

        if let Some(timeout) = options.timeout {
            let expired = active
                .iter()
                .filter(|(index, job)| {
                    !timed_out.contains(*index)
                        && unreported_deadline_expired(job.started, &job.reported, timeout)
                })
                .map(|(&index, _)| index)
                .collect::<Vec<_>>();
            for index in expired {
                timed_out.insert(index);
                if first_error
                    .as_ref()
                    .is_none_or(|(candidate, _)| index < *candidate)
                {
                    first_error = Some((index, CanonicalFailure::TimedOut(timeout)));
                }
                if let Some(job) = active.get(&index) {
                    job.cancellation.cancel();
                }
            }
        }

        if let Some((candidate, _)) = &first_error {
            for (&index, job) in &active {
                if index > *candidate {
                    job.cancellation.cancel();
                }
            }
        }

        while !externally_cancelled && first_error.is_none() && next < total && !idle.is_empty() {
            if options
                .external_cancellation
                .as_ref()
                .is_some_and(CancellationToken::is_cancelled)
            {
                externally_cancelled = true;
                for job in active.values() {
                    job.cancellation.cancel();
                }
                break;
            }
            if let Some(timeout) = options.timeout {
                let expired = active
                    .iter()
                    .filter(|(index, job)| {
                        !timed_out.contains(*index)
                            && unreported_deadline_expired(job.started, &job.reported, timeout)
                    })
                    .map(|(&index, _)| index)
                    .collect::<Vec<_>>();
                for index in expired {
                    timed_out.insert(index);
                    if first_error
                        .as_ref()
                        .is_none_or(|(candidate, _)| index < *candidate)
                    {
                        first_error = Some((index, CanonicalFailure::TimedOut(timeout)));
                    }
                    if let Some(job) = active.get(&index) {
                        job.cancellation.cancel();
                    }
                }
                if first_error.is_some() {
                    break;
                }
            }
            if active
                .values()
                .any(|job| job.reported.load(Ordering::Acquire))
            {
                break;
            }
            let worker = idle.pop_first().expect("idle worker exists");
            match dispatch(next, worker, &mut jobs, &mut active) {
                Ok(true) => next += 1,
                Ok(false) => {
                    idle.insert(worker);
                    externally_cancelled = true;
                    for job in active.values() {
                        job.cancellation.cancel();
                    }
                    break;
                }
                Err(()) => {
                    for job in active.values() {
                        job.cancellation.cancel();
                    }
                    drop(cancellation_hook);
                    let active_indices = active.keys().copied().collect();
                    let _ = stop_workers(&commands, handles);
                    return Err(ScheduleError::CoordinatorLostWorker { active_indices });
                }
            }
        }

        if active.is_empty() {
            break;
        }

        let deadline_wait = options.timeout.and_then(|timeout| {
            active
                .iter()
                .filter(|(index, job)| {
                    !timed_out.contains(*index) && !job.reported.load(Ordering::Acquire)
                })
                .map(|(_, job)| timeout.saturating_sub(job.started.elapsed()))
                .min()
        });
        let wait = deadline_wait
            .unwrap_or(CONTROL_POLL_INTERVAL)
            .min(CONTROL_POLL_INTERVAL);
        let message = result_receiver.recv_timeout(wait);
        let message = match message {
            Ok(message) => message,
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                for job in active.values() {
                    job.cancellation.cancel();
                }
                drop(cancellation_hook);
                let _ = stop_workers(&commands, handles);
                return Err(ScheduleError::CoordinatorLostWorker {
                    active_indices: active.keys().copied().collect(),
                });
            }
        };

        if matches!(&message, WorkerMessage::ExternalCancellation) {
            externally_cancelled = true;
            for job in active.values() {
                job.cancellation.cancel();
            }
            continue;
        }
        if let WorkerMessage::Stopped { worker } = &message {
            if active.values().any(|candidate| candidate.worker == *worker) {
                for job in active.values() {
                    job.cancellation.cancel();
                }
                drop(cancellation_hook);
                let _ = stop_workers(&commands, handles);
                return Err(ScheduleError::CoordinatorLostWorker {
                    active_indices: active.keys().copied().collect(),
                });
            }
            continue;
        }

        let (worker, index, elapsed) = match &message {
            WorkerMessage::Finished {
                worker,
                index,
                elapsed,
                ..
            }
            | WorkerMessage::Panicked {
                worker,
                index,
                elapsed,
                ..
            }
            | WorkerMessage::Cancelled {
                worker,
                index,
                elapsed,
            } => (*worker, *index, *elapsed),
            WorkerMessage::Stopped { .. } | WorkerMessage::ExternalCancellation => unreachable!(),
        };
        if let Some(timeout) = options.timeout
            && let Some(job) = active.get(&index)
            && elapsed >= timeout
            && timed_out.insert(index)
        {
            if first_error
                .as_ref()
                .is_none_or(|(candidate, _)| index < *candidate)
            {
                first_error = Some((index, CanonicalFailure::TimedOut(timeout)));
            }
            job.cancellation.cancel();
            if let Some((candidate, _)) = &first_error {
                for (&active_index, active_job) in &active {
                    if active_index > *candidate {
                        active_job.cancellation.cancel();
                    }
                }
            }
        }
        active.remove(&index);
        idle.insert(worker);

        let failure = match message {
            WorkerMessage::Finished {
                index,
                result: Ok(value),
                ..
            } => {
                results[index] = Some(value);
                None
            }
            WorkerMessage::Finished {
                result: Err(source),
                ..
            } => Some(CanonicalFailure::Failed(source)),
            WorkerMessage::Panicked { message, .. } => Some(CanonicalFailure::Panicked(message)),
            WorkerMessage::Cancelled { .. } => None,
            WorkerMessage::Stopped { .. } | WorkerMessage::ExternalCancellation => unreachable!(),
        };
        if first_error.as_ref().is_some_and(|(candidate, failure)| {
            *candidate == index && matches!(failure, CanonicalFailure::TimedOut(_))
        }) {
            // A cooperatively timed-out job can return either success or a
            // cancellation-shaped error. Its deadline remains authoritative.
        } else if let Some(failure) = failure
            && first_error
                .as_ref()
                .is_none_or(|(candidate, _)| index < *candidate)
        {
            first_error = Some((index, failure));
            for (&active_index, job) in &active {
                if active_index > index {
                    job.cancellation.cancel();
                }
            }
        }
    }

    drop(cancellation_hook);
    let teardown_panic = stop_workers(&commands, handles);

    if externally_cancelled
        || options
            .external_cancellation
            .as_ref()
            .is_some_and(CancellationToken::is_cancelled)
    {
        return Err(ScheduleError::Cancelled);
    }
    if let Some((index, failure)) = first_error {
        return Err(failure.into_schedule_error(index));
    }
    if let Some((worker, message)) = teardown_panic {
        return Err(ScheduleError::WorkerTeardownPanicked { worker, message });
    }

    Ok(results
        .into_iter()
        .enumerate()
        .map(|(index, result)| result.unwrap_or_else(|| panic!("job {index} returned no result")))
        .collect())
}

pub(crate) fn run_bounded_with_state<J, T, E, S, I, F>(
    jobs: impl IntoIterator<Item = J>,
    workers: usize,
    initialise: I,
    run: F,
) -> Result<Vec<T>, ScheduleError<E>>
where
    J: Send + 'static,
    T: Send + 'static,
    E: Send + 'static,
    I: Fn(usize) -> S + Send + Sync + 'static,
    F: Fn(usize, &mut S, J, CancellationToken) -> Result<T, E> + Send + Sync + 'static,
{
    run_bounded_with_state_controlled(jobs, workers, ScheduleOptions::default(), initialise, run)
}

/// Run independent jobs without persistent worker-local state.
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
    run_bounded_with_state(
        jobs,
        workers,
        |_| (),
        move |index, (), job, cancellation| run(index, job, cancellation),
    )
}

pub(crate) fn run_bounded_controlled<J, T, E, F>(
    jobs: impl IntoIterator<Item = J>,
    workers: usize,
    options: ScheduleOptions,
    run: F,
) -> Result<Vec<T>, ScheduleError<E>>
where
    J: Send + 'static,
    T: Send + 'static,
    E: Send + 'static,
    F: Fn(usize, J, CancellationToken) -> Result<T, E> + Send + Sync + 'static,
{
    run_bounded_with_state_controlled(
        jobs,
        workers,
        options,
        |_| (),
        move |index, (), job, cancellation| run(index, job, cancellation),
    )
}

/// One canonical job in a dependency-checked, weighted execution graph.
pub(crate) struct DagJob<J> {
    id: &'static str,
    after: Vec<&'static str>,
    /// Heavyweight lane cost and the nested-worker allocation given to `run`.
    /// A family wrapper that only coordinates those workers is not another
    /// execution lane.
    weight: usize,
    timeout: Option<Duration>,
    payload: J,
}

impl<J> DagJob<J> {
    pub(crate) fn new(
        id: &'static str,
        after: impl IntoIterator<Item = &'static str>,
        weight: usize,
        payload: J,
    ) -> Self {
        Self {
            id,
            after: after.into_iter().collect(),
            weight,
            timeout: None,
            payload,
        }
    }

    /// Attach a cooperative wall-clock bound measured from dispatch. The job
    /// must observe its cancellation token; Rust threads are never detached.
    pub(crate) fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

/// A fully validated graph. Construction performs every structural check, so
/// no job can start from an ambiguous dependency declaration.
pub(crate) struct ExecutionGraph<J> {
    jobs: Vec<DagJob<J>>,
    dependencies: Vec<Vec<usize>>,
    capacity: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GraphError {
    InvalidCapacity,
    EmptyId {
        index: usize,
    },
    DuplicateId {
        id: String,
    },
    InvalidWeight {
        id: String,
        weight: usize,
        capacity: usize,
    },
    DuplicateDependency {
        id: String,
        dependency: String,
    },
    UnknownDependency {
        id: String,
        dependency: String,
    },
    SelfDependency {
        id: String,
    },
    Cycle {
        ids: Vec<String>,
    },
    NonCanonicalOrder {
        expected: String,
        derived: String,
    },
}

impl std::fmt::Display for GraphError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidCapacity => {
                formatter.write_str("execution graph capacity must be positive")
            }
            Self::EmptyId { index } => {
                write!(formatter, "execution graph job {index} has an empty id")
            }
            Self::DuplicateId { id } => write!(formatter, "execution graph repeats job id {id}"),
            Self::InvalidWeight {
                id,
                weight,
                capacity,
            } => write!(
                formatter,
                "execution graph job {id} has weight {weight}; expected 1 through {capacity}"
            ),
            Self::DuplicateDependency { id, dependency } => write!(
                formatter,
                "execution graph job {id} repeats dependency {dependency}"
            ),
            Self::UnknownDependency { id, dependency } => write!(
                formatter,
                "execution graph job {id} names unknown dependency {dependency}"
            ),
            Self::SelfDependency { id } => {
                write!(formatter, "execution graph job {id} depends on itself")
            }
            Self::Cycle { ids } => {
                write!(formatter, "execution graph contains a cycle among {ids:?}")
            }
            Self::NonCanonicalOrder { expected, derived } => write!(
                formatter,
                "execution graph dependency order {derived} differs from canonical presentation order {expected}"
            ),
        }
    }
}

impl std::error::Error for GraphError {}

impl<J> ExecutionGraph<J> {
    pub(crate) fn derive(jobs: Vec<DagJob<J>>, capacity: usize) -> Result<Self, GraphError> {
        if capacity == 0 {
            return Err(GraphError::InvalidCapacity);
        }
        let mut by_id = BTreeMap::new();
        for (index, job) in jobs.iter().enumerate() {
            if job.id.is_empty() {
                return Err(GraphError::EmptyId { index });
            }
            if by_id.insert(job.id, index).is_some() {
                return Err(GraphError::DuplicateId {
                    id: job.id.to_owned(),
                });
            }
            if !(1..=capacity).contains(&job.weight) {
                return Err(GraphError::InvalidWeight {
                    id: job.id.to_owned(),
                    weight: job.weight,
                    capacity,
                });
            }
        }

        let mut dependencies = vec![Vec::new(); jobs.len()];
        let mut successors = vec![Vec::new(); jobs.len()];
        let mut indegree = vec![0_usize; jobs.len()];
        for (index, job) in jobs.iter().enumerate() {
            let mut seen = BTreeSet::new();
            for dependency in &job.after {
                if !seen.insert(*dependency) {
                    return Err(GraphError::DuplicateDependency {
                        id: job.id.to_owned(),
                        dependency: (*dependency).to_owned(),
                    });
                }
                let Some(&dependency_index) = by_id.get(dependency) else {
                    return Err(GraphError::UnknownDependency {
                        id: job.id.to_owned(),
                        dependency: (*dependency).to_owned(),
                    });
                };
                if dependency_index == index {
                    return Err(GraphError::SelfDependency {
                        id: job.id.to_owned(),
                    });
                }
                dependencies[index].push(dependency_index);
                successors[dependency_index].push(index);
                indegree[index] += 1;
            }
            dependencies[index].sort_unstable();
        }

        let mut ready = indegree
            .iter()
            .enumerate()
            .filter_map(|(index, degree)| (*degree == 0).then_some(index))
            .collect::<BTreeSet<_>>();
        let mut derived = Vec::with_capacity(jobs.len());
        while let Some(index) = ready.pop_first() {
            derived.push(index);
            for &successor in &successors[index] {
                indegree[successor] -= 1;
                if indegree[successor] == 0 {
                    ready.insert(successor);
                }
            }
        }
        if derived.len() != jobs.len() {
            return Err(GraphError::Cycle {
                ids: indegree
                    .iter()
                    .enumerate()
                    .filter_map(|(index, degree)| (*degree > 0).then(|| jobs[index].id.to_owned()))
                    .collect(),
            });
        }
        let canonical = (0..jobs.len()).collect::<Vec<_>>();
        if derived != canonical {
            let names = |indices: &[usize]| {
                indices
                    .iter()
                    .map(|index| jobs[*index].id)
                    .collect::<Vec<_>>()
                    .join(",")
            };
            return Err(GraphError::NonCanonicalOrder {
                expected: names(&canonical),
                derived: names(&derived),
            });
        }
        Ok(Self {
            jobs,
            dependencies,
            capacity,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct JobTiming {
    pub(crate) started_offset_ms: u64,
    pub(crate) elapsed_ms: u64,
}

pub(crate) struct DagResult<T> {
    pub(crate) id: &'static str,
    pub(crate) value: T,
    pub(crate) timing: JobTiming,
}

#[derive(Debug)]
pub(crate) enum DagFailureKind<E> {
    Job(E),
    TimedOut(Duration),
    Panicked(String),
    LostWorker(Vec<String>),
    Cancelled,
}

pub(crate) struct DagFailure<E> {
    pub(crate) index: Option<usize>,
    pub(crate) id: Option<&'static str>,
    pub(crate) kind: DagFailureKind<E>,
    pub(crate) timing: JobTiming,
}

pub(crate) struct DagRun<T, E> {
    /// Every successful canonical result, or the complete successful prefix on
    /// failure. Speculatively completed later jobs are deliberately excluded.
    pub(crate) completed: Vec<DagResult<T>>,
    pub(crate) failure: Option<DagFailure<E>>,
    pub(crate) maximum_active_weight: usize,
    pub(crate) maximum_active_jobs: usize,
    /// Conservative scheduler-created thread ceiling: active family wrappers
    /// plus every declared nested execution lane. Direct weight-one families
    /// run on their wrapper, so this deliberately over-counts them.
    pub(crate) maximum_managed_thread_upper_bound: usize,
}

enum DagMessage<T, E> {
    Finished {
        index: usize,
        result: Result<T, E>,
        elapsed: Duration,
    },
    Panicked {
        index: usize,
        message: String,
        elapsed: Duration,
    },
    Cancelled {
        index: usize,
        elapsed: Duration,
    },
}

struct ActiveDagJob {
    cancellation: CancellationToken,
    handle: JoinHandle<()>,
    reported: Arc<AtomicBool>,
    started: Instant,
    started_offset_ms: u64,
    weight: usize,
    timeout: Option<Duration>,
    _external_link: Option<CancellationHookGuard>,
}

fn dag_timing(started_offset_ms: u64, elapsed: Duration) -> JobTiming {
    JobTiming {
        started_offset_ms,
        elapsed_ms: u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX),
    }
}

/// Execute a preflighted graph under one weighted execution-lane capacity.
///
/// Jobs start only as one contiguous canonical prefix. Dependencies and a full
/// capacity can pause that frontier, but the scheduler never skips over it.
/// Consequently, after a failure every lower-ranked candidate is already
/// either complete or active; waiting for those active jobs selects the same
/// earliest failure as a one-worker run without launching any new work.
/// A job owns one joinable family wrapper. Direct weight-one work executes on
/// that wrapper; nested families may create at most `weight` lane workers while
/// the wrapper waits. The conservative scheduler-created thread ceiling is
/// therefore twice the configured lane capacity. Timeouts and external
/// cancellation are cooperative: every active job is signalled and then joined
/// before this function returns.
pub(crate) fn run_dag<J, T, E, F>(
    graph: ExecutionGraph<J>,
    external_cancellation: Option<CancellationToken>,
    run: F,
) -> DagRun<T, E>
where
    J: Send + 'static,
    T: Send + 'static,
    E: Send + 'static,
    F: Fn(usize, &'static str, usize, J, CancellationToken) -> Result<T, E> + Send + Sync + 'static,
{
    let ExecutionGraph {
        jobs,
        dependencies,
        capacity,
    } = graph;
    let total = jobs.len();
    let job_ids = jobs.iter().map(|job| job.id).collect::<Vec<_>>();
    let mut jobs = jobs.into_iter().map(Some).collect::<Vec<_>>();
    let run = Arc::new(run);
    let (sender, receiver) = mpsc::channel();
    let mut sender = Some(sender);

    let schedule_started = Instant::now();
    let mut next = 0_usize;
    let mut active_weight = 0_usize;
    let mut maximum_active_weight = 0_usize;
    let mut maximum_active_jobs = 0_usize;
    let mut maximum_managed_thread_upper_bound = 0_usize;
    let mut active = BTreeMap::<usize, ActiveDagJob>::new();
    let mut successes = std::iter::repeat_with(|| None)
        .take(total)
        .collect::<Vec<Option<DagResult<T>>>>();
    let mut failures = BTreeMap::<usize, (DagFailureKind<E>, JobTiming)>::new();
    let mut externally_cancelled = external_cancellation
        .as_ref()
        .is_some_and(CancellationToken::is_cancelled);

    loop {
        if external_cancellation
            .as_ref()
            .is_some_and(CancellationToken::is_cancelled)
        {
            externally_cancelled = true;
            for job in active.values() {
                job.cancellation.cancel();
            }
        }

        if active
            .values()
            .any(|job| job.handle.is_finished() && !job.reported.load(Ordering::Acquire))
        {
            let ids = active
                .keys()
                .map(|index| job_ids[*index].to_owned())
                .collect::<Vec<_>>();
            for job in active.values() {
                job.cancellation.cancel();
            }
            for (_, job) in std::mem::take(&mut active) {
                let _ = job.handle.join();
            }
            return DagRun {
                completed: canonical_prefix(successes),
                failure: Some(DagFailure {
                    index: None,
                    id: None,
                    kind: DagFailureKind::LostWorker(ids),
                    timing: dag_timing(0, schedule_started.elapsed()),
                }),
                maximum_active_weight,
                maximum_active_jobs,
                maximum_managed_thread_upper_bound,
            };
        }

        for (&index, job) in &active {
            let Some(timeout) = job.timeout else {
                continue;
            };
            if !unreported_deadline_expired(job.started, &job.reported, timeout)
                || failures.contains_key(&index)
            {
                continue;
            }
            failures.insert(
                index,
                (
                    DagFailureKind::TimedOut(timeout),
                    dag_timing(job.started_offset_ms, timeout),
                ),
            );
            job.cancellation.cancel();
        }

        if let Some((&candidate, _)) = failures.first_key_value() {
            for (&index, job) in &active {
                if index > candidate {
                    job.cancellation.cancel();
                }
            }
        }

        while !externally_cancelled && failures.is_empty() && next < total {
            if external_cancellation
                .as_ref()
                .is_some_and(CancellationToken::is_cancelled)
            {
                externally_cancelled = true;
                for job in active.values() {
                    job.cancellation.cancel();
                }
                break;
            }
            for (&index, job) in &active {
                let Some(timeout) = job.timeout else {
                    continue;
                };
                if !unreported_deadline_expired(job.started, &job.reported, timeout)
                    || failures.contains_key(&index)
                {
                    continue;
                }
                failures.insert(
                    index,
                    (
                        DagFailureKind::TimedOut(timeout),
                        dag_timing(job.started_offset_ms, timeout),
                    ),
                );
                job.cancellation.cancel();
            }
            if !failures.is_empty() {
                break;
            }
            if active
                .values()
                .any(|job| job.reported.load(Ordering::Acquire))
            {
                break;
            }
            if !dependencies[next]
                .iter()
                .all(|dependency| successes[*dependency].is_some())
            {
                break;
            }
            let job = jobs[next].as_ref().expect("unstarted graph job is present");
            if active_weight + job.weight > capacity {
                break;
            }

            let cancellation = CancellationToken::new();
            let external_link = external_cancellation.as_ref().map(|external| {
                let scheduled = cancellation.clone();
                external.on_cancel(move || {
                    scheduled.cancel();
                })
            });
            if cancellation.is_cancelled() {
                externally_cancelled = true;
                for job in active.values() {
                    job.cancellation.cancel();
                }
                break;
            }

            let DagJob {
                id,
                weight,
                timeout,
                payload,
                ..
            } = jobs[next].take().expect("unstarted graph job is present");
            let worker_cancellation = cancellation.clone();
            let worker_run = Arc::clone(&run);
            let worker_sender = sender
                .as_ref()
                .expect("dispatcher retains the graph channel")
                .clone();
            let index = next;
            let started = Instant::now();
            let worker_started = started;
            let reported = Arc::new(AtomicBool::new(false));
            let worker_reported = Arc::clone(&reported);
            let started_offset_ms =
                u64::try_from(schedule_started.elapsed().as_millis()).unwrap_or(u64::MAX);
            let handle = thread::spawn(move || {
                let message = if worker_cancellation.is_cancelled() {
                    DagMessage::Cancelled {
                        index,
                        elapsed: worker_started.elapsed(),
                    }
                } else {
                    let outcome = panic::catch_unwind(AssertUnwindSafe(|| {
                        worker_run(index, id, weight, payload, worker_cancellation)
                    }));
                    match outcome {
                        Ok(result) => DagMessage::Finished {
                            index,
                            result,
                            elapsed: worker_started.elapsed(),
                        },
                        Err(payload) => DagMessage::Panicked {
                            index,
                            message: panic_message(payload),
                            elapsed: worker_started.elapsed(),
                        },
                    }
                };
                worker_reported.store(true, Ordering::Release);
                if worker_sender.send(message).is_err() {
                    worker_reported.store(false, Ordering::Release);
                }
            });
            active_weight += weight;
            active.insert(
                index,
                ActiveDagJob {
                    cancellation,
                    handle,
                    reported,
                    started,
                    started_offset_ms,
                    weight,
                    timeout,
                    _external_link: external_link,
                },
            );
            maximum_active_weight = maximum_active_weight.max(active_weight);
            maximum_active_jobs = maximum_active_jobs.max(active.len());
            maximum_managed_thread_upper_bound =
                maximum_managed_thread_upper_bound.max(active_weight + active.len());
            next += 1;
        }

        if externally_cancelled || !failures.is_empty() || next == total {
            sender.take();
        }

        if active.is_empty() {
            break;
        }

        let job_deadline = active
            .iter()
            .filter(|(index, job)| {
                !failures.contains_key(index) && !job.reported.load(Ordering::Acquire)
            })
            .filter_map(|(_, job)| {
                job.timeout
                    .map(|timeout| timeout.saturating_sub(job.started.elapsed()))
            })
            .min();
        let wait = job_deadline
            .unwrap_or(CONTROL_POLL_INTERVAL)
            .min(CONTROL_POLL_INTERVAL);
        let message = receiver.recv_timeout(wait);
        match message {
            Ok(DagMessage::Finished {
                index,
                result,
                elapsed,
            }) => {
                let Some(job) = active.remove(&index) else {
                    continue;
                };
                active_weight -= job.weight;
                let join_panic = job.handle.join().err().map(panic_message);
                let timing = dag_timing(job.started_offset_ms, elapsed);
                if let Some(timeout) = job.timeout.filter(|timeout| elapsed >= *timeout) {
                    failures
                        .entry(index)
                        .or_insert((DagFailureKind::TimedOut(timeout), timing));
                } else if let Some(message) = join_panic {
                    failures
                        .entry(index)
                        .or_insert((DagFailureKind::Panicked(message), timing));
                } else if !failures.contains_key(&index) {
                    match result {
                        Ok(value) => {
                            successes[index] = Some(DagResult {
                                id: job_ids[index],
                                value,
                                timing,
                            });
                        }
                        Err(source) => {
                            failures.insert(index, (DagFailureKind::Job(source), timing));
                        }
                    }
                }
            }
            Ok(DagMessage::Panicked {
                index,
                message,
                elapsed,
            }) => {
                let Some(job) = active.remove(&index) else {
                    continue;
                };
                active_weight -= job.weight;
                let join_panic = job.handle.join().err().map(panic_message);
                let timing = dag_timing(job.started_offset_ms, elapsed);
                if let Some(timeout) = job.timeout.filter(|timeout| elapsed >= *timeout) {
                    failures
                        .entry(index)
                        .or_insert((DagFailureKind::TimedOut(timeout), timing));
                } else {
                    failures.entry(index).or_insert((
                        DagFailureKind::Panicked(join_panic.unwrap_or(message)),
                        timing,
                    ));
                }
            }
            Ok(DagMessage::Cancelled { index, elapsed }) => {
                let Some(job) = active.remove(&index) else {
                    continue;
                };
                active_weight -= job.weight;
                let join_panic = job.handle.join().err().map(panic_message);
                if let Some(message) = join_panic {
                    failures.entry(index).or_insert((
                        DagFailureKind::Panicked(message),
                        dag_timing(job.started_offset_ms, elapsed),
                    ));
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                let ids = active
                    .keys()
                    .map(|index| job_ids[*index].to_owned())
                    .collect::<Vec<_>>();
                for job in active.values() {
                    job.cancellation.cancel();
                }
                for (_, job) in std::mem::take(&mut active) {
                    let _ = job.handle.join();
                }
                return DagRun {
                    completed: canonical_prefix(successes),
                    failure: Some(DagFailure {
                        index: None,
                        id: None,
                        kind: DagFailureKind::LostWorker(ids),
                        timing: dag_timing(0, schedule_started.elapsed()),
                    }),
                    maximum_active_weight,
                    maximum_active_jobs,
                    maximum_managed_thread_upper_bound,
                };
            }
        }
    }

    if externally_cancelled
        || external_cancellation
            .as_ref()
            .is_some_and(CancellationToken::is_cancelled)
    {
        return DagRun {
            completed: Vec::new(),
            failure: Some(DagFailure {
                index: None,
                id: None,
                kind: DagFailureKind::Cancelled,
                timing: dag_timing(0, schedule_started.elapsed()),
            }),
            maximum_active_weight,
            maximum_active_jobs,
            maximum_managed_thread_upper_bound,
        };
    }
    if let Some((index, (kind, timing))) = failures.into_iter().next() {
        return DagRun {
            completed: canonical_prefix(successes),
            failure: Some(DagFailure {
                index: Some(index),
                id: Some(job_ids[index]),
                kind,
                timing,
            }),
            maximum_active_weight,
            maximum_active_jobs,
            maximum_managed_thread_upper_bound,
        };
    }
    DagRun {
        completed: canonical_prefix(successes),
        failure: None,
        maximum_active_weight,
        maximum_active_jobs,
        maximum_managed_thread_upper_bound,
    }
}

fn canonical_prefix<T>(results: Vec<Option<DagResult<T>>>) -> Vec<DagResult<T>> {
    results.into_iter().map_while(|result| result).collect()
}

fn scheduler_self_test_require(
    condition: bool,
    detail: &'static str,
) -> Result<(), crate::cli::Error> {
    if condition {
        Ok(())
    } else {
        Err(crate::cli::Error::new(format!(
            "scheduler self-test: {detail}"
        )))
    }
}

fn engine_state_object_self_test() -> Result<(), crate::cli::Error> {
    use std::sync::{Condvar, atomic::AtomicUsize};

    struct MemoryOwningWorkerState {
        bytes: Box<[u8]>,
        live: Arc<AtomicUsize>,
    }

    impl Drop for MemoryOwningWorkerState {
        fn drop(&mut self) {
            self.live.fetch_sub(1, Ordering::SeqCst);
        }
    }

    for workers in 1..=MAX_WORKERS {
        let live = Arc::new(AtomicUsize::new(0));
        let maximum = Arc::new(AtomicUsize::new(0));
        let ready = Arc::new((Mutex::new(0_usize), Condvar::new()));
        let live_for_workers = Arc::clone(&live);
        let maximum_for_workers = Arc::clone(&maximum);
        let ready_for_jobs = Arc::clone(&ready);
        let ordered = run_bounded_with_state(
            0..workers,
            workers,
            move |_| {
                let now = live_for_workers.fetch_add(1, Ordering::SeqCst) + 1;
                maximum_for_workers.fetch_max(now, Ordering::SeqCst);
                MemoryOwningWorkerState {
                    bytes: vec![0_u8; 4 * 1024].into_boxed_slice(),
                    live: Arc::clone(&live_for_workers),
                }
            },
            move |index, state, job, _| {
                let (lock, changed) = &*ready_for_jobs;
                let mut count = lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                *count += 1;
                changed.notify_all();
                let (count, _) = changed
                    .wait_timeout_while(count, Duration::from_secs(1), |count| *count < workers)
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                if *count < workers {
                    return Err("worker-local state objects did not reach the observed bound");
                }
                state.bytes[0] = u8::try_from(index).unwrap_or(u8::MAX);
                Ok::<_, &'static str>((index, job))
            },
        )
        .map_err(|error| {
            crate::cli::Error::new(format!(
                "scheduler engine-state object self-test failed: {error}"
            ))
        })?;
        scheduler_self_test_require(
            maximum.load(Ordering::SeqCst) == workers
                && live.load(Ordering::SeqCst) == 0
                && ordered
                    .iter()
                    .enumerate()
                    .all(|(index, result)| *result == (index, index)),
            "memory-owning worker-local engine-state object bound was not observed",
        )?;
    }
    Ok(())
}

fn dag_panic_self_test() -> Result<(), crate::cli::Error> {
    for workers in 1..=MAX_WORKERS {
        let graph = ExecutionGraph::derive(
            vec![
                DagJob::new("panic-0", [], 1, 0),
                DagJob::new("panic-1", [], 1, 1),
                DagJob::new("panic-wait-2", [], 1, 2),
                DagJob::new("panic-wait-3", [], 1, 3),
            ],
            workers,
        )
        .map_err(|error| crate::cli::Error::new(format!("scheduler self-test graph: {error}")))?;
        let run = run_dag(
            graph,
            None,
            |_, _, _, payload, cancellation| match payload {
                0 => {
                    thread::sleep(Duration::from_millis(3));
                    panic::resume_unwind(Box::new("canonical self-test panic"))
                }
                1 => panic::resume_unwind(Box::new("later self-test panic")),
                _ => {
                    while !cancellation.is_cancelled() {
                        thread::yield_now();
                    }
                    Ok::<_, &'static str>(())
                }
            },
        );
        scheduler_self_test_require(
            matches!(
                run.failure,
                Some(DagFailure {
                    index: Some(0),
                    id: Some("panic-0"),
                    kind: DagFailureKind::Panicked(ref message),
                    ..
                }) if message == "canonical self-test panic"
            ),
            "canonical panic classification changed with worker count",
        )?;
    }
    Ok(())
}

#[cfg(unix)]
fn managed_process_self_test() -> Result<(), crate::cli::Error> {
    use std::process::{Command, Stdio};
    use std::sync::atomic::AtomicUsize;

    for workers in 1..=MAX_WORKERS {
        let live = Arc::new(AtomicUsize::new(0));
        let maximum = Arc::new(AtomicUsize::new(0));
        let observed = Arc::new(AtomicUsize::new(0));
        let reaped = Arc::new(AtomicUsize::new(0));
        let live_for_jobs = Arc::clone(&live);
        let maximum_for_jobs = Arc::clone(&maximum);
        let observed_for_jobs = Arc::clone(&observed);
        let reaped_for_jobs = Arc::clone(&reaped);
        let graph = ExecutionGraph::derive(
            vec![
                DagJob::new("process-0", [], 1, ()),
                DagJob::new("process-1", [], 1, ()),
                DagJob::new("process-2", [], 1, ()),
                DagJob::new("process-3", [], 1, ()),
            ],
            workers,
        )
        .map_err(|error| crate::cli::Error::new(format!("scheduler self-test graph: {error}")))?;
        let run = run_dag(graph, None, move |_, _, _, _, cancellation| {
            let mut child = Command::new("sh")
                .args(["-c", "read _; exit 0"])
                .stdin(Stdio::piped())
                .spawn()
                .map_err(|error| format!("cannot spawn managed child: {error}"))?;
            match child.try_wait() {
                Ok(None) => {}
                Ok(Some(_)) => return Err("managed child exited before observation".to_owned()),
                Err(error) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!("cannot observe managed child: {error}"));
                }
            }

            let now = live_for_jobs.fetch_add(1, Ordering::SeqCst) + 1;
            maximum_for_jobs.fetch_max(now, Ordering::SeqCst);
            observed_for_jobs.fetch_add(1, Ordering::SeqCst);
            let deadline = Instant::now() + Duration::from_secs(1);
            while observed_for_jobs.load(Ordering::SeqCst) < workers
                && !cancellation.is_cancelled()
                && Instant::now() < deadline
            {
                thread::yield_now();
            }
            drop(child.stdin.take());
            let outcome = wait_for_child(&mut child, &cancellation, Duration::from_millis(1));
            live_for_jobs.fetch_sub(1, Ordering::SeqCst);
            let outcome = match outcome {
                Ok(outcome) => outcome,
                Err(error) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!("cannot reap managed child: {error}"));
                }
            };
            if child
                .try_wait()
                .map_err(|error| format!("cannot confirm managed-child reap: {error}"))?
                .is_none()
            {
                let _ = child.kill();
                let _ = child.wait();
                return Err("managed child was not reaped".to_owned());
            }
            reaped_for_jobs.fetch_add(1, Ordering::SeqCst);
            if outcome.was_cancelled() || !outcome.status().success() {
                Err("managed child did not exit normally".to_owned())
            } else if observed_for_jobs.load(Ordering::SeqCst) < workers {
                Err("managed children did not reach the observed process bound".to_owned())
            } else {
                Ok(())
            }
        });
        scheduler_self_test_require(
            run.failure.is_none()
                && maximum.load(Ordering::SeqCst) == workers
                && live.load(Ordering::SeqCst) == 0
                && reaped.load(Ordering::SeqCst) == 4,
            "confirmed-live managed-child process bound or reap count was not observed",
        )?;
    }

    for workers in 1..=MAX_WORKERS {
        let reaped = Arc::new(AtomicUsize::new(0));
        let reaped_for_job = Arc::clone(&reaped);
        let graph = ExecutionGraph::derive(
            vec![
                DagJob::new("abnormal-child", [], 1, true),
                DagJob::new("abnormal-wait-1", [], 1, false),
                DagJob::new("abnormal-wait-2", [], 1, false),
                DagJob::new("abnormal-wait-3", [], 1, false),
            ],
            workers,
        )
        .map_err(|error| crate::cli::Error::new(format!("scheduler self-test graph: {error}")))?;
        let run = run_dag(graph, None, move |_, _, _, abnormal, cancellation| {
            if !abnormal {
                while !cancellation.is_cancelled() {
                    thread::yield_now();
                }
                return Ok::<_, String>(());
            }
            let mut child = Command::new("sh")
                .args(["-c", "exit 7"])
                .spawn()
                .map_err(|error| format!("cannot spawn abnormal managed child: {error}"))?;
            let outcome = wait_for_child(&mut child, &cancellation, Duration::from_millis(1))
                .map_err(|error| format!("cannot reap abnormal managed child: {error}"))?;
            let was_reaped = child
                .try_wait()
                .map_err(|error| format!("cannot confirm abnormal-child reap: {error}"))?
                .is_some();
            if outcome.was_cancelled() || outcome.status().code() != Some(7) || !was_reaped {
                return Err("abnormal managed-child observation failed".to_owned());
            }
            reaped_for_job.fetch_add(1, Ordering::SeqCst);
            Err("managed child exited abnormally".to_owned())
        });
        scheduler_self_test_require(
            matches!(
                run.failure,
                Some(DagFailure {
                    index: Some(0),
                    id: Some("abnormal-child"),
                    kind: DagFailureKind::Job(ref message),
                    ..
                }) if message == "managed child exited abnormally"
            ) && reaped.load(Ordering::SeqCst) == 1,
            "abnormal managed-child exit was not classified and reaped",
        )?;
    }
    Ok(())
}

fn dag_self_test() -> Result<(), crate::cli::Error> {
    let mut passing_reference = None;
    for workers in 1..=MAX_WORKERS {
        let graph = ExecutionGraph::derive(
            (0..6)
                .map(|index| {
                    DagJob::new(
                        ["pass-0", "pass-1", "pass-2", "pass-3", "pass-4", "pass-5"][index],
                        [],
                        1,
                        index,
                    )
                })
                .collect(),
            workers,
        )
        .map_err(|error| crate::cli::Error::new(format!("scheduler self-test graph: {error}")))?;
        let run = run_dag(graph, None, |index, id, _, payload, _| {
            Ok::<_, &'static str>((index, id, payload))
        });
        let semantic = run
            .completed
            .iter()
            .map(|result| result.value)
            .collect::<Vec<_>>();
        scheduler_self_test_require(run.failure.is_none(), "passing graph failed")?;
        scheduler_self_test_require(
            run.maximum_active_weight <= workers
                && run.maximum_active_jobs <= workers
                && run.maximum_managed_thread_upper_bound <= workers * 2,
            "passing graph exceeded its execution-lane or managed-thread bound",
        )?;
        if let Some(reference) = &passing_reference {
            scheduler_self_test_require(
                &semantic == reference,
                "passing output changed with worker count",
            )?;
        } else {
            passing_reference = Some(semantic);
        }
    }

    for workers in 1..=MAX_WORKERS {
        let graph = ExecutionGraph::derive(
            vec![
                DagJob::new("fail-0", [], 1, 0),
                DagJob::new("fail-1", [], 1, 1),
                DagJob::new("wait-2", [], 1, 2),
                DagJob::new("wait-3", [], 1, 3),
            ],
            workers,
        )
        .map_err(|error| crate::cli::Error::new(format!("scheduler self-test graph: {error}")))?;
        let run = run_dag(
            graph,
            None,
            |_, _, _, payload, cancellation| match payload {
                0 => {
                    thread::sleep(Duration::from_millis(3));
                    Err("canonical failure")
                }
                1 => Err("later failure"),
                _ => {
                    while !cancellation.is_cancelled() {
                        thread::yield_now();
                    }
                    Ok(payload)
                }
            },
        );
        scheduler_self_test_require(
            matches!(
                run.failure,
                Some(DagFailure {
                    index: Some(0),
                    id: Some("fail-0"),
                    kind: DagFailureKind::Job("canonical failure"),
                    ..
                })
            ),
            "canonical failure changed with worker count",
        )?;
    }

    let weighted = ExecutionGraph::derive(
        vec![
            DagJob::new("weight-2a", [], 2, ()),
            DagJob::new("weight-1a", [], 1, ()),
            DagJob::new("weight-2b", [], 2, ()),
            DagJob::new("weight-1b", [], 1, ()),
        ],
        3,
    )
    .map_err(|error| crate::cli::Error::new(format!("scheduler self-test graph: {error}")))?;
    let weighted = run_dag(weighted, None, |_, _, _, _, _| {
        thread::sleep(Duration::from_millis(1));
        Ok::<_, &'static str>(())
    });
    scheduler_self_test_require(
        weighted.failure.is_none()
            && weighted.maximum_active_weight == 3
            && weighted.maximum_active_jobs == 2
            && weighted.maximum_managed_thread_upper_bound == 5,
        "weighted graph violated its capacity",
    )?;

    let control_semantics = |run: &DagRun<(), &'static str>| {
        let (failure_index, failure_id, failure_kind, timeout) = match &run.failure {
            None => (None, None, "none", None),
            Some(failure) => (
                failure.index,
                failure.id,
                match &failure.kind {
                    DagFailureKind::Job(_) => "job",
                    DagFailureKind::TimedOut(_) => "timeout",
                    DagFailureKind::Panicked(_) => "panic",
                    DagFailureKind::LostWorker(_) => "lost-worker",
                    DagFailureKind::Cancelled => "cancelled",
                },
                match &failure.kind {
                    DagFailureKind::TimedOut(timeout) => Some(*timeout),
                    _ => None,
                },
            ),
        };
        (
            run.completed
                .iter()
                .map(|result| result.id)
                .collect::<Vec<_>>(),
            failure_index,
            failure_id,
            failure_kind,
            timeout,
        )
    };

    let mut timeout_reference = None;
    for workers in 1..=MAX_WORKERS {
        let started = Arc::new(AtomicU64::new(0));
        let finished = Arc::new(AtomicU64::new(0));
        let active = Arc::new(AtomicU64::new(0));
        let maximum = Arc::new(AtomicU64::new(0));
        let successor_started = Arc::new(AtomicBool::new(false));
        let started_for_jobs = Arc::clone(&started);
        let finished_for_jobs = Arc::clone(&finished);
        let active_for_jobs = Arc::clone(&active);
        let maximum_for_jobs = Arc::clone(&maximum);
        let successor_started_for_jobs = Arc::clone(&successor_started);
        let timeout_graph = ExecutionGraph::derive(
            vec![
                DagJob::new("timeout", [], 1, 0).with_timeout(Duration::from_millis(5)),
                DagJob::new("timeout-peer-1", [], 1, 1),
                DagJob::new("timeout-peer-2", [], 1, 2),
                DagJob::new("timeout-peer-3", [], 1, 3),
                DagJob::new("timeout-successor", ["timeout"], 1, 4),
            ],
            workers,
        )
        .map_err(|error| crate::cli::Error::new(format!("scheduler self-test graph: {error}")))?;
        let timed = run_dag(
            timeout_graph,
            None,
            move |_, _, _, payload, cancellation| {
                if payload == 4 {
                    successor_started_for_jobs.store(true, Ordering::SeqCst);
                }
                started_for_jobs.fetch_add(1, Ordering::SeqCst);
                let now = active_for_jobs.fetch_add(1, Ordering::SeqCst) + 1;
                maximum_for_jobs.fetch_max(now, Ordering::SeqCst);
                while !cancellation.is_cancelled() {
                    thread::yield_now();
                }
                active_for_jobs.fetch_sub(1, Ordering::SeqCst);
                finished_for_jobs.fetch_add(1, Ordering::SeqCst);
                Ok::<_, &'static str>(())
            },
        );
        scheduler_self_test_require(
            // The dispatch-time deadline can expire before a worker enters the
            // closure; the dependent successor must never enter either way.
            matches!(
                &timed.failure,
                Some(DagFailure {
                    index: Some(0),
                    id: Some("timeout"),
                    kind: DagFailureKind::TimedOut(timeout),
                    ..
                }) if *timeout == Duration::from_millis(5)
            ) && timed.completed.is_empty()
                && !successor_started.load(Ordering::SeqCst),
            "timeout failure or successor launch changed with graph capacity",
        )?;
        scheduler_self_test_require(
            timed.maximum_active_weight <= workers
                && timed.maximum_active_jobs <= workers
                && timed.maximum_managed_thread_upper_bound <= workers * 2
                && maximum.load(Ordering::SeqCst) <= workers as u64
                && active.load(Ordering::SeqCst) == 0
                && started.load(Ordering::SeqCst) == finished.load(Ordering::SeqCst),
            "timeout graph exceeded its bound or returned before joining active work",
        )?;
        let semantics = control_semantics(&timed);
        if let Some(reference) = &timeout_reference {
            scheduler_self_test_require(
                &semantics == reference,
                "timeout graph semantics changed from the W1 reference",
            )?;
        } else {
            timeout_reference = Some(semantics);
        }
    }

    let mut cancellation_reference = None;
    for workers in 1..=MAX_WORKERS {
        let external = CancellationToken::new();
        let external_for_thread = external.clone();
        let started = Arc::new(AtomicU64::new(0));
        let finished = Arc::new(AtomicU64::new(0));
        let active = Arc::new(AtomicU64::new(0));
        let maximum = Arc::new(AtomicU64::new(0));
        let successor_started = Arc::new(AtomicBool::new(false));
        let ready = Arc::new((Mutex::new(0_usize), std::sync::Condvar::new()));
        let ready_for_canceller = Arc::clone(&ready);
        let canceller = thread::spawn(move || {
            let (lock, changed) = &*ready_for_canceller;
            let count = lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            let (count, _) = changed
                .wait_timeout_while(count, Duration::from_secs(1), |count| *count < workers)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let reached_bound = *count == workers;
            external_for_thread.cancel();
            reached_bound
        });
        let started_for_jobs = Arc::clone(&started);
        let finished_for_jobs = Arc::clone(&finished);
        let active_for_jobs = Arc::clone(&active);
        let maximum_for_jobs = Arc::clone(&maximum);
        let successor_started_for_jobs = Arc::clone(&successor_started);
        let ready_for_jobs = Arc::clone(&ready);
        let cancellation_graph = ExecutionGraph::derive(
            vec![
                DagJob::new("cancel", [], 1, 0),
                DagJob::new("cancel-peer-1", [], 1, 1),
                DagJob::new("cancel-peer-2", [], 1, 2),
                DagJob::new("cancel-peer-3", [], 1, 3),
                DagJob::new("cancel-successor", ["cancel"], 1, 4),
            ],
            workers,
        )
        .map_err(|error| crate::cli::Error::new(format!("scheduler self-test graph: {error}")))?;
        let cancelled = run_dag(
            cancellation_graph,
            Some(external),
            move |_, _, _, payload, cancellation| {
                if payload == 4 {
                    successor_started_for_jobs.store(true, Ordering::SeqCst);
                }
                started_for_jobs.fetch_add(1, Ordering::SeqCst);
                let now = active_for_jobs.fetch_add(1, Ordering::SeqCst) + 1;
                maximum_for_jobs.fetch_max(now, Ordering::SeqCst);
                {
                    let (lock, changed) = &*ready_for_jobs;
                    let mut count = lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                    *count += 1;
                    changed.notify_all();
                }
                while !cancellation.is_cancelled() {
                    thread::yield_now();
                }
                active_for_jobs.fetch_sub(1, Ordering::SeqCst);
                finished_for_jobs.fetch_add(1, Ordering::SeqCst);
                Ok::<_, &'static str>(())
            },
        );
        let canceller_reached_bound = canceller.join().unwrap_or(false);
        scheduler_self_test_require(
            canceller_reached_bound
                && matches!(
                    &cancelled.failure,
                    Some(DagFailure {
                        index: None,
                        id: None,
                        kind: DagFailureKind::Cancelled,
                        ..
                    })
                )
                && cancelled.completed.is_empty()
                && !successor_started.load(Ordering::SeqCst),
            "external cancellation classification or successor launch changed with graph capacity",
        )?;
        scheduler_self_test_require(
            cancelled.maximum_active_weight <= workers
                && cancelled.maximum_active_jobs <= workers
                && cancelled.maximum_managed_thread_upper_bound <= workers * 2
                && maximum.load(Ordering::SeqCst) == workers as u64
                && active.load(Ordering::SeqCst) == 0
                && started.load(Ordering::SeqCst) == workers as u64
                && started.load(Ordering::SeqCst) == finished.load(Ordering::SeqCst),
            "external cancellation graph exceeded its bound or returned before joining active work",
        )?;
        let semantics = control_semantics(&cancelled);
        if let Some(reference) = &cancellation_reference {
            scheduler_self_test_require(
                &semantics == reference,
                "external cancellation semantics changed from the W1 reference",
            )?;
        } else {
            cancellation_reference = Some(semantics);
        }
    }

    let invalid = ExecutionGraph::derive(
        vec![
            DagJob::new("cycle-a", ["cycle-b"], 1, ()),
            DagJob::new("cycle-b", ["cycle-a"], 1, ()),
        ],
        1,
    );
    scheduler_self_test_require(
        matches!(invalid, Err(GraphError::Cycle { .. })),
        "dependency cycle passed graph preflight",
    )
}

/// Cheap behavioral check used by both quick and full verification.
/// It exercises graph preflight, the sliding and weighted bounds, worker-count
/// equivalence, timeout/cancellation cleanup, deterministic panic selection,
/// canonical output/failure selection, and a measured memory-owning worker-state
/// object bound. On Unix it also starts short-lived managed children to observe
/// their live-process high-water mark, abnormal-exit classification, and reaping.
pub(crate) fn self_test() -> Result<String, crate::cli::Error> {
    use std::sync::atomic::{AtomicUsize, Ordering};

    if !cfg!(panic = "unwind") {
        return Err(crate::cli::Error::new(
            "scheduler requires panic=unwind so worker crashes are classified and joined",
        ));
    }

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

    engine_state_object_self_test()?;
    dag_self_test()?;
    dag_panic_self_test()?;
    #[cfg(unix)]
    managed_process_self_test()?;
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
/// This portable path terminates only the immediate child. Unix callers that
/// start a dedicated process group should use [`wait_for_child_group`].
pub(crate) fn wait_for_child(
    child: &mut Child,
    cancellation: &CancellationToken,
    poll_interval: Duration,
) -> io::Result<ChildWait> {
    wait_for_child_inner(child, cancellation, poll_interval, false)
}

/// Poll, terminate, and reap a child that was created as the leader of its own
/// Unix process group. Killing the whole group prevents a shell precondition's
/// grandchildren from surviving either cancellation or an early leader exit.
#[cfg(unix)]
pub(crate) fn wait_for_child_group(
    child: &mut Child,
    cancellation: &CancellationToken,
    poll_interval: Duration,
) -> io::Result<ChildWait> {
    wait_for_child_inner(child, cancellation, poll_interval, true)
}

fn wait_for_child_inner(
    child: &mut Child,
    cancellation: &CancellationToken,
    poll_interval: Duration,
    process_group: bool,
) -> io::Result<ChildWait> {
    loop {
        if cancellation.is_cancelled() {
            #[cfg(unix)]
            if process_group {
                terminate_process_group(child.id())?;
            }
            // A child can exit between try_wait and kill. Ignore InvalidInput,
            // which is how some platforms report an already-exited process,
            // then always wait to release the process-table entry.
            if !process_group
                && let Err(error) = child.kill()
                && error.kind() != io::ErrorKind::InvalidInput
            {
                return Err(error);
            }
            return child.wait().map(ChildWait::Cancelled);
        }
        if let Some(status) = child.try_wait()? {
            #[cfg(unix)]
            if process_group {
                // A shell leader can exit while a background grandchild keeps
                // its pipes and process group alive. A managed precondition
                // never transfers ownership of such a process, so close the
                // whole group even on the leader's normal exit.
                terminate_process_group(child.id())?;
            }
            return Ok(ChildWait::Exited(status));
        }
        thread::sleep(poll_interval.max(Duration::from_millis(1)));
    }
}

#[cfg(unix)]
fn terminate_process_group(child_id: u32) -> io::Result<()> {
    let process_group_id = i32::try_from(child_id)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "child PID exceeds i32"))?;
    // SAFETY: the child was explicitly placed in a new process group whose id
    // equals its pid. A negative id targets that group, and SIGKILL requires no
    // Rust-side memory contract.
    if unsafe { libc::kill(-process_group_id, libc::SIGKILL) } != 0 {
        let error = io::Error::last_os_error();
        if error.raw_os_error() != Some(libc::ESRCH) {
            return Err(error);
        }
    }
    Ok(())
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
        let initial_jobs_started = Arc::new(Barrier::new(3));
        let observed_cancels = Arc::new(AtomicUsize::new(0));
        let started_for_jobs = Arc::clone(&started);
        let initial_jobs_started_for_jobs = Arc::clone(&initial_jobs_started);
        let observed_for_jobs = Arc::clone(&observed_cancels);

        let error = run_bounded(0..8, 3, move |_, job, cancellation| {
            started_for_jobs.lock().expect("started lock").push(job);
            initial_jobs_started_for_jobs.wait();
            if job == 0 {
                return Err("watched failure");
            }
            while !cancellation.is_cancelled() {
                thread::sleep(Duration::from_millis(1));
            }
            observed_for_jobs.fetch_add(1, Ordering::SeqCst);
            Ok(job)
        })
        .expect_err("one job fails");

        assert!(matches!(
            error,
            ScheduleError::JobFailed {
                index: 0,
                source: "watched failure"
            }
        ));
        let mut started = started.lock().expect("started lock").clone();
        started.sort_unstable();
        assert_eq!(started, [0, 1, 2]);
        assert_eq!(observed_cancels.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn lowest_canonical_failure_is_independent_of_worker_count() {
        for workers in 1..=4 {
            let error = run_bounded(0..8, workers, move |_, job, cancellation| match job {
                0 => {
                    thread::sleep(Duration::from_millis(15));
                    Err("canonical failure")
                }
                2 => {
                    thread::sleep(Duration::from_millis(2));
                    Err("later fast failure")
                }
                _ => {
                    for _ in 0..30 {
                        if cancellation.is_cancelled() {
                            return Ok(job);
                        }
                        thread::sleep(Duration::from_millis(1));
                    }
                    Ok(job)
                }
            })
            .expect_err("controlled failures stop the schedule");

            assert!(matches!(
                error,
                ScheduleError::JobFailed {
                    index: 0,
                    source: "canonical failure"
                }
            ));
        }
    }

    #[test]
    fn persistent_state_is_initialised_once_per_used_worker() {
        struct WorkerState {
            worker: usize,
            ordinal: usize,
            active: Arc<AtomicUsize>,
        }

        impl Drop for WorkerState {
            fn drop(&mut self) {
                self.active.fetch_sub(1, Ordering::SeqCst);
            }
        }

        let initialisations = Arc::new(AtomicUsize::new(0));
        let active_states = Arc::new(AtomicUsize::new(0));
        let maximum_states = Arc::new(AtomicUsize::new(0));
        let initialisations_for_workers = Arc::clone(&initialisations);
        let active_for_workers = Arc::clone(&active_states);
        let maximum_for_workers = Arc::clone(&maximum_states);
        let results = run_bounded_with_state(
            0..12,
            3,
            move |worker| {
                initialisations_for_workers.fetch_add(1, Ordering::SeqCst);
                let active = active_for_workers.fetch_add(1, Ordering::SeqCst) + 1;
                maximum_for_workers.fetch_max(active, Ordering::SeqCst);
                WorkerState {
                    worker,
                    ordinal: 0,
                    active: Arc::clone(&active_for_workers),
                }
            },
            |index, state, job, _| {
                state.ordinal += 1;
                Ok::<_, &'static str>((index, job, state.worker, state.ordinal))
            },
        )
        .expect("stateful jobs pass");

        let used_workers = initialisations.load(Ordering::SeqCst);
        assert!((1..=3).contains(&used_workers));
        assert_eq!(maximum_states.load(Ordering::SeqCst), used_workers);
        assert_eq!(active_states.load(Ordering::SeqCst), 0);
        assert_eq!(results.len(), 12);
        assert!(
            results
                .iter()
                .all(|(index, job, _, ordinal)| { index == job && (1..=12).contains(ordinal) })
        );
    }

    #[test]
    fn worker_configuration_rejects_retired_knobs_without_mutating_process_environment() {
        assert_eq!(resolve_worker_configuration(None, &[], 16), Ok(4));
        assert_eq!(resolve_worker_configuration(None, &[], 0), Ok(1));
        assert_eq!(resolve_worker_configuration(Some("1"), &[], 16), Ok(1));
        assert_eq!(resolve_worker_configuration(Some("4"), &[], 1), Ok(4));
        for invalid in ["", "0", "5", "not-a-number"] {
            assert!(resolve_worker_configuration(Some(invalid), &[], 4).is_err());
        }
        let error = resolve_worker_configuration(
            Some("2"),
            &["STATE_FORM_MAX_PARALLEL", "RED_TEAM_JOBS"],
            4,
        )
        .expect_err("retired settings fail closed");
        assert!(error.contains("STATE_FORM_MAX_PARALLEL"));
        assert!(error.contains("RED_TEAM_JOBS"));
        assert!(error.contains("use RIGHTS_VERIFY_JOBS"));
    }

    #[test]
    fn process_worker_configuration_caches_the_first_value_or_error() {
        let evaluations = AtomicUsize::new(0);
        let value_cache = OnceLock::new();
        assert_eq!(
            cached_worker_configuration(&value_cache, || {
                evaluations.fetch_add(1, Ordering::SeqCst);
                Ok(4)
            }),
            Ok(4)
        );
        assert_eq!(
            cached_worker_configuration(&value_cache, || {
                evaluations.fetch_add(1, Ordering::SeqCst);
                Ok(1)
            }),
            Ok(4)
        );
        assert_eq!(evaluations.load(Ordering::SeqCst), 1);

        let error_evaluations = AtomicUsize::new(0);
        let error_cache = OnceLock::new();
        assert_eq!(
            cached_worker_configuration(&error_cache, || {
                error_evaluations.fetch_add(1, Ordering::SeqCst);
                Err("first resolution failed closed".to_owned())
            }),
            Err("first resolution failed closed".to_owned())
        );
        assert_eq!(
            cached_worker_configuration(&error_cache, || {
                error_evaluations.fetch_add(1, Ordering::SeqCst);
                Ok(2)
            }),
            Err("first resolution failed closed".to_owned())
        );
        assert_eq!(error_evaluations.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn graph_preflight_rejects_every_ambiguous_shape_before_execution() {
        assert!(matches!(
            ExecutionGraph::derive(vec![DagJob::new("", [], 1, ())], 1),
            Err(GraphError::EmptyId { index: 0 })
        ));
        assert!(matches!(
            ExecutionGraph::derive(
                vec![
                    DagJob::new("same", [], 1, ()),
                    DagJob::new("same", [], 1, ())
                ],
                1
            ),
            Err(GraphError::DuplicateId { .. })
        ));
        assert!(matches!(
            ExecutionGraph::derive(vec![DagJob::new("a", ["missing"], 1, ())], 1),
            Err(GraphError::UnknownDependency { .. })
        ));
        assert!(matches!(
            ExecutionGraph::derive(vec![DagJob::new("a", ["a"], 1, ())], 1),
            Err(GraphError::SelfDependency { .. })
        ));
        assert!(matches!(
            ExecutionGraph::derive(
                vec![
                    DagJob::new("a", ["b"], 1, ()),
                    DagJob::new("b", ["a"], 1, ())
                ],
                1
            ),
            Err(GraphError::Cycle { .. })
        ));
        assert!(matches!(
            ExecutionGraph::derive(
                vec![
                    DagJob::new("later", ["earlier"], 1, ()),
                    DagJob::new("earlier", [], 1, ())
                ],
                1
            ),
            Err(GraphError::NonCanonicalOrder { .. })
        ));
        assert!(matches!(
            ExecutionGraph::derive(vec![DagJob::new("heavy", [], 2, ())], 1),
            Err(GraphError::InvalidWeight { .. })
        ));
        assert!(matches!(
            ExecutionGraph::derive(Vec::<DagJob<()>>::new(), 0),
            Err(GraphError::InvalidCapacity)
        ));
    }

    #[test]
    fn dag_passing_output_and_failure_are_equivalent_for_all_worker_counts() {
        let mut passing_reference = None;
        for workers in 1..=MAX_WORKERS {
            let graph = ExecutionGraph::derive(
                vec![
                    DagJob::new("a", [], 1, 10),
                    DagJob::new("b", [], 1, 20),
                    DagJob::new("c", [], 1, 30),
                    DagJob::new("d", [], 1, 40),
                ],
                workers,
            )
            .expect("valid passing graph");
            let run = run_dag(graph, None, |index, id, _, value, _| {
                Ok::<_, &'static str>((index, id, value))
            });
            assert!(run.failure.is_none());
            assert!(run.maximum_active_weight <= workers);
            assert!(run.maximum_active_jobs <= workers);
            assert!(run.maximum_managed_thread_upper_bound <= workers * 2);
            let semantic = run
                .completed
                .into_iter()
                .map(|result| result.value)
                .collect::<Vec<_>>();
            if let Some(reference) = &passing_reference {
                assert_eq!(&semantic, reference);
            } else {
                passing_reference = Some(semantic);
            }

            let graph = ExecutionGraph::derive(
                vec![
                    DagJob::new("first", [], 1, 0),
                    DagJob::new("later", [], 1, 1),
                    DagJob::new("wait-c", [], 1, 2),
                    DagJob::new("wait-d", [], 1, 3),
                ],
                workers,
            )
            .expect("valid failing graph");
            let run = run_dag(graph, None, |_, _, _, value, cancellation| match value {
                0 => {
                    thread::sleep(Duration::from_millis(5));
                    Err("first failure")
                }
                1 => Err("later failure"),
                _ => {
                    while !cancellation.is_cancelled() {
                        thread::yield_now();
                    }
                    Ok(value)
                }
            });
            assert!(matches!(
                run.failure,
                Some(DagFailure {
                    index: Some(0),
                    id: Some("first"),
                    kind: DagFailureKind::Job("first failure"),
                    ..
                })
            ));
        }
    }

    #[test]
    fn dag_weighted_capacity_is_bounded_and_results_remain_canonical() {
        let graph = ExecutionGraph::derive(
            vec![
                DagJob::new("two-a", [], 2, "two-a"),
                DagJob::new("one-a", [], 1, "one-a"),
                DagJob::new("two-b", [], 2, "two-b"),
                DagJob::new("one-b", [], 1, "one-b"),
            ],
            3,
        )
        .expect("valid weighted graph");
        let run = run_dag(graph, None, |_, _, _, value, _| {
            thread::sleep(Duration::from_millis(2));
            Ok::<_, &'static str>(value)
        });
        assert!(run.failure.is_none());
        assert_eq!(run.maximum_active_weight, 3);
        assert_eq!(run.maximum_active_jobs, 2);
        assert_eq!(run.maximum_managed_thread_upper_bound, 5);
        assert_eq!(
            run.completed
                .into_iter()
                .map(|result| result.value)
                .collect::<Vec<_>>(),
            ["two-a", "one-a", "two-b", "one-b"]
        );
    }

    #[test]
    fn dag_failure_never_launches_a_dependency_successor() {
        let started = Arc::new(Mutex::new(Vec::new()));
        let started_for_jobs = Arc::clone(&started);
        let graph = ExecutionGraph::derive(
            vec![
                DagJob::new("root", [], 1, "root"),
                DagJob::new("successor", ["root"], 1, "successor"),
            ],
            4,
        )
        .expect("valid dependency graph");
        let run = run_dag(graph, None, move |_, _, _, value, _| {
            started_for_jobs.lock().expect("started lock").push(value);
            Err::<(), _>("root failure")
        });
        assert!(matches!(
            run.failure,
            Some(DagFailure {
                index: Some(0),
                kind: DagFailureKind::Job("root failure"),
                ..
            })
        ));
        assert_eq!(*started.lock().expect("started lock"), ["root"]);
    }

    #[test]
    fn dag_timeout_and_external_cancellation_are_equivalent_and_joined() {
        for workers in 1..=MAX_WORKERS {
            let active = Arc::new(AtomicUsize::new(0));
            let active_for_jobs = Arc::clone(&active);
            let graph = ExecutionGraph::derive(
                vec![
                    DagJob::new("slow-a", [], 1, ()).with_timeout(Duration::from_millis(5)),
                    DagJob::new("slow-b", [], 1, ()).with_timeout(Duration::from_millis(5)),
                    DagJob::new("slow-c", [], 1, ()).with_timeout(Duration::from_millis(5)),
                    DagJob::new("slow-d", [], 1, ()).with_timeout(Duration::from_millis(5)),
                ],
                workers,
            )
            .expect("valid timeout graph");
            let run = run_dag(graph, None, move |_, _, _, _, cancellation| {
                active_for_jobs.fetch_add(1, Ordering::SeqCst);
                while !cancellation.is_cancelled() {
                    thread::yield_now();
                }
                active_for_jobs.fetch_sub(1, Ordering::SeqCst);
                Ok::<_, &'static str>(())
            });
            assert!(matches!(
                run.failure,
                Some(DagFailure {
                    index: Some(0),
                    id: Some("slow-a"),
                    kind: DagFailureKind::TimedOut(_),
                    ..
                })
            ));
            assert_eq!(active.load(Ordering::SeqCst), 0);

            let external = CancellationToken::new();
            let external_for_thread = external.clone();
            let (started_sender, started_receiver) = mpsc::channel();
            let canceller = thread::spawn(move || {
                let _ = started_receiver.recv();
                external_for_thread.cancel();
            });
            let graph = ExecutionGraph::derive(
                vec![
                    DagJob::new("cancel-a", [], 1, true),
                    DagJob::new("cancel-b", [], 1, false),
                    DagJob::new("cancel-c", [], 1, false),
                    DagJob::new("cancel-d", [], 1, false),
                ],
                workers,
            )
            .expect("valid cancellation graph");
            let run = run_dag(
                graph,
                Some(external),
                move |_, _, _, signals_start, cancellation| {
                    if signals_start {
                        let _ = started_sender.send(());
                    }
                    while !cancellation.is_cancelled() {
                        thread::yield_now();
                    }
                    Ok::<_, &'static str>(())
                },
            );
            canceller.join().expect("join external canceller");
            assert!(matches!(
                run.failure,
                Some(DagFailure {
                    kind: DagFailureKind::Cancelled,
                    ..
                })
            ));
            assert!(run.completed.is_empty());
        }

        let external = CancellationToken::new();
        external.cancel();
        let started = Arc::new(AtomicUsize::new(0));
        let started_for_job = Arc::clone(&started);
        let graph = ExecutionGraph::derive(vec![DagJob::new("never-start", [], 1, ())], 1)
            .expect("valid pre-cancelled graph");
        let run = run_dag(graph, Some(external), move |_, _, _, _, _| {
            started_for_job.fetch_add(1, Ordering::SeqCst);
            Ok::<_, &'static str>(())
        });
        assert!(matches!(
            run.failure,
            Some(DagFailure {
                kind: DagFailureKind::Cancelled,
                ..
            })
        ));
        assert_eq!(started.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn persistent_scheduler_timeout_and_external_cancellation_are_joined() {
        for workers in 1..=MAX_WORKERS {
            let active = Arc::new(AtomicUsize::new(0));
            let active_for_jobs = Arc::clone(&active);
            let result = run_bounded_controlled(
                0..4,
                workers,
                ScheduleOptions::default().timeout_after(Duration::from_millis(5)),
                move |_, value, cancellation| {
                    active_for_jobs.fetch_add(1, Ordering::SeqCst);
                    while !cancellation.is_cancelled() {
                        thread::yield_now();
                    }
                    active_for_jobs.fetch_sub(1, Ordering::SeqCst);
                    Ok::<_, &'static str>(value)
                },
            );
            assert!(matches!(
                result,
                Err(ScheduleError::JobTimedOut { index: 0, .. })
            ));
            assert_eq!(active.load(Ordering::SeqCst), 0);

            let external = CancellationToken::new();
            let external_for_thread = external.clone();
            let (started_sender, started_receiver) = mpsc::channel();
            let canceller = thread::spawn(move || {
                let _ = started_receiver.recv();
                external_for_thread.cancel();
            });
            let active = Arc::new(AtomicUsize::new(0));
            let active_for_jobs = Arc::clone(&active);
            let result = run_bounded_controlled(
                0..4,
                workers,
                ScheduleOptions::cancelled_by(external),
                move |_, value, cancellation| {
                    active_for_jobs.fetch_add(1, Ordering::SeqCst);
                    if value == 0 {
                        let _ = started_sender.send(());
                    }
                    while !cancellation.is_cancelled() {
                        thread::yield_now();
                    }
                    active_for_jobs.fetch_sub(1, Ordering::SeqCst);
                    Ok::<_, &'static str>(value)
                },
            );
            canceller.join().expect("join external canceller");
            assert!(matches!(result, Err(ScheduleError::Cancelled)));
            assert_eq!(active.load(Ordering::SeqCst), 0);
        }
    }

    #[test]
    fn expired_work_never_refills_persistent_or_graph_capacity() {
        let persistent_started = Arc::new(AtomicUsize::new(0));
        let persistent_for_jobs = Arc::clone(&persistent_started);
        let result = run_bounded_controlled(
            0..8,
            MAX_WORKERS,
            ScheduleOptions::default().timeout_after(Duration::ZERO),
            move |_, value, _| {
                persistent_for_jobs.fetch_add(1, Ordering::SeqCst);
                Ok::<_, &'static str>(value)
            },
        );
        assert!(matches!(
            result,
            Err(ScheduleError::JobTimedOut { index: 0, .. })
        ));
        assert!(persistent_started.load(Ordering::SeqCst) <= 1);

        let graph_started = Arc::new(AtomicUsize::new(0));
        let graph_for_jobs = Arc::clone(&graph_started);
        let graph = ExecutionGraph::derive(
            vec![
                DagJob::new("deadline-a", [], 1, 0).with_timeout(Duration::ZERO),
                DagJob::new("deadline-b", [], 1, 1),
                DagJob::new("deadline-c", [], 1, 2),
                DagJob::new("deadline-d", [], 1, 3),
            ],
            MAX_WORKERS,
        )
        .expect("valid deadline graph");
        let run = run_dag(graph, None, move |_, _, _, value, _| {
            graph_for_jobs.fetch_add(1, Ordering::SeqCst);
            Ok::<_, &'static str>(value)
        });
        assert!(matches!(
            run.failure,
            Some(DagFailure {
                index: Some(0),
                kind: DagFailureKind::TimedOut(_),
                ..
            })
        ));
        assert!(graph_started.load(Ordering::SeqCst) <= 1);
    }

    #[test]
    fn queued_completion_is_not_reclassified_by_wall_clock_delay() {
        let reported = AtomicBool::new(true);
        let started = Instant::now() - Duration::from_secs(1);
        assert!(!unreported_deadline_expired(
            started,
            &reported,
            Duration::from_millis(1)
        ));
        reported.store(false, Ordering::Release);
        assert!(unreported_deadline_expired(
            started,
            &reported,
            Duration::from_millis(1)
        ));
    }

    #[test]
    fn dag_panics_select_the_same_canonical_job_for_all_worker_counts() {
        for workers in 1..=MAX_WORKERS {
            let graph = ExecutionGraph::derive(
                vec![
                    DagJob::new("panic-a", [], 1, 0),
                    DagJob::new("panic-b", [], 1, 1),
                    DagJob::new("wait-c", [], 1, 2),
                    DagJob::new("wait-d", [], 1, 3),
                ],
                workers,
            )
            .expect("valid panic graph");
            let run = run_dag(graph, None, |_, _, _, value, cancellation| match value {
                0 => {
                    thread::sleep(Duration::from_millis(5));
                    panic!("canonical panic")
                }
                1 => panic!("later panic"),
                _ => {
                    while !cancellation.is_cancelled() {
                        thread::yield_now();
                    }
                    Ok::<_, &'static str>(())
                }
            });
            assert!(matches!(
                run.failure,
                Some(DagFailure {
                    index: Some(0),
                    id: Some("panic-a"),
                    kind: DagFailureKind::Panicked(ref message),
                    ..
                }) if message == "canonical panic"
            ));
        }
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

    #[test]
    fn persistent_worker_teardown_panics_are_reported() {
        struct PanicOnDrop;

        impl Drop for PanicOnDrop {
            fn drop(&mut self) {
                panic!("controlled teardown panic");
            }
        }

        let error = run_bounded_with_state(
            [()],
            1,
            |_| PanicOnDrop,
            |_, _, _, _| Ok::<_, &'static str>(()),
        )
        .expect_err("teardown panic fails the schedule");
        assert!(matches!(
            error,
            ScheduleError::WorkerTeardownPanicked { worker: 0, message }
                if message == "controlled teardown panic"
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
        assert!(child.try_wait().expect("child remains queryable").is_some());
    }

    #[cfg(unix)]
    #[test]
    fn exited_process_group_leader_cannot_leave_a_running_descendant() {
        use std::io::{BufRead as _, BufReader};
        use std::os::unix::process::CommandExt as _;
        use std::process::Stdio;

        let token = CancellationToken::new();
        let mut command = std::process::Command::new("sh");
        command
            .args(["-c", "sleep 30 & printf '%s\\n' \"$!\""])
            .stdout(Stdio::piped())
            .process_group(0);
        let mut child = command.spawn().expect("spawn process-group leader");
        let mut pid = String::new();
        BufReader::new(child.stdout.take().expect("child stdout"))
            .read_line(&mut pid)
            .expect("read descendant pid");
        let descendant = pid.trim().parse::<i32>().expect("numeric descendant pid");

        let outcome = wait_for_child_group(&mut child, &token, Duration::from_millis(1))
            .expect("wait for process-group leader");
        assert!(!outcome.was_cancelled());
        assert!(outcome.status().success());

        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            // SAFETY: signal zero only queries the recorded process id.
            let exists = unsafe { libc::kill(descendant, 0) } == 0;
            #[cfg(target_os = "linux")]
            let running = exists
                && std::fs::read_to_string(format!("/proc/{descendant}/stat"))
                    .ok()
                    .and_then(|stat| stat.rsplit_once(") ").map(|(_, fields)| fields.to_owned()))
                    .is_some_and(|fields| !matches!(fields.as_bytes().first(), Some(b'Z' | b'X')));
            #[cfg(not(target_os = "linux"))]
            let running = exists;
            if !running {
                break;
            }
            assert!(Instant::now() < deadline, "descendant remained running");
            thread::sleep(Duration::from_millis(5));
        }
    }

    #[cfg(unix)]
    #[test]
    fn managed_child_process_count_never_exceeds_lane_capacity() {
        for lanes in 1..=MAX_WORKERS {
            let active = Arc::new(AtomicUsize::new(0));
            let maximum = Arc::new(AtomicUsize::new(0));
            let active_for_jobs = Arc::clone(&active);
            let maximum_for_jobs = Arc::clone(&maximum);
            let graph = ExecutionGraph::derive(
                vec![
                    DagJob::new("process-a", [], 1, ()),
                    DagJob::new("process-b", [], 1, ()),
                    DagJob::new("process-c", [], 1, ()),
                    DagJob::new("process-d", [], 1, ()),
                ],
                lanes,
            )
            .expect("valid managed-process graph");
            let run = run_dag(graph, None, move |_, _, _, _, cancellation| {
                let mut child = std::process::Command::new("sleep")
                    .arg("0.02")
                    .spawn()
                    .expect("spawn managed child");
                let now = active_for_jobs.fetch_add(1, Ordering::SeqCst) + 1;
                maximum_for_jobs.fetch_max(now, Ordering::SeqCst);
                let outcome = wait_for_child(&mut child, &cancellation, Duration::from_millis(1))
                    .expect("wait for managed child");
                active_for_jobs.fetch_sub(1, Ordering::SeqCst);
                if outcome.status().success() {
                    Ok::<_, &'static str>(())
                } else {
                    Err("managed child failed")
                }
            });
            assert!(run.failure.is_none());
            assert!(maximum.load(Ordering::SeqCst) <= lanes);
            assert_eq!(active.load(Ordering::SeqCst), 0);
        }
    }

    #[cfg(unix)]
    #[test]
    fn abnormally_exited_managed_children_are_reaped_for_all_worker_counts() {
        use std::os::unix::process::ExitStatusExt;

        for workers in 1..=MAX_WORKERS {
            let graph = ExecutionGraph::derive(
                vec![
                    DagJob::new("crash", [], 1, true),
                    DagJob::new("wait-b", [], 1, false),
                    DagJob::new("wait-c", [], 1, false),
                    DagJob::new("wait-d", [], 1, false),
                ],
                workers,
            )
            .expect("valid child-crash graph");
            let run = run_dag(graph, None, |_, _, _, crashes, cancellation| {
                if crashes {
                    let mut child = std::process::Command::new("sh")
                        .args(["-c", "kill -SEGV $$"])
                        .spawn()
                        .expect("spawn controlled crashing child");
                    let outcome =
                        wait_for_child(&mut child, &cancellation, Duration::from_millis(1))
                            .expect("wait for crashing child");
                    assert_eq!(outcome.status().signal(), Some(libc::SIGSEGV));
                    assert!(child.try_wait().expect("child remains queryable").is_some());
                    return Err("managed child crashed");
                }
                while !cancellation.is_cancelled() {
                    thread::yield_now();
                }
                Ok(())
            });
            assert!(matches!(
                run.failure,
                Some(DagFailure {
                    index: Some(0),
                    id: Some("crash"),
                    kind: DagFailureKind::Job("managed child crashed"),
                    ..
                })
            ));
        }
    }
}
