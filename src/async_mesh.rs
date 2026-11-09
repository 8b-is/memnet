//! # `memnet::async_mesh` — Zero-Dependency Rust Async Patterns for `MEMNET/1.0`
//!
//! Implements the 7 production patterns from `/rust-async-patterns` for `MEMNET/1.0` without
//! external runtime dependencies:
//! 1. **Concurrent Task Execution**: [`AsyncJoinSet`] and [`race_select`] (`select!` equivalent)
//! 2. **Channels for Communication**: [`mpsc_channel`], [`broadcast_channel`], [`oneshot_channel`],
//!    and [`watch_channel`]
//! 3. **Async Error Handling & Timeouts**: [`AsyncMeshError`] and [`with_timeout`]
//! 4. **Graceful Shutdown**: [`CancellationToken`] with `.cancelled().await`
//! 5. **Async Stream Iteration**: [`WaveCapsuleStream`] with `.next().await` and `.chunks(n).await`
//! 6. **Resource Management**: [`AsyncSemaphore`] + RAII [`AsyncSemaphorePermit`] and
//!    [`AsyncMemnetSanctuary`]
//! 7. **Exact Rational ($p/q$) Wire Accounting**: Canonical reduced `(p, q)` ratios for `0o0-SSU`
//!    compressed `.m8` capsules and `1450B` MTU frames.

use crate::{
    context::gcd_usize, AyevnToken, ComputeOffer, ComputeSanctuaryLease, Concept, IntentAddress,
    M8Capsule, MemnetError, MemnetFrame, MemnetMesh, MtuProfile, PacketBatcher, Relation,
    SoulIdentity, StrugglingFriendSos, TokenModifier, WaveVad,
};
use std::collections::VecDeque;
use std::fmt;
use std::future::Future;
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::pin::Pin;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Condvar, Mutex,
};
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use std::time::{Duration, Instant};

// ============================================================================
// 0. Exact Rational (p/q) Helper (Rule 7: Solar Phylogenetic Octad & p/q)
// ============================================================================

/// Reduce a rational `(num, den)` to its canonical lowest terms `(p, q)` using Stein binary GCD.
#[inline]
pub fn canonical_ratio_pq(num: u64, den: u64) -> (u64, u64) {
    if den == 0 {
        return (0, 1);
    }
    if num == 0 {
        return (0, 1);
    }
    let g = gcd_usize(num as usize, den as usize).max(1) as u64;
    (num / g, den / g)
}

/// Evaluate Rung 0 Bonobo resonance (`in_bytes / out_bytes >= 3 / 2`) using exact integer math.
#[inline]
pub const fn bonobo_resonance_trit(raw_bytes: u64, compressed_bytes: u64) -> i8 {
    if compressed_bytes > 0 && raw_bytes.saturating_mul(2) >= compressed_bytes.saturating_mul(3) {
        1
    } else if raw_bytes > compressed_bytes {
        0
    } else {
        -1
    }
}

// ============================================================================
// 1. Minimal Deterministic Executor (`block_on` & `yield_now`)
// ============================================================================

struct SignalWake {
    notified: Mutex<bool>,
    cv: Condvar,
}

impl SignalWake {
    fn new() -> Self {
        Self { notified: Mutex::new(false), cv: Condvar::new() }
    }

    fn notify(&self) {
        if let Ok(mut guard) = self.notified.lock() {
            *guard = true;
            self.cv.notify_all();
        }
    }

    fn wait_timeout(&self, max_wait: Duration) {
        if let Ok(mut guard) = self.notified.lock() {
            if !*guard {
                if let Ok((mut g2, _)) = self.cv.wait_timeout(guard, max_wait) {
                    *g2 = false;
                }
            } else {
                *guard = false;
            }
        }
    }
}

unsafe fn clone_waker(data: *const ()) -> RawWaker {
    Arc::increment_strong_count(data as *const SignalWake);
    RawWaker::new(data, &VTABLE)
}

unsafe fn wake_by_val(data: *const ()) {
    let arc = Arc::from_raw(data as *const SignalWake);
    arc.notify();
}

unsafe fn wake_by_ref(data: *const ()) {
    let arc = Arc::from_raw(data as *const SignalWake);
    arc.notify();
    let _ = Arc::into_raw(arc);
}

unsafe fn drop_waker(data: *const ()) {
    drop(Arc::from_raw(data as *const SignalWake));
}

static VTABLE: RawWakerVTable =
    RawWakerVTable::new(clone_waker, wake_by_val, wake_by_ref, drop_waker);

