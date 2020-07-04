#![warn(unused_extern_crates)]
#![deny(dead_code)]
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate enum_primitive_derive;
extern crate num_traits;
// use num_traits::FromPrimitive;

pub mod combat_predictor;
pub mod combat_unit;
mod enums;
pub mod generated_enums;
mod unit_type_data;
pub mod weapon;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

macro_rules! max {
    ($x: expr) => ($x);
    ($x: expr, $($z: expr),+) => {{
        let y = max!($($z),*);
        if $x > y {
            $x
        } else {
            y
        }
    }}
}
macro_rules! min {
    ($x: expr) => ($x);
    ($x: expr, $($z: expr),+) => {{
        let y = min!($($z),*);
        if $x < y {
            $x
        } else {
            y
        }
    }}
}

#[pyfunction]
pub fn circles_intersect(pos1: (f64, f64), pos2: (f64, f64), r1: f64, r2: f64) -> bool {
    let x1 = pos1.0;
    let x2 = pos2.0;
    let y1 = pos1.1;
    let y2 = pos2.1;

    let dist_sq: f64 = (x1 - x2) * (x1 - x2) + (y1 - y2) * (y1 - y2);
    let rad_sum_sq = (r1 + r2) * (r1 + r2);
    if (dist_sq - rad_sum_sq).abs() < std::f64::EPSILON {
        false
    } else {
        !(dist_sq > rad_sum_sq)
    }
}

#[pyfunction]
pub fn in_circle(position: (f64, f64), tile: (usize, usize), radius: f64) -> bool {
    let dx = position.0 - tile.0 as f64;
    let dy = position.1 - tile.1 as f64;

    ((dx * dx + dy * dy) as f64) < radius * radius
}

#[pyfunction]
fn find_points_inside_circle(
    position: (f64, f64),
    radius: f64,
    h: usize,
    w: usize,
) -> Vec<(usize, usize)> {
    let top = max!(0.0, position.1 - radius) as usize;
    let bottom = min!(h as f64, position.1 + radius) as usize;
    let left = max!(0.0, position.0 - radius) as usize;
    let right = min!(w as f64, position.0 + radius) as usize;
    let mut points_in_circle: Vec<(usize, usize)> = Vec::with_capacity(radius.ceil() as usize * 4);
    for y in top..bottom {
        for x in left..right {
            if in_circle(position, (x, y), radius) {
                points_in_circle.push((x, y))
            }
        }
    }
    points_in_circle
}

#[pymodule]
fn sc2_helper(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<combat_predictor::CombatPredictor>()?;
    // m.add_class::<combat_unit::CombatUnit>()?;
    m.add_class::<combat_predictor::CombatSettings>()?;
    m.add_wrapped(wrap_pyfunction!(circles_intersect))?;
    m.add_wrapped(wrap_pyfunction!(find_points_inside_circle))?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;

    use combat_predictor::{CombatPredictor, CombatSettings};
    use enums::Attribute;
    use generated_enums::UnitTypeId;
    use unit_type_data::{Cost, UnitTypeData};
    use weapon::{Weapon, WeaponTargetType};
    use crate::combat_unit::CombatUnit;

    #[test]
    fn test_combat_predictor() {
        let mut combat_settings = CombatSettings::new();
        combat_settings.debug = true;
        let mut predictor = CombatPredictor::new();
        let marine = CombatUnit {
            type_id: UnitTypeId::MARINE,
            type_data: UnitTypeData::new(
                vec![Attribute::LIGHT, Attribute::BIOLOGICAL],
                Cost {
                    minerals: 50,
                    vespene: 0,
                    time: 400.0,
                },
            ),
            name: "Marine".to_string(),
            is_light: true,
            is_armored: false,
            is_biological: true,
            is_mechanical: false,
            is_massive: false,
            is_psionic: false,
            weapons: Some(vec![Weapon {
                w_type: WeaponTargetType::ANY,
                damage: 6.0,
                attacks: 1,
                range: 5.0,
                speed: 0.86083984,
                damage_bonus: None,
            }]),
            ground_dps: 6.969938,
            ground_range: 5.0,
            air_dps: 6.969938,
            air_range: 5.0,
            armor: 0.0,
            movement_speed: 2.25,
            health: 45.0,
            health_max: 45.0,
            shield: 0.0,
            shield_max: 0.0,
            energy: 0.0,
            energy_max: 0.0,
            radius: 0.375,
            is_flying: false,
            attack_upgrade_level: 0,
            armor_upgrade_level: 0,
            buff_timer: 0.0,
            shield_upgrade_level: 0,
        };
        let zergling: CombatUnit = CombatUnit {
            type_id: UnitTypeId::ZERGLING,
            type_data: UnitTypeData::new(
                vec![Attribute::LIGHT, Attribute::BIOLOGICAL],
                Cost {
                    minerals: 25,
                    vespene: 0,
                    time: 384.0,
                },
            ),
            name: "Zergling".to_string(),
            is_light: true,
            is_armored: false,
            is_biological: true,
            is_mechanical: false,
            is_massive: false,
            is_psionic: false,
            weapons: Some(vec![Weapon {
                w_type: WeaponTargetType::GROUND,
                damage: 5.0,
                attacks: 1,
                range: 0.100097656,
                speed: 0.6960449,
                damage_bonus: None,
            }]),
            ground_dps: 7.1834445,
            ground_range: 0.100097656,
            air_dps: 0.0,
            air_range: 0.0,
            armor: 0.0,
            movement_speed: 2.953125,
            health: 35.0,
            health_max: 35.0,
            shield: 0.0,
            shield_max: 0.0,
            energy: 0.0,
            energy_max: 0.0,
            radius: 0.375,
            is_flying: false,
            attack_upgrade_level: 0,
            armor_upgrade_level: 0,
            buff_timer: 0.0,
            shield_upgrade_level: 0,
        };
        let battlecruiser: CombatUnit = CombatUnit {
            type_id: UnitTypeId::BATTLECRUISER,
            type_data: UnitTypeData::new(
                vec![
                    Attribute::ARMORED,
                    Attribute::MECHANICAL,
                    Attribute::MASSIVE,
                ],
                Cost {
                    minerals: 400,
                    vespene: 300,
                    time: 1440.0,
                },
            ),
            name: "Battlecruiser".to_string(),
            is_light: false,
            is_armored: true,
            is_biological: false,
            is_mechanical: true,
            is_massive: true,
            is_psionic: false,
            weapons: Some(Weapon::battlecruiser()),
            ground_dps: 35.71428571,
            ground_range: 6.0,
            air_dps: 22.32142857,
            air_range: 6.0,
            armor: 3.0,
            movement_speed: 1.875,
            health: 550.0,
            health_max: 550.0,
            shield: 0.0,
            shield_max: 0.0,
            energy: 0.0,
            energy_max: 0.0,
            radius: 1.25,
            is_flying: true,
            attack_upgrade_level: 0,
            armor_upgrade_level: 0,
            buff_timer: 0.0,
            shield_upgrade_level: 0,
        };
        let mut units1: Vec<CombatUnit> = vec![];
        let mut units2: Vec<CombatUnit> = vec![];
        for _ in 0..13 {
            units2.push(marine.clone());
        }
        for _ in 0..1 {
            units1.push(battlecruiser.clone());
        }
        let result = predictor
            .predict_engage(units1, units2, 1, &combat_settings)
            .unwrap();
        assert!(result.0 == 2u32);
    }
}