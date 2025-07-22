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
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use crossbeam::atomic::AtomicCell;
use pyo3::{
    conversion::{FromPyObject, IntoPyObject, IntoPyObjectExt as _},
    exceptions::{PyRuntimeError, PyTypeError},
    prelude::*,
    sync::GILOnceCell,
    types::{PyCFunction, PyType},
};
use tokio::sync::{oneshot, Notify};

struct Future(PyObject);

impl Future {
    fn py_future_class<'py>(py: Python<'py>) -> PyResult<&'py Bound<'py, PyType>> {
        static FUTURE_CLASS: GILOnceCell<Py<PyType>> = GILOnceCell::new();
        FUTURE_CLASS.import(py, "concurrent.futures", "Future")
    }
}

impl<'py> FromPyObject<'py> for Future {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let py = ob.py();
        let future_class = Self::py_future_class(py)?;
        if !ob.is_instance(future_class)? {
            return Err(PyTypeError::new_err(format!(
                "Expected a {future_class}, got {}",
                ob.get_type()
            )));
        }
        Ok(Future(ob.clone().unbind()))
    }
}

impl<'py> IntoPyObject<'py> for Future {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.into_bound(py))
    }
}

impl Future {
    pub fn clone_ref(&self, py: Python<'_>) -> PyResult<Self> {
        let this = self.0.clone_ref(py);
        Ok(Self(this))
    }
}

impl Future {
    pub fn new(py: Python<'_>) -> PyResult<Self> {
        let future_class = Self::py_future_class(py)?;
        let future_instance = future_class.call0()?;
        Ok(Self(future_instance.into()))
    }

    pub fn cancel(&self, py: Python<'_>) -> PyResult<bool> {
        self.0.call_method0(py, "cancel")?.extract(py)
    }

    pub fn cancelled(&self, py: Python<'_>) -> PyResult<bool> {
        self.0.call_method0(py, "cancelled")?.extract(py)
    }

    pub fn running(&self, py: Python<'_>) -> PyResult<bool> {
        self.0.call_method0(py, "running")?.extract(py)
    }

    pub fn done(&self, py: Python<'_>) -> PyResult<bool> {
        self.0.call_method0(py, "done")?.extract(py)
    }

    pub fn add_done_callback(
        &self,
        py: Python<'_>,
        cb: impl for<'p> FnOnce(Python<'p>, Self) -> PyResult<()> + Send + 'static,
    ) -> PyResult<()> {
        #[pyclass(frozen)]
        struct OnceCallback(
            AtomicCell<
                Option<
                    Box<
                        dyn for<'p> FnOnce(Python<'p>, Future) -> PyResult<()>
                            + Send
                            + Sync
                            + 'static,
                    >,
                >,
            >,
        );

        impl OnceCallback {
            fn new(
                cb: impl for<'p> FnOnce(Python<'p>, Future) -> PyResult<()> + Send + Sync + 'static,
            ) -> Self {
                OnceCallback(AtomicCell::new(Some(Box::new(cb))))
            }
        }

        #[pymethods]
        impl OnceCallback {
            fn __call__(&self, py: Python<'_>, this: Future) -> PyResult<()> {
                let cb = self.0.take().ok_or_else(|| {
                    PyRuntimeError::new_err("This callback can only be called once")
                })?;
                cb(py, this)
            }
        }

        // let cb = OnceCallback::new(cb);

        let this = self.clone_ref(py)?;
        let cb = Cell::new(Some(cb));
        let cb = PyCFunction::new_closure(py, None, None, move |args, _kwargs| {
            let py = args.py();
            let cb = cb
                .take()
                .ok_or_else(|| PyRuntimeError::new_err("This callback can only be called once"))?;
            cb(py, this.clone_ref(py)?)
        })?;

        self.0.call_method1(py, "add_done_callback", (cb,))?;
        Ok(())
    }

    pub fn result(&self, py: Python<'_>, timeout: Option<f64>) -> PyResult<PyObject> {
        match timeout {
            Some(t) => self.0.call_method1(py, "result", (t,)),
            None => self.0.call_method0(py, "result"),
        }
    }

    pub fn set_running_or_notify_cancel(&self, py: Python<'_>) -> PyResult<bool> {
        self.0
            .call_method0(py, "set_running_or_notify_cancel")?
            .extract(py)
    }

    pub fn set_result(&self, py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<()> {
        self.0.call_method1(py, "set_result", (value,))?;
        Ok(())
    }

    pub fn exception(&self, py: Python<'_>, timeout: Option<f64>) -> PyResult<Option<PyErr>> {
        let ret = match timeout {
            Some(t) => self.0.call_method1(py, "exception", (t,))?,
            None => self.0.call_method0(py, "exception")?,
        };
        if ret.is_none(py) {
            Ok(None)
        } else {
            Ok(Some(PyErr::from_value(ret.into_bound(py))))
        }
    }

    pub fn set_exception(&self, py: Python<'_>, exception: Option<PyErr>) -> PyResult<()> {
        self.0.call_method1(py, "set_exception", (exception,))?;
        Ok(())
    }
}

