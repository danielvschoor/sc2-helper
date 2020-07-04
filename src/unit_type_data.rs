use crate::enums::Attribute;
use pyo3::{FromPyObject, PyAny, PyResult, Python, ToPyObject};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub struct Cost {
    pub minerals: i32,
    pub vespene: i32,
    pub time: f32,
}

impl<'source> FromPyObject<'source> for Cost {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        //        println!("{:?}", "Extracting DamageBonus");
        unsafe {
            let py = Python::assume_gil_acquired();
            let obj = ob.to_object(py);
            Ok(Self {
                minerals: obj.getattr(py, "minerals")?.extract(py)?,
                vespene: obj.getattr(py, "vespene")?.extract(py)?,
                time: obj.getattr(py, "time")?.extract(py)?,
            })
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitTypeData {
    // pub id: UnitTypeId,
    // pub name: String,
    pub attributes: Vec<Attribute>,
    pub cost: Cost,
}
impl UnitTypeData {
    pub fn new(attributes: Vec<Attribute>, cost: Cost) -> Self {
        Self { attributes, cost }
    }
}
impl<'source> FromPyObject<'source> for UnitTypeData {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        //        println!("{:?}", "Extracting DamageBonus");
        unsafe {
            let py = Python::assume_gil_acquired();
            let obj = ob.to_object(py);
            Ok(Self {
                // id: obj.getattr(py, "id")?.extract(py)?,
                // name: obj.getattr(py, "name")?.extract(py)?,
                attributes: obj.getattr(py, "attributes")?.extract(py)?,
                cost: obj.getattr(py, "cost")?.extract(py)?,
            })
        }
    }
}
