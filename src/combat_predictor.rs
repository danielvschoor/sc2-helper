use crate::combat_unit::CombatUnit;
use crate::weapon::Weapon;
#[cfg(feature = "python")]
use pyo3::prelude::*;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rust_sc2::prelude::UnitTypeId;
use std::borrow::BorrowMut;
use std::f32::consts::PI;
use std::f32::EPSILON;

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
        enemy_ground_unit_area /= 0.6;
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
#[cfg_attr(feature = "python", pyclass)]
#[derive(Clone, Debug)]
pub struct CombatSettings {
    #[cfg_attr(feature = "python", pyo3(get, set))]
    pub bad_micro: bool,
    #[cfg_attr(feature = "python", pyo3(get, set))]
    pub debug: bool,
    #[cfg_attr(feature = "python", pyo3(get, set))]
    pub enable_splash: bool,
    #[cfg_attr(feature = "python", pyo3(get, set))]
    pub enable_timing_adjustment: bool,
    #[cfg_attr(feature = "python", pyo3(get, set))]
    pub enable_surround_limits: bool,
    #[cfg_attr(feature = "python", pyo3(get, set))]
    pub enable_melee_blocking: bool,
    #[cfg_attr(feature = "python", pyo3(get, set))]
    pub workers_do_no_damage: bool,
    #[cfg_attr(feature = "python", pyo3(get, set))]
    pub assume_reasonable_positioning: bool,
    #[cfg_attr(feature = "python", pyo3(get, set))]
    pub max_time: f32,
    #[cfg_attr(feature = "python", pyo3(get, set))]
    pub start_time: f32,
    #[cfg_attr(feature = "python", pyo3(get, set))]
    pub multi_threaded: bool,
}

#[cfg_attr(feature = "python", pymethods)]
impl CombatSettings {
    #[cfg_attr(feature = "python", new)]
    pub fn new() -> Self {
        CombatSettings {
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
        }
    }
}
impl Default for CombatSettings {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg_attr(feature = "python", pyclass)]
pub struct CombatPredictor {}

#[cfg_attr(feature = "python", pymethods)]
impl CombatPredictor {
    #[cfg_attr(feature = "python", new)]
    pub fn new() -> Self {
        CombatPredictor {}
    }

    pub fn predict_engage(
        &mut self,
        units1: Vec<CombatUnit>,
        units2: Vec<CombatUnit>,
        defender_player: u32,
        settings: &CombatSettings,
    ) -> (u32, f32) {
        self._predict_engage(units1, units2, defender_player, settings)
    }
}
impl Default for CombatPredictor {
    fn default() -> Self {
        Self::new()
    }
}
impl CombatPredictor {
    fn get_zealot_radius(&self) -> f32 {
        0.5
    }

    fn get_fastest_attacker_speed(units: &[CombatUnit]) -> f32 {
        let mut fastest_attacker_speed = 0.0;

        for u in units {
            if u.movement_speed > fastest_attacker_speed {
                fastest_attacker_speed = u.movement_speed;
            }
        }
        fastest_attacker_speed
    }
    #[cfg(feature = "python")]
    fn get_fastest_attacker_speed(units: &[CombatUnit]) -> f32 {
        let mut fastest_attacker_speed = 0.0;

        for u in units {
            if u.movement_speed > fastest_attacker_speed {
                fastest_attacker_speed = u.movement_speed;
            }
        }
        fastest_attacker_speed
    }

