// See: <https://doc.rust-lang.org/rustdoc/unstable-features.html#extensions-to-the-doc-attribute>
#![cfg_attr(
    docsrs,
    feature(doc_cfg, doc_auto_cfg, doc_cfg_hide),
    doc(cfg_hide(doc))
)]

use std::{
    cell::Cell,
    convert::Infallible,
    future,
    marker::Unpin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use crossbeam::atomic::AtomicCell;
use pyo3::{
    conversion::{FromPyObject, IntoPyObject, IntoPyObjectExt as _},
    exceptions::{PyRuntimeError, PyTypeError},
    ffi, intern,
    prelude::*,
    sync::GILOnceCell,
    type_object::PyTypeInfo,
    types::{DerefToPyAny, PyCFunction, PyType},
};
use tokio::sync::{oneshot, Notify};

pyo3::import_exception!(concurrent.futures, CancelledError);

#[repr(transparent)]
pub struct PyFuture(PyAny);

impl DerefToPyAny for PyFuture {}

unsafe impl PyTypeInfo for PyFuture {
    const NAME: &'static str = "Future";
    const MODULE: Option<&'static str> = Some("concurrent.futures");

    fn type_object_raw(py: Python<'_>) -> *mut ffi::PyTypeObject {
        static FUTURE_CLASS: GILOnceCell<Py<PyType>> = GILOnceCell::new();
        FUTURE_CLASS
            .import(py, "concurrent.futures", "Future")
            .unwrap()
            .as_type_ptr()
    }
}

#[derive(bon::Builder)]
pub struct CancelArgs {}

#[derive(bon::Builder)]
pub struct CancelledArgs {}

#[derive(bon::Builder)]
pub struct RunningArgs {}

#[derive(bon::Builder)]
pub struct DoneArgs {}

#[derive(bon::Builder)]
pub struct AddDoneCallbackArgs<Fn_>
where
    Fn_: for<'p> FnOnce(Bound<'p, PyFuture>) -> PyResult<()> + Send + 'static,
{
    #[builder(start_fn)]
    fn_: Fn_,
}

#[derive(bon::Builder)]
pub struct ResultArgs {
    timeout: Option<f64>,
}

#[derive(bon::Builder)]
pub struct SetRunningOrNotifyCancelArgs {}

#[derive(bon::Builder)]
pub struct SetResultArgs<'a, 'py> {
    #[builder(start_fn)]
    result: &'a Bound<'py, PyAny>,
}

#[derive(bon::Builder)]
pub struct ExceptionArgs {
    timeout: Option<f64>,
}

#[derive(bon::Builder)]
pub struct SetExceptionArgs {
    #[builder(start_fn)]
    exception: Option<PyErr>,
}

#[bon::bon]
impl PyFuture {
    #[builder]
    pub fn new(#[builder(start_fn)] py: Python<'_>) -> PyResult<Bound<'_, Self>> {
        let future_class = Self::type_object(py);
        let future_instance = future_class.call0()?;
        // Safety: As long as we correctly implement `PyTypeInfo` for `PyFuture`, this cast is safe.
        unsafe { Ok(future_instance.downcast_into_unchecked()) }
    }
}

mod sealed {
    use super::*;

    pub trait Sealed {}

    impl Sealed for Bound<'_, super::PyFuture> {}
}

#[doc(alias = "PyFuture")]
pub trait PyFutureMethods<'py>: sealed::Sealed {
    fn cancel(&self, args: CancelArgs) -> PyResult<bool>;
    fn cancelled(&self, args: CancelledArgs) -> PyResult<bool>;
    fn running(&self, args: RunningArgs) -> PyResult<bool>;
    fn done(&self, args: DoneArgs) -> PyResult<bool>;
    fn add_done_callback(
        &self,
        args: AddDoneCallbackArgs<
            impl for<'p> FnOnce(Bound<'p, PyFuture>) -> PyResult<()> + Send + 'static,
        >,
    ) -> PyResult<()>;
    fn result(&self, args: ResultArgs) -> PyResult<Bound<'py, PyAny>>;
    fn set_running_or_notify_cancel(&self, args: SetRunningOrNotifyCancelArgs) -> PyResult<bool>;
    fn set_result(&self, args: SetResultArgs) -> PyResult<()>;
    fn exception(&self, args: ExceptionArgs) -> PyResult<Option<PyErr>>;
    fn set_exception(&self, args: SetExceptionArgs) -> PyResult<()>;
}

