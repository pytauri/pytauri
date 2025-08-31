use std::{borrow::Cow, convert::Infallible, path::PathBuf};

use pyo3::{exceptions::PyValueError, prelude::*, types::PyString, FromPyObject, IntoPyObject};

use crate::utils::non_exhaustive_panic;

type TauriUrl = tauri::Url;

/// See also: [tauri::Url]
pub struct Url<'a>(pub Cow<'a, TauriUrl>);

impl From<TauriUrl> for Url<'_> {
    fn from(url: TauriUrl) -> Self {
        Self(Cow::Owned(url))
    }
}
impl<'a> From<&'a TauriUrl> for Url<'a> {
    fn from(url: &'a TauriUrl) -> Self {
        Self(Cow::Borrowed(url))
    }
}

impl From<Url<'_>> for TauriUrl {
    fn from(url: Url<'_>) -> Self {
        url.0.into_owned()
    }
}

impl AsRef<TauriUrl> for Url<'_> {
    fn as_ref(&self) -> &TauriUrl {
        &self.0
    }
}

impl<'py> FromPyObject<'py> for Url<'_> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        // TODO, PERF: once we drop py39, we can use `&str` directly
        let url: Cow<'_, str> = ob.extract()?;
        // TODO: unify this error type
        let url = TauriUrl::parse(&url).map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(Self::from(url))
    }
}

impl<'py> IntoPyObject<'py> for &Url<'_> {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let url = PyString::new(py, self.0.as_str());
        Ok(url)
    }
}

impl<'py> IntoPyObject<'py> for Url<'_> {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (&self).into_pyobject(py)
    }
}

/// See also: [tauri::WebviewUrl]
#[pyclass(frozen)]
#[non_exhaustive]
pub enum WebviewUrl {
    External(Url<'static>),
    App(PathBuf),
    CustomProtocol(Url<'static>),
    _NonExhaustive(),
}

impl WebviewUrl {
    #[expect(dead_code)] // TODO
    pub(crate) fn from_tauri(value: tauri::WebviewUrl) -> Self {
        match value {
            tauri::WebviewUrl::External(url) => Self::External(url.into()),
            tauri::WebviewUrl::App(path) => Self::App(path),
            tauri::WebviewUrl::CustomProtocol(url) => Self::CustomProtocol(url.into()),
            _ => Self::_NonExhaustive(),
        }
    }

    pub(crate) fn to_tauri(&self) -> PyResult<tauri::WebviewUrl> {
        // TODO, XXX, FIXME: avoid clone
        let value = match self {
            Self::External(url) => tauri::WebviewUrl::External(url.0.clone().into_owned()),
            Self::App(path) => tauri::WebviewUrl::App(path.clone()),
            Self::CustomProtocol(url) => {
                tauri::WebviewUrl::CustomProtocol(url.0.clone().into_owned())
            }
            Self::_NonExhaustive() => non_exhaustive_panic(),
        };
        Ok(value)
    }
}
