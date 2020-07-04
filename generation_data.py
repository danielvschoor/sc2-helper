base_data = [
    "use crate::num_traits::{FromPrimitive, ToPrimitive};",
    "use pyo3::{FromPyObject, PyResult, PyObject, ToPyObject, Python, FromPy, IntoPy};",
    "use pyo3::types::{PyAny};",
    "use std::fmt;",
    "use serde::{Deserialize, Serialize};"

]

macros = [
    "#[allow(missing_docs)]",
    "#[derive(Primitive,Debug, Eq, PartialEq, Copy, Clone, Hash,Serialize, Deserialize)]",
    "#[allow(non_camel_case_types)]",
    "#[allow(dead_code)]"
]


def implementations(enum_name):
    default_mapping = {
        "UnitTypeId": "UnitTypeId::NOTAUNIT",
        "AbilityId": "AbilityId::NULL_NULL",
        "EffectId": "EffectId::NULL",
        "UpgradeId": "UpgradeId::NULL",
        "BuffId": "BuffId::NULL"
    }
    impl = [
        f"impl fmt::Display for {enum_name} {{\n\tfn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {{"
        f"\n\t\twrite!(f, \"{{:?}}\", self)\n\t}}\n}}",
        f"impl Default for {enum_name} {{\n\tfn default() -> Self {{\n\t\t{default_mapping[enum_name]}\n\t}}\n}}",
        f"impl ToPyObject for {enum_name}{{\n\tfn to_object(&self, py: Python) -> PyObject {{\n\t\tself.to_i32("
        f").unwrap().to_object(py)\n\t}}\n}}",
        f"impl FromPy<{enum_name}> for PyObject {{\n\tfn from_py(other: {enum_name}, py: Python) -> Self {{\n\t\tlet "
        f"_other: i32 = other.to_i32().unwrap();\n\t\t_other.into_py(py)\n\t}}\n}}",
        f"impl<'source> FromPyObject<'source> for {enum_name}{{\n\tfn extract(ob: &'source PyAny)-> "
        f"PyResult<{enum_name}>{{\n\t\tlet ob1: i32 = ob.getattr(\"value\")?.extract()?;\n\t\tlet x : "
        f"{enum_name}={enum_name}::from_i32(ob1).unwrap_or_default();\n\t\tOk(x)\n\t}}\n}}"
    ]
    return impl


def test():
    print(implementations("UnitTypeId")[0])


if __name__ == "__main__":
    test()
