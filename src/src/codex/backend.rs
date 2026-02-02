use anyhow::{Error, Result};
use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, TryRecvError},
        Arc, Mutex,
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};

#[cfg(test)]
use std::thread;

/// Unique identifier for Codex requests routed through the backend.
pub type JobId = u64;

/// User-facing mode describing how Codex should treat the request.
#[derive(Debug, Clone, Copy)]
pub enum RequestMode {
    Chat,
}

/// Payload variants supported by the backend.
#[derive(Debug, Clone)]
pub enum RequestPayload {
    Chat { prompt: String },
}

/// Structured Codex request routed through the backend.
#[derive(Debug, Clone)]
pub struct CodexRequest {
    pub payload: RequestPayload,
    pub timeout: Option<Duration>,
    pub workspace_files: Vec<PathBuf>,
}

impl CodexRequest {
    pub fn chat(prompt: String) -> Self {
        Self {
            payload: RequestPayload::Chat { prompt },
            timeout: None,
            workspace_files: Vec::new(),
        }
    }
}

/// Telemetry produced for every Codex job so latency regressions can be audited.
#[derive(Debug, Clone)]
pub struct BackendStats {
    pub backend_type: &'static str,
    pub started_at: Instant,
    pub first_token_at: Option<Instant>,
    pub finished_at: Instant,
    pub tokens_received: usize,
    pub bytes_transferred: usize,
    pub pty_attempts: u32,
    pub cli_fallback_used: bool,
    pub disable_pty: bool,
}

impl BackendStats {
    pub(super) fn new(now: Instant) -> Self {
        Self {
            backend_type: "cli",
            started_at: now,
            first_token_at: None,
            finished_at: now,
            tokens_received: 0,
            bytes_transferred: 0,
            pty_attempts: 0,
            cli_fallback_used: false,
            disable_pty: false,
        }
    }
}

/// Event emitted by the backend describing job progress.
#[derive(Debug, Clone)]
pub struct BackendEvent {
    pub job_id: JobId,
    pub kind: BackendEventKind,
}

/// Classified event payload.
#[derive(Debug, Clone)]
pub enum BackendEventKind {
    Started {
        mode: RequestMode,
    },
    Status {
        message: String,
    },
    Token {
        text: String,
    },
    RecoverableError {
        phase: &'static str,
        message: String,
        retry_available: bool,
    },
    FatalError {
        phase: &'static str,
        message: String,
        disable_pty: bool,
    },
    Finished {
        lines: Vec<String>,
        status: String,
        stats: BackendStats,
    },
    Canceled {
        disable_pty: bool,
    },
}

/// Errors surfaced synchronously when a backend cannot start a job.
#[derive(Debug)]
pub enum BackendError {
    InvalidRequest(&'static str),
    BackendDisabled(String),
}

/// Runtime implementation of the Codex backend interface.
pub trait CodexBackend: Send + Sync {
    fn start(&self, request: CodexRequest) -> Result<BackendJob, BackendError>;
    fn cancel(&self, job_id: JobId);
    fn working_dir(&self) -> &Path;
}

/// Handle to an asynchronous Codex invocation routed through the backend.
pub struct BackendJob {
    pub id: JobId,
    events: Arc<BoundedEventQueue>,
    signal_rx: Receiver<()>,
    handle: Option<JoinHandle<()>>,
    cancel_token: CancelToken,
}

impl BackendJob {
    pub(super) fn new(
        id: JobId,
        events: Arc<BoundedEventQueue>,
        signal_rx: Receiver<()>,
        handle: JoinHandle<()>,
        cancel_token: CancelToken,
    ) -> Self {
        Self {
            id,
            events,
            signal_rx,
            handle: Some(handle),
            cancel_token,
        }
    }

    /// Request cancellation; the worker best-effort terminates subprocesses and emits a
    /// `BackendEventKind::Canceled` terminal event.
    pub fn cancel(&self) {
        self.cancel_token.cancel();
    }

    /// Poll the signal channel without blocking to determine whether new events exist.
    pub fn try_recv_signal(&self) -> Result<(), TryRecvError> {
        self.signal_rx.try_recv().map(|_| ())
    }

    /// Drain any queued backend events.
    pub fn drain_events(&self) -> Vec<BackendEvent> {
        self.events.drain()
    }

