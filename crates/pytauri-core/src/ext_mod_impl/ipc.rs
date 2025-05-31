use std::{borrow::Cow, str::FromStr as _};

use pyo3::{
    exceptions::PyValueError,
    intern,
    prelude::*,
    types::{PyBytes, PyDict, PyList, PyMapping, PyString},
};
use pyo3_utils::py_wrapper::{PyWrapper, PyWrapperT0, PyWrapperT2};
use tauri::ipc::{
    self, CommandArg as _, CommandItem, InvokeBody, InvokeMessage, InvokeResponseBody,
};

use crate::{
    ext_mod::{
        webview::{Webview, WebviewWindow},
        PyAppHandleExt as _,
    },
    tauri_runtime::Runtime,
    utils::TauriError,
};

type IpcInvoke = tauri::ipc::Invoke<Runtime>;
type IpcInvokeResolver = tauri::ipc::InvokeResolver<Runtime>;
type TauriWebviewWindow = tauri::webview::WebviewWindow<Runtime>;

/// Please refer to the Python-side documentation
#[pyclass(frozen, generic)]
#[non_exhaustive]
pub struct InvokeResolver {
    inner: PyWrapper<PyWrapperT2<IpcInvokeResolver>>,
    #[pyo3(get)]
    arguments: Py<PyDict>,
}

impl InvokeResolver {
    #[inline]
    fn new(resolver: IpcInvokeResolver, arguments: Py<PyDict>) -> Self {
        Self {
            inner: PyWrapper::new2(resolver),
            arguments,
        }
    }
}

#[pymethods]
// NOTE: These pymethods implementation must not block
impl InvokeResolver {
    // NOTE: use `Cow<[u8]>` instead of `Vec<u8>`,
    // see: <https://github.com/PyO3/pyo3/issues/2888>
    fn resolve(&self, py: Python<'_>, value: Cow<'_, [u8]>) -> PyResult<()> {
        // NOTE: This function implementation must not block
        py.allow_threads(|| {
            let resolver = self.inner.try_take_inner()??;
            resolver.resolve(InvokeResponseBody::Raw(value.into_owned()));
            Ok(())
        })
    }

    // TODO: Support more Python types. Tauri seems to only support `serde` types,
    // and not support `Raw: Vec<[u8]>`. We should open an issue to ask them about this.
    //
    // TODO, PERF: once we drop py39, we can use `value: &str` instead of `Cow<'_, str>`.
    fn reject(&self, py: Python<'_>, value: Cow<'_, str>) -> PyResult<()> {
        // NOTE: This function implementation must not block
        py.allow_threads(|| {
            let resolver = self.inner.try_take_inner()??;
            resolver.reject(value);
            Ok(())
        })
    }
}

/// Please refer to the Python-side documentation
#[pyclass(frozen)]
#[non_exhaustive]
pub struct Invoke {
    inner: PyWrapper<PyWrapperT2<IpcInvoke>>,
    #[pyo3(get)]
    command: Py<PyString>,
}

impl Invoke {
    /// If the frontend makes an illegal IPC call, it will automatically reject and return [None]
    #[cfg(feature = "__private")]
    pub fn new(py: Python<'_>, invoke: IpcInvoke) -> Option<Self> {
        let func_name = match Self::get_func_name_from_message(&invoke.message) {
            Ok(name) => name,
            Err(e) => {
                invoke.resolver.reject(e);
                return None;
            }
        };
        // TODO, PERF: may be we should use [PyString::intern] ?
        //     > However, for security reasons, since the input can be any string,
        //     > unconditionally using [PyString::intern] will cause continuous memory growth issues.
        //     > TODO, XXX: 👆 is this right?
        let command = PyString::new(py, func_name).unbind();

        let slf = Self {
            inner: PyWrapper::new2(invoke),
            command,
        };
        Some(slf)
    }

    const PYFUNC_HEADER_KEY: &str = "pyfunc";

    #[inline]
    fn get_func_name_from_message(message: &InvokeMessage<Runtime>) -> Result<&str, String> {
        let func_name = message
            .headers()
            .get(Self::PYFUNC_HEADER_KEY)
            .ok_or_else(|| format!("There is no {} header", Self::PYFUNC_HEADER_KEY))?
            .to_str()
            .map_err(|e| format!("{e}"))?;
        Ok(func_name)
    }
}

#[pymethods]
// NOTE: These pymethods implementation must not block
impl Invoke {
    // NOTE: remember to use `pyo3::intern!` for performance,
    // see: <https://github.com/PyO3/pyo3/discussions/2266#discussioncomment-2491646>.
    const BODY_KEY: &str = "body";
    const APP_HANDLE_KEY: &str = "app_handle";
    const WEBVIEW_WINDOW_KEY: &str = "webview_window";
    const HEADERS_KEY: &str = "headers";

