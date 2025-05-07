use std::{borrow::Cow, iter::Iterator};

use pyo3::{
    intern,
    prelude::*,
    types::{PyBytes, PyIterator, PyString},
};
use tauri::{
    utils::assets::{AssetKey as TauriAssetKey, AssetsIter, CspHash},
    Assets,
};

use crate::{
    ext_mod::{PyAppHandleExt as _, TauriApp},
    tauri_runtime::Runtime,
    utils::PyResultExt as _,
};

/// see also: [tauri::utils::assets::AssetKey]
//
// TODO: export this type in [ext_mod_impl::utils::assets] namespace
type AssetKey = PyString;

/// The [Iterator] is only implemented for [Bound], so we manually implement it for [Py] here.
struct PyAssetsIter(Py<PyIterator>);

impl Iterator for PyAssetsIter {
    type Item = (String, Vec<u8>);

    fn next(&mut self) -> Option<Self::Item> {
        let item: Option<Self::Item> = Python::with_gil(|py| {
            let mut slf = self
                .0
                // TODO, PERF, XXX: we can't iterate on [pyo3::Borrowed], so we have to convert into [Bound],
                // this is pyo3 limitation, create a issue for it.
                .bind(py)
                .clone();
            let next_result = slf.next()?;
            let item_result = (|| {
                let item = next_result?;
                // TODO: support `PyByteArray` also, ref impl: <https://github.com/PyO3/pyo3/issues/2888#issuecomment-1398307069>.
                //
                // NOTE: DO NOT `extract::<Vec<u8>>` directly, use `Cow<[u8]>` instead,
                // see: <https://github.com/PyO3/pyo3/issues/2888>.
                let (key, bytes) = item.extract::<(Bound<'_, PyString>, Bound<'_, PyBytes>)>()?;
                // TODO, PERF: once we drop py39, we can use `&str` instead of `Cow`
                let key = key.to_cow()?;
                let bytes = bytes.as_bytes();
                // TODO, PERF: how to avoid copy?
                let item = (key.into_owned(), bytes.to_vec());
                PyResult::Ok(item)
            })();
            let item = item_result.unwrap_unraisable_py_result(py, Some(&slf), || {
                "Python exception occurred during calling `PyIterator.next()`"
            });
            Some(item)
        });
        item
    }
}

pub(crate) struct PyAssets(pub(crate) PyObject);

impl Assets<Runtime> for PyAssets {
    fn get(&self, key: &TauriAssetKey) -> Option<Cow<'_, [u8]>> {
        const METHOD_NAME: &str = "get";

        let result = Python::with_gil(|py| {
            let key: Bound<AssetKey> = AssetKey::new(py, key.as_ref()); // intern it?
            let slf = self.0.bind(py);

            let result = (|| {
                let ret = slf.call_method1(intern!(py, METHOD_NAME), (key,))?;
                if ret.is_none() {
                    return Ok(None);
                }
                let ret_py_bytes = ret.downcast_into::<PyBytes>()?.unbind();
                let ret_bytes = ret_py_bytes.as_bytes(py);

                // TODO, PERF: how to avoid copy?
                let ret_bytes: Cow<'_, [u8]> = Cow::Owned(ret_bytes.to_vec());
                PyResult::Ok(Some(ret_bytes))
            })();
            result.unwrap_unraisable_py_result(py, Some(slf), || {
                "Python exception occurred during calling `Assets.get()`"
            })
        });
        result
    }

    fn iter(&self) -> Box<AssetsIter<'_>> {
        const METHOD_NAME: &str = "iter";

        let assets_iter = Python::with_gil(|py| {
            let slf = self.0.bind(py);
            let result = (|| {
                let ret = slf.call_method0(intern!(py, METHOD_NAME))?;
                let ret_iter = ret.try_iter()?;
                let unbound_iter = PyAssetsIter(ret_iter.unbind());
                let assets_iter = unbound_iter.map(|item| {
                    let (key, bytes) = item;
                    (Cow::Owned(key), Cow::Owned(bytes))
                });
                PyResult::Ok(assets_iter)
            })();
            result.unwrap_unraisable_py_result(py, Some(slf), || {
                "Python exception occurred during calling `Assets.iter()`"
            })
        });
        Box::new(assets_iter)
    }

    fn csp_hashes(&self, _html_path: &TauriAssetKey) -> Box<dyn Iterator<Item = CspHash<'_>> + '_> {
        todo!("Blocked by: <https://github.com/tauri-apps/tauri/issues/12756>")
    }

    fn setup(&self, app: &TauriApp) {
        const METHOD_NAME: &str = "setup";

        let app_handle = app.py_app_handle();
        Python::with_gil(|py| {
            let slf = self.0.bind(py);
            let result = slf.call_method1(intern!(py, METHOD_NAME), (app_handle,));
            result.unwrap_unraisable_py_result(py, Some(slf), || {
                "Python exception occurred during calling `Assets.setup()`"
            });
        })
    }
}