impl<'py> PyFutureMethods<'py> for Bound<'py, PyFuture> {
    fn cancel(&self, args: CancelArgs) -> PyResult<bool> {
        let py = self.py();
        let CancelArgs {} = args;
        self.call_method0(intern!(py, "cancel"))?.extract()
    }

    fn cancelled(&self, args: CancelledArgs) -> PyResult<bool> {
        let py = self.py();
        let CancelledArgs {} = args;
        self.call_method0(intern!(py, "cancelled"))?.extract()
    }

    fn running(&self, args: RunningArgs) -> PyResult<bool> {
        let py = self.py();
        let RunningArgs {} = args;
        self.call_method0(intern!(py, "running"))?.extract()
    }

    fn done(&self, args: DoneArgs) -> PyResult<bool> {
        let py = self.py();
        let DoneArgs {} = args;
        self.call_method0(intern!(py, "done"))?.extract()
    }

    fn add_done_callback(
        &self,
        args: AddDoneCallbackArgs<
            impl for<'p> FnOnce(Bound<'p, PyFuture>) -> PyResult<()> + Send + 'static,
        >,
    ) -> PyResult<()> {
        // #[pyclass(frozen)]
        // struct OnceCallback(
        //     AtomicCell<
        //         Option<
        //             Box<
        //                 dyn for<'p> FnOnce(Python<'p>, Future) -> PyResult<()>
        //                     + Send
        //                     + Sync
        //                     + 'static,
        //             >,
        //         >,
        //     >,
        // );

        // impl OnceCallback {
        //     fn new(
        //         cb: impl for<'p> FnOnce(Python<'p>, Future) -> PyResult<()> + Send + Sync + 'static,
        //     ) -> Self {
        //         OnceCallback(AtomicCell::new(Some(Box::new(cb))))
        //     }
        // }

        // #[pymethods]
        // impl OnceCallback {
        //     fn __call__(&self, py: Python<'_>, this: Future) -> PyResult<()> {
        //         let cb = self.0.take().ok_or_else(|| {
        //             PyRuntimeError::new_err("This callback can only be called once")
        //         })?;
        //         cb(py, this)
        //     }
        // }

        // let cb = OnceCallback::new(cb);
        let py = self.py();
        let AddDoneCallbackArgs { fn_ } = args;
        // TODO: cyclic reference?
        let fn_args = Cell::new(Some((fn_, self.clone().unbind())));
        let closure = PyCFunction::new_closure(py, None, None, move |args, _kwargs| {
            let py = args.py();
            let (fn_, self_) = fn_args
                .take()
                .ok_or_else(|| PyRuntimeError::new_err("This callback can only be called once"))?;
            fn_(self_.into_bound(py))
        })?;

        self.call_method1(intern!(py, "add_done_callback"), (closure,))?;
        Ok(())
    }

    fn result(&self, args: ResultArgs) -> PyResult<Bound<'py, PyAny>> {
        let py = self.py();
        let ResultArgs { timeout } = args;
        let method_name = intern!(py, "result");
        match timeout {
            Some(t) => self.call_method1(method_name, (t,)),
            None => self.call_method0(method_name),
        }
    }

    fn set_running_or_notify_cancel(&self, args: SetRunningOrNotifyCancelArgs) -> PyResult<bool> {
        let py = self.py();
        let SetRunningOrNotifyCancelArgs {} = args;
        self.call_method0(intern!(py, "set_running_or_notify_cancel"))?
            .extract()
    }

    fn set_result(&self, args: SetResultArgs) -> PyResult<()> {
        let py = self.py();
        let SetResultArgs { result } = args;
        self.call_method1(intern!(py, "set_result"), (result,))?;
        Ok(())
    }

    fn exception(&self, args: ExceptionArgs) -> PyResult<Option<PyErr>> {
        let py = self.py();
        let ExceptionArgs { timeout } = args;
        let method_name = intern!(py, "exception");
        let ret = match timeout {
            Some(t) => self.call_method1(method_name, (t,))?,
            None => self.call_method0(method_name)?,
        };
        if ret.is_none() {
            Ok(None)
        } else {
            Ok(Some(PyErr::from_value(ret)))
        }
    }

    fn set_exception(&self, args: SetExceptionArgs) -> PyResult<()> {
        let py = self.py();
        let SetExceptionArgs { exception } = args;
        self.call_method1(intern!(py, "set_exception"), (exception,))?;
        Ok(())
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CancelFutures {
    NoSet,
    True,
    False,
}

const _: () = {
    assert!(AtomicCell::<CancelFutures>::is_lock_free());
};

/// A executor like python `concurrent.futures.ThreadPoolExecutor`
pub struct TokioPoolExecutor {
    runtime: tokio::runtime::Handle,
    tracker: tokio_util::task::TaskTracker,
    shutdown: Arc<AtomicCell<CancelFutures>>,
}

#[bon::bon]
impl TokioPoolExecutor {
    #[builder]
    pub fn new(#[builder(start_fn)] runtime: tokio::runtime::Handle) -> Self {
        Self {
            runtime,
            tracker: tokio_util::task::TaskTracker::new(),
            shutdown: Arc::new(AtomicCell::new(CancelFutures::NoSet)),
        }
    }

    #[builder]
    pub fn submit<'py, Args, Ret, Fut>(
        &self,
        #[builder(start_fn)] py: Python<'py>,
        #[builder(start_fn)] fn_: impl FnOnce(Args) -> Fut + Send + 'static,
        #[builder(start_fn)] args: Args,
    ) -> PyResult<Bound<'py, PyFuture>>
    where
        Args: Send + 'static,
        Fut: future::Future<Output = PyResult<Ret>> + Send,
        for<'p> Ret: IntoPyObject<'p>,
    {
        if self.shutdown.load() != CancelFutures::NoSet {
            panic!("Already Shutdown")
        }

        let py_future = PyFuture::builder(py).build()?;
        let py_future1 = py_future.clone();
        let py_future2 = py_future.clone().unbind();
        let shutdown = self.shutdown.clone();

        // If `spawn_on` is cancelled/dropped, ensure the `PyFuture` is also cancelled
        let defer = Defer::new(move || {
            let err = Some(CancelledError::new_err("rust future cancelled"));
            let args = SetExceptionArgs::builder(err).build();
            Python::with_gil(|py| {
                let py_future2 = py_future2.bind(py);
                if let Err(e) = py_future2.set_exception(args) {
                    e.write_unraisable(py, Some(py_future2));
                }
            })
        });

        let py_future = py_future.unbind();
        self.tracker.spawn_on(
            async move {
                let defer = defer;
                let script = async || {
                    if shutdown.load() == CancelFutures::True {
                        Python::with_gil(|py| {
                            py_future.bind(py).cancel(CancelArgs::builder().build())
                        })?;
                        defer.suppress();
                        return Ok(());
                    }

                    if !Python::with_gil(|py| {
                        let args = SetRunningOrNotifyCancelArgs::builder().build();
                        py_future.bind(py).set_running_or_notify_cancel(args)
                    })? {
                        // `PyFuture` was already cancelled
                        defer.suppress();
                        return Ok(());
                    }

                    let ret = fn_(args).await;

                    Python::with_gil(|py| match ret {
                        Ok(result) => {
                            let result = result.into_bound_py_any(py)?;
                            let args = SetResultArgs::builder(&result).build();
                            py_future.bind(py).set_result(args)
                        }
                        Err(err) => {
                            let args = SetExceptionArgs::builder(Some(err)).build();
                            py_future.bind(py).set_exception(args)
                        }
                    })?;

                    defer.suppress();
                    PyResult::Ok(())
                };
                // TODO: `write_unraisable`?
                script().await.expect("Failed to run future script")
            },
            &self.runtime,
        );

        Ok(py_future1)
    }

    #[builder]
    // We may use parameters with `'py` in the future,
    // and `async fn` would cause the generated `Future` to capture `'py`,
    // so we use `impl Future` instead.
    #[expect(clippy::manual_async_fn)]
    pub fn shutdown(
        &self,
        #[builder(default = true)] wait: bool,
        #[builder(default = false)] cancel_futures: bool,
    ) -> impl future::Future<Output = ()> + Send + Sync + '_ {
        async move {
            if self.shutdown.swap(if cancel_futures {
                CancelFutures::True
            } else {
                CancelFutures::False
            }) != CancelFutures::NoSet
            {
                return;
            }

            self.tracker.close();

            if wait {
                self.tracker.wait().await;
            }
        }
    }
}