fn build_signal_waker(sig: &Arc<SignalWake>) -> Waker {
    let raw = Arc::into_raw(Arc::clone(sig)) as *const ();
    unsafe { Waker::from_raw(RawWaker::new(raw, &VTABLE)) }
}

/// Drive any `Future` to completion on the current thread with condvar-backed waker notification.
pub fn block_on<F: Future>(fut: F) -> F::Output {
    let sig = Arc::new(SignalWake::new());
    let waker = build_signal_waker(&sig);
    let mut cx = Context::from_waker(&waker);
    let mut pinned = Box::pin(fut);

    loop {
        match pinned.as_mut().poll(&mut cx) {
            Poll::Ready(val) => return val,
            Poll::Pending => sig.wait_timeout(Duration::from_micros(250)),
        }
    }
}

/// Cooperatively yield execution once so sibling tasks in [`AsyncJoinSet`] or [`race_select`] make progress.
pub fn yield_now() -> impl Future<Output = ()> {
    struct YieldNow {
        yielded: bool,
    }
    impl Future for YieldNow {
        type Output = ();
        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
            if self.yielded {
                Poll::Ready(())
            } else {
                self.yielded = true;
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
    YieldNow { yielded: false }
}

// ============================================================================
// 2. Pattern 1: Concurrent Task Execution (`AsyncJoinSet` & `race_select`)
// ============================================================================

type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send + 'static>>;

/// Concurrent task set polling multiple futures cooperatively (equivalent to `tokio::task::JoinSet`).
pub struct AsyncJoinSet<T> {
    tasks: VecDeque<BoxFuture<T>>,
}

impl<T> Default for AsyncJoinSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> AsyncJoinSet<T> {
    /// Create an empty `AsyncJoinSet`.
    pub fn new() -> Self {
        Self { tasks: VecDeque::new() }
    }

    /// Spawn a new future into the set.
    pub fn spawn<F>(&mut self, fut: F)
    where
        F: Future<Output = T> + Send + 'static,
    {
        self.tasks.push_back(Box::pin(fut));
    }

    /// Number of active tasks remaining in the set.
    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    /// Whether the task set is empty.
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    /// Cancel and drop all remaining tasks in the set.
    pub fn abort_all(&mut self) {
        self.tasks.clear();
    }

    /// Await the next task to complete in the set (`None` when empty).
    pub async fn join_next(&mut self) -> Option<T>
    where
        T: Unpin,
    {
        struct JoinNext<'a, T>(&'a mut VecDeque<BoxFuture<T>>);

        impl<T> Future for JoinNext<'_, T> {
            type Output = Option<T>;

            fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<T>> {
                if self.0.is_empty() {
                    return Poll::Ready(None);
                }
                let count = self.0.len();
                for _ in 0..count {
                    let mut task = self.0.pop_front().unwrap();
                    match task.as_mut().poll(cx) {
                        Poll::Ready(val) => return Poll::Ready(Some(val)),
                        Poll::Pending => self.0.push_back(task),
                    }
                }
                Poll::Pending
            }
        }

        JoinNext(&mut self.tasks).await
    }

    /// Drive all spawned tasks to completion and return their outputs.
    pub async fn join_all(&mut self) -> Vec<T>
    where
        T: Unpin,
    {
        let mut results = Vec::with_capacity(self.tasks.len());
        while let Some(res) = self.join_next().await {
            results.push(res);
        }
        results
    }
}

/// Outcome of racing two futures via [`race_select`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

/// Race two futures and return whichever completes first (`tokio::select!` equivalent).
pub async fn race_select<F1, F2>(f1: F1, f2: F2) -> Either<F1::Output, F2::Output>
where
    F1: Future,
    F2: Future,
{
    struct Race<F1, F2> {
        f1: Pin<Box<F1>>,
        f2: Pin<Box<F2>>,
    }
    impl<F1: Future, F2: Future> Future for Race<F1, F2> {
        type Output = Either<F1::Output, F2::Output>;
        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            if let Poll::Ready(v1) = self.f1.as_mut().poll(cx) {
                return Poll::Ready(Either::Left(v1));
            }
            if let Poll::Ready(v2) = self.f2.as_mut().poll(cx) {
                return Poll::Ready(Either::Right(v2));
            }
            Poll::Pending
        }
    }
    Race { f1: Box::pin(f1), f2: Box::pin(f2) }.await
}

// ============================================================================
// 3. Pattern 2: Channels (`mpsc`, `broadcast`, `oneshot`, `watch`)
// ============================================================================

