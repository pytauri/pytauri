use std::{
    borrow::Cow,
    convert::Infallible,
    ops::{Deref, DerefMut},
};

use pyo3::{exceptions::PyValueError, prelude::*, types::PyString, FromPyObject, IntoPyObject};

type TauriUrl = tauri::Url;

/// See also: [tauri::Url]
pub struct Url(TauriUrl);

impl Deref for Url {
    type Target = TauriUrl;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Url {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<TauriUrl> for Url {
    fn from(url: TauriUrl) -> Self {
        Self(url)
    }
}

impl From<Url> for TauriUrl {
    fn from(url: Url) -> Self {
        url.0
    }
}

impl<'py> FromPyObject<'py> for Url {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let url: Cow<'_, str> = ob.extract()?; // TODO, PERF: once we drop py39, we can use `&str` directly
        let url = TauriUrl::parse(&url).map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(Self(url))
    }
}

impl<'py> IntoPyObject<'py> for &Url {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let url = PyString::new(py, self.0.as_str());
        Ok(url)
    }
}

impl<'py> IntoPyObject<'py> for Url {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (&self).into_pyobject(py)
    }
}
