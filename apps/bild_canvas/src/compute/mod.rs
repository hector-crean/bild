

use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use std::sync::mpsc::{self, Sender, Receiver};
use core::future::Future;
use core::pin::Pin;
use std::marker::PhantomData;

// Generic compute traits/backends for Bevy

pub trait ComputeSnapshot: Send + Sync + 'static {}
pub trait ComputeOutput: Send + Sync + 'static {}
pub trait ComputeError: Send + Sync + 'static + std::error::Error {}

impl<T: Send + Sync + 'static> ComputeSnapshot for T {}
impl<T: Send + Sync + 'static> ComputeOutput for T {}
impl<T: Send + Sync + 'static + std::error::Error> ComputeError for T {}

pub trait ComputeBackend<S, O, E>
where
    S: ComputeSnapshot,
    O: ComputeOutput,
    E: ComputeError,
{
    type Job: ComputeJob<O, E>;
    fn start(&self, snapshot: S) -> Self::Job;
}

pub trait ComputeJob<O, E>
where
    O: ComputeOutput,
    E: ComputeError,
{
    fn poll(&mut self) -> Option<Result<O, E>>;
    fn cancel(self) where Self: Sized { let _ = self; }
}

// CPU sync backend
#[derive(Clone, Copy, Default)]
pub struct CpuSync;

// Intentionally no blanket impl for (CpuSync, F).

pub struct CpuSyncJob<O, E> { result: Option<Result<O, E>> }

impl<O, E> CpuSyncJob<O, E> {
    pub fn new(run: impl FnOnce() -> Result<O, E>) -> Self {
        Self { result: Some(run()) }
    }
}

impl<O, E> ComputeJob<O, E> for CpuSyncJob<O, E>
where
    O: ComputeOutput,
    E: ComputeError,
{
    fn poll(&mut self) -> Option<Result<O, E>> { self.result.take() }
}

// CPU async backend
#[derive(Clone, Copy, Default)]
pub struct CpuAsync;

pub struct CpuAsyncJob<O: Send + 'static, E: Send + 'static>(Task<Result<O, E>>);

// Intentionally no blanket impl for (CpuAsync, F).

impl<O, E> ComputeJob<O, E> for CpuAsyncJob<O, E>
where
    O: ComputeOutput + Send + 'static,
    E: ComputeError + Send + 'static,
{
    fn poll(&mut self) -> Option<Result<O, E>> {
        bevy::tasks::futures_lite::future::block_on(
            bevy::tasks::futures_lite::future::poll_once(&mut self.0)
        )
    }
}

// Unified enum wrapper for ergonomics
#[derive(Clone, Debug)]
pub enum ComputeMsg<O: ComputeOutput> {
    Progress(f32),
    Chunk(O),
    Log(String),
}


#[derive(Clone, Copy, Debug)]
pub enum ComputeCmd { Cancel }

type BoxResultFuture<O, E> = Pin<Box<dyn Future<Output = Result<O, E>> + Send + 'static>>;

pub enum BackendKind<F, Fs, L, Fa>
{
    CpuSync(F),
    CpuAsync(F),
    /// Worker sends messages via tx_msg and listens for cmds via rx_cmd
    CpuAsyncStreaming(Fs),
    /// GPU launch hook. Provide a closure that enqueues/render-graph work and
    /// eventually sends the final Result through tx_done and progress/partials through tx_msg.
    /// The job polls rx_done and rx_msg on the main thread.
    Gpu(L),
    /// External async source (e.g., HTTP API). Closure returns a Future; we spawn it on the task pool.
    ExternalAsync(Fa),
}

pub struct Backend<S, O, E, F, Fs, L, Fa>
where
    S: ComputeSnapshot,
    O: ComputeOutput,
    E: ComputeError,
    F: FnOnce(S) -> Result<O, E> + Send + 'static,
    Fs: FnOnce(S, Sender<ComputeMsg<O>>, Receiver<ComputeCmd>) -> Result<O, E> + Send + 'static,
    L: FnOnce(S, Sender<ComputeMsg<O>>, Sender<Result<O, E>>, Receiver<ComputeCmd>) + Send + 'static,
    Fa: FnOnce(S) -> BoxResultFuture<O, E> + Send + 'static,
{
    kind: BackendKind<F, Fs, L, Fa>,
    _pd: PhantomData<(S, O, E)>,
}

