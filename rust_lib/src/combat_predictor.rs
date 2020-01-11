use crate::num_traits::{FromPrimitive, ToPrimitive};
use pyo3::prelude::*;
use crate::combat_unit::{CombatUnit, clone_vec};
use crate::game_info::{GameInfo, UnitTypeData, WeaponInfo, can_be_attacked_by_air_weapons};
use crate::generated_enums::UnitTypeId;
use sc2_techtree::TechData;
use std::fs::File;
use std::io::prelude::*;
use rand::seq::{SliceRandom};
use rand::{thread_rng};
use std::borrow::{Borrow, BorrowMut};
use std::f32::consts::PI;
use rand::{Rng};
use std::f32::EPSILON;
use std::collections::HashSet;
use stopwatch::Stopwatch;


#[derive(Clone, Copy)]
pub struct SurroundInfo{
    max_attackers_per_defender: i32,
    max_melee_attackers: i32
}


pub fn max_surround(mut enemy_ground_unit_area: f32, enemy_ground_units: i32, zealot_radius: f32) -> SurroundInfo{
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
    SurroundInfo{max_attackers_per_defender, max_melee_attackers}
}

#[pyclass]
#[derive(Clone, Debug)]
pub struct CombatSettings{
    #[pyo3(get,set)]
    bad_micro: bool,
    #[pyo3(get,set)]
    debug: bool,
    #[pyo3(get,set)]
    enable_splash: bool,
    #[pyo3(get,set)]
    enable_timing_adjustment: bool,
    #[pyo3(get,set)]
    enable_surround_limits: bool,
    #[pyo3(get,set)]
    enable_melee_blocking: bool,
    #[pyo3(get,set)]
    workers_do_no_damage: bool,
    #[pyo3(get,set)]
    assume_reasonable_positioning: bool,
    #[pyo3(get,set)]
    max_time: f32,
    #[pyo3(get,set)]
    start_time: f32
}

#[pymethods]
impl CombatSettings {
    #[new]
    fn new(obj: &PyRawObject) {
        obj.init(CombatSettings {
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
        })
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
            Some(p) => {
                let mut file = File::open(p).unwrap();

                let mut contents = String::with_capacity(100);
                file.read_to_string(&mut contents).unwrap();

                match TechData::from_path(contents.as_ref()){
                    Err(e) => {
                        println!("{:?}", e);
                        TechData::current()
                    },
                    Ok(t) => t
                }
            },
            None => TechData::current()

        };

        obj.init(CombatPredictor {
            data: _game_info,
            tech_data: td
        }

        )
     }

    fn init(&mut self){
        self.data.load_available_units();
    }

    fn predict_engage(&mut self, py: Python<'_> ,mut _units1: Vec<&CombatUnit>, mut _units2: Vec<&CombatUnit>, defender_player:u32, settings: &CombatSettings)->PyResult<(u32, f32)>{
        pyo3::prepare_freethreaded_python();
//        let sw = Stopwatch::start_new();

        let units1: Vec<CombatUnit> = py.allow_threads(move||{
            clone_vec(_units1)
        });
        let units2:Vec<CombatUnit>= py.allow_threads(move ||{
            clone_vec(_units2)
        });

//        println!("Memory swap took {:?}", sw.elapsed());
        let (x,y) = self._predict_engage(units1, units2,defender_player,settings);
        Ok((x,y))



    }

 }

