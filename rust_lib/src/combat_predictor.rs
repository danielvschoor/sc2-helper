use crate::combat_unit::{clone_vec, CombatUnit};
use crate::game_info::{can_be_attacked_by_air_weapons, Attribute, GameInfo, WeaponInfo};
use crate::generated_enums::UnitTypeId;
use pyo3::prelude::*;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rustc_hash::{FxHashSet};
use sc2_techtree::TechData;
use serde_json;
use std::borrow::{Borrow, BorrowMut};
use std::f32::consts::PI;
use std::f32::EPSILON;
use std::fs::File;
use std::io::prelude::*;
use rayon::prelude::*;

#[derive(Clone, Copy)]
pub struct SurroundInfo {
    max_attackers_per_defender: i32,
    max_melee_attackers: i32,
}

pub fn max_surround(
    mut enemy_ground_unit_area: f32,
    enemy_ground_units: i32,
    zealot_radius: f32,
) -> SurroundInfo {
    if enemy_ground_units > 0 {
        enemy_ground_unit_area /= 0.0;
    }
    let radius: f32 = (enemy_ground_unit_area / PI).sqrt();
    let representative_melee_unit_radius = zealot_radius;
    let circumference_defenders: f32 = radius * (2.0 * PI);
    let circumference_attackers: f32 = (radius + representative_melee_unit_radius) * (2.0 * PI);
    let approximate_defenders_in_melee_range: f32;
    let value1: f32 = circumference_defenders / (2.0 * representative_melee_unit_radius);
    if value1 < enemy_ground_units as f32 {
        approximate_defenders_in_melee_range = value1;
    } else {
        approximate_defenders_in_melee_range = enemy_ground_units as f32
    }
    let approximate_attackers_in_melee_range: f32 =
        circumference_attackers / (2.0 * representative_melee_unit_radius);
    let max_attackers_per_defender: i32 = if approximate_defenders_in_melee_range > 0.0 {
        (approximate_attackers_in_melee_range.ceil() / approximate_defenders_in_melee_range) as i32
    } else {
        1
    };

    let max_melee_attackers: i32 = approximate_attackers_in_melee_range.ceil() as i32;
    SurroundInfo {
        max_attackers_per_defender,
        max_melee_attackers,
    }
}

#[pyclass]
#[derive(Clone, Debug)]
pub struct CombatSettings {
    #[pyo3(get, set)]
    bad_micro: bool,
    #[pyo3(get, set)]
    debug: bool,
    #[pyo3(get, set)]
    enable_splash: bool,
    #[pyo3(get, set)]
    enable_timing_adjustment: bool,
    #[pyo3(get, set)]
    enable_surround_limits: bool,
    #[pyo3(get, set)]
    enable_melee_blocking: bool,
    #[pyo3(get, set)]
    workers_do_no_damage: bool,
    #[pyo3(get, set)]
    assume_reasonable_positioning: bool,
    #[pyo3(get, set)]
    max_time: f32,
    #[pyo3(get, set)]
    start_time: f32,
    #[pyo3(get, set)]
    multi_threaded: bool,
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
            max_time: 100_000.0,
            start_time: 0.0,
            multi_threaded: false,
        })
    }
}

#[pyclass]
pub struct CombatPredictor {
    data: GameInfo,
    tech_data: TechData,
}

#[pymethods]
impl CombatPredictor {
    #[new]
    fn new(obj: &PyRawObject, _game_info: GameInfo, path: Option<String>) {
        let td: TechData = match path {
            Some(p) => {
                let mut file = File::open(p).unwrap();

                let mut contents = String::with_capacity(100);
                file.read_to_string(&mut contents).unwrap();

                match TechData::from_path(contents.as_ref()) {
                    Err(e) => {
                        println!("{:?}", e);
                        TechData::current()
                    }
                    Ok(t) => t,
                }
            }
            None => TechData::current(),
        };

        obj.init(CombatPredictor {
            data: _game_info,
            tech_data: td,
        })
    }

    fn serialize_game_data(&self) {
        let mut file = File::create("game_info.json").unwrap();
        let serialized = serde_json::to_string(&self.data).unwrap();
        file.write_all(serialized.as_ref()).unwrap();
    }

    fn init(&mut self) {
        //        self.data.load_available_units();
    }

    fn predict_engage(
        &mut self,
        mut _units1: Vec<&CombatUnit>,
        mut _units2: Vec<&CombatUnit>,
        defender_player: u32,
        settings: &CombatSettings,
    ) -> PyResult<(u32, f32)> {
        //        for mut u in _units1{
        //            println!("{:?}",u.health);
        //        }
        let units1: Vec<CombatUnit> = clone_vec(_units1);
        let units2: Vec<CombatUnit> = clone_vec(_units2);

        let (x, y) = self._predict_engage(units1, units2, defender_player, settings);
        Ok((x, y))
    }
}

impl CombatPredictor {
    fn nop_new(_game_info: GameInfo, path: Option<String>) -> Self {
        let td: TechData = match path {
            Some(p) => {
                let mut file = File::open(p).unwrap();

                let mut contents = String::with_capacity(100);
                file.read_to_string(&mut contents).unwrap();

                match TechData::from_path(contents.as_ref()) {
                    Err(e) => {
                        println!("{:?}", e);
                        TechData::current()
                    }
                    Ok(t) => t,
                }
            }
            None => TechData::current(),
        };
        CombatPredictor {
            data: _game_info,
            tech_data: td,
        }
    }

