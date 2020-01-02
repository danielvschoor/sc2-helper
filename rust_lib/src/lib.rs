mod combat_predictor;
mod combat_unit;
use pyo3::prelude::*;

#[pymodule]
fn sc2_helper(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<combat_unit::CombatUnits>()?;
    m.add_class::<combat_predictor::CombatPredictor>()?;
    m.add_class::<combat_unit::CombatUnit>()?;
    Ok(())
}