    /// Pass in a Python dictionary, which can contain the following
    /// optional keys (values are arbitrary):
    ///
    /// - [Self::BODY_KEY] : [PyBytes]
    /// - [Self::APP_HANDLE_KEY] : [crate::ext_mod::AppHandle]
    /// - [Self::WEBVIEW_WINDOW_KEY] : [crate::ext_mod::webview::WebviewWindow]
    ///
    /// # Returns
    ///
    /// - On successful parsing of [Invoke], this function will set
    ///     the corresponding types for the existing keys and return [InvokeResolver].
    ///
    ///     The return value [InvokeResolver::arguments] is not the same object as
    ///     the input `parameters`.
    /// - On failure, it returns [None], consumes and rejects [Invoke];
    fn bind_to(&self, parameters: Bound<'_, PyMapping>) -> PyResult<Option<InvokeResolver>> {
        // NOTE: This function implementation must not block

        // see <https://docs.rs/tauri/2.1.1/tauri/ipc/trait.CommandArg.html#implementors>
        // for how to parse the arguments

        let py = parameters.py();
        let invoke = self.inner.try_take_inner()??;
        let IpcInvoke {
            message,
            resolver,
            acl,
        } = invoke;

        let arguments = PyDict::new(py);

        let body_key = intern!(py, Invoke::BODY_KEY);
        if parameters.contains(body_key)? {
            match message.payload() {
                InvokeBody::Json(_) => {
                    resolver.reject(
                        "Please use `ArrayBuffer` or `Uint8Array` raw request, it's more efficient",
                    );
                    return Ok(None);
                }
                InvokeBody::Raw(body) => arguments.set_item(body_key, PyBytes::new(py, body))?,
            }
        }

        let app_handle_key = intern!(py, Invoke::APP_HANDLE_KEY);
        if parameters.contains(app_handle_key)? {
            let py_app_handle = message.webview_ref().try_py_app_handle()?;
            arguments.set_item(app_handle_key, py_app_handle)?;
        }

        let webview_window_key = intern!(py, Invoke::WEBVIEW_WINDOW_KEY);
        if parameters.contains(webview_window_key)? {
            let command_webview_window_item = CommandItem {
                plugin: None,
                name: "__whatever__pyfunc",
                key: "__whatever__webviewWindow",
                message: &message,
                acl: &acl,
            };
            // TODO, PERF: maybe we should release the GIL here?
            let webview_window = match TauriWebviewWindow::from_command(command_webview_window_item)
            {
                Ok(webview_window) => webview_window,
                Err(e) => {
                    resolver.invoke_error(e);
                    return Ok(None);
                }
            };
            arguments.set_item(webview_window_key, WebviewWindow::new(webview_window))?;
        }

        let headers_key = intern!(py, Invoke::HEADERS_KEY);
        if parameters.contains(headers_key)? {
            let headers: Vec<(&[u8], &[u8])> = message
                .headers()
                // PERF:
                // > Each key will be yielded once per associated value.
                // > So, if a key has 3 associated values, it will be yielded 3 times.
                //
                // This means the same key may generate multiple PyBytes objects
                // (although this is consistent with the Python `h11` implementation).
                // We need to use [HeaderMap::into_iter] to improve this (but this requires ownership, need a Tauri feature request):
                // when get [None], we only need to clone the previous PyBytes.
                .iter()
                // PERF: Perhaps we don't need to filter out [PYFUNC_HEADER_KEY], just pass it to Python as is.
                //
                // TODO: Ideally, we should use [HeaderMap::remove] in [Self::get_func_name_from_message]
                // to pop [PYFUNC_HEADER_KEY], but currently, we cannot obtain ownership/mutable reference
                // of `headers` from `invoke`. We should submit a feature request to Tauri.
                .filter(|(key, _)| **key != Self::PYFUNC_HEADER_KEY)
                .map(|(key, value)| (key.as_ref(), value.as_bytes()))
                .collect();
            // TODO: Unify and export this type in [crate::ext_mod::ipc], see Python [pytauri.ipc.Headers] type.
            let py_headers = PyList::new(py, headers)?;
            arguments.set_item(headers_key, py_headers)?;
        }

        Ok(Some(InvokeResolver::new(resolver, arguments.unbind())))
    }

    // NOTE: use `Cow<[u8]>` instead of `Vec<u8>`,
    // see: <https://github.com/PyO3/pyo3/issues/2888>
    fn resolve(&self, py: Python<'_>, value: Cow<'_, [u8]>) -> PyResult<()> {
        // NOTE: This function implementation must not block

        py.allow_threads(|| {
            let resolver = self.inner.try_take_inner()??.resolver;
            resolver.resolve(InvokeResponseBody::Raw(value.into_owned()));
            Ok(())
        })
    }