    fn get_max_range_defender(units: &[CombatUnit]) -> f32 {
        let mut max_range_defender: f32 = 0.0;
        for u in units {
            if u.get_max_range() > max_range_defender {
                max_range_defender = u.get_max_range();
            }
        }
        max_range_defender
    }
    #[cfg(feature = "python")]
    fn get_max_range_defender(units: &[CombatUnit]) -> f32 {
        let mut max_range_defender: f32 = 0.0;
        for u in units {
            if u.get_max_range() > max_range_defender {
                max_range_defender = u.get_max_range();
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
                has_air += unit.can_be_attacked_by_air() as i32;
                has_ground += !unit.is_flying as i32;
                let r: f32 = unit.radius;
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
    #[cfg(feature = "python")]
    fn get_unit_info(units: &[CombatUnit], time: f32) -> (i32, i32, f32, f32, f32) {
        let mut has_air: i32 = 0;
        let mut has_ground: i32 = 0;
        let mut ground_area: f32 = 0.0;
        let mut average_health_by_time: f32 = 0.0;
        let mut average_health_by_time_weight: f32 = 0.0;
        for unit in units {
            if unit.health > 0.0 {
                has_air += unit.can_be_attacked_by_air() as i32;
                has_ground += !unit.is_flying as i32;
                let r: f32 = unit.radius;
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

    // fn find_best_target_multi_threaded<'a>(
    //     unit: &'a CombatUnit,
    //     units: &[CombatUnit],
    //     combat_settings: &CombatSettings,
    //     has_ground: bool,
    //     has_air: bool,
    //     is_unit_melee: bool,
    //     melee_unit_attack_count: &[i32],
    //     surround: &SurroundInfo,
    //     opponent_fraction_melee_units: f32,
    //     _best_weapon: Option<&'a Weapon>,
    // ) -> (Option<&'a CombatUnit>, usize, Option<&'a Weapon>, f32) {
    //     let (_, best_target, best_weapon, best_dps, best_target_index) = units
    //         .into_par_iter()
    //         .enumerate()
    //         .map(|(j, other)| {
    //             let air_dps2: f32 = match unit.air_weapons() {
    //                 Some(t) => t.calculate_dps(unit, other),
    //                 None => 0.0,
    //             };
    //             let ground_dps2: f32 = match unit.ground_weapons() {
    //                 Some(t) => t.calculate_dps(unit, other),
    //                 None => 0.0,
    //             };
    //
    //             let dps: f32 = air_dps2.max(ground_dps2);
    //             let mut score: f32 = dps * target_score(other, has_ground, has_air) * 0.001;
    //
    //             if is_unit_melee {
    //                 if combat_settings.enable_surround_limits
    //                     && melee_unit_attack_count[j] >= surround.max_attackers_per_defender
    //                 {
    //                     score = 0.0;
    //                 }
    //
    //                 if !combat_settings.bad_micro && combat_settings.assume_reasonable_positioning {
    //                     score = -score;
    //                 }
    //                 if combat_settings.enable_melee_blocking && other.is_melee() {
    //                     score += 1000.00;
    //                 } else if combat_settings.enable_melee_blocking
    //                     && unit.movement_speed < 1.05 * other.movement_speed
    //                 {
    //                     score += 500.00;
    //                 }
    //             } else if !unit.is_flying {
    //                 let range_diff: f32 = other.get_max_range() - unit.get_max_range();
    //                 if opponent_fraction_melee_units > 0.5 && range_diff > 0.5 {
    //                     score -= 1000.00;
    //                 } else if opponent_fraction_melee_units > 0.3 && range_diff > 1.0 {
    //                     score -= 1000.00
    //                 }
    //             }
    //             (score, Some(other), _best_weapon, dps, j)
    //         })
    //         .reduce(
    //             || (0.0, None, None, 0.0, 0),
    //             |a, b| (if a.0 > b.0 { a } else { b }),
    //         );
    //
    //     (best_target, best_target_index, best_weapon, best_dps)
    // }
    #[allow(clippy::too_many_arguments)]
    fn find_best_target<'a>(
        unit: &CombatUnit,
        units: &'a [CombatUnit],
        combat_settings: &CombatSettings,
        has_ground: bool,
        has_air: bool,
        is_unit_melee: bool,
        melee_unit_attack_count: &[i32],
        surround: &SurroundInfo,
        opponent_fraction_melee_units: f32,
        _best_weapon: Option<&'a Weapon>,
    ) -> (Option<&'a CombatUnit>, usize, Option<&'a Weapon>, f32) {
        let mut best_target: Option<&CombatUnit> = None;
        let mut best_target_index: usize = 0;
        let mut best_score: f32 = 0.0;
        let mut best_weapon: Option<&Weapon> = None;
        let mut best_dps: f32 = 0.0;

        for (j, other) in units.iter().enumerate() {
            let air_dps2: f32 = match unit.air_weapons() {
                Some(t) => t.calculate_dps(unit, other),
                None => 0.0,
            };
            let ground_dps2: f32 = match unit.ground_weapons() {
                Some(t) => t.calculate_dps(unit, other),
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
                    && unit.movement_speed < 1.05 * other.movement_speed
                {
                    score += 500.00;
                }
            } else if !unit.is_flying {
                let range_diff: f32 = other.get_max_range() - unit.get_max_range();
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
                    best_dps = dps;
                }
                Some(t) => {
                    if (score > best_score || (score - best_score).abs() < EPSILON)
                        && unit.health + unit.shield < t.health + t.shield
                    {
                        best_score = score;
                        best_target = Some(other);
                        best_target_index = j;
                        best_dps = dps;
                        best_weapon = _best_weapon;
                    }
                }
            }
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
        const MAX_ITERATIONS: u32 = 100;

        let debug: bool = combat_settings.debug;
        let zealot_radius: f32 = self.get_zealot_radius();

        let mut time: f32 = combat_settings.start_time;
        // let reset_buff: bool = time == 0.00;

        let mut average_health_by_time: [f32; 2] = [0.0, 0.0];
        let mut average_health_by_time_weight: [f32; 2] = [0.0, 0.0];
        let max_range_defender: f32;
        let fastest_attacker_speed: f32;
        let mut changed: bool = true;

        let mut rng = thread_rng();
        units1.shuffle(&mut rng);
        units2.shuffle(&mut rng);

        if defender_player == 1 || defender_player == 2 {
            if defender_player == 1 {
                max_range_defender = Self::get_max_range_defender(&units1);
                fastest_attacker_speed = Self::get_fastest_attacker_speed(&units2);
            } else {
                max_range_defender = Self::get_max_range_defender(&units2);
                fastest_attacker_speed = Self::get_fastest_attacker_speed(&units1);
            }
        } else {
            max_range_defender =
                Self::get_max_range_defender(&units1).max(Self::get_max_range_defender(&units2));
            fastest_attacker_speed = Self::get_fastest_attacker_speed(&units2)
                .max(Self::get_fastest_attacker_speed(&units1))
        }

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

            let pi_ga1 = ground_area1 * PI;
            let pi_ga2 = ground_area2 * PI;
            let surround_info1: SurroundInfo = max_surround(pi_ga2, has_ground2, zealot_radius);
            let surround_info2: SurroundInfo = max_surround(pi_ga1, has_ground1, zealot_radius);

            let dt = if 5 < 1 + (it / 10) {
                5_f32
            } else {
                (1 + (it / 10)) as f32
            };
            if debug {
                println!("Iteration: {:?} Time:  {:?}", it, time);
            }
            changed = false;

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

                let mut opponent_fraction_melee_units: f32 = 0.0;

                for u in g2.iter() {
                    if u.health > 0.0 && u.is_melee() {
                        opponent_fraction_melee_units += 1.0;
                    }
                }

                if !g2.is_empty() {
                    opponent_fraction_melee_units /= g2.len() as f32;
                }

                let mut has_been_healed: Vec<bool> = vec![false; g1.len()];
                let mut melee_unit_attack_count: Vec<i32> = vec![0; g2.len()];

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
                        println!("Processing {:?}, health: {:?}, shield: {:?}, energy: {:?}, ground_dps: {:?}, air_dps: {:?}",
                                 unit.name,
                                 unit.health,
                                 unit.shield,
                                 unit.energy,
                                 ground_dps,
                                 air_dps);
                    }

                    if unit.type_id == UnitTypeId::Medivac {
                        if unit.energy > 0.0 {
                            let offset: usize = rand::random::<usize>() % g1_len;

                            for j in 0..g1_len {
                                let index: usize = (j + offset) % g1_len;

                                let other = g1[index].borrow_mut();
                                if index != i
                                    && !has_been_healed[index]
                                    && other.health < other.health_max
                                    && other.is_biological
                                {
                                    if debug {
                                        println!(
                                            "Unit {:?} being healed. Health before ={:?}",
                                            other.type_id, other.health
                                        );
                                    }
                                    other.modify_health(HEALING_PER_SECOND * dt);
                                    if debug {
                                        println!(
                                            "Unit {:?} being healed. Health after ={:?}",
                                            other.type_id, other.health
                                        );
                                    }
                                    has_been_healed[index] = true;
                                    break;
                                }
                            }
                        }

                        continue;
                    }

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
                                (max_range_defender - unit.get_max_range()) / fastest_attacker_speed
                            } else {
                                10000_f32
                            };
                            if time < time_to_reach_enemy {
                                changed = true;
                                continue;
                            }
                        }
                    }

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
                    let _best_weapon = if air_dps > ground_dps {
                        unit.air_weapons()
                    } else {
                        unit.ground_weapons()
                    };
                    let (best_target, best_target_index, best_weapon, best_dps) =
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
                        );

                    if best_target.is_some() {
                        if is_unit_melee {
                            num_melee_units_used += 1;
                        }
                        melee_unit_attack_count[best_target_index] += 1;

                        let best_weapon_splash = best_weapon.unwrap().splash();
                        let remaining_splash = best_weapon_splash.max(1.0);

                        let other: &mut CombatUnit = g2[best_target_index].borrow_mut();
                        changed = true;

                        let dps: f32 = best_dps * remaining_splash.max(1.0);
                        let damage_multiplier: f32 = 1.0;
                        if debug {
                            println!(
                                "Modify health of {:?}, current health={:?}, delta={:?}",
                                other.name,
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
            if debug {
                println!("{:?} has {:?} health left", u.type_id, u.health + u.shield);
            }
            total_health1 += u.health + u.shield;
        }
        for u in &units2 {
            if debug {
                println!("{:?} has {:?} health left", u.type_id, u.health + u.shield);
            }
            total_health2 += u.health + u.shield;
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
    let cost: f32 = unit.get_adjusted_cost() as f32;

    let air_dps: f32 = unit.get_dps(true);
    let ground_dps: f32 = unit.get_dps(false);

    score += 0.01 * cost;

    score += 1000.00 * unit.get_max_dps();

    if !has_air && ground_dps == 0.0 || !has_ground && air_dps == 0.0 {
        score *= 0.01;
    }
    score
}
#[cfg(feature = "python")]
pub fn target_score(unit: &CombatUnit, has_ground: bool, has_air: bool) -> f32 {
    let mut score: f32 = 0.0;
    let cost: f32 = unit.get_adjusted_cost() as f32;

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
    if unit.movement_speed > 0.0 {
        if distance_to_enemy - unit.get_max_range() > 0.0 {
            (distance_to_enemy - unit.get_max_range()) / unit.movement_speed
        } else {
            0.0
        }
    } else {
        10000.0
    }
}

#[cfg(feature = "python")]
pub fn time_to_be_able_to_attack(unit: &CombatUnit, distance_to_enemy: f32) -> f32 {
    if unit.movement_speed > 0.0 {
        if distance_to_enemy - unit.get_max_range() > 0.0 {
            (distance_to_enemy - unit.get_max_range()) / unit.movement_speed
        } else {
            0.0
        }
    } else {
        10000.0
    }
}
