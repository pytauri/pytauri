// See: <https://doc.rust-lang.org/rustdoc/unstable-features.html#extensions-to-the-doc-attribute>
#![cfg_attr(
    docsrs,
    feature(doc_cfg, doc_auto_cfg, doc_cfg_hide),
    doc(cfg_hide(doc))
)]

use std::{convert::Infallible, future, sync::Arc};

use pyo3::{
    conversion::{FromPyObject, IntoPyObject, IntoPyObjectExt as _},
    exceptions::PyTypeError,
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
        cb: impl for<'p> Fn(Python<'p>, Self) -> PyResult<()> + Send + 'static,
    ) -> PyResult<()> {
        let this = self.clone_ref(py)?;
        let cb = PyCFunction::new_closure(py, None, None, move |args, _kwargs| {
            let py = args.py();
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
    runtime: &tokio::runtime::Runtime,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_future() {
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

        Python::with_gil(|py| {
            async fn rs_future() -> PyResult<i32> {
                Ok(42)
            }

            let py_future = future_to_py(py, &runtime, rs_future()).unwrap();

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
}
