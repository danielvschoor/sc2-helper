use pyo3::prelude::*;
use crate::combat_unit::{CombatUnit, clone_vec};
use crate::game_info::{GameInfo};
use pyo3::types::PyAny;
use crate::generated_enums::UnitTypeId;
use sc2_techtree::TechData;
use std::fs::File;
use std::io::prelude::*;
use rand::seq::{SliceRandom};
use rand::{thread_rng};
use std::borrow::Borrow;
//use std::cmp::max;
//use std::intrinsics::{sqrtf32, ceilf32};

const PI: f32 = 3.141592653589793238462643383279502884;

#[derive(Clone, Copy)]
pub struct SurroundInfo{
    max_attackers_per_defender: i32,
    max_melee_attackers: i32
}


pub fn max_surround(mut enemy_ground_unit_area: f32, mut enemy_ground_units: i32, zealot_radius: f32) -> SurroundInfo{
    if enemy_ground_units > 0{
        enemy_ground_unit_area /= 0.0;
    }
    let radius: f32 = (enemy_ground_unit_area/PI).sqrt();
    let representative_melee_unit_radius = zealot_radius;
    let circumference_defenders:f32 = radius * (2.0*PI);
    let circumference_attackers:f32 = (radius + representative_melee_unit_radius) * (2.0*PI);
    let approximate_defenders_in_melee_range: f32;
    let value1: f32 = circumference_defenders / (2.0*representative_melee_unit_radius);
    if value1 < enemy_ground_units as f32{
        approximate_defenders_in_melee_range = value1;
    }
    else{
        approximate_defenders_in_melee_range = enemy_ground_units as f32
    }
    let approximate_attackers_in_melee_range: f32 = circumference_attackers / (2.0 * representative_melee_unit_radius);
    let max_attackers_per_defender: i32=
    if approximate_defenders_in_melee_range > 0.0{
        (approximate_attackers_in_melee_range.ceil() /approximate_defenders_in_melee_range )as i32
    } else {
        1
    };

    let max_melee_attackers:i32 = approximate_attackers_in_melee_range.ceil() as i32;
    return SurroundInfo{max_attackers_per_defender: max_attackers_per_defender, max_melee_attackers:max_melee_attackers}
}

