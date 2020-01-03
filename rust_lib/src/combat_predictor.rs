use pyo3::prelude::*;
use crate::combat_unit::{CombatUnits, CombatUnit};

#[pyclass]
pub struct CombatPredictor{
    #[pyo3(get)]
    units1: CombatUnits,
    #[pyo3(get)]
    units2: CombatUnits ,
    // data: Vec<UnitTypeData>
}

#[pymethods]
impl CombatPredictor{
    #[new]
    fn new(obj: &PyRawObject, mut _units1: &mut CombatUnits, mut _units2: &mut CombatUnits){
        let mem_units1: CombatUnits = std::mem::replace(&mut _units1, CombatUnits{units: Vec::<CombatUnit>::new()});
        let mem_units2: CombatUnits = std::mem::replace(&mut _units2, CombatUnits{units: Vec::<CombatUnit>::new()});
        obj.init(CombatPredictor{
           units1: mem_units1,
           units2: mem_units2
            })
     }

//    #[getter]
//    fn get_units1(&self) -> PyResult<CombatUnits>{
//        Ok(self.units1.clone())
//    }
//    #[getter]
//    fn get_units2(&self) -> PyResult<CombatUnits>{
//        Ok(self.units2.clone())
//    }
    // #[setter]
    // fn set_units1(&mut self, value: Vec<CombatUnit>){
    //     self.units1 = value;
    //     Ok(())
    // }
    // }
    fn predict_engage(&mut self)->PyResult<u32>{
        let mut total_health1: f64 = 0.0;
        let mut total_health2: f64 = 0.0;

        for x in self.units1.units.iter(){
            total_health1 += x.health;
        }
        for y in self.units2.units.iter(){
            total_health2 += y.health;
            }
        if total_health1 > total_health2{
            Ok(1)
        }
        else{
            Ok(2)
        }
    }
// #[pyfunction]
// fn predict_engage(units1: PyList, units2:PyList)->PyResult<i64>{
//     Ok(1*2)
 }