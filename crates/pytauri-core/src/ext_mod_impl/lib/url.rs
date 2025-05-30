use std::{borrow::Cow, convert::Infallible};

use pyo3::{exceptions::PyValueError, prelude::*, types::PyString, FromPyObject, IntoPyObject};

type TauriUrl = tauri::Url;

/// See also: [tauri::Url]
pub struct Url<'a>(Cow<'a, TauriUrl>);

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

impl<'py> FromPyObject<'py> for Url<'_> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let url: Cow<'_, str> = ob.extract()?; // TODO, PERF: once we drop py39, we can use `&str` directly
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
