base_data = [
    "use crate::num_traits::{FromPrimitive, ToPrimitive};",
    "use pyo3::{FromPyObject, PyResult, ObjectProtocol, PyObject, ToPyObject, Python, FromPy, IntoPy};",
    "use pyo3::types::{PyAny};",
    "use pyo3::derive_utils::IntoPyResult;",
    "use std::fmt;"
    "use sc2_techtree::{UnitTypeId as UTI, AbilityId as AI, UpgradeId as UI };"
    ]

macros = [
    "#[allow(missing_docs)]",
    "#[derive(Primitive,Debug, Eq, PartialEq, Copy, Clone, Hash)]",
    "#[allow(non_camel_case_types)]",
    "#[allow(dead_code)]"
    ]

def implementations(enum_name):
    tech_tree_mapping = {
        "UnitTypeId":"UTI",
        "AbilityId": "AI",
        "UpgradeId":"UI"
        }
    default_mapping = {
        "UnitTypeId": "UnitTypeId::NOTAUNIT",
        "AbilityId":"AbilityId::NULL_NULL",
        "EffectId":"EffectId::NULL",
        "UpgradeId":"UpgradeId::NULL",
        "BuffId":"BuffId::NULL"
        }
    get_tech_tree = tech_tree_mapping.get(enum_name)
    if get_tech_tree:
        tech_tree_impl = f"impl {enum_name}{{\n\tpub fn to_tt(self)-> {get_tech_tree}{{\n\t\t{get_tech_tree}::new(self.to_u32().unwrap())\n\t}}\n}}"
    else:
        tech_tree_impl = "\n\n"
    impl = [
        f"impl fmt::Display for {enum_name} {{\n\tfn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {{\n\t\twrite!(f, \"{{:?}}\", self)\n\t}}\n}}",
        f"impl Default for {enum_name} {{\n\tfn default() -> Self {{\n\t\t{default_mapping[enum_name]}\n\t}}\n}}",
        f"impl ToPyObject for {enum_name}{{\n\tfn to_object(&self, py: Python) -> PyObject {{\n\t\tself.to_i32().unwrap().to_object(py)\n\t}}\n}}",
        f"impl FromPy<{enum_name}> for PyObject {{\n\tfn from_py(other: {enum_name}, py: Python) -> Self {{\n\t\tlet _other: i32 = other.to_i32().unwrap();\n\t\t_other.into_py(py)\n\t}}\n}}",
        f"impl<'source> FromPyObject<'source> for {enum_name}{{\n\tfn extract(ob: &'source PyAny)-> PyResult<{enum_name}>{{\n\t\tlet ob1: i32 = ob.extract::<i32>().unwrap();\n\t\tlet x : {enum_name}={enum_name}::from_i32(ob1).unwrap_or_default();\n\t\tOk(x).into_py_result()\n\t}}\n}}",
        tech_tree_impl
    ]
    return impl


def test():
    print(implementations("UnitTypeId")[0])

if __name__ == "__main__":
    test()