    fn get_zealot_radius(&self) -> f32 {
        self.tech_data
            .unittype(UnitTypeId::ZEALOT.to_tt())
            .unwrap()
            .radius
            .into()
    }

    fn load_data(
        &self,
        units: &mut [CombatUnit],
        reset_buff: bool,
        unit_types_scope: &FxHashSet<usize>,
        muti_threaded: bool,
    ) {
        for u in units {
            u.load_data(
                self.data.borrow(),
                self.tech_data.borrow(),
                None,
                None,
                &unit_types_scope,
                muti_threaded,
            );
            if reset_buff {
                u.buff_timer = 0.0;
            }
        }
    }

    fn get_fastest_attacker_speed(units: &[CombatUnit]) -> f32 {
        let mut fastest_attacker_speed = 0.0;

        for u in units {
            if u.get_movement_speed() > fastest_attacker_speed {
                fastest_attacker_speed = u.get_movement_speed();
            }
        }
        fastest_attacker_speed
    }

    fn get_max_range_defender(units: &[CombatUnit]) -> f32 {
        let mut max_range_defender: f32 = 0.0;
        for u in units {
            if u.get_attack_range() > max_range_defender {
                max_range_defender = u.get_attack_range();
            }
        }
        max_range_defender
    }

    fn get_unit_info(units: &[CombatUnit], time: f32) -> (i32, i32, f32, f32, f32) {
        let mut has_air: i32 = 0;
        let mut has_ground: i32 = 0;
        let mut ground_area: f32 = 0.0;
        let mut average_health_by_time: f32 = 0.0;
        let mut average_health_by_time_weight: f32 = 0.0;
        for unit in units {
            if unit.health > 0.0 {
                has_air += can_be_attacked_by_air_weapons(unit) as i32;
                has_ground += !unit.is_flying as i32;
                let r: f32 = unit.get_radius();
                ground_area += r * r;

                average_health_by_time += time * unit.health + unit.shield;
                average_health_by_time_weight += unit.health + unit.shield;
            }
        }
        (
            has_air,
            has_ground,
            ground_area,
            average_health_by_time,
            average_health_by_time_weight,
        )
    }

    fn find_best_target_multi_threaded(
        unit: &CombatUnit,
        units: &'a [CombatUnit],
        combat_settings: &CombatSettings,
        has_ground: bool,
        has_air: bool,
        is_unit_melee: bool,
        melee_unit_attack_count: &[i32],
        surround: &SurroundInfo,
        opponent_fraction_melee_units: f32,
        _best_weapon: Option<&'b WeaponInfo>,
    ) -> (Option<&'a CombatUnit>, usize, Option<&'b WeaponInfo>, f32) {
        //        let best_target: Option<&'a CombatUnit> = None;
        //        let best_target_index: usize = 0;
        //        let best_score: f32 = 0.0;
        //        let best_weapon: Option<&'b WeaponInfo> = None;
        //        let best_dps: f32 = 0.0;

        //        let best_units:Vec<_> =
        let (_, best_target, best_weapon, best_dps, best_target_index) = units
            .into_par_iter()
            .enumerate()
            .map(|(j, other)| {
                let air_dps2: f32 = match &unit.air_weapons {
                    Some(t) => t.get_dps(other.unit_type),
                    None => 0.0,
                };
                let ground_dps2: f32 = match &unit.ground_weapons {
                    Some(t) => t.get_dps(other.unit_type),
                    None => 0.0,
                };

                let dps: f32 = air_dps2.max(ground_dps2);
                let mut score: f32 = dps * target_score(other, has_ground, has_air) * 0.001;

                if is_unit_melee {
                    if combat_settings.enable_surround_limits
                        && melee_unit_attack_count[j] >= surround.max_attackers_per_defender
                    {
                        score = 0.0;
                    }

                    if !combat_settings.bad_micro && combat_settings.assume_reasonable_positioning {
                        score = -score;
                    }
                    if combat_settings.enable_melee_blocking && other.is_melee() {
                        score += 1000.00;
                    } else if combat_settings.enable_melee_blocking
                        && unit.get_movement_speed() < 1.05 * other.get_movement_speed()
                    {
                        score += 500.00;
                    }
                } else if !unit.is_flying {
                    let range_diff: f32 = other.get_attack_range() - unit.get_attack_range();
                    if opponent_fraction_melee_units > 0.5 && range_diff > 0.5 {
                        score -= 1000.00;
                    } else if opponent_fraction_melee_units > 0.3 && range_diff > 1.0 {
                        score -= 1000.00
                    }
                }
                (score, Some(other), _best_weapon, unit.get_max_dps(), j)
            })
            .reduce(
                || (0.0, None, None, 0.0, 0),
                |a, b| (if a.0 > b.0 { a } else { b }),
            );
        //.collect();

        //        for y in best_units {
        //            if y.0 > best_score {
        //                best_target = y.1;
        //                best_weapon = y.2;
        //                best_dps = y.3;
        //                best_target_index = y.4;
        //            }
        //        }

        (best_target, best_target_index, best_weapon, best_dps)
    }

