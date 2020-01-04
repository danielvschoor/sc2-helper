mod combat_predictor;
mod combat_unit;
mod generated_enums;
mod game_info;
use pyo3::prelude::*;

#[macro_use]
extern crate enum_primitive_derive;
extern crate num_traits;


#[pymodule]
fn sc2_helper(_py: Python, m: &PyModule) -> PyResult<()> {
//    m.add_class::<combat_unit::CombatUnits>()?;
    m.add_class::<combat_predictor::CombatPredictor>()?;
    m.add_class::<combat_unit::CombatUnit>()?;
    Ok(())
}