    /// Take ownership of the worker handle so the caller can join it once the job finishes.
    pub fn take_handle(&mut self) -> Option<JoinHandle<()>> {
        self.handle.take()
    }
}

#[cfg(test)]
impl BackendJob {
    pub(crate) fn is_cancelled(&self) -> bool {
        self.cancel_token.is_cancelled()
    }
}

#[cfg(test)]
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub(crate) enum TestSignal {
    Ready,
    Disconnected,
    Empty,
}

#[cfg(test)]
pub(crate) fn build_test_backend_job(events: Vec<BackendEvent>, signal: TestSignal) -> BackendJob {
    let queue = Arc::new(BoundedEventQueue::new(32));
    for event in events {
        let _ = queue.push(event);
    }
    let (tx, rx) = mpsc::channel();
    match signal {
        TestSignal::Ready => {
            let _ = tx.send(());
        }
        TestSignal::Disconnected => {
            drop(tx);
        }
        TestSignal::Empty => {
            let _ = tx;
        }
    }
    let handle = thread::spawn(|| {});
    BackendJob::new(0, queue, rx, handle, CancelToken::new())
}

pub(super) const BACKEND_EVENT_CAPACITY: usize = 1024;

/// Backing store for all events emitted by a job. Ensures bounded capacity with drop-oldest semantics.
pub(super) struct BoundedEventQueue {
    capacity: usize,
    inner: Mutex<VecDeque<BackendEvent>>,
}

impl BoundedEventQueue {
    pub(super) fn new(capacity: usize) -> Self {
        Self {
            capacity,
            inner: Mutex::new(VecDeque::with_capacity(capacity)),
        }
    }

    pub(super) fn push(&self, event: BackendEvent) -> Result<(), BackendQueueError> {
        let mut queue = self.inner.lock().unwrap();
        if queue.len() >= self.capacity && !Self::drop_non_terminal(&mut queue) {
            return Err(BackendQueueError);
        }
        queue.push_back(event);
        Ok(())
    }

    pub(super) fn drain(&self) -> Vec<BackendEvent> {
        let mut queue = self.inner.lock().unwrap();
        queue.drain(..).collect()
    }

    fn drop_non_terminal(queue: &mut VecDeque<BackendEvent>) -> bool {
        if let Some(idx) = queue
            .iter()
            .position(|event| matches!(event.kind, BackendEventKind::Token { .. }))
        {
            queue.remove(idx);
            return true;
        }
        if let Some(idx) = queue
            .iter()
            .position(|event| matches!(event.kind, BackendEventKind::Status { .. }))
        {
            queue.remove(idx);
            return true;
        }
        if let Some(idx) = queue.iter().position(|event| {
            matches!(
                event.kind,
                BackendEventKind::RecoverableError { .. } | BackendEventKind::Started { .. }
            )
        }) {
            queue.remove(idx);
            return true;
        }
        false
    }
}

#[derive(Debug)]
pub(super) struct BackendQueueError;

/// Sender that writes backend events into the bounded queue and notifies the UI.
pub(super) struct EventSender {
    queue: Arc<BoundedEventQueue>,
    signal_tx: mpsc::Sender<()>,
}

impl EventSender {
    pub(super) fn new(queue: Arc<BoundedEventQueue>, signal_tx: mpsc::Sender<()>) -> Self {
        Self { queue, signal_tx }
    }

    pub(super) fn emit(&self, event: BackendEvent) -> Result<(), BackendQueueError> {
        self.queue.push(event)?;
        let _ = self.signal_tx.send(());
        Ok(())
    }
}

#[derive(Clone)]
pub(super) struct CancelToken {
    flag: Arc<AtomicBool>,
}

impl CancelToken {
    pub(super) fn new() -> Self {
        Self {
            flag: Arc::new(AtomicBool::new(false)),
        }
    }

    pub(super) fn cancel(&self) {
        self.flag.store(true, Ordering::SeqCst);
    }

    pub(super) fn is_cancelled(&self) -> bool {
        self.flag.load(Ordering::SeqCst)
    }
}

#[derive(Debug)]
pub(super) enum CodexCallError {
    Cancelled,
    Failure(Error),
}

impl From<Error> for CodexCallError {
    fn from(err: Error) -> Self {
        Self::Failure(err)
    }
}

impl From<std::io::Error> for CodexCallError {
    fn from(err: std::io::Error) -> Self {
        Self::Failure(err.into())
    }
}
