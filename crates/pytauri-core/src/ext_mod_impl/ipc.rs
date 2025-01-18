use std::borrow::Cow;

use pyo3::{
    prelude::*,
    types::{PyByteArray, PyDict, PyMapping, PyString},
};
use pyo3_utils::py_wrapper::{PyWrapper, PyWrapperT2};
use tauri::{
    ipc::{CommandArg as _, CommandItem, InvokeBody, InvokeMessage, InvokeResponseBody},
    webview::WebviewWindow as TauriWebviewWindow,
};

use crate::{
    ext_mod_impl::{webview::WebviewWindow, PyAppHandleExt as _},
    tauri_runtime::Runtime,
};

type IpcInvoke = tauri::ipc::Invoke<Runtime>;
type IpcInvokeResolver = tauri::ipc::InvokeResolver<Runtime>;

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
    fn get_func_name_from_message(message: &InvokeMessage) -> Result<&str, String> {
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
    /// - [Self::BODY_KEY] : [PyByteArray]
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
                    arguments.set_item(Self::BODY_KEY, PyByteArray::new(py, body))?
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