impl CombatPredictor{
    fn _predict_engage(&mut self, mut units1: Vec<CombatUnit>, mut units2: Vec<CombatUnit>, defender_player:u32, combat_settings: &CombatSettings) -> (u32, f32){
        let debug: bool = combat_settings.debug;

        let zealot_radius = self.tech_data.unittype(UnitTypeId::ZEALOT.to_tt()).unwrap().radius;

        let temporary_units: Vec<CombatUnit> = Vec::<CombatUnit>::with_capacity(100);
        let mut rng = thread_rng();

//        let sw = Stopwatch::start_new();

        units1.shuffle(&mut rng);
        units2.shuffle(&mut rng);
//        println!("Shuffle took {:?}", sw.elapsed());

        if debug{
            println!("Loading data");
        }

//        let sw = Stopwatch::start_new();
//        let mut unit_types_scope: Vec<UnitTypeId> = vec![];
        let mut unit_types_scope: HashSet<usize> = HashSet::with_capacity(100);

        for u in &units1{
            unit_types_scope.insert(u.unit_type.to_usize().unwrap());

        }

        for u in &units2{
            unit_types_scope.insert(u.unit_type.to_usize().unwrap());

        }
        for (u1,u2) in units1.iter_mut().zip(units2.iter_mut()){
            u1.load_data(self.data.borrow(), self.tech_data.borrow(),None,None, &unit_types_scope);
            u2.load_data(self.data.borrow(), self.tech_data.borrow(),None,None, &unit_types_scope);
        }
//        for u in &mut units1{
//            u.load_data(self.data.borrow(), self.tech_data.borrow(),None,None, &unit_types_scope)
//        }
//
//        for u in &mut units2{
//            u.load_data(self.data.borrow(), self.tech_data.borrow(),None,None, &unit_types_scope)
//        }
        if debug{
            println!("Data loaded");
        }
//        println!("Load data took {:?}", sw.elapsed());

//        let sw = Stopwatch::start_new();

        let mut average_health_by_time: Vec<f32> = vec![0.0, 0.0];
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
//        println!("get_attack_range took {:?}", sw.elapsed_ms());
        let mut time: f32 = combat_settings.start_time;
        let mut changed :bool = true;
        const MAX_ITERATIONS: u32=100;
        if combat_settings.start_time == 0.00{
            for u in &mut units1{
                u.buff_timer = 0.0;
            }
             for u in &mut units2{
                u.buff_timer = 0.0;
            }
        }
//       println!("Pre-Main loop took {:?}", sw.elapsed());
//       let sw = Stopwatch::start_new();
        for it in 0..MAX_ITERATIONS {
            if !changed {
                break;
            }
            let mut has_air1: i32 = 0;
            let mut has_air2: i32 = 0;
            let mut has_ground1: i32 = 0;
            let mut has_ground2: i32 = 0;
            let mut ground_area1: f32 = 0.0;
            let mut ground_area2: f32 = 0.0;


            for u in &mut units1 {
                if u.health > 0.0 {
                    has_air1 += can_be_attacked_by_air_weapons(u) as i32;
                    has_ground1 += !u.is_flying as i32;
                    let r: f32 = u.get_radius();
                    ground_area1 += r * r;

                    average_health_by_time[0] += time * u.health + u.shield;
                    average_health_by_time_weight[0] += u.health + u.shield;
                }
            }

            for u in &mut units2 {
                if u.health > 0.0 {
                    has_air2 += can_be_attacked_by_air_weapons(u) as i32;
                    has_ground2 += !u.is_flying as i32;
                    let r: f32 = u.get_radius();
                    ground_area2 += r * r;

                    average_health_by_time[1] += time * u.health + u.shield;
                    average_health_by_time_weight[1] += u.health + u.shield;
                }
            }
            let surround_info1: SurroundInfo = max_surround(ground_area2 * PI, has_ground2, zealot_radius.into());
            let surround_info2: SurroundInfo = max_surround(ground_area1 * PI, has_ground1, zealot_radius.into());

            let dt: f32;
            let dt = if 5 < 1 + (it / 10) { 5_f32 } else { (1 + (it / 10)) as f32 };
            if debug {
                println!("Iteration: {:?} Time:  {:?}", it, time);
            }
            changed = false;

            const GUARDIAN_SHIELD_UNITS: f32 = 4.5 * 4.5 * PI * 0.4;

            let mut guardian_shield_unit_fraction: Vec<f32> = vec![0.0, 0.0];
            let mut guardian_shield_covers_all_units: Vec<bool> = vec![false, false];
//            let mut combined_units: Vec<CombatUnit> = &units1 + &units2;
//            let mut damage_to_do: Vec<(&mut CombatUnit, f32)> = vec![];

            for group in 0..2 {
                let mut guardian_shield_area: f32 = 0.0;


                let g: &mut Vec<CombatUnit> = match group {
                    0 =>&mut units1,
                    _=> &mut units2
                };

                for u in g.iter_mut() {
                    if u.unit_type == UnitTypeId::SENTRY && u.buff_timer > 0.0 {
                        u.buff_timer -= dt;
                        guardian_shield_area += GUARDIAN_SHIELD_UNITS;
                    }
                }
                let mut total_area: f32 = 0.0;
                let len = g.len();
                for i in 0..len {
                    let r: f32 = g[i].get_radius();
                    total_area += r * r * PI;
                }
                guardian_shield_covers_all_units[group] = guardian_shield_area > total_area;
                guardian_shield_unit_fraction[group] = if guardian_shield_area / (0.001 + total_area) > 0.8 { guardian_shield_area / (0.001 + total_area) } else { 0.8 }
            }
//            let mut group_units: Vec<Vec<CombatUnit>> = vec![units1, units2];
            let previous_group: usize = 3;
            for group in 0..2 {
//                let random_group: usize = if previous_group == 3{ rng.gen_range(0,2) } else { if previous_group ==1 {0} else {1} };
//                previous_group = random_group;
                let random_group = group;
//                println!("Random group chosen == {:?}", random_group);
                if debug {
                    println!("Processing group {:?}", group);
                }
                let (g1, g2) : (&mut Vec<CombatUnit>, &mut Vec<CombatUnit>) = match random_group {
                    0 => (&mut units1, &mut units2),
                    1 => (&mut units2, &mut units1),
                    _ => unreachable!(),
                };

                let surround: SurroundInfo = if random_group == 0 { surround_info1 } else { surround_info2 };

                let max_extra_melee_distance = (ground_area1 / PI).sqrt() * PI + (ground_area2 / PI).sqrt() * PI;

                let mut num_melee_units_used: i32 = 0;

                let did_activate_guardian_shield: bool = false;

                let mut opponent_fraction_melee_units: f32 = 0.0;

                for u in g2.iter(){
                    if u.is_melee() && u.health > 0.0 {
                        opponent_fraction_melee_units += 1.0;
                    }
                }

                if !g2.is_empty() {
                    opponent_fraction_melee_units /= g2.len() as f32;
                }

                let has_been_healed: Vec<bool> = vec![false; g2.len()];
                let mut melee_unit_attack_count: Vec<i32> = vec![0; g2.len()];
//                let melee_units_in_attack_range: Vec<i32> = vec![0; g2.len()];

                if debug {
                    println!("Max melee attackers: {:?} {:?} num units: {:?}", surround.max_melee_attackers, surround.max_attackers_per_defender, g1.len())
                }

                for (i, unit) in g1.iter().enumerate() {
                    if unit.health == 0.0 {
                        continue
                    }

                    let unit_type_data = unit.type_data.as_ref().unwrap();
                    let air_dps = unit.get_dps(true);
                    let ground_dps = unit.get_dps(false);

                    if debug {
                        println!("Processing {:?}, health: {:?}, shield: {:?}, energy: {:?}, ground_dps: {:?}, air_dps: {:?}", unit.get_name(), unit.health, unit.shield, unit.energy, ground_dps, air_dps);
                    }

                    /*
                    INSERT SPECIAL UNIT CODE HERE
                    */

                    if air_dps == 0.0 && ground_dps == 0.0 {
                        continue
                    }


                    if combat_settings.workers_do_no_damage && unit.is_basic_harvester() {
                        continue
                    }

                    let is_unit_melee: bool = unit.is_melee();

                    if is_unit_melee && combat_settings.enable_surround_limits && num_melee_units_used > surround.max_melee_attackers {
                        continue
                    }

                    //Timing adjustment
                    if combat_settings.enable_timing_adjustment {
                        if group + 1 != defender_player {
                            let mut distance_to_enemy = max_range_defender;
                            if is_unit_melee {
                                distance_to_enemy += max_extra_melee_distance * (i as f32 / g1.len() as f32);
                            }

                            let time_to_reach_enemy = time_to_be_able_to_attack(unit, distance_to_enemy);
                            if time < time_to_reach_enemy {
                                changed = true;
                                continue
                            }
                        } else {
                            let time_to_reach_enemy = if fastest_attacker_speed > 0.0 { (max_range_defender - unit.get_attack_range()) / fastest_attacker_speed } else { 10000_f32 };
                            if time < time_to_reach_enemy {
                                changed = true;
                                continue
                            }
                        }
                    }

                    let mut best_target: Option<&CombatUnit> = None;
                    let mut best_target_index: usize = 0;
                    let mut best_score: f32 = 0.0;
                    let mut best_weapon: &Option<WeaponInfo> = &None;

                    for (j,other) in g2.iter().enumerate() {
//                        let other: &CombatUnit = &g2[j];
                        let other_data: &UnitTypeData = other.type_data.as_ref().unwrap();

                        let air_dps2: f32 = match &unit.air_weapons{
                            Some(t) => t.get_dps(other.unit_type),
                            None => 0.0
                        };
                        let ground_dps2: f32 = match &unit.ground_weapons{
                            Some(t) => t.get_dps(other.unit_type),
                            None => 0.0
                        };

                        let dps: f32 = air_dps2.max(ground_dps2);
                        let mut score: f32 = dps * target_score(other, other_data,if random_group == 0 { has_ground1 != 0 } else { has_ground2 != 0 }, if random_group == 0 { has_air1 != 0 } else { has_air2 != 0 }) * 0.001;

                        if is_unit_melee {
                            if combat_settings.enable_surround_limits && melee_unit_attack_count[j] >= surround.max_attackers_per_defender {
                                continue
                            }

                            if !combat_settings.bad_micro && combat_settings.assume_reasonable_positioning {
                                score = -score;
                            }
                            if combat_settings.enable_melee_blocking && other.is_melee() {
                                score += 1000.00;
                            } else if combat_settings.enable_melee_blocking && unit_type_data.get_movement_speed() < 1.05 * other_data.get_movement_speed() {
                                score += 500.00;
                            }
                        } else {
                            if !unit.is_flying {
                                let range_diff: f32 = other.get_attack_range() - unit.get_attack_range();
                                if opponent_fraction_melee_units > 0.5 && range_diff > 0.5 {
                                    score -= 1000.00;
                                } else if opponent_fraction_melee_units > 0.3 && range_diff > 1.0 {
                                    score -= 1000.00
                                }
                            }
                        }

                        match best_target{
                            None => {
                                if score > best_score{
                                    best_score = score;
                                    best_target = Some(other);
                                    best_target_index = j;
                                    best_weapon = if ground_dps > air_dps { &unit.ground_weapons } else { &unit.air_weapons };
                                }
                            },
                            Some(t) => {
                                if (score - best_score).abs() < EPSILON && unit.health + unit.shield < t.health + t.shield{
                                    best_score = score;
                                    best_target = Some(other);
                                    best_target_index = j;
                                    best_weapon = if ground_dps > air_dps { &unit.ground_weapons } else { &unit.air_weapons }
                                }
                            }
                        }

                    }


                        if best_target.is_some() {
                            if is_unit_melee {
                                num_melee_units_used += 1;
                            }
                            melee_unit_attack_count[best_target_index] += 1;

                            let best_weapon_splash: f32 = best_weapon.as_ref().unwrap().splash;
                            let remaining_splash: f32 = best_weapon_splash.max(1.0);

                            let other: &mut CombatUnit = g2[best_target_index].borrow_mut();
                            changed = true;
                            let mut rng = rand::thread_rng();
                            let val: f32 = rng.gen();


                            let shielded: bool = !is_unit_melee && val < guardian_shield_unit_fraction[(1 - random_group) as usize];
                            let dps: f32 = best_weapon.as_ref().unwrap().get_dps(other.unit_type) * remaining_splash.max(1.0);
                            let damage_multiplier: f32 = 1.0;
                            if debug{
                                println!("Modify health of {:?}, current health={:?}, delta={:?}",other.get_name(), other.health, -dps * damage_multiplier * dt);
                            }
                            other.modify_health(-dps * damage_multiplier * dt);


                            if debug {
                                println!("Health of unit after modification ={:?}", other.health);
                            }
                            if other.health == 0.0 {
                                let last_element: usize = g2.len()-1;
                                g2.swap(best_target_index, last_element);
//                                g2[best_target_index as usize] = g2.last().unwrap().clone();
                                melee_unit_attack_count[best_target_index] = *melee_unit_attack_count.last().unwrap();
                                g2.pop();
                                melee_unit_attack_count.pop();
                                best_target = None;
                            }
                        }

//                            remaining_splash -= 1.0;
                }


                if combat_settings.enable_splash {
                    if debug {
                        println!("Splash!")
                    }
                    /*TODO: SPLASH*/
                }

                if debug {
                    println!("Melee attackers used: {:?} did change in the last iteration {:?}", num_melee_units_used, changed);
                }
            }

            time += dt;
            if time > combat_settings.max_time {
                break;
            }
        }

//        println!("Main loop took {:?}", sw.elapsed());
        average_health_by_time[0] /= if average_health_by_time_weight[0] > 0.01 {average_health_by_time_weight[0]} else {0.01};
        average_health_by_time[1] /= if average_health_by_time_weight[1] > 0.01 {average_health_by_time_weight[1]} else {0.01};

        if debug {
            println!("1: {:?}, 2: {:?}", average_health_by_time[0], average_health_by_time[1]);
        }

        let mut total_health1:f32 = 0.0;
        let mut total_health2:f32 = 0.0;
        for u in &units1{
            total_health1 += u.health + u.shield
        }
        for u in &units2{
            total_health2 += u.health + u.shield
        }

        if total_health1 > total_health2{
            if debug {
                println!("Player 1 wins with health={:?}", total_health1);
            }
            (1, total_health1)

        }
        else{
            if debug {
                println!("Player 2 wins with health={:?}", total_health2);
            }
            (2, total_health2)
        }

    }
}
pub fn target_score(unit: &CombatUnit, type_data: &UnitTypeData, has_ground:bool, has_air:bool)->f32{
    const VESPENE_MULTIPLIER: f32 = 1.5;
//    let unit_type_data: &UnitTypeData = unit.type_data.as_ref().unwrap();
    let mut score: f32 = 0.0;
    let cost: f32 = type_data.get_mineral_cost() as f32 + VESPENE_MULTIPLIER * type_data.get_vespene_cost() as f32;

    let air_dps: f32 = unit.get_dps(true);
    let ground_dps: f32 = unit.get_dps(false);

    score += 0.01 * cost;

    score += 1000.00 * air_dps.max(ground_dps);

    if !has_air && ground_dps ==0.0 || !has_ground && air_dps ==0.0{
        score *= 0.01;
    }
    score
}

pub fn time_to_be_able_to_attack(unit: &CombatUnit, distance_to_enemy: f32) -> f32{
    if unit.get_movement_speed() > 0.0 {
        if distance_to_enemy-unit.get_attack_range() > 0.0 {
            (distance_to_enemy-unit.get_attack_range())/unit.get_movement_speed()
        } else {
            0.0
        }
    } else {
        10000.0
    }
}