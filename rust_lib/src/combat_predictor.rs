use pyo3::prelude::*;
use crate::combat_unit::{CombatUnit, clone_vec};
use crate::game_info::{GameInfo};
use pyo3::types::PyAny;
use crate::generated_enums::UnitTypeId;
use sc2_techtree::TechData;
use std::fs::File;
use std::io::prelude::*;

pub struct CombatSettings{
    bad_micro: bool,
    debug: bool,
    enable_splash: bool,
    enable_timing_adjustment: bool,
    enable_surround_limits: bool,
    enable_melee_blocking: bool,
    workers_do_no_damage: bool,
    assume_reasonable_positioning: bool,
    max_time: u64,
    start_time: u64
}

impl<'source> FromPyObject<'source> for CombatSettings {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        unsafe {
            let py = Python::assume_gil_acquired();
            let obj = ob.to_object(py);
            Ok(Self {
                bad_micro: obj.getattr(py, "bad_micro")?.extract(py)?,
                debug: obj.getattr(py, "debug")?.extract(py)?,
                enable_splash: obj.getattr(py, "enable_splash")?.extract(py)?,
                enable_timing_adjustment: obj.getattr(py, "enable_timing_adjustment")?.extract(py)?,
                enable_surround_limits: obj.getattr(py, "enable_surround_limits")?.extract(py)?,
                enable_melee_blocking: obj.getattr(py, "enable_melee_blocking")?.extract(py)?,
                workers_do_no_damage: obj.getattr(py, "workers_do_no_damage")?.extract(py)?,
                assume_reasonable_positioning: obj.getattr(py, "assume_reasonable_positioning")?.extract(py)?,
                max_time: obj.getattr(py, "max_time")?.extract(py)?,
                start_time: obj.getattr(py, "start_time")?.extract(py)?
            })
        }
    }
}


#[pyclass]
pub struct CombatPredictor{
    data: GameInfo,
    tech_data: TechData
}

#[pymethods]
impl CombatPredictor{
    #[new]
    fn new(obj: &PyRawObject, _game_info: GameInfo, path: Option<String>){

        let td: TechData = match path {
            None => TechData::current(),
            Some(p) => {
                let mut file = File::open(p).unwrap();

                let mut contents = String::new();
                file.read_to_string(&mut contents).unwrap();

                match TechData::from_path(contents.as_ref()){
                    Err(e) => {
                        println!("{:?}", e);
                        TechData::current()
                    },
                    Ok(t) => t,
                    _ => {
                        println!("Not supposed to reach this");
                        TechData::current()
                    }
                }
            }

        };

        obj.init(CombatPredictor{
            data: _game_info,
            tech_data: td
        })
     }


    fn predict_engage(&mut self, mut _units1: Vec<&CombatUnit>, mut _units2: Vec<&CombatUnit>, settings: Option<CombatSettings>)->PyResult<u32>{
        let combat_settings: CombatSettings = match settings{
            None => CombatSettings{
                bad_micro: false,
                debug: false,
                enable_splash: true,
                enable_timing_adjustment: false,
                enable_surround_limits: true,
                enable_melee_blocking: true,
                workers_do_no_damage: false,
                assume_reasonable_positioning: true,
                max_time: 100000,
                start_time: 0
            },
            Some(u) => u
        };

        let units1 =  clone_vec(std::mem::replace(&mut _units1, Vec::<&CombatUnit>::new()));

        let units2 =  clone_vec(std::mem::replace(&mut _units2, Vec::<&CombatUnit>::new()));

        let debug: bool = combat_settings.debug;
        let zealot_radius = self.tech_data.unittype(UnitTypeId::ZEALOT.to_tt()).unwrap().radius;
        let mut total_health1: f64 = 0.0;
        let mut total_health2: f64 = 0.0;

        for x in units1.clone().into_iter(){
            total_health1 += x.health;
        }
        for y in units2.clone().into_iter(){
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