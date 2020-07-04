use crate::num_traits::{FromPrimitive, ToPrimitive};
use pyo3::{FromPyObject, PyAny, PyObject, PyResult, Python, ToPyObject};
use serde::{Deserialize, Serialize};

/// Attributes Enum
#[derive(Primitive, Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub enum Attribute {
    NULL = 0,
    LIGHT = 1,
    ARMORED = 2,
    BIOLOGICAL = 3,
    MECHANICAL = 4,
    ROBOTIC = 5,
    PSIONIC = 6,
    MASSIVE = 7,
    STRUCTURE = 8,
    HOVER = 9,
    HEROIC = 10,
    SUMMONED = 11,
}

impl Default for Attribute {
    fn default() -> Self {
        Attribute::NULL
    }
}

impl ToPyObject for Attribute {
    fn to_object(&self, py: Python) -> PyObject {
        self.to_i32().unwrap().to_object(py)
    }
}

impl<'source> FromPyObject<'source> for Attribute {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        let ob1: i32 = ob.extract::<i32>().unwrap();
        let x: Attribute = Attribute::from_i32(ob1).unwrap_or_default();
        Ok(x)
    }
}
