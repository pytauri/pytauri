use std::{borrow::Cow, str::FromStr as _};

use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{PyBytes, PyDict, PyMapping, PyString, PyType},
};
use pyo3_utils::py_wrapper::{PyWrapper, PyWrapperT0, PyWrapperT2};
use tauri::ipc::{
    self, CommandArg as _, CommandItem, InvokeBody, InvokeMessage, InvokeResponseBody,
};

use crate::{
    ext_mod_impl::{
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
// `subclass` for Generic type hint
#[pyclass(frozen, subclass)]
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
    fn resolve(&self, py: Python<'_>, value: Vec<u8>) -> PyResult<()> {
        // NOTE: This function implementation must not block
        py.allow_threads(|| {
            let resolver = self.inner.try_take_inner()??;
            resolver.resolve(InvokeResponseBody::Raw(value));
            Ok(())
        })
    }

    // TODO: Support more Python types. Tauri seems to only support `serde` types,
    // and not `Raw: [u8]`. We should open an issue to ask them about this.
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
        let command = PyString::new(py, func_name).unbind();

        let slf = Self {
            inner: PyWrapper::new2(invoke),
            command,
        };
        Some(slf)
    }

    #[inline]
    fn get_func_name_from_message(message: &InvokeMessage<Runtime>) -> Result<&str, String> {
        const PYFUNC_HEADER_KEY: &str = "pyfunc";

        let func_name = message
            .headers()
            .get(PYFUNC_HEADER_KEY)
            .ok_or_else(|| format!("There is no {PYFUNC_HEADER_KEY} header"))?
            .to_str()
            .map_err(|e| format!("{e}"))?;
        Ok(func_name)
    }
}

#[pymethods]
// NOTE: These pymethods implementation must not block
impl Invoke {
    // TODO, PERF: may be we should use [PyString::intern] ?
    const BODY_KEY: &str = "body";
    const APP_HANDLE_KEY: &str = "app_handle";
    const WEBVIEW_WINDOW_KEY: &str = "webview_window";

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

        if parameters.contains(Self::BODY_KEY)? {
            match message.payload() {
                InvokeBody::Json(_) => {
                    resolver.reject(
                        "Please use `ArrayBuffer` or `Uint8Array` raw request, it's more efficient",
                    );
                    return Ok(None);
                }
                InvokeBody::Raw(body) => {
                    arguments.set_item(Self::BODY_KEY, PyBytes::new(py, body))?
                }
            }
        }

        if parameters.contains(Self::APP_HANDLE_KEY)? {
            let py_app_handle = message.webview_ref().try_py_app_handle()?;
            arguments.set_item(Self::APP_HANDLE_KEY, py_app_handle.clone_ref(py))?;
        }

        if parameters.contains(Self::WEBVIEW_WINDOW_KEY)? {
            let command_webview_window_item = CommandItem {
                plugin: None,
                name: "__whatever__pyfunc",
                key: "__whatever__webviewWindow",
                message: &message,
                acl: &acl,
            };
            let webview_window = match TauriWebviewWindow::from_command(command_webview_window_item)
            {
                Ok(webview_window) => webview_window,
                Err(e) => {
                    resolver.invoke_error(e);
                    return Ok(None);
                }
            };
            arguments.set_item(Self::WEBVIEW_WINDOW_KEY, WebviewWindow::new(webview_window))?;
        }

        Ok(Some(InvokeResolver::new(resolver, arguments.unbind())))
    }

    fn resolve(&self, py: Python<'_>, value: Vec<u8>) -> PyResult<()> {
        // NOTE: This function implementation must not block

        py.allow_threads(|| {
            let resolver = self.inner.try_take_inner()??.resolver;
            resolver.resolve(InvokeResponseBody::Raw(value));
            Ok(())
        })
    }

    // TODO: Support more Python types. Tauri seems to only support `serde` types,
    // and not `Raw: [u8]`. We should open an issue to ask them about this.
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
    #[classmethod]
    fn from_str(cls: &Bound<'_, PyType>, value: &str) -> PyResult<Self> {
        let py = cls.py();
        let result = ipc::JavaScriptChannelId::from_str(value);
        match result {
            Ok(js_channel_id) => Ok(Self::new(js_channel_id)),
            Err(err) => {
                let msg: &'static str = err;
                // because the `err` is `static`, so we use `PyString::intern`.
                let msg = PyString::intern(py, msg).unbind();
                Err(PyValueError::new_err(msg))
            }
        }
    }

    /// PERF, TODO: maybe we should accept `Union[Webview, WebviewWindow]`,
    /// so that user dont need create new `Webview` pyobject for `WebviewWindow`.
    fn channel_on(&self, webview: Py<Webview>) -> Channel {
        let js_channel_id = self.0.inner_ref();
        let webview = webview.get().0.inner_ref().clone();
        // TODO, FIXME, PERF:
        // Why [JavaScriptChannelId::channel_on] need take the ownership of [Webview]?
        // We should ask tauri developers.
        let channel = js_channel_id.channel_on(webview);
        Channel::new(channel)
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

    fn send(&self, py: Python<'_>, data: Vec<u8>) -> PyResult<()> {
        // [tauri::ipc::Channel::send] is not a very fast operation,
        // so we need to release the GIL
        py.allow_threads(|| {
            self.0
                .inner_ref()
                .send(InvokeResponseBody::Raw(data))
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