pub struct FutureNursery {
    runtime: tokio::runtime::Handle,
}

#[bon::bon]
impl FutureNursery {
    #[builder]
    pub fn new(#[builder(start_fn)] runtime: tokio::runtime::Handle) -> Self {
        Self { runtime }
    }

    #[builder]
    pub fn wait<'py>(
        &self,
        #[builder(start_fn)] future: Bound<'py, PyFuture>,
    ) -> PyResult<impl future::Future<Output = PyResult<PyObject>> + Send + Sync + 'static> {
        let (tx, rx) = oneshot::channel::<PyResult<PyObject>>();

        let args = AddDoneCallbackArgs::builder(move |future| {
            if future.cancelled(CancelledArgs::builder().build())? {
                // maybe cancelled
                let _ = tx.send(Err(CancelledError::new_err(())));
                return Ok(());
            }

            if let Some(err) = future.exception(ExceptionArgs::builder().build())? {
                // maybe cancelled
                let _ = tx.send(Err(err));
                return Ok(());
            }

            let ret = future.result(ResultArgs::builder().build())?;
            // maybe cancelled
            let _ = tx.send(Ok(ret.unbind()));
            Ok(())
        })
        .build();
        future.add_done_callback(args)?;

        let future = future.unbind();
        let defer = Defer::new(move || {
            Python::with_gil(|py| {
                let future = future.bind(py);
                if let Err(e) = future.cancel(CancelArgs::builder().build()) {
                    e.write_unraisable(py, Some(future));
                }
            })
        });
        Ok(async {
            let defer = defer;
            let ret = rx.await.unwrap();
            defer.suppress();
            ret
        })
    }
}