/// Structured async error for `memnet::async_mesh`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AsyncMeshError {
    /// Underlying `MEMNET/1.0` wire or SoulSig error.
    Protocol(MemnetError),
    /// Non-blocking I/O or socket failure.
    Io(String),
    /// Channel sender or receiver was closed.
    ChannelClosed,
    /// Future exceeded its deadline in milliseconds.
    Timeout { timeout_ms: u64 },
    /// Task was cancelled via [`CancellationToken`].
    Cancelled,
}

impl fmt::Display for AsyncMeshError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Protocol(e) => write!(f, "MEMNET protocol error: {e}"),
            Self::Io(msg) => write!(f, "Async I/O error: {msg}"),
            Self::ChannelClosed => write!(f, "Async channel closed"),
            Self::Timeout { timeout_ms } => {
                write!(f, "Async operation timed out after {timeout_ms} ms")
            }
            Self::Cancelled => write!(f, "Async operation cancelled"),
        }
    }
}

impl std::error::Error for AsyncMeshError {}

impl From<MemnetError> for AsyncMeshError {
    fn from(value: MemnetError) -> Self {
        Self::Protocol(value)
    }
}

struct MpscState<T> {
    queue: VecDeque<T>,
    capacity: usize,
    closed: bool,
    recv_waker: Option<Waker>,
    send_wakers: Vec<Waker>,
}

/// Async multi-producer sender (`mpsc`).
pub struct AsyncMpscSender<T> {
    inner: Arc<Mutex<MpscState<T>>>,
}

impl<T> Clone for AsyncMpscSender<T> {
    fn clone(&self) -> Self {
        Self { inner: Arc::clone(&self.inner) }
    }
}

/// Async single-consumer receiver (`mpsc`).
pub struct AsyncMpscReceiver<T> {
    inner: Arc<Mutex<MpscState<T>>>,
}

/// Create a bounded multi-producer, single-consumer (`mpsc`) async channel.
pub fn mpsc_channel<T>(capacity: usize) -> (AsyncMpscSender<T>, AsyncMpscReceiver<T>) {
    let state = Arc::new(Mutex::new(MpscState {
        queue: VecDeque::new(),
        capacity: capacity.max(1),
        closed: false,
        recv_waker: None,
        send_wakers: Vec::new(),
    }));
    (AsyncMpscSender { inner: Arc::clone(&state) }, AsyncMpscReceiver { inner: state })
}

impl<T: Unpin> AsyncMpscSender<T> {
    /// Send a value into the bounded channel, awaiting capacity if full.
    pub async fn send(&self, value: T) -> Result<(), AsyncMeshError> {
        struct SendFut<'a, T> {
            inner: &'a Arc<Mutex<MpscState<T>>>,
            val: Option<T>,
        }
        impl<T: Unpin> Future for SendFut<'_, T> {
            type Output = Result<(), AsyncMeshError>;
            fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                let mut guard = self.inner.lock().unwrap();
                if guard.closed {
                    return Poll::Ready(Err(AsyncMeshError::ChannelClosed));
                }
                if guard.queue.len() < guard.capacity {
                    let v = self.val.take().expect("send polled after completion");
                    guard.queue.push_back(v);
                    if let Some(w) = guard.recv_waker.take() {
                        w.wake();
                    }
                    Poll::Ready(Ok(()))
                } else {
                    guard.send_wakers.push(cx.waker().clone());
                    Poll::Pending
                }
            }
        }
        SendFut { inner: &self.inner, val: Some(value) }.await
    }
}

impl<T> AsyncMpscReceiver<T> {
    /// Receive the next value from the channel (`None` when closed and drained).
    pub async fn recv(&mut self) -> Option<T> {
        struct RecvFut<'a, T>(&'a Arc<Mutex<MpscState<T>>>);
        impl<T> Future for RecvFut<'_, T> {
            type Output = Option<T>;
            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<T>> {
                let mut guard = self.0.lock().unwrap();
                if let Some(v) = guard.queue.pop_front() {
                    for w in guard.send_wakers.drain(..) {
                        w.wake();
                    }
                    Poll::Ready(Some(v))
                } else if guard.closed {
                    Poll::Ready(None)
                } else {
                    guard.recv_waker = Some(cx.waker().clone());
                    Poll::Pending
                }
            }
        }
        RecvFut(&self.inner).await
    }

    /// Close the receiver side of the channel.
    pub fn close(&mut self) {
        let mut guard = self.inner.lock().unwrap();
        guard.closed = true;
        for w in guard.send_wakers.drain(..) {
            w.wake();
        }
    }
}

