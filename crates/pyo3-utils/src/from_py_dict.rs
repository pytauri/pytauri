//! See: <https://github.com/PyO3/pyo3/issues/5163>

use pyo3::{
    conversion::{FromPyObjectBound, IntoPyObjectExt as _},
    exceptions::PyTypeError,
    prelude::*,
    types::{PyDict, PyString},
};

/// Inspired by [`typing.NotRequired`](https://docs.python.org/3/library/typing.html#typing.NotRequired)
///
/// See also: [derive_from_py_dict].
#[derive(Debug, Clone)]
pub struct NotRequired<T>(pub Option<T>);

// DO NOT use `#[derive(Default)]`, it requires `T: Default`.
impl<T> Default for NotRequired<T> {
    fn default() -> Self {
        NotRequired(None)
    }
}

impl<'py, T> FromPyObject<'py> for NotRequired<T>
where
    for<'a, 'py_a> T: FromPyObjectBound<'a, 'py_a>,
{
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let value = ob.extract::<T>()?;
        Ok(NotRequired(Some(value)))
    }
}

fn not_required_into_pyobject_err(py: Python<'_>) -> PyErr {
    const NOT_REQUIRED_INTO_PYOBJECT_ERR: &str =
        "`NotRequired` value does not exist, cannot convert to PyObject";

    PyTypeError::new_err(
        pyo3::intern!(py, NOT_REQUIRED_INTO_PYOBJECT_ERR)
            .clone()
            .unbind(),
    )
}

impl<'py, T> IntoPyObject<'py> for NotRequired<T>
where
    T: IntoPyObject<'py>,
{
    type Output = <T as IntoPyObject<'py>>::Output;
    type Target = <T as IntoPyObject<'py>>::Target;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        match self.0 {
            Some(value) => value.into_pyobject_or_pyerr(py),
            None => Err(not_required_into_pyobject_err(py)),
        }
    }
}

impl<'a, 'py, T> IntoPyObject<'py> for &'a NotRequired<T>
where
    &'a T: IntoPyObject<'py>,
{
    type Output = <&'a T as IntoPyObject<'py>>::Output;
    type Target = <&'a T as IntoPyObject<'py>>::Target;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        match &self.0 {
            Some(value) => value.into_pyobject_or_pyerr(py),
            None => Err(not_required_into_pyobject_err(py)),
        }
    }
}

// TODO: once <https://github.com/PyO3/pyo3/issues/5163> is resolved, we can deprecate this trait.
/// See also: [derive_from_py_dict]
pub trait FromPyDict: Sized {
    fn from_py_dict(dict: &Bound<'_, PyDict>) -> PyResult<Self>;
}

#[doc(hidden)]
pub fn __get_item_with_default<T>(
    dict: &Bound<'_, PyDict>,
    key: &Bound<'_, PyString>,
) -> PyResult<T>
where
    for<'a, 'py> T: FromPyObjectBound<'a, 'py> + Default,
{
    let value = match dict.get_item(key)? {
        Some(value) => value.extract::<T>()?,
        None => Default::default(),
    };
    Ok(value)
}

#[doc(hidden)]
pub fn __get_item<T>(dict: &Bound<'_, PyDict>, key: &Bound<'_, PyString>) -> PyResult<T>
where
    for<'a, 'py> T: FromPyObjectBound<'a, 'py>,
{
    let value = dict.as_any().get_item(key)?.extract::<T>()?;
    Ok(value)
}

// ref: <https://github.com/PyO3/pyo3/blob/3914daff760fc23aae4602378b4c010332baa920/src/impl_/frompyobject.rs#L82-L93>
#[doc(hidden)]
pub fn __failed_to_extract_struct_field<T>(
    py: Python<'_>,
    result: PyResult<T>,
    struct_name: &'static str,
    field_name: &'static str,
) -> PyResult<T> {
    result.map_err(|err| {
        let new_err = PyTypeError::new_err(format!(
            "failed to extract field {struct_name}.{field_name}"
        ));
        new_err.set_cause(py, Some(err));
        new_err
    })
}

/// Derives the [FromPyDict] trait for a struct.
///
/// > Why we need this trait?
/// >
/// > ref: <https://github.com/PyO3/pyo3/issues/5163>
///
/// # Example:
/**
```rust
use pyo3_utils::from_py_dict::{derive_from_py_dict, FromPyDict as _, NotRequired};
use pyo3::{
    prelude::*,
    types::{IntoPyDict as _, PyDict},
};

fn main() -> PyResult<()> {
    pub struct Foo {
        a: i32,
        b: NotRequired<i32>,
        c: NotRequired<Option<i32>>,
    }

    derive_from_py_dict!(Foo {
        a,
        #[default]
        b,
        #[default]
        c,
    });

    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        // optional default `b`
        let dict_0 = [("a", 1)].into_py_dict(py)?;
        let foo_0 = Foo::from_py_dict(&dict_0)?;
        assert_eq!(foo_0.a, 1);
        assert_eq!(foo_0.b.0, None);

        // missing required field `a`
        let dict_1 = [("b", 2)].into_py_dict(py)?;
        assert!(Foo::from_py_dict(&dict_1).is_err());

        // provide a value for the optional field `b`
        let dict_2 = [("a", 1), ("b", 2)].into_py_dict(py)?;
        let foo_2 = Foo::from_py_dict(&dict_2)?;
        assert_eq!(foo_2.a, 1);
        assert_eq!(foo_2.b.0, Some(2));

        // provide a value for the optional field `c: NotRequired[Optional[int]]`
        let dict_3 = [("a", 1), ("c", 2)].into_py_dict(py)?;
        let foo_3 = Foo::from_py_dict(&dict_3)?;
        assert_eq!(foo_3.c.0, Some(Some(2)));

        // provide `None` for the optional field `c: NotRequired[Optional[int]]`
        let dict_4 = PyDict::new(py);
        dict_4.set_item("a", 1)?;
        dict_4.set_item("c", py.None())?;
        let foo_4 = Foo::from_py_dict(&dict_4)?;
        assert_eq!(foo_4.c.0, Some(None));

        Ok(())
    })
}
```
*/
#[macro_export]
macro_rules! __derive_from_py_dict {
    ($dict:expr, $key:expr, #) => {
        $crate::from_py_dict::__get_item($dict, $key)
    };
    ($dict:expr, $key:expr, #default) => {
        $crate::from_py_dict::__get_item_with_default($dict, $key)
    };
    ($dict:expr, $key:expr, #$attribute:ident) => {
        compile_error!(concat!(
            "Invalid attribute: #[",
            stringify!($attribute),
            "]. Only accepted optional `#[default]` attribute."
        ))
    };

    (
        $name:ty {
            $($( #[$meta:ident] )? $field:ident, )*
        }
    ) => {
        impl $crate::from_py_dict::FromPyDict for $name {
            fn from_py_dict(dict: &::pyo3::Bound<'_, ::pyo3::types::PyDict>) -> ::pyo3::PyResult<Self> {
                use $name as __name;
                Ok(__name {
                    $(
                        $field: $crate::from_py_dict::__failed_to_extract_struct_field(
                            dict.py(),
                            {
                                let key = ::pyo3::intern!(dict.py(), stringify!($field));
                                $crate::from_py_dict::derive_from_py_dict!(dict, key, #$($meta)?)
                            },
                            stringify!($name),
                            stringify!($field),
                        )?,
                    )*
                })
            }
        }
    };
}

pub use __derive_from_py_dict as derive_from_py_dict;
