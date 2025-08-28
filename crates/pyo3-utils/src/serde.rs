use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{PyBytes, PyString},
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub use pythonize;
pub use serde;
pub use serde_json;

struct SerdeJsonError(serde_json::Error);

impl From<serde_json::Error> for SerdeJsonError {
    fn from(e: serde_json::Error) -> Self {
        Self(e)
    }
}

impl From<SerdeJsonError> for PyErr {
    fn from(e: SerdeJsonError) -> Self {
        PyValueError::new_err(e.0.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PySerde<T>(T);

impl<T> PySerde<T> {
    pub fn new(de: T) -> Self {
        Self(de)
    }

    pub fn into_inner(self) -> T {
        self.0
    }

    pub fn as_ref(&self) -> PySerde<&T> {
        PySerde(&self.0)
    }

    pub fn as_mut(&mut self) -> PySerde<&mut T> {
        PySerde(&mut self.0)
    }
}

impl<'de, T> PySerde<T>
where
    T: Deserialize<'de>,
{
    pub fn from_json_str<'py>(ob: &'de Bound<'py, PyString>) -> PyResult<Self> {
        let de = serde_json::from_str(ob.to_str()?).map_err(SerdeJsonError::from)?;
        Ok(Self(de))
    }

    pub fn from_json_bytes<'py>(ob: &'de Bound<'py, PyBytes>) -> PyResult<Self> {
        let de = serde_json::from_slice(ob.as_bytes()).map_err(SerdeJsonError::from)?;
        Ok(Self(de))
    }

    pub fn from_object<'py>(ob: &'de Bound<'py, PyAny>) -> PyResult<Self> {
        let de = pythonize::depythonize(ob)?;
        Ok(Self(de))
    }

    pub fn extract<'py>(ob: &'de Bound<'py, PyAny>) -> PyResult<Self> {
        if let Ok(v) = ob.downcast::<PyBytes>() {
            Self::from_json_bytes(v)
        } else if let Ok(v) = ob.downcast::<PyString>() {
            Self::from_json_str(v)
        } else {
            Self::from_object(ob)
        }
    }
}

impl<'py, T> FromPyObject<'py> for PySerde<T>
where
    T: DeserializeOwned,
{
    /// TODO: We have to use [DeserializeOwned] because in `pyo3 v0.25` it cannot borrow data from the object.
    /// We need to wait for [pyo3::conversion::FromPyObjectBound].
    /// See: <https://github.com/PyO3/pyo3/pull/4390>.
    ///
    /// Use [PySerde::extract] as a workaround for now.
    #[inline]
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        Self::extract(ob)
    }
}

/// Benchmark(less is better):
/// - [Self::to_object] : 0.9
/// - [Self::to_json_str] : 0.6
/// - json.loads([Self::to_json_str]): 1.2
impl<T> PySerde<T>
where
    T: Serialize,
{
    pub fn to_json_str<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyString>> {
        let val = serde_json::to_string(&self.0).map_err(SerdeJsonError::from)?;
        Ok(PyString::new(py, &val))
    }

    pub fn to_json_bytes<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        let val = serde_json::to_vec(&self.0).map_err(SerdeJsonError::from)?;
        Ok(PyBytes::new(py, &val))
    }

    pub fn to_object<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let val = pythonize::pythonize(py, &self.0)?;
        Ok(val)
    }
}

impl<'py, T> IntoPyObject<'py> for &PySerde<T>
where
    T: Serialize,
{
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        self.to_object(py)
    }
}

impl<'py, T> IntoPyObject<'py> for PySerde<T>
where
    T: Serialize,
{
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (&self).into_pyobject(py)
    }
}