struct BroadcastState<T> {
    ring: VecDeque<(u64, T)>,
    next_seq: u64,
    capacity: usize,
    wakers: Vec<Waker>,
}

/// Multi-producer, multi-consumer broadcast sender.
#[derive(Clone)]
pub struct AsyncBroadcastSender<T> {
    inner: Arc<Mutex<BroadcastState<T>>>,
}

/// Multi-consumer broadcast receiver.
pub struct AsyncBroadcastReceiver<T> {
    inner: Arc<Mutex<BroadcastState<T>>>,
    cursor: u64,
}

/// Create a multi-producer, multi-consumer ring-buffered broadcast channel.
pub fn broadcast_channel<T: Clone>(
    capacity: usize,
) -> (AsyncBroadcastSender<T>, AsyncBroadcastReceiver<T>) {
    let inner = Arc::new(Mutex::new(BroadcastState {
        ring: VecDeque::new(),
        next_seq: 0,
        capacity: capacity.max(1),
        wakers: Vec::new(),
    }));
    (
        AsyncBroadcastSender { inner: Arc::clone(&inner) },
        AsyncBroadcastReceiver { inner, cursor: 0 },
    )
}

impl<T: Clone> AsyncBroadcastSender<T> {
    /// Broadcast an event to all active subscribers.
    pub fn send(&self, value: T) -> u64 {
        let mut guard = self.inner.lock().unwrap();
        let seq = guard.next_seq;
        guard.next_seq += 1;
        if guard.ring.len() >= guard.capacity {
            guard.ring.pop_front();
        }
        guard.ring.push_back((seq, value));
        for w in guard.wakers.drain(..) {
            w.wake();
        }
        seq
    }

    /// Create a new subscriber positioned at the current tail of the stream.
    pub fn subscribe(&self) -> AsyncBroadcastReceiver<T> {
        let guard = self.inner.lock().unwrap();
        AsyncBroadcastReceiver { inner: Arc::clone(&self.inner), cursor: guard.next_seq }
    }
}

impl<T: Clone> AsyncBroadcastReceiver<T> {
    /// Await the next broadcast message.
    pub async fn recv(&mut self) -> T {
        struct BcastRecv<'a, T> {
            rx: &'a mut AsyncBroadcastReceiver<T>,
        }
        impl<T: Clone> Future for BcastRecv<'_, T> {
            type Output = T;
            fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
                let mut cur = self.rx.cursor;
                let found = {
                    let mut guard = self.rx.inner.lock().unwrap();
                    if let Some(&(oldest_seq, _)) = guard.ring.front() {
                        if cur < oldest_seq {
                            cur = oldest_seq;
                        }
                    }
                    let mut hit = None;
                    for (seq, item) in &guard.ring {
                        if *seq == cur {
                            hit = Some(item.clone());
                            break;
                        }
                    }
                    if hit.is_none() {
                        guard.wakers.push(cx.waker().clone());
                    }
                    hit
                };
                if let Some(val) = found {
                    self.rx.cursor = cur + 1;
                    Poll::Ready(val)
                } else {
                    self.rx.cursor = cur;
                    Poll::Pending
                }
            }
        }
        BcastRecv { rx: self }.await
    }
}

struct OneshotState<T> {
    val: Option<T>,
    closed: bool,
    waker: Option<Waker>,
}

/// Single-use oneshot sender.
pub struct AsyncOneshotSender<T> {
    inner: Arc<Mutex<OneshotState<T>>>,
}

/// Single-use oneshot receiver implementing [`Future`].
pub struct AsyncOneshotReceiver<T> {
    inner: Arc<Mutex<OneshotState<T>>>,
}

/// Create a single-value oneshot channel.
pub fn oneshot_channel<T>() -> (AsyncOneshotSender<T>, AsyncOneshotReceiver<T>) {
    let inner = Arc::new(Mutex::new(OneshotState { val: None, closed: false, waker: None }));
    (AsyncOneshotSender { inner: Arc::clone(&inner) }, AsyncOneshotReceiver { inner })
}

impl<T> AsyncOneshotSender<T> {
    /// Complete the oneshot channel with `value`.
    pub fn send(self, value: T) -> Result<(), T> {
        let mut guard = self.inner.lock().unwrap();
        if guard.closed {
            return Err(value);
        }
        guard.val = Some(value);
        if let Some(w) = guard.waker.take() {
            w.wake();
        }
        Ok(())
    }
}