// fn future_to_py<R>(
//     py: Python<'_>,
//     rs_future: impl future::Future<Output = PyResult<R>> + Send + 'static,
// ) -> PyResult<Future>
// where
//     for<'p> R: IntoPyObject<'p>,
// {
//     let py_future = Future::new(py)?;
//     let cancel_notify = Notify::new();
//     let cancel_notified = cancel_notify.notified();

//     py_future.add_done_callback(py, move |py, fut| {
//         if fut.cancelled(py)? {
//             cancel_notify.notify_waiters();
//         }
//         Ok(())
//     })?;

//     tokio::spawn(async move {
//         let ret = tokio::select! {
//             _ = cancel_notified => None,
//             ret = rs_future => Some(ret),
//         };
//         if let Some(ret) = ret {
//             match ret {
//                 Ok(value) => {
//                     let py_value = value.into_pyobject(py)?;
//                     py_future.set_result(py, py_value.unbind())?;
//                 }
//                 Err(err) => {
//                     py_future.set_exception(py, Some(err))?;
//                 }
//             }
//         }

//         todo!()
//     });

//     todo!()
// }

fn future_to_py<R>(
    py: Python<'_>,
    runtime: &tokio::runtime::Handle,
    rs_future: impl future::Future<Output = PyResult<R>> + Send + 'static,
) -> PyResult<Future>
where
    for<'p> R: IntoPyObject<'p>,
{
    let py_future = Future::new(py)?;
    let py_future1 = py_future.clone_ref(py)?;

    runtime.spawn(async move {
        let script = async || {
            if !Python::with_gil(|py| py_future.set_running_or_notify_cancel(py))? {
                return Ok(());
            }

            let ret = rs_future.await;
            Python::with_gil(|py| match ret {
                Ok(value) => {
                    let value = value.into_bound_py_any(py)?;
                    py_future.set_result(py, &value)
                }
                Err(err) => py_future.set_exception(py, Some(err)),
            })?;

            PyResult::Ok(())
        };
        script().await.expect("Failed to run future script")
    });

    Ok(py_future1)
}

struct Defer<T>
where
    T: FnOnce() + Send + 'static,
{
    func: Option<T>,
    suppressed: bool,
}

impl<T> Defer<T>
where
    T: FnOnce() + Send + 'static,
{
    fn new(func: T) -> Self {
        Self {
            func: Some(func),
            suppressed: false,
        }
    }

    fn suppress(mut self) {
        self.suppressed = true;
    }
}

impl<T> Drop for Defer<T>
where
    T: FnOnce() + Send + 'static,
{
    fn drop(&mut self) {
        if self.suppressed {
            return;
        }
        self.func.take().unwrap()();
    }
}

pyo3::import_exception!(concurrent.futures, CancelledError);

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
struct TokioPoolExecutor {
    runtime: tokio::runtime::Handle,
    tracker: tokio_util::task::TaskTracker,
    shutdown: Arc<AtomicCell<CancelFutures>>,
}

#[bon::bon]
impl TokioPoolExecutor {
    pub fn new(runtime: tokio::runtime::Handle) -> Self {
        Self {
            runtime,
            tracker: tokio_util::task::TaskTracker::new(),
            shutdown: Arc::new(AtomicCell::new(CancelFutures::NoSet)),
        }
    }

    pub fn submit<Func, Args, Ret>(
        &self,
        py: Python<'_>,
        func: impl FnOnce(Args) -> Func + Send + 'static,
        args: Args,
    ) -> PyResult<Future>
    where
        Args: Send + 'static,
        Func: future::Future<Output = PyResult<Ret>> + Send,
        for<'p> Ret: IntoPyObject<'p>,
    {
        if self.shutdown.load() != CancelFutures::NoSet {
            panic!("Already Shutdown")
        }

        let py_future = Future::new(py)?;
        let py_future1 = py_future.clone_ref(py)?;
        let py_future2 = py_future.clone_ref(py)?;
        let shutdown = self.shutdown.clone();

        let defer = Defer::new(move || {
            Python::with_gil(|py| py_future2.set_exception(py, Some(CancelledError::new_err(()))))
                .unwrap();
        });

        self.tracker.spawn_on(
            async move {
                let defer = defer;
                let script = async || {
                    if shutdown.load() == CancelFutures::True {
                        Python::with_gil(|py| py_future.cancel(py))?;
                        defer.suppress();
                        return Ok(());
                    }

                    if !Python::with_gil(|py| py_future.set_running_or_notify_cancel(py))? {
                        defer.suppress();
                        return Ok(());
                    }

                    let ret = func(args).await;

                    Python::with_gil(|py| match ret {
                        Ok(value) => {
                            let value = value.into_bound_py_any(py)?;
                            py_future.set_result(py, &value)
                        }
                        Err(err) => py_future.set_exception(py, Some(err)),
                    })?;

                    defer.suppress();
                    PyResult::Ok(())
                };
                script().await.expect("Failed to run future script")
            },
            &self.runtime,
        );

        Ok(py_future1)
    }

