use pyo3::{prelude::*, types::PyTuple};

/// See also: [tauri::Rect]
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
    pub(crate) fn from_tauri(py: Python<'_>, rect: tauri::Rect) -> PyResult<Self> {
        let position = Position::from_tauri(py, rect.position)?
            .into_pyobject(py)?
            .unbind();
        let size = Size::from_tauri(py, rect.size)?.into_pyobject(py)?.unbind();
        Ok(Self { position, size })
    }

    #[expect(dead_code)] // TODO
    pub(crate) fn to_tauri(&self, py: Python<'_>) -> PyResult<tauri::Rect> {
        let ret = tauri::Rect {
            position: self.position.get().to_tauri(py)?,
            size: self.size.get().to_tauri(py)?,
        };
        Ok(ret)
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

pub(crate) type TauriPhysicalRect = tauri::PhysicalRect<i32, u32>;

/// See also: [tauri::PhysicalRect]
#[pyclass(frozen)]
pub struct PhysicalRect {
    #[pyo3(get)]
    #[expect(private_interfaces)]
    pub position: PhysicalPositionI32,
    #[pyo3(get)]
    #[expect(private_interfaces)]
    pub size: PhysicalSizeU32,
}

impl PhysicalRect {
    pub(crate) fn from_tauri(py: Python<'_>, rect: TauriPhysicalRect) -> PyResult<Self> {
        let position = PhysicalPositionI32::from_tauri(py, rect.position)?;
        let size = PhysicalSizeU32::from_tauri(py, rect.size)?;
        Ok(Self { position, size })
    }

    #[expect(dead_code)] // TODO
    pub(crate) fn to_tauri(&self, py: Python<'_>) -> PyResult<TauriPhysicalRect> {
        let ret = TauriPhysicalRect {
            position: self.position.to_tauri(py)?,
            size: self.size.to_tauri(py)?,
        };
        Ok(ret)
    }
}

#[pymethods]
impl PhysicalRect {
    #[new]
    #[pyo3(signature = (*, position, size))]
    fn __new__(position: PhysicalPositionI32, size: PhysicalSizeU32) -> Self {
        Self { position, size }
    }
}

pub(crate) type TauriLogicalRect = tauri::LogicalRect<f64, f64>;

/// See also: [tauri::LogicalRect]
#[pyclass(frozen)]
pub struct LogicalRect {
    #[pyo3(get)]
    #[expect(private_interfaces)]
    pub position: LogicalPositionF64,
    #[pyo3(get)]
    #[expect(private_interfaces)]
    pub size: LogicalSizeF64,
}

impl LogicalRect {
    #[expect(dead_code)] // TODO
    pub(crate) fn from_tauri(py: Python<'_>, rect: TauriLogicalRect) -> PyResult<Self> {
        let position = LogicalPositionF64::from_tauri(py, rect.position)?;
        let size = LogicalSizeF64::from_tauri(py, rect.size)?;
        Ok(Self { position, size })
    }

    #[expect(dead_code)] // TODO
    pub(crate) fn to_tauri(&self, py: Python<'_>) -> PyResult<TauriLogicalRect> {
        let ret = TauriLogicalRect {
            position: self.position.to_tauri(py)?,
            size: self.size.to_tauri(py)?,
        };
        Ok(ret)
    }
}

#[pymethods]
impl LogicalRect {
    #[new]
    #[pyo3(signature = (*, position, size))]
    fn __new__(position: LogicalPositionF64, size: LogicalSizeF64) -> Self {
        Self { position, size }
    }
}

/// See also: [tauri::Position]
#[pyclass(frozen)]
pub enum Position {
    #[expect(private_interfaces)]
    Physical(PhysicalPositionI32),
    #[expect(private_interfaces)]
    Logical(LogicalPositionF64),
}

impl Position {
    pub(crate) fn from_tauri(py: Python<'_>, position: tauri::Position) -> PyResult<Self> {
        let ret = match position {
            tauri::Position::Physical(pos) => {
                Position::Physical(PhysicalPositionI32::from_tauri(py, pos)?)
            }
            tauri::Position::Logical(pos) => {
                Position::Logical(LogicalPositionF64::from_tauri(py, pos)?)
            }
        };
        Ok(ret)
    }

    pub(crate) fn to_tauri(&self, py: Python<'_>) -> PyResult<tauri::Position> {
        match self {
            Position::Physical(pos) => pos.to_tauri(py).map(tauri::Position::Physical),
            Position::Logical(pos) => pos.to_tauri(py).map(tauri::Position::Logical),
        }
    }
}

/// See also: [tauri::Size]
#[pyclass(frozen)]
pub enum Size {
    #[expect(private_interfaces)]
    Physical(PhysicalSizeU32),
    #[expect(private_interfaces)]
    Logical(LogicalSizeF64),
}

impl Size {
    pub(crate) fn from_tauri(py: Python<'_>, size: tauri::Size) -> PyResult<Self> {
        let ret = match size {
            tauri::Size::Physical(size) => Size::Physical(PhysicalSizeU32::from_tauri(py, size)?),
            tauri::Size::Logical(size) => Size::Logical(LogicalSizeF64::from_tauri(py, size)?),
        };
        Ok(ret)
    }

    pub(crate) fn to_tauri(&self, py: Python<'_>) -> PyResult<tauri::Size> {
        match self {
            Size::Physical(size) => size.to_tauri(py).map(tauri::Size::Physical),
            Size::Logical(size) => size.to_tauri(py).map(tauri::Size::Logical),
        }
    }
}

macro_rules! position {
    ($vis:vis, $name:ident, $ty:ty => $from_tauri:ident, $to_tauri:ident, $tauri_ty:ty) => {
        /// See also: [tauri::PhysicalPosition] and [tauri::LogicalPosition]
        ///
        /// `(x, y)`
        #[derive(FromPyObject, IntoPyObject, IntoPyObjectRef)]
        #[pyo3(transparent)]
        $vis struct $name($vis Py<PyTuple>);

        impl $name {
            #[allow(dead_code)]
            $vis fn $from_tauri(
                py: Python<'_>,
                value: $tauri_ty,
            ) -> PyResult<Self> {
                let x_y: ($ty, $ty) = (value.x, value.y); // typing assertion
                Ok(Self(x_y.into_pyobject(py)?.unbind()))
            }

            #[allow(dead_code)]
            $vis fn $to_tauri(
                &self,
                py: Python<'_>,
            ) -> PyResult<$tauri_ty> {
                let (x, y): ($ty, $ty) = self.0.extract(py)?;
                type TauriTy = $tauri_ty; // convert to expr `TauriTy`
                Ok(TauriTy { x, y })
            }
        }
    };
}

macro_rules! size {
    ($vis:vis, $name:ident, $ty:ty => $from_tauri:ident, $to_tauri:ident, $tauri_ty:ty) => {
        /// See also: [tauri::PhysicalSize] and [tauri::LogicalSize]
        ///
        /// `(width, height)`
        #[derive(FromPyObject, IntoPyObject, IntoPyObjectRef)]
        #[pyo3(transparent)]
        $vis struct $name($vis Py<PyTuple>);

        impl $name {
            #[allow(dead_code)]
            $vis fn $from_tauri(
                py: Python<'_>,
                value: $tauri_ty,
            ) -> PyResult<Self> {
                let width_height: ($ty, $ty) = (value.width, value.height); // typing assertion
                Ok(Self(width_height.into_pyobject(py)?.unbind()))
            }

            #[allow(dead_code)]
            $vis fn $to_tauri(
                &self,
                py: Python<'_>,
            ) -> PyResult<$tauri_ty> {
                let (width, height): ($ty, $ty) = self.0.extract(py)?;
                type TauriTy = $tauri_ty; // convert to expr `TauriTy`
                Ok(TauriTy { width, height })
            }
        }
    };
}

position!(pub(crate), PhysicalPositionF64, f64 => from_tauri, to_tauri, tauri::PhysicalPosition::<f64>);
position!(pub(crate), PhysicalPositionI32, i32 => from_tauri, to_tauri, tauri::PhysicalPosition::<i32>);
position!(pub(crate), LogicalPositionF64, f64 => from_tauri, to_tauri, tauri::LogicalPosition::<f64>);
size!(pub(crate), PhysicalSizeU32, u32 => from_tauri, to_tauri, tauri::PhysicalSize::<u32>);
size!(pub(crate), LogicalSizeF64, f64 => from_tauri, to_tauri, tauri::LogicalSize::<f64>);