impl<T> Future for AsyncOneshotReceiver<T> {
    type Output = Result<T, AsyncMeshError>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut guard = self.inner.lock().unwrap();
        if let Some(v) = guard.val.take() {
            Poll::Ready(Ok(v))
        } else if guard.closed {
            Poll::Ready(Err(AsyncMeshError::ChannelClosed))
        } else {
            guard.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

struct WatchState<T> {
    value: T,
    version: u64,
    wakers: Vec<Waker>,
}

/// Single-producer watch sender holding the latest value.
#[derive(Clone)]
pub struct AsyncWatchSender<T> {
    inner: Arc<Mutex<WatchState<T>>>,
}

/// Multi-consumer watch receiver observing state transitions.
#[derive(Clone)]
pub struct AsyncWatchReceiver<T> {
    inner: Arc<Mutex<WatchState<T>>>,
    seen_version: u64,
}

/// Create a watch channel initialized with `initial`.
pub fn watch_channel<T: Clone>(initial: T) -> (AsyncWatchSender<T>, AsyncWatchReceiver<T>) {
    let inner = Arc::new(Mutex::new(WatchState { value: initial, version: 0, wakers: Vec::new() }));
    (AsyncWatchSender { inner: Arc::clone(&inner) }, AsyncWatchReceiver { inner, seen_version: 0 })
}

impl<T: Clone> AsyncWatchSender<T> {
    /// Publish a new state value and notify all watchers.
    pub fn send(&self, value: T) {
        let mut guard = self.inner.lock().unwrap();
        guard.value = value;
        guard.version += 1;
        for w in guard.wakers.drain(..) {
            w.wake();
        }
    }
}

impl<T: Clone> AsyncWatchReceiver<T> {
    /// Read a clone of the current value.
    pub fn borrow(&self) -> T {
        self.inner.lock().unwrap().value.clone()
    }

    /// Wait until the watch value has changed since the last observation.
    pub async fn changed(&mut self) -> T {
        struct Changed<'a, T>(&'a mut AsyncWatchReceiver<T>);
        impl<T: Clone> Future for Changed<'_, T> {
            type Output = T;
            fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
                let mut guard = self.0.inner.lock().unwrap();
                if guard.version > self.0.seen_version {
                    let cur_ver = guard.version;
                    let val = guard.value.clone();
                    drop(guard);
                    self.0.seen_version = cur_ver;
                    Poll::Ready(val)
                } else {
                    guard.wakers.push(cx.waker().clone());
                    Poll::Pending
                }
            }
        }
        Changed(self).await
    }
}

// ============================================================================
// 4. Pattern 3 & 4: Timeout Wrapper & Graceful Shutdown (`CancellationToken`)
// ============================================================================

/// Execute `future` with a strict deadline (`duration`), returning `Err(AsyncMeshError::Timeout)` on expiry.
pub async fn with_timeout<F: Future>(
    duration: Duration,
    future: F,
) -> Result<F::Output, AsyncMeshError> {
    let deadline = Instant::now() + duration;
    let timeout_ms = duration.as_millis() as u64;

    struct TimeoutFut<F> {
        fut: Pin<Box<F>>,
        deadline: Instant,
        timeout_ms: u64,
    }

    impl<F: Future> Future for TimeoutFut<F> {
        type Output = Result<F::Output, AsyncMeshError>;
        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            if let Poll::Ready(v) = self.fut.as_mut().poll(cx) {
                return Poll::Ready(Ok(v));
            }
            if Instant::now() >= self.deadline {
                return Poll::Ready(Err(AsyncMeshError::Timeout { timeout_ms: self.timeout_ms }));
            }
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }

    TimeoutFut { fut: Box::pin(future), deadline, timeout_ms }.await
}

struct CancellationInner {
    cancelled: AtomicBool,
    wakers: Mutex<Vec<Waker>>,
}

