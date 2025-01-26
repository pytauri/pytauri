//! When you want to expose an **state machine** enum to Python,
//! you should implement the trait in this module.
//!
//! # Tips
//!
//! In most cases, `enum` is just like Python's `Union` type, rather than a state machine.
//! For such cases, you can directly return the matched `FooEnum`,
//! without creating a newtype `struct Foo(third_party::Foo)`.
//!
//! # Example:
/*!
```rust
use pyo3::prelude::*;
use pyo3_utils::py_match::PyMatchRef;

mod third_party {
    pub enum Foo {
        A { a: i32 },
    }
}

#[pyclass(frozen)]
#[non_exhaustive]
enum FooEnum {
    A { a: i32 },
}

#[pyclass(frozen)]
#[non_exhaustive]
struct Foo(third_party::Foo);

impl PyMatchRef for Foo {
    type Output = FooEnum;

    fn match_ref(&self) -> Self::Output {
        match &self.0 {
            third_party::Foo::A { a } => FooEnum::A { a: *a },
        }
    }
}

// In the future, we might provide a macro to automatically generate this pymethod,
// for now, please do it manually.
#[pymethods]
impl Foo {
    fn match_ref(&self) -> <Self as PyMatchRef>::Output {
        <Self as PyMatchRef>::match_ref(self)
    }
}
```
*/

pub trait PyMatchRef {
    type Output;

    fn match_ref(&self) -> Self::Output;
}

pub trait PyMatchMut {
    type Output;

    fn match_mut(&mut self) -> Self::Output;
}

/// It is recommended to implement this trait only when using `clone` in [PyMatchRef]/[PyMatchMut]
/// would significantly impact memory/performance.
pub trait PyMatchInto {
    type Output;

    fn match_into(self) -> Self::Output;
}