    fn find_best_target(
        unit: &CombatUnit,
        units: &'a [CombatUnit],
        combat_settings: &CombatSettings,
        has_ground: bool,
        has_air: bool,
        is_unit_melee: bool,
        melee_unit_attack_count: &[i32],
        surround: &SurroundInfo,
        opponent_fraction_melee_units: f32,
        _best_weapon: Option<&'b WeaponInfo>,
    ) -> (Option<&'a CombatUnit>, usize, Option<&'b WeaponInfo>, f32) {
        let mut best_target: Option<&'a CombatUnit> = None;
        let mut best_target_index: usize = 0;
        let mut best_score: f32 = 0.0;
        let mut best_weapon: Option<&'b WeaponInfo> = None;
        let mut best_dps: f32 = 0.0;

        for (j, other) in units.iter().enumerate() {
            let air_dps2: f32 = match &unit.air_weapons {
                Some(t) => t.get_dps(other.unit_type),
                None => 0.0,
            };
            let ground_dps2: f32 = match &unit.ground_weapons {
                Some(t) => t.get_dps(other.unit_type),
                None => 0.0,
            };

            let dps: f32 = air_dps2.max(ground_dps2);

            let mut score: f32 = dps * target_score(other, has_ground, has_air) * 0.001;

            if is_unit_melee {
                if combat_settings.enable_surround_limits
                    && melee_unit_attack_count[j] >= surround.max_attackers_per_defender
                {
                    continue;
                }

                if !combat_settings.bad_micro && combat_settings.assume_reasonable_positioning {
                    score = -score;
                }
                if combat_settings.enable_melee_blocking && other.is_melee() {
                    score += 1000.00;
                } else if combat_settings.enable_melee_blocking
                    && unit.get_movement_speed() < 1.05 * other.get_movement_speed()
                {
                    score += 500.00;
                }
            } else if !unit.is_flying {
                let range_diff: f32 = other.get_attack_range() - unit.get_attack_range();
                if opponent_fraction_melee_units > 0.5 && range_diff > 0.5 {
                    score -= 1000.00;
                } else if opponent_fraction_melee_units > 0.3 && range_diff > 1.0 {
                    score -= 1000.00
                }
            }

            match best_target {
                None => {
                    best_score = score;
                    best_target = Some(other);
                    best_target_index = j;
                    best_weapon = _best_weapon;
                    best_dps = unit.get_max_dps();
                }
                Some(t) => {
                    if (score - best_score).abs() < EPSILON
                        && unit.health + unit.shield < t.health + t.shield
                    {
                        best_score = score;
                        best_target = Some(other);
                        best_target_index = j;
                        best_dps = unit.get_max_dps();
                        best_weapon = _best_weapon;
                    }
                }
            }
            //            if score > 1.0{
            //                break
            //            }
        }

        (best_target, best_target_index, best_weapon, best_dps)
    }
    fn _predict_engage(
        &mut self,
        mut units1: Vec<CombatUnit>,
        mut units2: Vec<CombatUnit>,
        defender_player: u32,
        combat_settings: &CombatSettings,
    ) -> (u32, f32) {
        const HEALING_PER_SECOND: f32 = 12.6 / 1.4;
        const SHIELDS_PER_NORMAL_SPEED_SECOND: f32 = 50.4 / 1.4;
        const ENERGY_USE_PER_SHIELD: f32 = 1.0 / 3.0;
        const MAX_ITERATIONS: u32 = 100;

        let debug: bool = combat_settings.debug;
        let zealot_radius: f32 = self.get_zealot_radius();

        //        let mut unit_types_scope: FnvHashSet<usize> =
        //            FnvHashSet::with_capacity_and_hasher(100, Default::default());
        let mut time: f32 = combat_settings.start_time;
        let reset_buff: bool = time == 0.00;

        let mut average_health_by_time: [f32; 2] = [0.0, 0.0];
        let mut average_health_by_time_weight: [f32; 2] = [0.0, 0.0];
        let mut max_range_defender: f32 = 0.0;
        let mut fastest_attacker_speed: f32 = 0.0;
        let mut changed: bool = true;
        let mut unit_types_scope = FxHashSet::with_capacity_and_hasher(100, Default::default());

//        if combat_settings.multi_threaded {
//            unit_types_scope.par_extend(units1.par_iter().map(|x| x.unit_type as usize));
//            unit_types_scope.par_extend(units2.par_iter().map(|x| x.unit_type as usize));
//        } else {
            for unit in &units1 {
                unit_types_scope.insert(unit.unit_type as usize);
            }
            for unit in &units2 {
                unit_types_scope.insert(unit.unit_type as usize);
            }
//        }
        //       units2.par_iter().for_each(|unit|{
        //            unit_types_scope.insert(unit.unit_type as usize);
        //        });
        //        for unit in &units2 {
        //            unit_types_scope.insert(unit.unit_type as usize);
        //        }
        //        unit_types_scope.extend(
        //            &units1
        //                .iter()
        //                .map(|n| n.unit_type as usize)
        //                .collect::<Vec<usize>>(),
        //        );
        //        unit_types_scope.extend(
        //            units2
        //                .iter()
        //                .map(|n| n.unit_type as usize)
        //                .collect::<Vec<usize>>(),
        //        );

        if debug {
            println!("Loading data");
        }
        //        let units1_pointer = Arc::new(units1);
        //        let units2_pointer = Arc::new(units2);
        //
        //        let mut t_pointer1 = units1_pointer.clone();
        //        let mut t_pointer2 = units2_pointer.clone();
        //

        let mut rng = thread_rng();
        units1.shuffle(&mut rng);
        units2.shuffle(&mut rng);
        if combat_settings.multi_threaded {
            units1.par_iter_mut().for_each(|u| {
                u.load_data(
                    self.data.borrow(),
                    self.tech_data.borrow(),
                    None,
                    None,
                    &unit_types_scope,
                    combat_settings.multi_threaded,
                );
                if reset_buff {
                    u.buff_timer = 0.0;
                }
            });

            units2.par_iter_mut().for_each(|u| {
                u.load_data(
                    self.data.borrow(),
                    self.tech_data.borrow(),
                    None,
                    None,
                    &unit_types_scope,
                    combat_settings.multi_threaded,
                );
                if reset_buff {
                    u.buff_timer = 0.0;
                }
            });
        } else {
            self.load_data(
                &mut units1,
                reset_buff,
                &unit_types_scope,
                combat_settings.multi_threaded,
            );
            self.load_data(
                &mut units2,
                reset_buff,
                &unit_types_scope,
                combat_settings.multi_threaded,
            )
        }

        //        let _data_pointer = Arc::new(&self.data);
        //        let _tech_data_pointer = Arc::new(&self.tech_data);
        //
        //        let pointer = Arc::new(&unit_types_scope[..]);
        //        let mut units1_pointer = Arc::new(units1).clone();
        //        let mut units2_pointer = Arc::new(units2).clone();

        //        thread::scope(|s| {
        //            let shuffle_handle = s.spawn(|_| {
        //                let mut rng = thread_rng();
        //                units1_pointer.shuffle(&mut rng);
        //            });
        //            let shuffle_handle2 = s.spawn(|_| {
        //                let mut rng = thread_rng();
        //                units2_pointer.shuffle(&mut rng);
        //            });
        //            shuffle_handle.join();
        //            shuffle_handle2.join();
        //
        //            let handle = s.spawn(move |_|{
        //
        //              for unit in units1.to_owned().iter().unique_by(|s| s.unit_type) {
        //                    unit_types_scope.push(unit.unit_type as usize)
        //                }}
        //            );
        //            let handle2 = s.spawn(move |_| {
        //                for unit in units2.to_owned().iter().unique_by(|s| s.unit_type) {
        //                    unit_types_scope.push(unit.unit_type as usize)
        //                }
        //            });
        //
        //
        //            let _t_data_pointer = *_data_pointer.clone();
        //            let _t_tech_data_pointer = *_tech_data_pointer.clone();
        ////            let t_pointer = (*pointer.clone());
        //            handle.join();
        //            handle2.join();
        //            for u in units1.iter_mut() {
        //               let t_pointer = (*pointer.clone());
        //                s.spawn(move |_| {
        //                    u.load_data(
        //                        &(_t_data_pointer),
        //                        &(_t_tech_data_pointer),
        //                        None,
        //                        None,
        //                        t_pointer,
        //                    );
        //                    if reset_buff {
        //                        u.buff_timer = 0.0;
        //                    }
        //                });
        //            }
        //            let _t_data_pointer = *_data_pointer.clone();
        //            let _t_tech_data_pointer = *_tech_data_pointer.clone();
        ////            let t_pointer = ;
        //            for u in units2.iter_mut() {
        //                let t_pointer = (*pointer.clone());
        //                s.spawn(move |_| {
        //                    u.load_data(
        //                        &(_t_data_pointer),
        //                        &(_t_tech_data_pointer),
        //                        None,
        //                        None,
        //                        t_pointer,
        //                    );
        //                    if reset_buff {
        //                        u.buff_timer = 0.0;
        //                    }
        //                });
        //            }
        //
        //            //            s.spawn(move |_| {
        //            //                let mut rng = thread_rng();
        //            //                for u in units2.iter_mut() {
        //            //                    u.load_data(
        //            //                        self.data.borrow(),
        //            //                        self.tech_data.borrow(),
        //            //                        None,
        //            //                        None,
        //            //                        &unit_types_scope,
        //            //                    );
        //            //                    if reset_buff {
        //            //                        u.buff_timer = 0.0;
        //            //                    }
        //            //                }
        //            //                units2.shuffle(&mut rng);
        //            //            });
        //        }).unwrap();
        //        let handle = thread::Builder::new()
        //            .name("units1".to_owned())
        //            .spawn(move||{
        //               for u in t_pointer1 {
        //                    u.load_data(
        //                        &_t_data_pointer,
        //                        &_t_tech_data_pointer,
        //                        None,
        //                        None,
        //                        &unit_types_scope,
        //                    );
        //            if reset_buff {
        //                u.buff_timer = 0.0;
        //            }
        //        }
        //
        ////
        //            });
        //        self.load_data(&mut units1, reset_buff, &unit_types_scope);
        //        self.load_data(&mut units2, reset_buff, &unit_types_scope);
        //        units1.shuffle(&mut rng);
        //        units2.shuffle(&mut rng);

        if debug {
            println!("Data loaded");
        }

        if defender_player == 1 || defender_player == 2 {
            if defender_player == 1 {
                max_range_defender = Self::get_max_range_defender(&units1);
                fastest_attacker_speed = Self::get_fastest_attacker_speed(&units2);
            } else {
                max_range_defender = Self::get_max_range_defender(&units2);
                fastest_attacker_speed = Self::get_fastest_attacker_speed(&units1);
            }
        }

        //        units1.shuffle(&mut rng);
        //        units2.shuffle(&mut rng);

        for it in 0..MAX_ITERATIONS {
            if !changed {
                break;
            }
            if debug {
                let mut total_health1 = 0.0;
                let mut total_health2 = 0.0;
                for u in &units1 {
                    total_health1 += u.health + u.shield
                }
                for u in &units2 {
                    total_health2 += u.health + u.shield
                }
                println!(
                    "units1-health={:?}, total={:?}, unit2-health={:?}, total={:?}",
                    total_health1,
                    units1.len(),
                    total_health2,
                    units2.len()
                );
            }
            //            let mut has_air1: i32 = 0;
            //            let mut has_air2: i32 = 0;
            //            let mut has_ground1: i32 = 0;
            //            let mut has_ground2: i32 = 0;
            //            let mut ground_area1: f32 = 0.0;
            //            let mut ground_area2: f32 = 0.0;

            let (
                has_air1,
                has_ground1,
                ground_area1,
                _average_health_by_time,
                _average_health_by_time_weight,
            ) = Self::get_unit_info(&units1, time);
            average_health_by_time[0] = _average_health_by_time;
            average_health_by_time_weight[0] = _average_health_by_time_weight;

            let (
                has_air2,
                has_ground2,
                ground_area2,
                _average_health_by_time,
                _average_health_by_time_weight,
            ) = Self::get_unit_info(&units2, time);
            average_health_by_time[1] = _average_health_by_time;
            average_health_by_time_weight[1] = _average_health_by_time_weight;

            let surround_info1: SurroundInfo =
                max_surround(ground_area2 * PI, has_ground2, zealot_radius);
            let surround_info2: SurroundInfo =
                max_surround(ground_area1 * PI, has_ground1, zealot_radius);

            let dt = if 5 < 1 + (it / 10) {
                5_f32
            } else {
                (1 + (it / 10)) as f32
            };
            if debug {
                println!("Iteration: {:?} Time:  {:?}", it, time);
            }
            changed = false;

            //            const GUARDIAN_SHIELD_UNITS: f32 = 4.5 * 4.5 * PI * 0.4;

            //            let mut guardian_shield_unit_fraction: Vec<f32> = vec![0.0, 0.0];
            //            let mut guardian_shield_covers_all_units: Vec<bool> = vec![false, false];
            //            let mut combined_units: Vec<CombatUnit> = &units1 + &units2;
            //            let mut damage_to_do: Vec<(&mut CombatUnit, f32)> = vec![];

            //            for group in 0..2 {
            //                let mut guardian_shield_area: f32 = 0.0;
            //
            //
            //                let g: &mut Vec<CombatUnit> = match group {
            //                    0 =>&mut units1,
            //                    _=> &mut units2
            //                };

            //                for u in g.iter_mut() {
            //                    if u.unit_type == UnitTypeId::SENTRY && u.buff_timer > 0.0 {
            //                        u.buff_timer -= dt;
            //                        guardian_shield_area += GUARDIAN_SHIELD_UNITS;
            //                    }
            //                }
            //                let mut total_area: f32 = 0.0;
            //                let len = g.len();
            //                for i in 0..len {
            //                    let r: f32 = g[i].get_radius();
            //                    total_area += r * r * PI;
            //                }
            //                guardian_shield_covers_all_units[group] = guardian_shield_area > total_area;
            //                guardian_shield_unit_fraction[group] = if guardian_shield_area / (0.001 + total_area) > 0.8 { guardian_shield_area / (0.001 + total_area) } else { 0.8 }
            //            }

            for group in 0..2 {
                if debug {
                    println!("Processing group {:?}", group);
                }
                let (g1, g2): (&mut Vec<CombatUnit>, &mut Vec<CombatUnit>) = match group {
                    0 => (&mut units1, &mut units2),
                    1 => (&mut units2, &mut units1),
                    _ => unreachable!(),
                };

                let surround: SurroundInfo = if group == 0 {
                    surround_info1
                } else {
                    surround_info2
                };

                let max_extra_melee_distance =
                    (ground_area1 / PI).sqrt() * PI + (ground_area2 / PI).sqrt() * PI;

                let mut num_melee_units_used: i32 = 0;

                //                let did_activate_guardian_shield: bool = false;

                let mut opponent_fraction_melee_units: f32 = 0.0;

                for u in g2.iter() {
                    if u.health > 0.0 && u.is_melee() {
                        opponent_fraction_melee_units += 1.0;
                    }
                }

                if !g2.is_empty() {
                    opponent_fraction_melee_units /= g2.len() as f32;
                }

                let mut has_been_healed: Vec<bool> = vec![false; g2.len()];
                let mut melee_unit_attack_count: Vec<i32> = vec![0; g2.len()];
                //                let melee_units_in_attack_range: Vec<i32> = vec![0; g2.len()];

                if debug {
                    println!(
                        "Max melee attackers: {:?} {:?} num units: {:?}",
                        surround.max_melee_attackers,
                        surround.max_attackers_per_defender,
                        g1.len()
                    )
                }
                let g1_len: usize = g1.len();
                for i in 0..g1_len {
                    let unit = &g1[i];
                    if unit.health == 0.0 {
                        continue;
                    }

                    let air_dps = unit.get_dps(true);
                    let ground_dps = unit.get_dps(false);

                    if debug {
                        println!("Processing {:?}, health: {:?}, shield: {:?}, energy: {:?}, ground_dps: {:?}, air_dps: {:?}", unit.get_name(), unit.health, unit.shield, unit.energy, ground_dps, air_dps);
                    }

                    if unit.unit_type == UnitTypeId::MEDIVAC {
                        let mut index_to_heal: i32 = -1;
                        if unit.energy > 0.0 {
                            let offset: usize = (rand::random::<usize>() % g1_len) as usize;

                            for j in 0..g1_len {
                                let index: usize = (j + offset) % g1_len;

                                let other = g1[index].borrow();
                                if index != i
                                    && !has_been_healed[index]
                                    && other.health < other.health_max
                                    && other
                                        .type_data
                                        .as_ref()
                                        .unwrap()
                                        .get_attributes()
                                        .contains(&Attribute::BIOLOGICAL)
                                {
                                    index_to_heal = index as i32;
                                    has_been_healed[index] = true;
                                    break;
                                }
                            }

                            if index_to_heal != -1 {
                                let t = &mut g1[index_to_heal as usize];
                                t.modify_health(HEALING_PER_SECOND * dt);
                                changed = true;
                            }
                        }

                        continue;
                    }

                    //                    if unit.unit_type == UnitTypeId::SHIELDBATTERY {
                    //                        if unit.energy > 0_f32 {
                    //                            let mut index_to_heal: i32 = -1;
                    //                            let delta: f32 = 0_f32;
                    //                            let offset: usize = (rand::random::<usize>() % g1_len) as usize;
                    //                            for j in 0..g1_len {
                    //                                let index: usize = (j + offset) % g1_len;
                    //                                let other = g1[index].borrow();
                    //                                if index != i && !has_been_healed[index] && other.health > 0_f32 && other.shield < other.shield_max {
                    //                                    delta: f32 = (other.shield_max - other.shield).min(SHIELDS_PER_NORMAL_SPEED_SECOND *dt).min(unit.energy / ENERGY_USE_PER_SHIELD);
                    //                                    debug_assert!(delta >=0_f32);
                    ////                                    other.shield += delta;
                    //                                    index_to_heal = index as i32;
                    //                                    has_been_healed[index] = true;
                    //                                    break;
                    //                                }
                    //                            }
                    //                            if index_to_heal != -1 {
                    //                                let t = &mut g1[index_to_heal as usize];
                    //
                    //                                t.shield += delta;
                    //                                let
                    ////                                t.modify_health(HEALING_PER_SECOND * dt);
                    //                                changed = true;
                    //                            }
                    //                        }
                    //                        continue
                    //                    }

                    if air_dps == 0.0 && ground_dps == 0.0 {
                        continue;
                    }

                    if combat_settings.workers_do_no_damage && unit.is_basic_harvester() {
                        continue;
                    }

                    let is_unit_melee: bool = unit.is_melee();

                    if is_unit_melee
                        && combat_settings.enable_surround_limits
                        && num_melee_units_used > surround.max_melee_attackers
                    {
                        continue;
                    }

                    //Timing adjustment
                    if combat_settings.enable_timing_adjustment {
                        if group + 1 != defender_player {
                            let mut distance_to_enemy = max_range_defender;
                            if is_unit_melee {
                                distance_to_enemy +=
                                    max_extra_melee_distance * (i as f32 / g1.len() as f32);
                            }

                            let time_to_reach_enemy =
                                time_to_be_able_to_attack(unit, distance_to_enemy);
                            if time < time_to_reach_enemy {
                                changed = true;
                                continue;
                            }
                        } else {
                            let time_to_reach_enemy = if fastest_attacker_speed > 0.0 {
                                (max_range_defender - unit.get_attack_range())
                                    / fastest_attacker_speed
                            } else {
                                10000_f32
                            };
                            if time < time_to_reach_enemy {
                                changed = true;
                                continue;
                            }
                        }
                    }

                    //                    let mut best_target: Option<&CombatUnit> = None;
                    //                    let mut best_target_index: usize = 0;
                    //                    let mut best_score: f32 = 0.0;
                    //                    let mut best_weapon: &Option<WeaponInfo> = &None;
                    //                    let mut best_dps: f32 = 0.0;
                    let has_ground: bool = if group == 0 {
                        has_ground1 != 0
                    } else {
                        has_ground2 != 0
                    };
                    let has_air: bool = if group == 0 {
                        has_air1 != 0
                    } else {
                        has_air2 != 0
                    };
                    let _best_weapon: Option<&WeaponInfo> = if air_dps > ground_dps {
                        unit.air_weapons.as_ref()
                    } else {
                        unit.ground_weapons.as_ref()
                    };
                    let (best_target, best_target_index, best_weapon, best_dps) =
                        if combat_settings.multi_threaded {
                            Self::find_best_target(
                                unit,
                                g2,
                                combat_settings,
                                has_ground,
                                has_air,
                                is_unit_melee,
                                &melee_unit_attack_count,
                                &surround,
                                opponent_fraction_melee_units,
                                _best_weapon,
                            )
                        } else {
                            Self::find_best_target_multi_threaded(
                                unit,
                                g2,
                                combat_settings,
                                has_ground,
                                has_air,
                                is_unit_melee,
                                &melee_unit_attack_count,
                                &surround,
                                opponent_fraction_melee_units,
                                _best_weapon,
                            )
                        };
                    if debug {
                        println!(
                            "Best unit={:?}, health={:?}, for unit={:?},health={:?}",
                            best_target.unwrap().unit_type,
                            best_target.unwrap().health,
                            unit.unit_type,
                            unit.health
                        );
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
                        //                        let rng = rand::thread_rng();

                        //                            let shielded: bool = !is_unit_melee && val < guardian_shield_unit_fraction[(1 - random_group) as usize];
                        let dps: f32 = best_dps//best_weapon.as_ref().unwrap().get_dps(other.unit_type)
                            * remaining_splash.max(1.0);
                        let damage_multiplier: f32 = 1.0;
                        if debug {
                            println!(
                                "Modify health of {:?}, current health={:?}, delta={:?}",
                                other.get_name(),
                                other.health,
                                -dps * damage_multiplier * dt
                            );
                        }
                        other.modify_health(-dps * damage_multiplier * dt);

                        if debug {
                            println!("Health of unit after modification ={:?}", other.health);
                        }

                        if other.health == 0.0 {
                            g2.swap_remove(best_target_index);
                            melee_unit_attack_count.swap_remove(best_target_index);
                            //                            let last_element: usize = g2.len() - 1;
                            //                            g2.swap(best_target_index, last_element);
                            //
                            //                            melee_unit_attack_count[best_target_index] =
                            //                                *melee_unit_attack_count.last().unwrap();
                            //                            g2.pop();
                            //                            melee_unit_attack_count.pop();
                            //                            best_target = None;
                        }
                    }
                }

                if debug {
                    println!(
                        "Melee attackers used: {:?} did change in the last iteration {:?}",
                        num_melee_units_used, changed
                    );
                }
            }

            time += dt;
            if time > combat_settings.max_time {
                break;
            }
        }

        //        println!("Main loop took {:?}", sw.elapsed());
        average_health_by_time[0] /= average_health_by_time_weight[0].max(0.01);
        average_health_by_time[1] /= average_health_by_time_weight[1].max(0.01);

        if debug {
            println!(
                "1: {:?}, 2: {:?}",
                average_health_by_time[0], average_health_by_time[1]
            );
        }

        let mut total_health1: f32 = 0.0;
        let mut total_health2: f32 = 0.0;
        for u in &units1 {
            total_health1 += u.health + u.shield
        }
        for u in &units2 {
            total_health2 += u.health + u.shield
        }

        if total_health1 > total_health2 {
            if debug {
                println!("Player 1 wins with health={:?}", total_health1);
            }
            (1, total_health1)
        } else {
            if debug {
                println!("Player 2 wins with health={:?}", total_health2);
            }
            (2, total_health2)
        }
    }
}

