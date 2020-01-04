use pyo3::prelude::*;
//use dict_derive::{FromPyObject, IntoPyObject};
use crate::num_traits::FromPrimitive;
use crate::generated_enums::{UnitTypeId};
//use pyo3::types::PyAny;
//use crate::game_info::{UnitInfo};
//use pyo3::types::PyAny;

#[pyclass]
#[derive(Clone)]
pub struct CombatUnit {
    #[pyo3(get,set)]
    pub owner: i64,
    #[pyo3(get,set)]
    pub unit_type: UnitTypeId,
    #[pyo3(get,set)]
    pub health: f64,
    #[pyo3(get,set)]
    pub health_max: f64,
    #[pyo3(get,set)]
    pub shield: f64,
    #[pyo3(get,set)]
    pub shield_max: f64,
    #[pyo3(get,set)]
    pub energy: f64,
    #[pyo3(get,set)]
    pub is_flying: bool,
    #[pyo3(get,set)]
    pub buff_timer: f64,

}

#[pymethods]
impl CombatUnit{
    fn dup(&self) -> Self {
        CombatUnit{
            owner: self.owner, 
            unit_type: self.unit_type,
            health: self.health,
            health_max: self.health_max,
            shield: self.shield,
            shield_max: self.shield_max,
            energy: self.energy,
            is_flying: self.is_flying,
            buff_timer: self.buff_timer,
        }
    }
//    #[args(_owner, _unit_type, _health, _health_max=0.0, _shield, _shield_max=0.0, _energy=0.0, _flying, _buff_timer=0.0)]
    #[new]
     fn new(obj: &PyRawObject,
            _owner: i64,
            _unit_type: i32,
            _health: f64,
            mut _health_max: Option<f64>,
            _shield:f64,
            mut _shield_max:Option<f64>,
            mut _energy:Option<f64>,
            _flying:bool,
            mut _buff_timer:Option<f64>,
        ){
        obj.init(CombatUnit{
             owner: _owner,
             unit_type: UnitTypeId::from_i32(_unit_type).unwrap_or_default(),
             health: _health,
             is_flying: _flying,
             buff_timer:_buff_timer.get_or_insert(0.0).to_owned(),
             energy:_energy.get_or_insert(0.0).to_owned(),
             health_max: _health_max.get_or_insert(_health).to_owned(),
             shield_max: _shield_max.get_or_insert(_shield).to_owned(),
             shield:_shield,
        })
     }
    fn show_unit_type(&self)-> PyResult<String> {
        Ok(self.unit_type.to_string())
    }

}
impl CombatUnit{
    fn modify_health(&mut self, mut delta: f64){
        if delta < 0.0 {
            delta = -delta;
            self.shield -= delta;
            if self.shield < 0.0{
                delta = -self.shield;
                self.shield = 0.0;
                self.health += -delta;
                if self.health < 0.0{
                    self.health = 0.0;
                }
            }
        }
        else{
            self.health += delta;
            if self.health > self.health_max{
                self.health = self.health_max;
            }

        }
    }
}
//
//impl ToPyObject for CombatUnit{
//	fn to_object(&self, py: Python) -> PyObject {
//		self.to_object(py)
//	}
//}
//
//impl FromPy<CombatUnit> for PyObject {
//    fn from_py(other: CombatUnit, py: Python) -> Self {
//        let _other: CombatUnit = other;
//        _other.into_py(py)
//    }
//}
//#[derive(Clone)]
//pub struct CombatUnits(Vec<CombatUnit>);
//impl<'source> FromPyObject<'source> for CombatUnits {
//    fn extract(ob: &'source PyAny) -> PyResult<Self> {
//    }
//}
//impl IntoIterator for CombatUnits {
//    type Item = CombatUnit;
//    type IntoIter = std::vec::IntoIter<Self::Item>;
//
//    fn into_iter(self) -> Self::IntoIter {
//        self.0.into_iter()
//    }
//}

pub fn clone_vec(vec: Vec<&CombatUnit>) -> Vec<CombatUnit> {
    vec.into_iter().map(|f| f.dup()).collect()
    }

//#[pyclass]
//#[derive(Clone)]
//// #[derive(FromPyObject, IntoPyObject)]
//pub struct CombatUnits{
//    pub units: Vec<CombatUnit>
//}
//
//#[pymethods]
//impl CombatUnits{
//    #[new]
//    fn new(obj: &PyRawObject, _units1:  Vec<&CombatUnit>){
//        let new_vec: Vec<CombatUnit> = clone_vec(_units1);
//        obj.init(CombatUnits{units: new_vec})
//
//    }
//    fn len(&self)-> PyResult<usize>{
//       Ok(self.units.len())
//    }
//
//    fn clear(&mut self){
//        self.units = Vec::<CombatUnit>::new()
//    }
//    #[getter]
//    fn get_units(&mut self)->PyResult<Vec<CombatUnit>>{
//        Ok(self.units.clone())
//    }
//}