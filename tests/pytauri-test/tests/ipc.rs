use std::{collections::HashMap, env::var, error::Error, path::PathBuf, sync::LazyLock};

use pyo3::{prelude::*, wrap_pymodule};
use pytauri::standalone::{
    PythonInterpreter, PythonInterpreterBuilder, PythonInterpreterEnv, PythonScript,
};
use serde::{de::DeserializeOwned, ser::Serialize};
use serde_json::json;
use tauri::{
    ipc::{Channel, InvokeBody, InvokeResponseBody},
    test::{get_ipc_response, MockRuntime, INVOKE_KEY},
    webview::{InvokeRequest, Webview, WebviewWindowBuilder},
};

use pytauri_test::test::{ext_mod, Runtime};

static PYI: LazyLock<PythonInterpreter> = LazyLock::new(|| {
    let virtual_env = var("VIRTUAL_ENV").unwrap();
    let py_env = PythonInterpreterEnv::Venv(PathBuf::from(virtual_env).into());
    let py_script = PythonScript::REPL;
    let builder =
        PythonInterpreterBuilder::new(py_env, py_script, |py| wrap_pymodule!(ext_mod)(py));
    builder.build().unwrap()
});

fn app_handle_fixture<F>(f: F) -> PyResult<()>
where
    F: FnOnce(&tauri::AppHandle<Runtime>) -> PyResult<()> + Send,
{
    PYI.with_gil(|py| {
        let test_mod = py.import("pytauri_test")?;
        let fixture = test_mod.getattr("app_handle_fixture")?;

        let context_manager = fixture.call0()?;

        let py_app_handle = context_manager
            .call_method0("__enter__")?
            .downcast_into::<pytauri::ext_mod::AppHandle>()?;

        let app_handle_guard = py_app_handle.get();

        let result = py.allow_threads(|| {
            let app_handle = app_handle_guard.0.inner_ref();
            f(&app_handle)
        });

        let _exit = match result {
            Ok(_) => {
                let py_none = py.None();
                context_manager.call_method1("__exit__", (&py_none, &py_none, &py_none))?
            }
            Err(e) => context_manager
                .call_method1("__exit__", (e.get_type(py), e.value(py), e.traceback(py)))?,
        };

        Ok(())
    })
}

pub fn get_pytauri_ipc_response<D: DeserializeOwned>(
    webview: &impl AsRef<Webview<MockRuntime>>,
    func_name: String,
    body: &impl Serialize,
) -> D {
    let mut headers = HashMap::new();
    headers.insert("pyfunc".to_string(), func_name);
    let headers = (&headers).try_into().unwrap();

    let body = serde_json::to_vec(body).unwrap();
    let body = InvokeBody::Raw(body);

    // see: <https://github.com/tauri-apps/tauri/blob/e3b0260871008e4d213a6036690198ea637d555b/crates/tauri/src/manager/mod.rs#L354>
    const URL: &str = {
        if cfg!(windows) {
            "http://tauri.localhost/"
        } else {
            "tauri://localhost/"
        }
    };

    let resp = get_ipc_response(
        webview,
        InvokeRequest {
            cmd: "plugin:pytauri|pyfunc".into(),
            callback: tauri::ipc::CallbackFn(0),
            error: tauri::ipc::CallbackFn(1),
            url: URL.parse().unwrap(),
            body,
            headers,
            invoke_key: INVOKE_KEY.to_string(),
        },
    )
    .unwrap();

    let data = match resp {
        InvokeResponseBody::Json(_) => panic!("ipc should return raw `Vec<u8>`"),
        InvokeResponseBody::Raw(data) => data,
    };

    serde_json::from_slice(&data).unwrap()
}

#[test]
fn test_ipc() -> Result<(), Box<dyn Error>> {
    app_handle_fixture(|app| {
        let webview = WebviewWindowBuilder::new(app, "main", Default::default())
            .build()
            .unwrap();

        let channel = Channel::<()>::new(|reps| {
            // TODO, FIXME, XXX: This test actually does not work.
            // Because the `channel` created by python using [JavaScriptChannelId::channel_on] does not call this callback.
            // We should submit a PR to tauri.

            if let InvokeResponseBody::Raw(data) = reps {
                let data: String = serde_json::from_slice(&data)?;
                assert_eq!(data, "ping");
                Ok(())
            } else {
                panic!("expected raw data");
            }
        });

        let body = json!(
            {
                "ping": "ping",
                "channelId": channel
            }
        );

        let resp: String = get_pytauri_ipc_response(&webview, "command".into(), &body);
        assert_eq!(resp, "pong");

        Ok(())
    })?;
    Ok(())
}