    #[builder]
    pub async fn shutdown(
        &self,
        #[builder(default = true)] wait: bool,
        #[builder(default = false)] cancel_futures: bool,
    ) {
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

struct FutureNursery {
    runtime: tokio::runtime::Handle,
}

impl FutureNursery {
    pub fn new(runtime: tokio::runtime::Handle) -> Self {
        Self { runtime }
    }

    pub fn wait(
        &self,
        py: Python<'_>,
        future: Future,
    ) -> PyResult<impl std::future::Future<Output = PyResult<PyObject>> + Send + Sync + 'static>
    {
        let (tx, rx) = oneshot::channel::<PyResult<PyObject>>();

        future.add_done_callback(py, move |py, future| {
            if future.cancelled(py)? {
                tx.send(Err(CancelledError::new_err(()))).unwrap();
                return Ok(());
            }

            if let Some(err) = future.exception(py, None)? {
                tx.send(Err(err)).unwrap();
                return Ok(());
            }

            let ret = future.result(py, None)?;
            tx.send(Ok(ret)).unwrap();
            Ok(())
        })?;

        Ok(async {
            // TODO: cancelled exception handling
            rx.await.unwrap()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_future() {
        let foo: TokioPoolExecutor = todo!();
        foo.shutdown().wait(true).cancel_futures(false).call();

        pyo3::prepare_freethreaded_python();

        Python::with_gil(|py| {
            let future = Future::new(py).unwrap();
            future
                .add_done_callback(py, |py, fut| {
                    let ret = fut.result(py, Some(0.0))?.extract::<i32>(py)?;
                    assert_eq!(ret, 42);
                    Ok(())
                })
                .unwrap();
            let input: Bound<'_, pyo3::types::PyInt> = 42.into_pyobject(py).unwrap();
            let input = input.into_any();
            future.set_result(py, &input).unwrap();

            let ret = future
                .result(py, Some(0.0))
                .unwrap()
                .extract::<i32>(py)
                .unwrap();
            assert_eq!(ret, 42);
        });
    }

    #[test]
    fn test_rs_future() {
        pyo3::prepare_freethreaded_python();

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed building the Runtime");

        let executor = TokioPoolExecutor::new(runtime.handle().clone());

        Python::with_gil(|py| {
            async fn rs_future() -> PyResult<i32> {
                Ok(42)
            }

            let py_future = executor.submit(py, |_| rs_future(), ()).unwrap();

            py_future
                .add_done_callback(py, |py, fut| {
                    let ret = fut.result(py, Some(0.001))?.extract::<i32>(py)?;
                    assert_eq!(ret, 42);
                    Ok(())
                })
                .unwrap();

            let future_nursery = FutureNursery::new(runtime.handle().clone());
            let fut = future_nursery
                .wait(py, py_future.clone_ref(py).unwrap())
                .unwrap();
            let ret = py
                .allow_threads(|| runtime.handle().block_on(fut))
                .unwrap()
                .extract::<i32>(py)
                .unwrap();
            assert_eq!(ret, 42);

            let ret = py_future
                .result(py, Some(0.001))
                .unwrap()
                .extract::<i32>(py)
                .unwrap();
            assert_eq!(ret, 42);
        });
    }

    #[test]
    fn test_shutdown() {
        console_subscriber::init();

        pyo3::prepare_freethreaded_python();

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed building the Runtime");

        let executor = TokioPoolExecutor::new(runtime.handle().clone());

        Python::with_gil(|py| {
            async fn rs_future() -> PyResult<i32> {
                println!("1");
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                println!("2");
                Ok(42)
            }

            let py_future = executor.submit(py, |_| rs_future(), ()).unwrap();

            py.allow_threads(|| {
                let wait = executor.shutdown().wait(true).cancel_futures(true).call();
                runtime.handle().block_on(wait);
            });

            py_future
                .add_done_callback(py, |py, fut| {
                    let ret = fut.result(py, Some(0.001))?.extract::<i32>(py)?;
                    assert_eq!(ret, 42);
                    Ok(())
                })
                .unwrap();

            let ret = py_future
                .result(py, Some(0.001))
                .unwrap()
                .extract::<i32>(py)
                .unwrap();
            assert_eq!(ret, 42);
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
        println!("Is AtomicCell lock-free? {}", foo);

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