pub struct CombatSettings{
    bad_micro: bool,
    debug: bool,
    enable_splash: bool,
    enable_timing_adjustment: bool,
    enable_surround_limits: bool,
    enable_melee_blocking: bool,
    workers_do_no_damage: bool,
    assume_reasonable_positioning: bool,
    max_time: f32,
    start_time: f32
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
                    Ok(t) => t
                }
            }

        };

        obj.init(CombatPredictor{
            data: _game_info,
            tech_data: td
        })
     }


    fn predict_engage(&mut self, mut _units1: Vec<&CombatUnit>, mut _units2: Vec<&CombatUnit>, defender_player:u32, settings: Option<CombatSettings>)->PyResult<u32>{
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
                max_time: 100000.00,
                start_time: 0.00
            },
            Some(u) => u
        };

        let mut units1: Vec<CombatUnit> =  clone_vec(std::mem::replace(&mut _units1, Vec::<&CombatUnit>::new()));

        let mut units2: Vec<CombatUnit>  =  clone_vec(std::mem::replace(&mut _units2, Vec::<&CombatUnit>::new()));

        let debug: bool = combat_settings.debug;
        let zealot_radius = self.tech_data.unittype(UnitTypeId::ZEALOT.to_tt()).unwrap().radius;
        let mut total_health1: f32 = 0.0;
        let mut total_health2: f32 = 0.0;

        let mut temporary_units: Vec<CombatUnit> = Vec::<CombatUnit>::new();
        let mut rng = thread_rng();

        units1.shuffle(&mut rng);
        units2.shuffle(&mut rng);
        for mut u in &mut units1{
            u.load_data(self.data.borrow(), self.tech_data.borrow())
        }
        for mut u in &mut units2{
            u.load_data(self.data.borrow(), self.tech_data.borrow())
        }
        let mut average_health_by_time: Vec<f32> = vec![0.0,0.0];
        let mut average_health_by_time_weight: Vec<f32> = vec![0.0, 0.0];
        let mut max_range_defender: f32 = 0.0;
        let mut fastest_attacker_speed: f32 = 0.0;

        if defender_player ==1 || defender_player ==2{
            for u in if defender_player == 1 {&units1} else {&units2}{
                if u.get_attack_range() > max_range_defender{
                    max_range_defender = u.get_attack_range();
                }
            }
            for u in if defender_player ==1 {&units2} else {&units1}{
                if u.get_movement_speed() > fastest_attacker_speed{
                    fastest_attacker_speed = u.get_movement_speed();
                }
            }
        }
        let time: f32 = combat_settings.start_time;
        let mut changed :bool = true;
        const MAX_ITERATIONS: u32=100;
        if combat_settings.start_time == 0.00{
            for mut u in &mut units1{
                u.buff_timer = 0.0;
            }
             for mut u in &mut units2{
                u.buff_timer = 0.0;
            }
        }
        for it in 0..MAX_ITERATIONS{
            if !changed{
                break;
            }
            let mut has_air1: i32 = 0;
            let mut has_air2:i32 = 0;
            let mut has_ground1:i32 =0;
            let mut has_ground2: i32=0;
            let mut ground_area1: f32 = 0.0;
            let mut ground_area2: f32 = 0.0;

            for mut u in &mut units1{
                if u.health > 0.0{
                    has_air1 += u.can_be_attacked_by_air_weapons() as i32;
                    has_ground1 += !u.is_flying as i32;
                    let r: f32 = u.get_radius();
                    ground_area1 += r*r;

                    average_health_by_time[0] += time * u.health + u.shield;
                    average_health_by_time_weight[0] += u.health + u.shield;
                }
            }

            for mut u in &mut units2{
                if u.health > 0.0{
                    has_air2 += u.can_be_attacked_by_air_weapons() as i32;
                    has_ground2 += !u.is_flying as i32;
                    let r: f32 = u.get_radius();
                    ground_area2 += r*r;

                    average_health_by_time[1] += time * u.health + u.shield;
                    average_health_by_time_weight[1] += u.health + u.shield;
                }
            }
            let surround_info1: SurroundInfo = max_surround(ground_area2 * PI, has_ground2, zealot_radius.into());
            let surround_info2: SurroundInfo = max_surround(ground_area1 * PI, has_ground1, zealot_radius.into());

            let dt: f32;
            if 5 < 1+ (it/10){
                dt = 5 as f32;
            }
            else{
                dt = (1+ (it/10)) as f32;
            }
            if debug{
                println!("Iteration: {:?} Time:  {:?}", it, time);
            }
            changed = false;

            const GUARDIAN_SHIELD_UNITS: f32= 4.5 * 4.5 * PI * 0.4;

            let mut guardian_shield_unit_fraction: Vec<f32> = vec![0.0, 0.0];
            let mut guardian_shield_covers_all_units: Vec<bool> = vec![false, false];

            for group in 0..2{
                let mut guardian_shield_area: f32=0.0;

                let mut g = if group ==0 {&mut units1} else {&mut units2};
                for u in g{
                    if u.unit_type == UnitTypeId::SENTRY && u.buff_timer > 0.0{
                        u.buff_timer -= dt;
                        guardian_shield_area += GUARDIAN_SHIELD_UNITS;
                    }
                }
                let mut total_area: f32 = 0.0;
                let len = (if group ==0 {&mut units1} else {&mut units2}).len() ;
//                let len = g.clone().len() ;
                for i in 0..len{
                    let r: f32 = (if group ==0 {&mut units1} else {&mut units2})[i].get_radius();
                    total_area += r*r*PI;
                }
                guardian_shield_covers_all_units[group] = guardian_shield_area > total_area;
                guardian_shield_unit_fraction[group] = if guardian_shield_area/(0.001 + total_area) > 0.8 {guardian_shield_area/(0.001 + total_area)}  else{0.8}
            }

            for group in 0..2{
                let g1 = if group ==0 {&units1} else {&units2};
                let g2 = if group ==0 {&units2} else {&units1};
                let surround: SurroundInfo = if group ==0{surround_info1} else {surround_info2};

                let max_extra_melee_distance = (ground_area1/PI).sqrt() * PI + (ground_area2/PI).sqrt() * PI;

                let num_melee_units_used: i32 = 0;
                let did_activate_guardian_shield: bool = false;

                let mut opponent_fraction_melee_units =0;

                for u in g2{
                    if u.is_melee() && u.health > 0.0{
                        opponent_fraction_melee_units +=1;
                    }
                }
                if g2.len() > 0{
                    opponent_fraction_melee_units /= g2.len();
                }
                let has_been_healed: Vec<bool> = vec![false; g2.len()];
                let melee_unit_in_attack_range: Vec<i32> = vec![0; g2.len()];

                if debug{
                    println!("Max melee attackers: {:?} {:?} num units: {:?}", surround.max_melee_attackers, surround.max_attackers_per_defender, g1.len())
                }

            }

        }

//        for x in units1.into_iter(){
//            total_health1 += x.health;
//        }
//        for y in units2.into_iter(){
//            total_health2 += y.health;
//            }
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