pub fn target_score(unit: &CombatUnit, has_ground: bool, has_air: bool) -> f32 {
    let mut score: f32 = 0.0;
    let cost: f32 = unit.get_adjusted_cost();

    let air_dps: f32 = unit.get_dps(true);
    let ground_dps: f32 = unit.get_dps(false);

    score += 0.01 * cost;

    score += 1000.00 * unit.get_max_dps();

    if !has_air && ground_dps == 0.0 || !has_ground && air_dps == 0.0 {
        score *= 0.01;
    }
    score
}

pub fn time_to_be_able_to_attack(unit: &CombatUnit, distance_to_enemy: f32) -> f32 {
    if unit.get_movement_speed() > 0.0 {
        if distance_to_enemy - unit.get_attack_range() > 0.0 {
            (distance_to_enemy - unit.get_attack_range()) / unit.get_movement_speed()
        } else {
            0.0
        }
    } else {
        10000.0
    }
}

#[cfg(test)] // Only compiles when running tests
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use test::Bencher;
    #[bench]
    fn bench_combat_predictor_10_units(b: &mut Bencher) {
        // Setup
        let mut file = File::open("D:\\Rust\\Combat Simulator\\game_info.json").unwrap();

        let mut contents = String::with_capacity(10000);
        file.read_to_string(&mut contents).unwrap();

        let game_info: GameInfo = serde_json::from_str(contents.as_ref()).unwrap();
        let mut cp = CombatPredictor::nop_new(
            game_info,
            Some(
                "D:\\Rust\\Combat Simulator\\sc2-techtree\\data\\data.json"
                    .parse()
                    .unwrap(),
            ),
        );
        let combat_unit1: CombatUnit = CombatUnit::nop_new(
            1,
            UnitTypeId::MARINE,
            45.0,
            45.0,
            0.0,
            0.0,
            0.0,
            false,
            0.0,
            None,
        );
        let combat_unit2: CombatUnit = CombatUnit::nop_new(
            1,
            UnitTypeId::ROACH,
            145.0,
            145.0,
            0.0,
            0.0,
            0.0,
            false,
            0.0,
            None,
        );
        let combat_units1: Vec<CombatUnit> = vec![combat_unit1.clone(); 10];

        let combat_units2: Vec<CombatUnit> = vec![combat_unit2.clone(); 10];

        let combat_settings = CombatSettings {
            bad_micro: false,
            debug: false,
            enable_splash: false,
            enable_timing_adjustment: false,
            enable_surround_limits: false,
            enable_melee_blocking: false,
            workers_do_no_damage: false,
            assume_reasonable_positioning: false,
            max_time: 0.0,
            start_time: 0.0,
            multi_threaded: true,
        };
        // Run bench
        b.iter(|| {
            cp._predict_engage(
                combat_units1.clone(),
                combat_units2.clone(),
                1,
                &combat_settings,
            )
        });
    }
    #[bench]
    fn bench_combat_predictor_100_units(b: &mut Bencher) {
        // Setup
        let mut file = File::open("D:\\Rust\\Combat Simulator\\game_info.json").unwrap();

        let mut contents = String::with_capacity(10000);
        file.read_to_string(&mut contents).unwrap();

        let game_info: GameInfo = serde_json::from_str(contents.as_ref()).unwrap();
        let mut cp = CombatPredictor::nop_new(
            game_info,
            Some(
                "D:\\Rust\\Combat Simulator\\sc2-techtree\\data\\data.json"
                    .parse()
                    .unwrap(),
            ),
        );
        let combat_unit1: CombatUnit = CombatUnit::nop_new(
            1,
            UnitTypeId::MARINE,
            45.0,
            45.0,
            0.0,
            0.0,
            0.0,
            false,
            0.0,
            None,
        );
        let combat_unit2: CombatUnit = CombatUnit::nop_new(
            1,
            UnitTypeId::ROACH,
            145.0,
            145.0,
            0.0,
            0.0,
            0.0,
            false,
            0.0,
            None,
        );
        let combat_units1: Vec<CombatUnit> = vec![combat_unit1.clone(); 100];

        let combat_units2: Vec<CombatUnit> = vec![combat_unit2.clone(); 100];

        let combat_settings = CombatSettings {
            bad_micro: false,
            debug: false,
            enable_splash: false,
            enable_timing_adjustment: false,
            enable_surround_limits: false,
            enable_melee_blocking: false,
            workers_do_no_damage: false,
            assume_reasonable_positioning: false,
            max_time: 0.0,
            start_time: 0.0,
            multi_threaded: true,
        };
        // Run bench
        b.iter(|| {
            cp._predict_engage(
                combat_units1.clone(),
                combat_units2.clone(),
                1,
                &combat_settings,
            )
        });
    }
    #[bench]
    fn bench_combat_predictor_1000_units(b: &mut Bencher) {
        // Setup
        let mut file = File::open("D:\\Rust\\Combat Simulator\\game_info.json").unwrap();

        let mut contents = String::with_capacity(10000);
        file.read_to_string(&mut contents).unwrap();

        let game_info: GameInfo = serde_json::from_str(contents.as_ref()).unwrap();
        let mut cp = CombatPredictor::nop_new(
            game_info,
            Some(
                "D:\\Rust\\Combat Simulator\\sc2-techtree\\data\\data.json"
                    .parse()
                    .unwrap(),
            ),
        );
        let combat_unit1: CombatUnit = CombatUnit::nop_new(
            1,
            UnitTypeId::MARINE,
            45.0,
            45.0,
            0.0,
            0.0,
            0.0,
            false,
            0.0,
            None,
        );
        let combat_unit2: CombatUnit = CombatUnit::nop_new(
            1,
            UnitTypeId::ROACH,
            145.0,
            145.0,
            0.0,
            0.0,
            0.0,
            false,
            0.0,
            None,
        );
        let combat_units1: Vec<CombatUnit> = vec![combat_unit1.clone(); 1000];

        let combat_units2: Vec<CombatUnit> = vec![combat_unit2.clone(); 1000];

        let combat_settings = CombatSettings {
            bad_micro: false,
            debug: false,
            enable_splash: false,
            enable_timing_adjustment: false,
            enable_surround_limits: false,
            enable_melee_blocking: false,
            workers_do_no_damage: false,
            assume_reasonable_positioning: false,
            max_time: 0.0,
            start_time: 0.0,
            multi_threaded: true,
        };
        // Run bench
        b.iter(|| {
            cp._predict_engage(
                combat_units1.clone(),
                combat_units2.clone(),
                1,
                &combat_settings,
            )
        });
    }
}