impl<S, O, E, F, Fs, L, Fa> Backend<S, O, E, F, Fs, L, Fa>
where
    S: ComputeSnapshot,
    O: ComputeOutput,
    E: ComputeError,
    F: FnOnce(S) -> Result<O, E> + Send + 'static,
    Fs: FnOnce(S, Sender<ComputeMsg<O>>, Receiver<ComputeCmd>) -> Result<O, E> + Send + 'static,
    L: FnOnce(S, Sender<ComputeMsg<O>>, Sender<Result<O, E>>, Receiver<ComputeCmd>) + Send + 'static,
    Fa: FnOnce(S) -> BoxResultFuture<O, E> + Send + 'static,
{
    pub fn start(self, snapshot: S) -> BackendJob<O, E> {
        match self.kind {
            BackendKind::CpuSync(f) => BackendJob::CpuSync(CpuSyncJob::new(|| f(snapshot))),
            BackendKind::CpuAsync(f) => {
                let task = AsyncComputeTaskPool::get().spawn(async move { f(snapshot) });
                BackendJob::CpuAsync(CpuAsyncJob(task))
            }
            BackendKind::CpuAsyncStreaming(f_stream) => {
                let (tx_msg, rx_msg) = mpsc::channel();
                let (tx_cmd, rx_cmd) = mpsc::channel();
                let task = AsyncComputeTaskPool::get().spawn(async move { f_stream(snapshot, tx_msg, rx_cmd) });
                BackendJob::CpuAsyncStreaming(CpuAsyncStreamingJob { task, rx_msg, tx_cmd })
            }
            BackendKind::Gpu(launch) => {
                let (tx_msg, rx_msg) = mpsc::channel();
                let (tx_cmd, rx_cmd) = mpsc::channel();
                let (tx_done, rx_done) = mpsc::channel();
                // Launch on a task to avoid blocking; user closure sets up render work and returns immediately
                AsyncComputeTaskPool::get().spawn(async move {
                    launch(snapshot, tx_msg, tx_done, rx_cmd)
                }).detach();
                BackendJob::Gpu(GpuJob { rx_done, rx_msg, tx_cmd })
            }
            BackendKind::ExternalAsync(fa) => {
                let fut: BoxResultFuture<O, E> = fa(snapshot);
                let task = AsyncComputeTaskPool::get().spawn(async move { fut.await });
                BackendJob::ExternalAsync(CpuAsyncJob(task))
            }
        }
    }

    pub fn cpu_sync(f: F) -> Self {
        Self { kind: BackendKind::CpuSync(f), _pd: PhantomData }
    }
    pub fn cpu_async(f: F) -> Self {
        Self { kind: BackendKind::CpuAsync(f), _pd: PhantomData }
    }
    pub fn cpu_async_streaming(fs: Fs) -> Self {
        Self { kind: BackendKind::CpuAsyncStreaming(fs), _pd: PhantomData }
    }
    pub fn gpu(l: L) -> Self {
        Self { kind: BackendKind::Gpu(l), _pd: PhantomData }
    }
    pub fn external_async(fa: Fa) -> Self {
        Self { kind: BackendKind::ExternalAsync(fa), _pd: PhantomData }
    }
}

pub struct CpuAsyncStreamingJob<O: Send + Sync + 'static, E: Send + Sync + 'static> {
    task: Task<Result<O, E>>,
    rx_msg: Receiver<ComputeMsg<O>>,
    tx_cmd: Sender<ComputeCmd>,
}

pub struct GpuJob<O: Send + Sync + 'static, E: Send + Sync + 'static> {
    rx_done: Receiver<Result<O, E>>,
    rx_msg: Receiver<ComputeMsg<O>>,
    tx_cmd: Sender<ComputeCmd>,
}



pub enum BackendJob<O, E>
where
    O: ComputeOutput,
    E: ComputeError,
{
    CpuSync(CpuSyncJob<O, E>),
    CpuAsync(CpuAsyncJob<O, E>),
    CpuAsyncStreaming(CpuAsyncStreamingJob<O, E>),
    Gpu(GpuJob<O, E>),
    ExternalAsync(CpuAsyncJob<O, E>),
}

impl<O, E> BackendJob<O, E>
where
    O: ComputeOutput,
    E: ComputeError,
{
    pub fn poll(&mut self) -> Option<Result<O, E>> {
        match self {
            BackendJob::CpuSync(j) => j.poll(),
            BackendJob::CpuAsync(j) => j.poll(),
            BackendJob::CpuAsyncStreaming(j) => bevy::tasks::futures_lite::future::block_on(
                bevy::tasks::futures_lite::future::poll_once(&mut j.task)
            ),
            BackendJob::Gpu(j) => j.rx_done.try_recv().ok(),
            BackendJob::ExternalAsync(j) => j.poll(),
        }
    }

    pub fn try_recv_msg(&mut self) -> Option<ComputeMsg<O>> {
        match self {
            BackendJob::CpuSync(_) => None,
            BackendJob::CpuAsync(_) => None,
            BackendJob::CpuAsyncStreaming(j) => j.rx_msg.try_recv().ok(),
            BackendJob::Gpu(j) => j.rx_msg.try_recv().ok(),
            BackendJob::ExternalAsync(_) => None,
        }
    }

    pub fn send_cmd(&self, cmd: ComputeCmd) -> bool {
        match self {
            BackendJob::CpuSync(_) => false,
            BackendJob::CpuAsync(_) => false,
            BackendJob::CpuAsyncStreaming(j) => j.tx_cmd.send(cmd).is_ok(),
            BackendJob::Gpu(j) => j.tx_cmd.send(cmd).is_ok(),
            BackendJob::ExternalAsync(_) => false,
        }
    }
}



