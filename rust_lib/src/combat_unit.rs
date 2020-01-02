use pyo3::prelude::*;

#[pyclass]
#[derive(Clone)]
pub struct CombatUnit{
    pub owner: i64,
    pub unit_type: i64,//todo
    pub health: f64,
    pub health_max: f64,
    pub shield: f64,
    pub shield_max: f64,
    pub energy: f64,
    pub is_flying: bool,
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
            buff_timer: self.buff_timer}
    }
    #[new]
    fn new(obj: &PyRawObject, _owner: i64, _unit_type: i64/*todo:*/, _health:f64, _flying: bool){
        obj.init(CombatUnit{
            owner: _owner, 
            unit_type: _unit_type, 
            health: _health, 
            is_flying: _flying, 
            buff_timer:0.0, 
            energy:0.0,
            health_max: _health,
            shield_max:0.0,
            shield:0.0
            })
    }
    
    #[getter]
    fn get_owner(&self) -> PyResult<i64>{
        Ok(self.owner)
    }
    
    #[setter]
    fn set_owner(&mut self, value: i64) -> PyResult<()>{
        self.owner = value;
        Ok(())
    }
    
    #[getter]
    fn get_unit_type(&self) -> PyResult<i64>{
        Ok(self.unit_type)
    }
    
    #[setter]
    fn set_unit_type(&mut self, value: i64) -> PyResult<()>{
        self.unit_type = value;
        Ok(())
    }

    #[getter]
    fn get_health(&self) -> PyResult<f64>{
        Ok(self.health)
    }
    
    #[setter]
    fn set_health(&mut self, value: f64) -> PyResult<()>{
        self.health = value;
        Ok(())
    }
    
    #[getter]
    fn get_health_max(&self) -> PyResult<f64>{
        Ok(self.health_max)
    }
    
    #[setter]
    fn set_health_max(&mut self, value: f64) -> PyResult<()>{
        self.health_max = value;
        Ok(())
    }
    
    #[getter]
    fn get_shield(&self) -> PyResult<f64>{
        Ok(self.shield)
    }
    
    #[setter]
    fn set_shield(&mut self, value: f64) -> PyResult<()>{
        self.shield = value;
        Ok(())
    }

    #[getter]
    fn get_shield_max(&self) -> PyResult<f64>{
        Ok(self.shield_max)
    }

    #[setter]
    fn set_shield_max(&mut self, value: f64) -> PyResult<()>{
        self.shield_max = value;
        Ok(())
    }

    #[getter]
    fn get_energy(&self) -> PyResult<f64>{
        Ok(self.energy)
    }

    #[setter]
    fn set_energy(&mut self, value:f64) -> PyResult<()>{
        self.energy = value;
        Ok(())
    }

    #[getter]
    fn get_is_flying(&self) -> PyResult<bool>{
        Ok(self.is_flying)
    }

    #[setter]
    fn set_is_flying(&mut self, value: bool) -> PyResult<()>{
        self.is_flying = value;
        Ok(())
    }

    #[getter]
    fn get_buff_timer(&self) -> PyResult<f64>{
        Ok(self.buff_timer)
    }
    
    #[setter]
    fn set_buff_timer(&mut self, value: f64) -> PyResult<()>{
        self.buff_timer = value;
        Ok(())
    }
    
}

pub fn clone_vec(vec: Vec<&CombatUnit>) -> Vec<CombatUnit> {
    vec.into_iter().map(|f| f.dup()).collect()
    }

#[pyclass]
#[derive(Clone)]
pub struct CombatUnits{
    pub units: Vec<CombatUnit>
}

#[pymethods]
impl CombatUnits{
    #[new]
    fn new(obj: &PyRawObject, mut _units1:  Vec<&CombatUnit>){
        let new_vec: Vec<CombatUnit> = clone_vec(_units1);
        obj.init(CombatUnits{units: new_vec})

    }
    fn len(&self)-> PyResult<usize>{
       Ok(self.units.len())
    }
    fn add(&mut self, _owner: i64, _unit_type: i64/*todo:*/, _health:f64, _flying: bool){
        let combat_unit: CombatUnit = CombatUnit{
            owner: _owner, 
            unit_type: _unit_type, 
            health: _health, 
            is_flying: _flying, 
            buff_timer:0.0, 
            energy:0.0,
            health_max: _health,
            shield_max:0.0,
            shield:0.0
            };
             self.units.push(combat_unit)
     }
    
    fn clear(&mut self){
        self.units = Vec::<CombatUnit>::new()
    }
    #[getter]
    fn get_units(&mut self)->PyResult<Vec<CombatUnit>>{
        Ok(self.units.clone())
    }
}