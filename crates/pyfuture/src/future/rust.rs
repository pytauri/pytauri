// TODO, XXX: `eprintln` and `println` is not async-safe and atomic,
// use `log` crate instead.
// See: <https://pyo3.rs/v0.22.5/ecosystem/logging>

use std::{
    future::Future,
    pin::{pin, Pin},
    task::{Context, Poll},
};

use pyo3::prelude::*;

use crate::future::py::PyFuture;

#[derive(Debug)]
struct InitRustFuture {
    pub(self) awaitable: PyObject,
    pub(self) runner: PyObject,
}

#[derive(Debug)]
struct RunningRustFuture {
    pub(self) py_future: Py<PyFuture>,
    pub(self) cancel_handle: PyObject,
    pub(self) cancellation_required: bool,
}

#[derive(Debug)]
enum RustFutureInner {
    Init(Option<InitRustFuture>),
    Running(RunningRustFuture),
    Done,
}

/// # NOTE
///
/// When calling the [RustFuture::poll] method, it will internally call [Python::with_gil],
/// which means it may block the Rust async runtime.
/// Therefore, it is best to use a separate Rust async runtime to schedule this future.

// The reason why we use Inner struct instead of directly use `enum RustFuture`:
//
// If we use `enum RustFuture`directly,
// when we change the state of enmu (e.g, `*(<self as &mut RustFuture> )= RustFuture::Done`),
// it will call the `Drop` of `RustFuture` unexpectedly
//     (it will raise `RustFuture dropped when PyFuture maybe still running` warning).
#[derive(Debug)]
pub struct RustFuture(RustFutureInner);

impl RustFuture {
    pub(crate) const fn new(runner: PyObject, awaitable: PyObject) -> Self {
        let inner = RustFutureInner::Init(Some(InitRustFuture { awaitable, runner }));
        Self(inner)
    }

    #[inline]
    pub fn is_init(&self) -> bool {
        matches!(&self.0, RustFutureInner::Init(_))
    }

    #[inline]
    pub fn is_running(&self) -> bool {
        matches!(&self.0, RustFutureInner::Running(_))
    }

    #[inline]
    pub fn is_done(&self) -> bool {
        matches!(&self.0, RustFutureInner::Done)
    }

    #[inline]
    pub fn is_cancellation_required(&self) -> bool {
        match &self.0 {
            RustFutureInner::Running(RunningRustFuture {
                cancellation_required,
                ..
            }) => *cancellation_required,
            // This is a *is_bool* function, don't panic even if it's not running
            _ => false,
        }
    }

    // NOTE: For developer, whatever if you need `&mut` to change this stcuture,
    // you have to use `&mut` to make sure only one thread can cancel the future at a time,
    // it's for thread-safe for python async runtime.
    pub fn cancel(&mut self, py: Python<'_>) -> PyResult<PyObject> {
        match &mut self.0 {
            RustFutureInner::Running(RunningRustFuture {
                cancel_handle,
                cancellation_required,
                ..
            }) => {
                let result = cancel_handle.call0(py)?;
                *cancellation_required = true;
                Ok(result)
            }
            _ => panic!("Cancel a non-running future"),
        }
    }
}

/// The reason why we don't implement cancellation on Drop of `RustFuture` directly,
/// See: <https://github.com/PyO3/pyo3/pull/4095#issuecomment-2064889181>.
/// In short, [RustFuture::cancel] need GIL,
/// but calling GIL on every `Drop` is much too prone to deadlocks.
///
/// The reason why we use newtype `CancelOnDrop` instead of a `is_cancel_on_drop` field in `RustFuture`
/// is that newtype will make you easier to find the place where GIL required for cancellation.
impl Drop for RustFuture {
    fn drop(&mut self) {
        if self.is_running() && !self.is_cancellation_required() {
            // TODO: use `log` crate, for recoding line number and file name
            eprintln!("[Warning] {self:?}: RustFuture dropped when PyFuture maybe still running");
        }
    }
}

impl Future for RustFuture {
    type Output = PyResult<PyObject>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let inner = &mut self.get_mut().0;
        match inner {
            RustFutureInner::Init(init) => {
                let InitRustFuture { awaitable, runner } = init
                    .take()
                    // unreachable, just for assertion
                    .expect("InitRustFuture already polled or constructor implemented incorrectly");
                // TODO, NOTE, XXX: here we need call GIL, but it maybe block the thread;
                // i don't have a better solution for now, so just use it :)
                //
                // But NOTE: DO NOT use any other lock in GIL, or it maybe cause deadlock;
                // and release the GIL as soon as possible.
                let running_rust_future = Python::with_gil(|py| {
                    let future = PyFuture::new(awaitable, cx.waker().clone());
                    let py_future = Bound::new(py, future).expect("Failed to create Py<PyFuture>");

                    let cancel_handle =
                        // we require the implementation of runner returns as soon as possible,
                        // so this should not block too long.
                        runner.call1(py, (py_future.clone(),)).unwrap_or_else(|e| {
                            match e.traceback(py).map(|t| t.format()) {
                                Some(Ok(traceback)) => {
                                    panic!(
                                        "Error while calling runner: {e}\n{traceback}"
                                    );
                                }
                                _ => {
                                    panic!("Error while calling runner: {e:?}");
                                }
                            }
                        });

                    RunningRustFuture {
                        py_future: py_future.unbind(),
                        cancel_handle,
                        cancellation_required: false,
                    }
                });
                *inner = RustFutureInner::Running(running_rust_future);
                Poll::Pending
            }
            RustFutureInner::Running(running_rust_future) => {
                let RunningRustFuture { py_future, .. } = running_rust_future;
                let result = Python::with_gil(|py| {
                    let mut py_future = py_future.borrow_mut(py);
                    match py_future.result_as_ref() {
                        None => {
                            py_future.waker_clone_from(cx.waker());
                            Poll::Pending
                        }
                        Some(result) => {
                            let result = result
                                .as_ref()
                                .map(|ok| ok.clone_ref(py))
                                .map_err(|err| err.clone_ref(py));
                            Poll::Ready(result)
                        }
                    }
                });
                if result.is_ready() {
                    *inner = RustFutureInner::Done;
                }
                result
            }
            RustFutureInner::Done => panic!("Polling a done future"),
        }
    }
}

#[derive(Debug)]
pub struct CancelOnDrop(pub RustFuture);

impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        // perf: maybe we can use `ManuallyDrop` to avoid the cost of `self.0.drop`?
        // But `ManuallyDrop` will require `unsafe` block, i don't like any `unsafe` block.
        let rs_future = &mut self.0;
        if rs_future.is_running() && !rs_future.is_cancellation_required() {
            Python::with_gil(|py| {
                let result = rs_future.cancel(py);
                if let Err(e) = result {
                    match e.traceback(py).map(|t| t.format()) {
                        // TODO: use `log` crate instead of `eprintln!`
                        Some(Ok(traceback)) => {
                            eprintln!("[Warning] Error while cancelling on drop: {e}\n{traceback}");
                        }
                        _ => {
                            eprintln!("Warning] Error while cancelling on drop: {e:?}");
                        }
                    }
                }
            });
        }
    }
}

impl Future for CancelOnDrop {
    type Output = PyResult<PyObject>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        pin!(&mut self.0).poll(&mut Context::from_waker(cx.waker()))
    }
}