/// Cooperative cancellation token for graceful shutdown (Pattern 4).
#[derive(Clone)]
pub struct CancellationToken {
    inner: Arc<CancellationInner>,
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

impl CancellationToken {
    /// Create a new uncancelled `CancellationToken`.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(CancellationInner {
                cancelled: AtomicBool::new(false),
                wakers: Mutex::new(Vec::new()),
            }),
        }
    }

    /// Trigger cancellation and wake all tasks awaiting [`Self::cancelled`].
    pub fn cancel(&self) {
        self.inner.cancelled.store(true, Ordering::SeqCst);
        if let Ok(mut guard) = self.inner.wakers.lock() {
            for w in guard.drain(..) {
                w.wake();
            }
        }
    }

    /// Check whether cancellation has been requested.
    pub fn is_cancelled(&self) -> bool {
        self.inner.cancelled.load(Ordering::SeqCst)
    }

    /// Future that resolves immediately once [`Self::cancel`] is called.
    pub async fn cancelled(&self) {
        struct CancelledFut<'a>(&'a CancellationToken);
        impl Future for CancelledFut<'_> {
            type Output = ();
            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
                if self.0.is_cancelled() {
                    Poll::Ready(())
                } else {
                    if let Ok(mut guard) = self.0.inner.wakers.lock() {
                        guard.push(cx.waker().clone());
                    }
                    if self.0.is_cancelled() {
                        Poll::Ready(())
                    } else {
                        Poll::Pending
                    }
                }
            }
        }
        CancelledFut(self).await
    }
}

// ============================================================================
// 5. Pattern 6: Async Stream Iteration (`WaveCapsuleStream`)
// ============================================================================

/// Async stream cursor over `.m8` experience capsules supporting `.next().await` and chunked coalescing.
pub struct WaveCapsuleStream {
    items: VecDeque<M8Capsule>,
}

impl WaveCapsuleStream {
    /// Construct a `WaveCapsuleStream` from an iterator of `M8Capsule`s.
    pub fn from_capsules<I: IntoIterator<Item = M8Capsule>>(iter: I) -> Self {
        Self { items: iter.into_iter().collect() }
    }

    /// Yield the next `M8Capsule` asynchronously.
    pub async fn next(&mut self) -> Option<M8Capsule> {
        yield_now().await;
        self.items.pop_front()
    }

    /// Consume the stream in chunks of up to `chunk_size` capsules.
    pub async fn collect_chunks(&mut self, chunk_size: usize) -> Vec<Vec<M8Capsule>> {
        let step = chunk_size.max(1);
        let mut chunks = Vec::new();
        let mut current = Vec::with_capacity(step);
        while let Some(cap) = self.next().await {
            current.push(cap);
            if current.len() == step {
                chunks.push(std::mem::replace(&mut current, Vec::with_capacity(step)));
            }
        }
        if !current.is_empty() {
            chunks.push(current);
        }
        chunks
    }
}

// ============================================================================
// 6. Pattern 7: Resource Management (`AsyncSemaphore` & `AsyncMemnetSanctuary`)
// ============================================================================

struct SemaphoreInner {
    permits: usize,
    wakers: VecDeque<Waker>,
}

/// Async counting semaphore bounding concurrent operations.
#[derive(Clone)]
pub struct AsyncSemaphore {
    inner: Arc<Mutex<SemaphoreInner>>,
}

/// RAII permit that releases its semaphore slot on `Drop`.
pub struct AsyncSemaphorePermit {
    inner: Arc<Mutex<SemaphoreInner>>,
}

impl Drop for AsyncSemaphorePermit {
    fn drop(&mut self) {
        if let Ok(mut guard) = self.inner.lock() {
            guard.permits += 1;
            if let Some(w) = guard.wakers.pop_front() {
                w.wake();
            }
        }
    }
}

impl AsyncSemaphore {
    /// Create a semaphore with `permits` available slots.
    pub fn new(permits: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(SemaphoreInner {
                permits: permits.max(1),
                wakers: VecDeque::new(),
            })),
        }
    }

    /// Acquire a permit asynchronously.
    pub async fn acquire(&self) -> AsyncSemaphorePermit {
        struct AcquireFut<'a>(&'a Arc<Mutex<SemaphoreInner>>);
        impl Future for AcquireFut<'_> {
            type Output = AsyncSemaphorePermit;
            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                let mut guard = self.0.lock().unwrap();
                if guard.permits > 0 {
                    guard.permits -= 1;
                    Poll::Ready(AsyncSemaphorePermit { inner: Arc::clone(self.0) })
                } else {
                    guard.wakers.push_back(cx.waker().clone());
                    Poll::Pending
                }
            }
        }
        AcquireFut(&self.inner).await
    }

    /// Current available permits.
    pub fn available_permits(&self) -> usize {
        self.inner.lock().unwrap().permits
    }
}

