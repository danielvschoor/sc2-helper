use crate::generated_enums::{UnitTypeId, UpgradeId};
// use crate::num_traits::FromPrimitive;
use crate::unit_type_data::UnitTypeData;
use crate::weapon::{Weapon, WeaponTargetType};
use pyo3::prelude::*;
use pyo3::types::PyAny;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Mutex;
lazy_static! {
    pub static ref UNIT_CACHE: Mutex<HashMap<UnitTypeId, CombatUnit>> =
        Mutex::new(HashMap::with_capacity(100));
}

const VESPENE_MULTIPLIER: f32 = 1.5;
lazy_static! {
    pub static ref IS_MELEE: HashSet<UnitTypeId> = [
        UnitTypeId::PROBE,
        UnitTypeId::ZEALOT,
        UnitTypeId::DARKTEMPLAR,
        UnitTypeId::SCV,
        UnitTypeId::HELLIONTANK,
        UnitTypeId::DRONE,
        UnitTypeId::ZERGLING,
        UnitTypeId::ZERGLINGBURROWED,
        UnitTypeId::BANELING,
        UnitTypeId::BANELINGBURROWED,
        UnitTypeId::ULTRALISK,
        UnitTypeId::BROODLING
    ]
    .iter()
    .cloned()
    .collect();
    pub static ref IS_BASIC_HARVESTER: HashSet<UnitTypeId> =
        [UnitTypeId::SCV, UnitTypeId::PROBE, UnitTypeId::DRONE]
            .iter()
            .cloned()
            .collect();
    pub static ref IS_UPGRADE_WITH_LEVELS: HashSet<UpgradeId> = [
        UpgradeId::TERRANINFANTRYWEAPONSLEVEL1,
        UpgradeId::TERRANINFANTRYARMORSLEVEL1,
        UpgradeId::TERRANVEHICLEWEAPONSLEVEL1,
        UpgradeId::TERRANSHIPWEAPONSLEVEL1,
        UpgradeId::PROTOSSGROUNDWEAPONSLEVEL1,
        UpgradeId::PROTOSSGROUNDARMORSLEVEL1,
        UpgradeId::PROTOSSSHIELDSLEVEL1,
        UpgradeId::ZERGMELEEWEAPONSLEVEL1,
        UpgradeId::ZERGGROUNDARMORSLEVEL1,
        UpgradeId::ZERGMISSILEWEAPONSLEVEL1,
        UpgradeId::ZERGFLYERWEAPONSLEVEL1,
        UpgradeId::ZERGFLYERARMORSLEVEL1,
        UpgradeId::PROTOSSAIRWEAPONSLEVEL1,
        UpgradeId::PROTOSSAIRARMORSLEVEL1,
        UpgradeId::TERRANVEHICLEANDSHIPARMORSLEVEL1
    ]
    .iter()
    .cloned()
    .collect();
    pub static ref TARGET_GROUND: HashSet<WeaponTargetType> =
        [WeaponTargetType::ANY, WeaponTargetType::GROUND]
            .iter()
            .cloned()
            .collect();
    pub static ref TARGET_AIR: HashSet<WeaponTargetType> =
        [WeaponTargetType::ANY, WeaponTargetType::AIR]
            .iter()
            .cloned()
            .collect();
}
#[derive(Clone, Debug)]
pub struct CombatUnit {
    pub type_id: UnitTypeId,
    pub type_data: UnitTypeData,
    pub name: String,
    // pub race: Race,
    // pub tag: i64,
    // pub is_structure: bool,
    pub is_light: bool,
    pub is_armored: bool,
    pub is_biological: bool,
    pub is_mechanical: bool,
    pub is_massive: bool,
    pub is_psionic: bool,
    pub weapons: Option<Vec<Weapon>>,
    // pub can_attack_both: bool,
    // pub can_attack_ground: bool,
    // pub can_attack_air: bool,
    pub ground_dps: f32,
    pub ground_range: f32,
    pub air_dps: f32,
    pub air_range: f32,
    pub armor: f32,
    // pub sight_range: f32,
    pub movement_speed: f32,
    pub health: f32,
    pub health_max: f32,
    pub shield: f32,
    pub shield_max: f32,
    pub energy: f32,
    pub energy_max: f32,
    // pub is_mine: bool,
    // pub is_enemy: bool,
    // pub owner_id: i64,
    pub radius: f32,
    // pub cloak: CloakState,
    // pub is_revealed: bool,
    // pub can_be_attacked: bool,
    // pub buffs: FxHashSet<BuffId>,
    pub is_flying: bool,
    pub attack_upgrade_level: i64,
    pub armor_upgrade_level: i64,
    pub shield_upgrade_level: i64,
    // pub buff_duration_remain: i64,
    // pub buff_duration_max: i64,
    // pub is_idle: bool,
    // pub is_moving: bool,
    // pub is_attacking: bool,
    // pub is_gathering: bool,
    // pub is_returning: bool,
    // pub is_collecting: bool,
    // pub is_constructing_scv: bool,
    // pub is_transforming: bool,
    // pub is_repairing: bool,
    // pub weapon_cooldown: f32,
    pub buff_timer: f32,
}
impl<'source> FromPyObject<'source> for CombatUnit {
    fn extract(obj: &'source PyAny) -> PyResult<Self> {
        let type_id: UnitTypeId = obj.getattr("type_id")?.extract()?;
        let mut cache = UNIT_CACHE.lock().unwrap();
        if let Some(x) = cache.get(&type_id) {
            Ok(Self {
                type_id,
                type_data: x.type_data.clone(),
                name: x.name.clone(),
                is_light: x.is_light,
                is_armored: x.is_armored,
                is_biological: x.is_biological,
                is_mechanical: x.is_mechanical,
                is_massive: x.is_massive,
                is_psionic: x.is_psionic,
                weapons: x.weapons.clone(),
                ground_dps: x.ground_dps,
                ground_range: x.ground_range,
                air_dps: x.air_dps,
                air_range: x.air_range,
                armor: x.armor,
                movement_speed: x.movement_speed,
                health: obj.getattr("health")?.extract()?,
                health_max: x.health_max,
                shield: obj.getattr("shield")?.extract()?,
                shield_max: x.shield_max,
                energy: obj.getattr("energy")?.extract()?,
                energy_max: x.energy_max,
                radius: x.radius,
                is_flying: x.is_flying,
                attack_upgrade_level: obj.getattr("attack_upgrade_level")?.extract()?,
                armor_upgrade_level: obj.getattr("armor_upgrade_level")?.extract()?,
                buff_timer: 0.0,
                shield_upgrade_level: obj.getattr("shield_upgrade_level")?.extract()?,
            })
        } else {
            let mut cu = Self {
                type_id,
                type_data: obj.getattr("_type_data")?.extract()?,
                name: obj.getattr("name")?.extract()?,
                // race: (),
                // tag: obj.getattr("tag")?.extract()?,
                // is_structure: obj.getattr("is_structure")?.extract()?,
                is_light: obj.getattr("is_light")?.extract()?,
                is_armored: obj.getattr("is_armored")?.extract()?,
                is_biological: obj.getattr("is_biological")?.extract()?,
                is_mechanical: obj.getattr("is_mechanical")?.extract()?,
                is_massive: obj.getattr("is_massive")?.extract()?,
                is_psionic: obj.getattr("is_psionic")?.extract()?,
                weapons: obj.getattr("_weapons")?.extract()?,
                // can_attack_both: obj.getattr( "can_attack_both")?.extract()?,
                // can_attack_ground: obj.getattr( "can_attack_ground")?.extract()?,
                // can_attack_air: obj.getattr( "can_attack_air")?.extract()?,
                ground_dps: obj.getattr("ground_dps")?.extract()?,
                ground_range: obj.getattr("ground_range")?.extract()?,
                air_dps: obj.getattr("air_dps")?.extract()?,
                air_range: obj.getattr("air_range")?.extract()?,
                // bonus_damage: None,
                armor: obj.getattr("armor")?.extract()?,
                // sight_range: obj.getattr(py, "sight_range")?.extract(py)?,
                movement_speed: obj.getattr("movement_speed")?.extract()?,
                health: obj.getattr("health")?.extract()?,
                health_max: obj.getattr("health_max")?.extract()?,
                shield: obj.getattr("shield")?.extract()?,
                shield_max: obj.getattr("shield_max")?.extract()?,
                energy: obj.getattr("energy")?.extract()?,
                energy_max: obj.getattr("energy_max")?.extract()?,
                // // alliance: (),
                // is_mine: obj.getattr( "is_mine")?.extract()?,
                // is_enemy: obj.getattr( "is_enemy")?.extract()?,
                // owner_id: obj.getattr( "owner_id")?.extract()?,
                radius: obj.getattr("radius")?.extract()?,
                // is_revealed: obj.getattr( "is_revealed")?.extract()?,
                // can_be_attacked: obj.getattr( "can_be_attacked")?.extract()?,
                // buffs: Default::default(),
                is_flying: obj.getattr("is_flying")?.extract()?,
                attack_upgrade_level: obj.getattr("attack_upgrade_level")?.extract()?,
                armor_upgrade_level: obj.getattr("armor_upgrade_level")?.extract()?,
                shield_upgrade_level: obj.getattr("shield_upgrade_level")?.extract()?,
                // buff_duration_remain: obj.getattr( "buff_duration_remain")?.extract()?,
                // buff_duration_max: obj.getattr( "buff_duration_max")?.extract()?,
                // is_idle: obj.getattr( "is_idle")?.extract()?,
                // is_moving: obj.getattr( "is_moving")?.extract()?,
                // is_attacking: obj.getattr( "is_attacking")?.extract()?,
                // is_gathering: obj.getattr(py, "is_gathering")?.extract(py)?,
                // is_returning: obj.getattr(py, "is_returning")?.extract(py)?,
                // is_collecting: obj.getattr(py, "is_collecting")?.extract(py)?,
                // is_constructing_scv: obj.getattr(py, "is_constructing_scv")?.extract(py)?,
                // is_transforming: obj.getattr(py, "is_transforming")?.extract(py)?,
                // is_repairing: obj.getattr(py, "is_repairing")?.extract(py)?,
                // weapon_cooldown: obj.getattr( "weapon_cooldown")?.extract()?,
                buff_timer: 0.0,
            };
            if type_id == UnitTypeId::BATTLECRUISER {
                cu.weapons = Some(Weapon::battlecruiser());
                cu.ground_dps = 35.714_287;
                cu.air_dps = 22.321_428;
            }
            cache.insert(type_id, cu.clone());
            Ok(cu)
        }
    }
}
impl CombatUnit {
    pub fn get_max_range(&self) -> f32 {
        if self.air_range > self.ground_range {
            self.air_range
        } else {
            self.ground_range
        }
    }
    pub fn can_be_attacked_by_air(&self) -> bool {
        self.is_flying || self.type_id == UnitTypeId::COLOSSUS
    }
    pub fn is_basic_harvester(&self) -> bool {
        IS_BASIC_HARVESTER.contains(&self.type_id)
    }
    pub fn modify_health(&mut self, mut delta: f32) {
        if delta < 0.0 {
            delta = -delta;
            self.shield -= delta;
            if self.shield < 0.0 {
                delta = -self.shield;
                self.shield = 0.0;
                self.health += -delta;
                if self.health < 0.0 {
                    self.health = 0.0;
                }
            }
        } else {
            self.health += delta;
            if self.health > self.health_max {
                self.health = self.health_max;
            }
        }
    }
    pub fn get_adjusted_cost(&self) -> i32 {
        self.get_mineral_cost() + (VESPENE_MULTIPLIER * self.get_vespene_cost() as f32) as i32
    }
    pub fn get_mineral_cost(&self) -> i32 {
        self.type_data.cost.minerals
    }
    pub fn get_vespene_cost(&self) -> i32 {
        self.type_data.cost.vespene
    }
    pub fn get_max_dps(&self) -> f32 {
        if self.air_dps > self.ground_dps {
            self.air_dps
        } else {
            self.ground_dps
        }
    }
    pub fn get_dps(&self, air: bool) -> f32 {
        if air {
            self.air_dps
        } else {
            self.ground_dps
        }
    }
    pub fn can_attack(&self) -> bool {
        if let Some(weapons) = &self.weapons {
            !weapons.is_empty()
        } else {
            false
        }
    }
    pub fn can_attack_ground(&self) -> bool {
        if self.type_id == UnitTypeId::ORACLE {
            true
        } else if let Some(weapons) = &self.weapons {
            weapons.iter().any(|x| TARGET_GROUND.contains(&x.w_type))
        } else {
            false
        }
    }
    pub fn can_attack_air(&self) -> bool {
        if let Some(weapons) = &self.weapons {
            weapons.iter().any(|x| TARGET_AIR.contains(&x.w_type))
        } else {
            false
        }
    }
    pub fn air_weapons(&self) -> Option<&Weapon> {
        match &self.weapons {
            Some(weapons) => {
                for weapon in weapons {
                    if weapon.w_type == WeaponTargetType::AIR
                        || weapon.w_type == WeaponTargetType::ANY
                    {
                        return Some(&weapon);
                    }
                }
            }
            None => return None,
        }
        None
    }
    pub fn is_melee(&self) -> bool {
        IS_MELEE.contains(&self.type_id)
    }
    pub fn ground_weapons(&self) -> Option<&Weapon> {
        match &self.weapons {
            Some(weapons) => {
                for weapon in weapons {
                    if weapon.w_type == WeaponTargetType::GROUND
                        || weapon.w_type == WeaponTargetType::ANY
                    {
                        return Some(&weapon);
                    }
                }
            }
            None => {
                return None;
            }
        }
        None
    }
}