    // TODO: Support more Python types. Tauri seems to only support `serde` types,
    // and not support `Raw: Vec<[u8]>`. We should open an issue to ask them about this.
    //
    // TODO, PERF: once we drop py39, we can use `value: &str` instead of `Cow<'_, str>`.
    fn reject(&self, py: Python<'_>, value: Cow<'_, str>) -> PyResult<()> {
        // NOTE: This function implementation must not block

        py.allow_threads(|| {
            let resolver = self.inner.try_take_inner()??.resolver;
            resolver.reject(value);
            Ok(())
        })
    }
}

/// see also: [tauri::ipc::JavaScriptChannelId]
#[pyclass(frozen)]
#[non_exhaustive]
pub struct JavaScriptChannelId(PyWrapper<PyWrapperT0<ipc::JavaScriptChannelId>>);

impl JavaScriptChannelId {
    fn new(js_channel_id: ipc::JavaScriptChannelId) -> Self {
        Self(PyWrapper::new0(js_channel_id))
    }
}

#[pymethods]
impl JavaScriptChannelId {
    #[staticmethod]
    fn from_str(py: Python<'_>, value: &str) -> PyResult<Self> {
        // PERF: it's short enough, so we don't release the GIL
        let result = ipc::JavaScriptChannelId::from_str(value);
        match result {
            Ok(js_channel_id) => Ok(Self::new(js_channel_id)),
            Err(err) => {
                let msg: &'static str = err;
                // because the `err` is `static`, so we use `PyString::intern`.
                // TODO, PERF: maybe we can just use `pyo3::intern!("failed to parse JavaScriptChannelId")`.
                let msg = PyString::intern(py, msg).unbind();
                Err(PyValueError::new_err(msg))
            }
        }
    }

    /// PERF, TODO: maybe we should accept `Union[Webview, WebviewWindow]`,
    /// so that user dont need create new `Webview` pyobject for `WebviewWindow`.
    fn channel_on(&self, py: Python<'_>, webview: Py<Webview>) -> Channel {
        py.allow_threads(|| {
            let js_channel_id = self.0.inner_ref();
            let webview = webview.get().0.inner_ref().clone();
            // TODO, FIXME, PERF:
            // Why [JavaScriptChannelId::channel_on] need take the ownership of [Webview]?
            // We should ask tauri developers.
            let channel = js_channel_id.channel_on(webview); // maybe block, so we release the GIL
            Channel::new(channel)
        })
    }
}

/// see also: [tauri::ipc::Channel]
#[pyclass(frozen)]
#[non_exhaustive]
pub struct Channel(PyWrapper<PyWrapperT0<ipc::Channel>>);

impl Channel {
    fn new(channel: ipc::Channel) -> Self {
        Self(PyWrapper::new0(channel))
    }
}

#[pymethods]
impl Channel {
    fn id(&self) -> u32 {
        self.0.inner_ref().id()
    }

    // NOTE: use `Cow<[u8]>` instead of `Vec<u8>`,
    // see: <https://github.com/PyO3/pyo3/issues/2888>
    fn send(&self, py: Python<'_>, data: Cow<'_, [u8]>) -> PyResult<()> {
        // [tauri::ipc::Channel::send] is not a very fast operation,
        // so we need to release the GIL
        py.allow_threads(|| {
            self.0
                .inner_ref()
                .send(InvokeResponseBody::Raw(data.into_owned()))
                .map_err(TauriError::from)?;
            Ok(())
        })
    }
}

// You can enable this comment and expand the macro
// with rust-analyzer to understand how tauri implements IPC
/*
#[expect(unused_variables)]
#[expect(dead_code)]
#[expect(unused_imports)]
mod foo {
    use super::*;

    use tauri::ipc::{Channel, CommandScope, GlobalScope, InvokeResponseBody, Request, Response};

    #[tauri::command]
    #[expect(clippy::too_many_arguments)]
    async fn foo(
        request: Request<'_>,
        command_scope: CommandScope<String>,
        global_scope: GlobalScope<String>,
        app_handle: tauri::AppHandle,
        webview: tauri::Webview,
        webview_window: tauri::WebviewWindow,
        window: tauri::Window,
        channel: Channel<InvokeResponseBody>,
        state: tauri::State<'_, String>,
    ) -> Result<Response, String> {
        Ok(Response::new(InvokeResponseBody::Raw(Vec::new())))
    }

    fn bar() {
        let _ = tauri::Builder::new().invoke_handler(tauri::generate_handler![foo]);
    }
}
 */