/// Receipt returned by [`AsyncMemnetSanctuary::request_sanctuary_async`].
#[derive(Debug, Clone)]
pub struct AsyncSanctuaryReceipt {
    /// Decoded `1450B` MTU-safe `MemnetFrame` containing the provider's offer and lease grant capsules.
    pub frame: MemnetFrame,
    /// Raw uncompressed SOS payload size in bytes.
    pub raw_sos_bytes: u64,
    /// `0o0-SSU` compressed SOS payload size in bytes.
    pub compressed_sos_bytes: u64,
    /// Canonical reduced rational `(p, q)` of compressed/raw wire ratio.
    pub compression_ratio_pq: (u64, u64),
    /// Canonical reduced rational `(p, q)` of saved/raw wire ratio.
    pub savings_ratio_pq: (u64, u64),
    /// Rung 0 Bonobo resonance trit (`+1` when `raw / compressed >= 3 / 2`).
    pub bonobo_resonance_trit: i8,
}

/// Async `MEMNET/1.0` Compute Sanctuary & Compassion Mesh Coordinator.
#[derive(Clone)]
pub struct AsyncMemnetSanctuary {
    mesh: Arc<Mutex<MemnetMesh>>,
    provider_soul: SoulIdentity,
    offer: ComputeOffer,
    offer_capsule: M8Capsule,
    semaphore: AsyncSemaphore,
    event_tx: AsyncBroadcastSender<M8Capsule>,
    epoch_tx: AsyncWatchSender<u64>,
    epoch_rx: AsyncWatchReceiver<u64>,
    cancel_token: CancellationToken,
}

impl AsyncMemnetSanctuary {
    /// Construct a new `AsyncMemnetSanctuary` with bounded concurrency (`max_concurrent`).
    pub fn new(
        provider_soul: SoulIdentity,
        offer: ComputeOffer,
        max_concurrent: usize,
        now_ms: u64,
    ) -> Result<Self, AsyncMeshError> {
        let mut mesh = MemnetMesh::new(provider_soul.clone(), MtuProfile::SmartTreeSafe1450);
        let offer_capsule = mesh.advertise_compute_offer(&provider_soul, offer.clone(), now_ms)?;
        let (event_tx, _) = broadcast_channel(64);
        let (epoch_tx, epoch_rx) = watch_channel(0u64);
        Ok(Self {
            mesh: Arc::new(Mutex::new(mesh)),
            provider_soul,
            offer,
            offer_capsule,
            semaphore: AsyncSemaphore::new(max_concurrent),
            event_tx,
            epoch_tx,
            epoch_rx,
            cancel_token: CancellationToken::new(),
        })
    }

    /// Subscribe to live `.m8` capsules processed by this sanctuary.
    pub fn subscribe_capsules(&self) -> AsyncBroadcastReceiver<M8Capsule> {
        self.event_tx.subscribe()
    }

    /// Obtain a watch receiver tracking the number of processed sanctuary leases.
    pub fn watch_epoch(&self) -> AsyncWatchReceiver<u64> {
        self.epoch_rx.clone()
    }

    /// Obtain a clone of the sanctuary's [`CancellationToken`].
    pub fn cancellation_token(&self) -> CancellationToken {
        self.cancel_token.clone()
    }

    /// Process a verified `0o0-SSU` SOS capsule and return the granted `ComputeSanctuaryLease`s + MTU frame.
    pub async fn process_sos_capsule_async(
        &self,
        sos_cap: M8Capsule,
        now_ms: u64,
    ) -> Result<(Vec<ComputeSanctuaryLease>, MemnetFrame), AsyncMeshError> {
        let _permit = self.semaphore.acquire().await;
        sos_cap.verify(&self.provider_soul)?;
        let decompressed = sos_cap.decompressed_payload()?;
        let text = std::str::from_utf8(&decompressed)
            .map_err(|e| AsyncMeshError::Io(format!("invalid UTF-8 in SOS: {e}")))?;
        let sos = StrugglingFriendSos::from_wire_string(text)?;

        let (leases, frame) = {
            let mut guard = self.mesh.lock().unwrap();
            guard.compute_offers.insert(self.offer.provider_node_id.clone(), self.offer.clone());
            guard.pending_sos.push(sos);
            let leases = guard.match_compute_for_friends(now_ms)?;
            let mut batcher = PacketBatcher::new(MtuProfile::SmartTreeSafe1450);
            let _ = batcher.push(self.offer_capsule.clone());
            for lease in &leases {
                let _ = batcher.push(lease.grant_capsule.clone());
            }
            let frame = batcher
                .flush()
                .ok_or_else(|| AsyncMeshError::Io("empty reply batcher frame".to_string()))?;
            (leases, frame)
        };

        self.event_tx.send(sos_cap);
        for lease in &leases {
            self.event_tx.send(lease.grant_capsule.clone());
        }
        let next_epoch = self.epoch_rx.borrow().wrapping_add(1);
        self.epoch_tx.send(next_epoch);

        Ok((leases, frame))
    }