struct Defer<T>
where
    T: FnOnce() + Send + 'static,
{
    func: Option<T>,
}

impl<T> Defer<T>
where
    T: FnOnce() + Send + 'static,
{
    fn new(func: T) -> Self {
        Self { func: Some(func) }
    }

    fn suppress(mut self) {
        self.func.take();
    }
}

impl<T> Drop for Defer<T>
where
    T: FnOnce() + Send + 'static,
{
    fn drop(&mut self) {
        if let Some(func) = self.func.take() {
            func();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_future() -> anyhow::Result<()> {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let callback_return = Arc::new(AtomicCell::new(None));
            let callback_return1 = callback_return.clone();

            let future = PyFuture::builder(py).build().unwrap();
            future.add_done_callback(
                AddDoneCallbackArgs::builder(move |future| {
                    let ret = future
                        .result(ResultArgs::builder().timeout(0.0).build())?
                        .extract::<i32>()?;
                    callback_return1.store(Some(ret));
                    Ok(())
                })
                .build(),
            )?;
            let input: Bound<'_, pyo3::types::PyInt> = 42.into_pyobject(py).unwrap();
            let input = input.into_any();
            future.set_result(SetResultArgs::builder(&input).build())?;

            let ret = future
                .result(ResultArgs::builder().timeout(0.0).build())
                .unwrap()
                .extract::<i32>()
                .unwrap();
            assert_eq!(ret, 42);
            assert_eq!(callback_return.load(), Some(42));
            assert!(!future.cancel(CancelArgs::builder().build())?);
            assert!(!future.cancelled(CancelledArgs::builder().build())?);
            assert!(!future.running(RunningArgs::builder().build())?);
            assert!(future.done(DoneArgs::builder().build())?);
            assert!(future
                .exception(ExceptionArgs::builder().build())?
                .is_none());
            Ok(())
        })
    }

    #[test]
    fn test_canceling_future() -> anyhow::Result<()> {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let future = PyFuture::builder(py).build().unwrap();
            future.cancel(CancelArgs::builder().build())?;
            assert!(future.cancelled(CancelledArgs::builder().build())?);
            Ok(())
        })
    }

    #[test]
    fn test_future_set_exception() -> anyhow::Result<()> {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let err = PyRuntimeError::new_err(());
            let future = PyFuture::builder(py).build().unwrap();
            future.set_exception(SetExceptionArgs::builder(Some(err)).build())?;
            let exception = future
                .exception(ExceptionArgs::builder().build())?
                .expect("should be Some");
            assert!(exception.is_instance_of::<PyRuntimeError>(py));
            Ok(())
        })
    }

    #[test]
    fn test_rs_future() {
        pyo3::prepare_freethreaded_python();

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed building the Runtime");

        let executor = TokioPoolExecutor::builder(runtime.handle().clone()).build();

        Python::with_gil(|py| {
            async fn rs_future() -> PyResult<i32> {
                Ok(42)
            }

            let py_future = executor.submit(py, |_| rs_future(), ()).call().unwrap();
            py_future
                .add_done_callback(
                    AddDoneCallbackArgs::builder(|fut| {
                        let ret = fut
                            .result(ResultArgs::builder().timeout(0.001).build())?
                            .extract::<i32>()?;
                        assert_eq!(ret, 42);
                        Ok(())
                    })
                    .build(),
                )
                .unwrap();

            let future_nursery = FutureNursery::builder(runtime.handle().clone()).build();
            let fut = future_nursery.wait(py_future.clone()).call().unwrap();
            let ret = py
                .allow_threads(|| runtime.handle().block_on(fut))
                .unwrap()
                .extract::<i32>(py)
                .unwrap();
            assert_eq!(ret, 42);

            let ret = py_future
                .result(ResultArgs::builder().timeout(0.001).build())
                .unwrap()
                .extract::<i32>()
                .unwrap();
            assert_eq!(ret, 42);
        });
    }

    // #[test]
    fn test_shutdown() {
        console_subscriber::init();

        pyo3::prepare_freethreaded_python();

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed building the Runtime");

        let executor = TokioPoolExecutor::builder(runtime.handle().clone()).build();

        Python::with_gil(|py| {
            async fn rs_future() -> PyResult<i32> {
                println!("1");
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                println!("2");
                Ok(42)
            }

            let py_future = executor.submit(py, |_| rs_future(), ()).call().unwrap();

            py.allow_threads(|| {
                let wait = executor.shutdown().wait(true).cancel_futures(true).call();
                runtime.handle().block_on(wait);
            });

            py_future
                .add_done_callback(
                    AddDoneCallbackArgs::builder(|fut| {
                        let cancelled = fut.cancelled(CancelledArgs::builder().build())?;
                        assert!(cancelled);
                        Ok(())
                    })
                    .build(),
                )
                .unwrap();

            let ret = py_future
                .cancelled(CancelledArgs::builder().build())
                .unwrap();
            assert!(ret);
        });
    }

    #[test]
    fn test_tokio_pool_executor() {
        enum Cancel {
            NoSet,
            True,
            False,
        }

        const _: () = {
            assert!(crossbeam::atomic::AtomicCell::<Cancel>::is_lock_free());
        };

        type Foo = Option<u8>;

        let foo = crossbeam::atomic::AtomicCell::<Option<u8>>::is_lock_free();
        println!("Is AtomicCell lock-free? {foo}");

        // let runtime = tokio::runtime::Builder::new_multi_thread()
        //     .enable_all()
        //     .build()
        //     .expect("Failed building the Runtime");

        // struct Defer(i32);

        // impl Drop for Defer {
        //     fn drop(&mut self) {
        //         println!("Defer dropped with value: {}", self.0);
        //     }
        // }

        // // for i in 0..8 {
        // //     runtime.spawn(async {
        // //         std::thread::sleep(std::time::Duration::from_secs(5));
        // //     });
        // // }

        // let handle = runtime.handle().clone();

        // // runtime.shutdown_background();

        // let tracker = tokio_util::task::TaskTracker::new();
        // tracker.close();

        // let mut buf = vec![];

        // for i in 0..1024 {
        //     let task = tracker.spawn_on(
        //         async move {
        //             let defer = Defer(i);
        //             defer;
        //         },
        //         &handle,
        //     );
        //     buf.push(task);
        // }
        // let all_finished = buf.into_iter().all(|task| task.is_finished());
        // assert!(all_finished, "Not all tasks finished before shutdown");
    }

    #[test]
    fn test_defer() {
        {
            let defer = Defer::new(|| println!("0"));
            defer.suppress();
        }

        {
            let _defer = Defer::new(|| println!("1"));
        }
    }
}
