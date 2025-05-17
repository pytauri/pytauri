use pyo3::{prelude::*, types::PyTuple};

/// see also: [tauri::Rect]
#[pyclass(frozen)]
pub struct Rect {
    // use `Py<T>` to avoid creating new obj every time visiting the field,
    // see: <https://pyo3.rs/v0.23.4/faq.html#pyo3get-clones-my-field>
    #[pyo3(get)]
    pub position: Py<Position>,
    #[pyo3(get)]
    pub size: Py<Size>,
}

impl Rect {
    #[expect(dead_code)] // TODO
    pub(crate) fn to_tauri(&self) -> tauri::Rect {
        tauri::Rect {
            position: (*self.position.get()).into(),
            size: (*self.size.get()).into(),
        }
    }

    pub(crate) fn from_tauri(py: Python<'_>, rect: tauri::Rect) -> PyResult<Self> {
        Ok(Self {
            position: Position::from(rect.position).into_pyobject(py)?.unbind(),
            size: Size::from(rect.size).into_pyobject(py)?.unbind(),
        })
    }
}

#[pymethods]
impl Rect {
    #[new]
    #[pyo3(signature = (*, position, size))]
    fn __new__(position: Py<Position>, size: Py<Size>) -> Self {
        Self { position, size }
    }
}

/// see also: [tauri::Position]
#[derive(Clone, Copy)]
#[pyclass(frozen)]
pub enum Position {
    /// `x, y`
    Physical(i32, i32),
    /// `x, y`
    Logical(f64, f64),
}

impl From<Position> for tauri::Position {
    fn from(val: Position) -> Self {
        match val {
            Position::Physical(x, y) => tauri::PhysicalPosition::new(x, y).into(),
            Position::Logical(x, y) => tauri::LogicalPosition::new(x, y).into(),
        }
    }
}

impl From<tauri::Position> for Position {
    fn from(val: tauri::Position) -> Self {
        match val {
            tauri::Position::Physical(tauri::PhysicalPosition { x, y }) => Position::Physical(x, y),
            tauri::Position::Logical(tauri::LogicalPosition { x, y }) => Position::Logical(x, y),
        }
    }
}

/// see also: [tauri::Size]
#[derive(Clone, Copy)]
#[pyclass(frozen)]
pub enum Size {
    /// `width, height`
    Physical(u32, u32),
    /// `width, height`
    Logical(f64, f64),
}

impl From<Size> for tauri::Size {
    fn from(val: Size) -> Self {
        match val {
            Size::Physical(width, height) => tauri::PhysicalSize::new(width, height).into(),
            Size::Logical(width, height) => tauri::LogicalSize::new(width, height).into(),
        }
    }
}

impl From<tauri::Size> for Size {
    fn from(val: tauri::Size) -> Self {
        match val {
            tauri::Size::Physical(tauri::PhysicalSize { width, height }) => {
                Size::Physical(width, height)
            }
            tauri::Size::Logical(tauri::LogicalSize { width, height }) => {
                Size::Logical(width, height)
            }
        }
    }
}

macro_rules! physical_position {
    ($vis:vis, $name:ident, $ty:ty) => {
        /// See also: [tauri::PhysicalPosition]
        ///
        /// `(x, y)`
        #[derive(FromPyObject, IntoPyObject, IntoPyObjectRef)]
        #[pyo3(transparent)]
        $vis struct $name($vis Py<PyTuple>);

        impl $name {
            $vis fn from_tauri(
                py: Python<'_>,
                position: tauri::PhysicalPosition<$ty>,
            ) -> PyResult<Self> {
                let x_y: ($ty, $ty) = (position.x, position.y); // typing assertion
                Ok(Self(x_y.into_pyobject(py)?.unbind()))
            }
        }
    };
}

physical_position!(pub(crate), PhysicalPositionF64, f64);