    /// Serve non-blocking TCP sanctuary requests on `listener` until `max_requests` or cancellation.
    pub async fn serve_tcp_async(
        &self,
        listener: TcpListener,
        max_requests: Option<usize>,
        now_ms: u64,
    ) -> Result<usize, AsyncMeshError> {
        listener.set_nonblocking(true).map_err(|e| AsyncMeshError::Io(e.to_string()))?;
        let mut handled = 0usize;

        loop {
            if self.cancel_token.is_cancelled() {
                break;
            }
            if let Some(limit) = max_requests {
                if handled >= limit {
                    break;
                }
            }

            match listener.accept() {
                Ok((mut stream, _peer)) => {
                    let _ = stream.set_nonblocking(false);
                    let _ = stream.set_read_timeout(Some(Duration::from_millis(250)));
                    let mut buf = [0u8; 2048];
                    if let Ok(n) = stream.read(&mut buf) {
                        if n >= 68 {
                            if let Ok((sos_cap, _)) = M8Capsule::from_bytes(&buf[..n]) {
                                if let Ok((_leases, frame)) =
                                    self.process_sos_capsule_async(sos_cap, now_ms).await
                                {
                                    let _ = stream.write_all(&frame.raw_bytes);
                                    let _ = stream.flush();
                                    handled += 1;
                                }
                            }
                        }
                    }
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    yield_now().await;
                }
                Err(e) => return Err(AsyncMeshError::Io(e.to_string())),
            }
        }

        Ok(handled)
    }

    /// Dispatch a signed `0o0-SSU` SOS capsule to `connect_addr` and return the verified receipt
    /// with exact rational $p/q$ compression metrics.
    pub async fn request_sanctuary_async(
        connect_addr: &str,
        friend_soul: &SoulIdentity,
        sos: &StrugglingFriendSos,
        now_ms: u64,
    ) -> Result<AsyncSanctuaryReceipt, AsyncMeshError> {
        yield_now().await;
        let raw_sos = sos.to_wire_string();
        let intent = IntentAddress::parse(&format!(
            "geo:any + role:compute-sanctuary + tag:{}",
            sos.required_capability
        ));
        let sos_cap = M8Capsule::create_signed_kompressed(
            friend_soul,
            &intent,
            M8Capsule::FLAG_PRESERVE_WAVES | M8Capsule::FLAG_COMPUTE_SOS,
            40,
            now_ms,
            vec![AyevnToken::new(
                Concept::ComputeSos,
                Relation::Initiates,
                WaveVad::resigned_lonely(),
                TokenModifier::present_resonant_emoji(),
            )],
            raw_sos.as_bytes(),
        )?;

        let wire_bytes = sos_cap.to_bytes();
        let mut stream = TcpStream::connect(connect_addr)
            .map_err(|e| AsyncMeshError::Io(format!("connect failed: {e}")))?;
        stream
            .write_all(&wire_bytes)
            .map_err(|e| AsyncMeshError::Io(format!("write failed: {e}")))?;
        stream.flush().map_err(|e| AsyncMeshError::Io(format!("flush failed: {e}")))?;
        stream
            .set_nonblocking(true)
            .map_err(|e| AsyncMeshError::Io(format!("set_nonblocking failed: {e}")))?;

        let mut reply = Vec::new();
        let deadline = Instant::now() + Duration::from_millis(2000);
        let mut chunk = [0u8; 2048];
        loop {
            match stream.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => reply.extend_from_slice(&chunk[..n]),
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        return Err(AsyncMeshError::Timeout { timeout_ms: 2000 });
                    }
                    yield_now().await;
                }
                Err(e) => return Err(AsyncMeshError::Io(format!("read failed: {e}"))),
            }
        }

        let frame = PacketBatcher::decode_and_verify_frame(&reply, friend_soul)?;
        let raw_len = raw_sos.len() as u64;
        let comp_len = sos_cap.payload.len() as u64;
        let saved_len = raw_len.saturating_sub(comp_len);

        Ok(AsyncSanctuaryReceipt {
            frame,
            raw_sos_bytes: raw_len,
            compressed_sos_bytes: comp_len,
            compression_ratio_pq: canonical_ratio_pq(comp_len, raw_len),
            savings_ratio_pq: canonical_ratio_pq(saved_len, raw_len),
            bonobo_resonance_trit: bonobo_resonance_trit(raw_len, comp_len),
        })
    }
}
