use pyo3::{prelude::*, types::PyBytes};
use tauri::image;

/// See also: [tauri::image::Image]
#[pyclass(frozen, subclass)] // subclass for `pillow`
#[non_exhaustive]
pub struct Image {
    // PERF: maybe we can use `memoryview` or `buffer protocol`.
    rgba: Py<PyBytes>,
    width: u32,
    height: u32,
}

impl Image {
    pub(crate) fn to_tauri<'a>(&'a self, py: Python<'_>) -> image::Image<'a> {
        image::Image::new(self.rgba.as_bytes(py), self.width, self.height)
    }

    pub(crate) fn from_tauri(py: Python<'_>, image: &image::Image) -> Self {
        Self {
            rgba: PyBytes::new(py, image.rgba()).unbind(),
            width: image.width(),
            height: image.height(),
        }
    }
}

#[pymethods]
impl Image {
    #[new]
    const fn __new__(rgba: Py<PyBytes>, width: u32, height: u32) -> Self {
        Self {
            rgba,
            width,
            height,
        }
    }

    const fn rgba(&self) -> &Py<PyBytes> {
        &self.rgba
    }

    const fn width(&self) -> u32 {
        self.width
    }

    const fn height(&self) -> u32 {
        self.height
    }
}
