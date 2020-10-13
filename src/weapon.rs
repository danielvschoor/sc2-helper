use serde::{Deserialize, Serialize};

use crate::combat_unit::CombatUnit;
#[cfg(feature = "python")]
use pyo3::{FromPy, FromPyObject, IntoPy, PyAny, PyObject, PyResult, Python, ToPyObject};
use rust_sc2::game_data::Attribute;
use rust_sc2::prelude::UnitTypeId;
use std::f32::EPSILON;

#[allow(missing_docs)]
#[derive(Primitive, Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub enum WeaponTargetType {
    NULL = 0,
    GROUND = 1,
    AIR = 2,
    ANY = 3,
}
impl Default for WeaponTargetType {
    fn default() -> Self {
        WeaponTargetType::NULL
    }
}
#[cfg(feature = "python")]
impl ToPyObject for WeaponTargetType {
    fn to_object(&self, py: Python) -> PyObject {
        self.to_i32().unwrap().to_object(py)
    }
}
#[cfg(feature = "python")]
impl FromPy<WeaponTargetType> for PyObject {
    fn from_py(other: WeaponTargetType, py: Python) -> Self {
        let _other: i32 = other.to_i32().unwrap();
        _other.into_py(py)
    }
}
#[cfg(feature = "python")]
impl<'source> FromPyObject<'source> for WeaponTargetType {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        //        println!("{:?}", "Extracting WeaponTargetType");
        let ob1: i32 = ob.extract::<i32>().unwrap();
        let x: WeaponTargetType = WeaponTargetType::from_i32(ob1).unwrap_or_default();
        Ok(x)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Weapon {
    pub w_type: WeaponTargetType,
    pub damage: f32,
    pub attacks: i32,
    pub range: f32,
    pub speed: f32,
    pub damage_bonus: Option<DamageBonus>,
}

#[cfg(feature = "python")]
impl<'source> FromPyObject<'source> for Weapon {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        unsafe {
            let py = Python::assume_gil_acquired();
            let obj = ob.to_object(py);
            Ok(Self {
                w_type: obj.getattr(py, "type")?.extract(py)?,
                damage: obj.getattr(py, "damage")?.extract(py)?,
                attacks: obj.getattr(py, "attacks")?.extract(py)?,
                range: obj.getattr(py, "range")?.extract(py)?,
                speed: obj.getattr(py, "speed")?.extract(py)?,
                damage_bonus: {
                    match obj.getattr(py, "damage_bonus") {
                        Ok(damage_bonus) => {
                            if let Ok(x) = damage_bonus.extract::<Vec<DamageBonus>>(py) {
                                if !x.is_empty() {
                                    Some(x[0])
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }
                        _ => None,
                    }
                },
            })
        }
    }
}

impl Weapon {
    pub fn battlecruiser() -> Vec<Self> {
        let air = Weapon {
            w_type: WeaponTargetType::AIR,
            damage: 5.0,
            damage_bonus: None,
            attacks: 1,
            range: 6.0,
            speed: 0.16 * 1.4,
        };
        let ground = Weapon {
            w_type: WeaponTargetType::GROUND,
            damage: 8.0,
            damage_bonus: None,
            attacks: 1,
            range: 6.0,
            speed: 0.16 * 1.4,
        };
        vec![air, ground]
    }
}
#[derive(Debug, Copy, Clone)]
pub struct DamageBonus {
    pub(crate) attribute: Attribute,
    pub(crate) bonus: f32,
}
impl PartialEq for DamageBonus {
    fn eq(&self, other: &Self) -> bool {
        self.attribute == other.attribute && (self.bonus - other.bonus).abs() < EPSILON
    }
}
impl Eq for DamageBonus {}
impl DamageBonus {
    /// Affected attribute.
    pub fn get_attribute(self) -> Attribute {
        self.attribute
    }

    /// Damage bonus.
    pub fn get_bonus(self) -> f32 {
        self.bonus
    }
}
#[cfg(feature = "python")]
impl<'source> FromPyObject<'source> for DamageBonus {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        //        println!("{:?}", "Extracting DamageBonus");
        unsafe {
            let py = Python::assume_gil_acquired();
            let obj = ob.to_object(py);
            Ok(Self {
                attribute: obj.getattr(py, "attribute")?.extract(py)?,
                bonus: obj.getattr(py, "bonus")?.extract(py)?,
            })
        }
    }
}

impl Weapon {
    /// Weapon's target type.
    pub fn get_target_type(&self) -> WeaponTargetType {
        self.w_type
    }
    /// Weapon damage.
    pub fn get_damage(&self) -> f32 {
        self.damage
    }

    /// Number of hits per attack (eg. Colossus has 2 beams).
    pub fn get_attacks(&self) -> i32 {
        self.attacks
    }
    pub fn splash(&self) -> f32 {
        // TODO: Add this
        0.0
    }
    /// Attack range.
    pub fn get_range(&self) -> f32 {
        self.range
    }
    /// Time between attacks.
    pub fn get_speed(&self) -> f32 {
        self.speed
    }

    pub fn get_dps(&self) -> f32 {
        self.attacks as f32 / self.speed
    }

    pub(crate) fn calculate_dps(&self, attacker: &CombatUnit, target: &CombatUnit) -> f32 {
        if self.w_type == WeaponTargetType::ANY
            || if target.type_id == UnitTypeId::Colossus || target.is_flying {
                self.w_type == WeaponTargetType::AIR
            } else {
                !target.is_flying
            }
        {
            let mut dmg: f32 = self.damage;
            if let Some(bonus_damage) = self.damage_bonus {
                match bonus_damage.attribute {
                    Attribute::Armored => {
                        if target.is_armored {
                            dmg += bonus_damage.bonus;
                        }
                    }
                    Attribute::Biological => {
                        if target.is_biological {
                            dmg += bonus_damage.bonus;
                        }
                    }
                    Attribute::Light => {
                        if target.is_light {
                            dmg += bonus_damage.bonus;
                        }
                    }
                    Attribute::Massive => {
                        if target.is_massive {
                            dmg += bonus_damage.bonus;
                        }
                    }
                    Attribute::Mechanical => {
                        if target.is_mechanical {
                            dmg += bonus_damage.bonus;
                        }
                    }
                    Attribute::Psionic => {
                        if target.is_psionic {
                            dmg += bonus_damage.bonus;
                        }
                    }
                    _ => {}
                }
            }

            dmg += attacker.attack_upgrade_level as f32;
            //        println!("DMG after getting damage bonus= {:?}", dmg);

            let mut armor: f32 = target.armor + target.armor_upgrade_level as f32;
            //        println!("Target {:?}, Armor {:?}",target, armor);

            // Note: cannot distinguish between damage to shields and damage to health yet, so average out so that the armor is about 0.5 over the whole shield+health of the unit
            // Important only for protoss
            let max_health: f32 = target.health_max;

            let max_shield: f32 = target.shield_max;
            //        println!("Past max_shield");
            armor = armor * max_health / (max_shield + max_health);
            //        println!("Target {:?}, Armor-calculated {:?}",target, armor);

            let time_between_attacks: f32 = self.speed;
            //        println!("Weapon speed={:?}", weapon.speed);

            return if dmg - armor > 0.0 {
                ((dmg - armor) * self.attacks as f32) / time_between_attacks
            } else {
                0.0
            };
        }

        0.0
    }
}
