use crate::game_info::{
    get_tech_data, get_unit_data, CombatUpgrades, GameInfo, UnitTypeData, Weapon, WeaponInfo,
    WeaponTargetType,
};
use crate::generated_enums::{UnitTypeId, UpgradeId};
use crate::num_traits::FromPrimitive;
use pyo3::prelude::*;
use rayon::prelude::*;
use sc2_techtree::{TechData, UnitType};
use std::borrow::Borrow;
use std::collections::{HashSet};
use std::f32::EPSILON;
use std::hash::{Hash, Hasher};
use rustc_hash::FxHashSet;

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
}

#[pyclass]
#[derive(Clone, Debug)]
pub struct CombatUnit {
    #[pyo3(get, set)]
    pub owner: i32,
    #[pyo3(get, set)]
    pub unit_type: UnitTypeId,
    #[pyo3(get, set)]
    pub health: f32,
    #[pyo3(get, set)]
    pub health_max: f32,
    #[pyo3(get, set)]
    pub shield: f32,
    #[pyo3(get, set)]
    pub shield_max: f32,
    #[pyo3(get, set)]
    pub energy: f32,
    #[pyo3(get, set)]
    pub is_flying: bool,
    #[pyo3(get, set)]
    pub buff_timer: f32,
    pub air_weapons: Option<WeaponInfo>,
    pub tech_data: Option<UnitType>,
    pub type_data: Option<UnitTypeData>,
    pub ground_weapons: Option<WeaponInfo>,
    movement_speed: Option<f32>,
    unit_radius: Option<f32>,
    can_be_attacked_by_air_weapons: Option<bool>,
    attack_range: Option<f32>,
    air_dps: Option<f32>,
    ground_dps: Option<f32>,
    max_dps: Option<f32>,
    mineral_cost: Option<f32>,
    vespene_cost: Option<f32>,
    adjusted_cost: Option<f32>,
}
impl Hash for CombatUnit {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.unit_type.hash(state);
        self.owner.hash(state);
    }
}
impl PartialEq for CombatUnit {
    fn eq(&self, other: &Self) -> bool {
        self.owner == other.owner
            && self.unit_type == other.unit_type
            && (self.health - other.health).abs() < EPSILON
    }
}
impl Eq for CombatUnit {}

impl CombatUnit {
    pub(crate) fn nop_new(
        _owner: i32,
        _unit_type: UnitTypeId,
        _health: f32,
        mut _health_max: f32,
        _shield: f32,
        mut _shield_max: f32,
        mut _energy: f32,
        _flying: bool,
        mut _buff_timer: f32,
        _type_data: Option<UnitTypeData>,
    ) -> Self {
        CombatUnit {
            owner: _owner,
            unit_type: _unit_type,
            health: _health,
            is_flying: _flying,
            buff_timer: _buff_timer,
            energy: _energy,
            health_max: _health_max,
            shield_max: _shield_max,
            shield: _shield,
            air_weapons: None,
            tech_data: None,
            type_data: _type_data,
            ground_weapons: None,
            movement_speed: None,
            unit_radius: None,
            can_be_attacked_by_air_weapons: None,
            attack_range: None,
            max_dps: None,
            mineral_cost: None,
            air_dps: None,
            ground_dps: None,
            vespene_cost: None,
            adjusted_cost: None,
        }
    }
}
#[pymethods]
impl CombatUnit {
    fn dup(&self) -> Self {
        CombatUnit {
            owner: self.owner,
            unit_type: self.unit_type,
            health: self.health,
            health_max: self.health_max,
            shield: self.shield,
            shield_max: self.shield_max,
            energy: self.energy,
            is_flying: self.is_flying,
            buff_timer: self.buff_timer,
            air_weapons: self.air_weapons.clone(),
            tech_data: None,
            type_data: None,
            ground_weapons: self.ground_weapons.clone(),
            movement_speed: None,
            unit_radius: None,
            can_be_attacked_by_air_weapons: None,
            attack_range: None,
            air_dps: None,
            ground_dps: None,
            max_dps: None,
            mineral_cost: None,
            vespene_cost: None,
            adjusted_cost: None,
        }
    }
    #[new]
    fn new(
        obj: &PyRawObject,
        _owner: i32,
        _unit_type: i32,
        _health: f32,
        mut _health_max: Option<f32>,
        _shield: f32,
        mut _shield_max: Option<f32>,
        mut _energy: Option<f32>,
        _flying: bool,
        mut _buff_timer: Option<f32>,
        _type_data: UnitTypeData,
    ) {
        obj.init(CombatUnit {
            owner: _owner,
            unit_type: UnitTypeId::from_i32(_unit_type).unwrap_or_default(),
            health: _health,
            is_flying: _flying,
            buff_timer: _buff_timer.get_or_insert(0.0).to_owned(),
            energy: _energy.get_or_insert(0.0).to_owned(),
            health_max: _health_max.get_or_insert(_health).to_owned(),
            shield_max: _shield_max.get_or_insert(_shield).to_owned(),
            shield: _shield,
            air_weapons: None,
            tech_data: None,
            type_data: Some(_type_data),
            ground_weapons: None,
            movement_speed: None,
            unit_radius: None,
            can_be_attacked_by_air_weapons: None,
            attack_range: None,
            air_dps: None,
            ground_dps: None,
            max_dps: None,
            mineral_cost: None,
            vespene_cost: None,
            adjusted_cost: None,
        })
    }
    fn show_unit_type(&self) -> PyResult<String> {
        Ok(self.unit_type.to_string())
    }
}
impl CombatUnit {
    pub fn can_be_attacked_by_air_weapons(&self) -> bool {
        self.can_be_attacked_by_air_weapons.unwrap()
    }

    pub fn get_movement_speed(&self) -> f32 {
        self.movement_speed.unwrap()
    }

    pub fn get_radius(&self) -> f32 {
        self.unit_radius.unwrap()
    }

    pub fn get_attack_range(&self) -> f32 {
        self.attack_range.unwrap()
    }

    pub fn get_name(&self) -> &str {
        self.type_data.as_ref().unwrap().get_name()
    }

    pub fn is_melee(&self) -> bool {
        IS_MELEE.contains(self.unit_type.borrow())
    }

    pub fn is_basic_harvester(&self) -> bool {
        IS_BASIC_HARVESTER.contains(self.unit_type.borrow())
    }

    pub fn load_attributes(&mut self) {
        match self.movement_speed {
            None => {
                self.movement_speed = Some(self.type_data.as_ref().unwrap().get_movement_speed());
            }
            _ => {}
        }

        match self.attack_range {
            None => {
                let ground_range: f32 = match &self.ground_weapons {
                    Some(t) => t.get_range(),
                    None => 0.0,
                };
                let air_range: f32 = match &self.air_weapons {
                    Some(t) => t.get_range(),
                    None => 0.0,
                };
                self.attack_range = Some(ground_range.max(air_range));
            }
            _ => {}
        };

        match self.can_be_attacked_by_air_weapons {
            None => {
                self.can_be_attacked_by_air_weapons =
                    Some(self.is_flying || self.unit_type == UnitTypeId::COLOSSUS)
            }
            _ => {}
        };
        match self.unit_radius {
            None => {
                self.unit_radius = Some(self.tech_data.as_ref().unwrap().radius.into());
            }
            _ => {}
        };
        match self.air_dps {
            None => {
                let air_weapon_dps: f32 = match &self.air_weapons {
                    Some(t) => t.base_dps,
                    None => 0.0,
                };
                self.air_dps = Some(air_weapon_dps)
            }
            _ => {}
        }

        match self.ground_dps {
            None => {
                let ground_weapon_dps: f32 = match &self.ground_weapons {
                    Some(t) => t.base_dps,
                    None => 0.0,
                };

                self.ground_dps = Some(ground_weapon_dps)
            }
            _ => {}
        }
        match self.max_dps {
            None => self.max_dps = Some(self.air_dps.unwrap().max(self.ground_dps.unwrap())),
            _ => {}
        }
        match self.mineral_cost {
            None => {
                self.mineral_cost = Some(self.type_data.as_ref().unwrap().get_mineral_cost() as f32)
            }
            _ => {}
        }
        match self.vespene_cost {
            None => {
                self.vespene_cost = Some(self.type_data.as_ref().unwrap().get_vespene_cost() as f32)
            }
            _ => {}
        }
        match self.adjusted_cost {
            None => {
                self.adjusted_cost =
                    Some(self.get_mineral_cost() + VESPENE_MULTIPLIER * self.get_vespene_cost())
            }
            _ => {}
        }
    }

    pub fn get_adjusted_cost(&self) -> f32 {
        self.adjusted_cost.unwrap()
    }
    pub fn get_max_dps(&self) -> f32 {
        self.max_dps.unwrap()
    }

    pub fn get_mineral_cost(&self) -> f32 {
        self.mineral_cost.unwrap()
    }
    pub fn get_vespene_cost(&self) -> f32 {
        self.vespene_cost.unwrap()
    }
    pub fn load_data(
        &mut self,
        data: &GameInfo,
        tech_tree: &TechData,
        _upgrades: Option<&CombatUpgrades>,
        _target_upgrades: Option<&CombatUpgrades>,
        unit_types_scope: &FxHashSet<usize>,
        multi_threaded: bool,
    ) {
        //        let data = &data;
        //        let tech_tree = &tech_tree;
        let upgrades: CombatUpgrades = match _upgrades {
            None => CombatUpgrades::new(vec![]),
            Some(t) => t.clone(),
        };

        let target_upgrades: CombatUpgrades = match _target_upgrades {
            None => CombatUpgrades::new(vec![]),
            Some(t) => t.clone(),
        };

        self.tech_data = Some(get_tech_data(self.unit_type, tech_tree));

        if self.type_data.is_none() {
            self.type_data = Some(get_unit_data(self.unit_type, data));
            //                match data.get_unit_data(self.unit_type) {
            //                Some(t) => Some(t.clone()),
            //                None => {
            //                    println!("Unit data for {:?} is none", self.unit_type);
            //                    None
            //                }
            //            }
        }
        self.init_weapons(
            data,
            tech_tree,
            &upgrades,
            &target_upgrades,
            unit_types_scope,
            multi_threaded,
        );
        self.load_attributes();
        //        let after_init_weapons = sw2.elapsed();
        //        let total_time = sw1.elapsed();
        //        println!("Total time before init_weapons {:?}", before_init_weapons);
        //        println!("Total time in init_weapons {:?}", after_init_weapons);
        //        println!("Total time in load_data {:?}", total_time);
    }

    pub fn init_weapons(
        &mut self,
        _data: &GameInfo,
        _tech_tree: &TechData,
        _upgrades: &CombatUpgrades,
        _target_upgrades: &CombatUpgrades,
        unit_types_scope: &FxHashSet<usize>,
        multi_threaded: bool,
    ) {
        //        println!("Loading weapons for {:?}, type_data: {:?}, tech_data: {:?}", self.unit_type, self.type_data.is_some(), self.tech_data.is_some());
        let unwrapped_type_data = self.type_data.as_ref().unwrap();

        let unwrapped_tech_data = self.tech_data.as_ref().unwrap();

        //        let mut air_weapons: Option<WeaponInfo> = None;
        //        let mut ground_weapons: Option<WeaponInfo> = None;
        //        self.ground_weapons = unwrapped_type_data.get_weapons().par_iter().map(|weapon|{
        ////            let target_type: WeaponTargetType = weapon.get_target_type();
        //            get_new_weapons(
        //                    weapon,
        //                    self.unit_type.clone(),
        //                    _upgrades,
        //                    _target_upgrades,
        //                    unwrapped_type_data,
        //                    unwrapped_tech_data,
        //                    _data,
        //                    _tech_tree,
        //                    unit_types_scope,
        //                )
        //
        //
        //        }).find_any(|x| x.get_target_type() == WeaponTargetType::GROUND || x.get_target_type() == WeaponTargetType::ANY);

        //            self.air_weapons =
        if multi_threaded {
            let weapons: Vec<(WeaponTargetType, Option<WeaponInfo>)> = unwrapped_type_data
                .get_weapons()
                .par_iter()
                .map(|weapon| {
                    (
                        weapon.get_target_type(),
                        Some(get_new_weapons(
                            weapon,
                            self.unit_type.clone(),
                            _upgrades,
                            _target_upgrades,
                            unwrapped_type_data,
                            unwrapped_tech_data,
                            _data,
                            _tech_tree,
                            unit_types_scope,
                            multi_threaded,
                        )),
                    )
                })
                .collect();
            for (target_type, weapon) in weapons {
                if target_type == WeaponTargetType::AIR || target_type == WeaponTargetType::ANY {
                    self.air_weapons = weapon.clone();
                };
                if target_type == WeaponTargetType::GROUND || target_type == WeaponTargetType::ANY {
                    self.ground_weapons = weapon.clone();
                }
            }
        }
        //         self.ground_weapons = unwrapped_type_data.get_weapons().par_iter().map(|weapon|{
        //            let target_type: WeaponTargetType = weapon.get_target_type();
        //             if target_type ==  WeaponTargetType::AIR || target_type == WeaponTargetType::ANY{
        //                 get_new_weapons(
        //                     weapon,
        //                     self.unit_type.clone(),
        //                     _upgrades,
        //                     _target_upgrades,
        //                     unwrapped_type_data,
        //                     unwrapped_tech_data,
        //                     _data,
        //                     _tech_tree,
        //                     unit_types_scope,
        //                 )
        //             }
        //             else{
        //                 None
        //             }
        //        }).collect();

        //        self.air_weapons = weapons.iter().filter(|x| x.get_target_type() == WeaponTargetType::AIR || x.get_target_type() == WeaponTargetType::ANY);
        //        self.ground_weapons= weapons.iter().filter(|x| x.get_target_type() == WeaponTargetType::GROUND || x.get_target_type() == WeaponTargetType::ANY)
        else {
            for weapon in unwrapped_type_data.get_weapons() {
                let target_type: WeaponTargetType = weapon.get_target_type();
                if target_type == WeaponTargetType::AIR || target_type == WeaponTargetType::ANY {
                    self.air_weapons = Some(get_new_weapons(
                        weapon,
                        self.unit_type,
                        _upgrades,
                        _target_upgrades,
                        unwrapped_type_data,
                        unwrapped_tech_data,
                        _data,
                        _tech_tree,
                        unit_types_scope,
                        multi_threaded,
                    ));
                }

                if target_type == WeaponTargetType::GROUND || target_type == WeaponTargetType::ANY {
                    self.ground_weapons = Some(get_new_weapons(
                        weapon,
                        self.unit_type,
                        _upgrades,
                        _target_upgrades,
                        unwrapped_type_data,
                        unwrapped_tech_data,
                        _data,
                        _tech_tree,
                        unit_types_scope,
                        multi_threaded,
                    ));
                }
            }
        }
        //        println!("Loading weapons complete");
    }

    pub fn get_dps(&self, air: bool) -> f32 {
        if air {
            self.air_dps.unwrap()
        } else {
            self.ground_dps.unwrap()
        }
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
}

pub fn clone_vec(vec: Vec<&CombatUnit>) -> Vec<CombatUnit> {
    //    let cache = LruCache::with_hasher(100,BuildHasherDefault::<FnvHasher>::default());
    //    BuildHasherDefault::<FxHasher>::default()
    vec.into_iter().cloned().collect()
}

//#[cache(LruCache : LruCache::new(1000))]
//#[cache_cfg(ignore_args = _weapon, _tech_tree, _data, _unit_data, _unit_tech_tree, _unit_types_scope)]
//#[cache_cfg(thread_local)]
pub fn get_new_weapons(
    _weapon: &Weapon,
    _type: UnitTypeId,
    _upgrades: &CombatUpgrades,
    _target_upgrades: &CombatUpgrades,
    _unit_data: &UnitTypeData,
    _unit_tech_tree: &UnitType,
    _data: &GameInfo,
    _tech_tree: &TechData,
    _unit_types_scope: &FxHashSet<usize>,
    multi_threaded: bool,
) -> WeaponInfo {
    WeaponInfo::new(
        _weapon,
        _type,
        _upgrades,
        _target_upgrades,
        _unit_data,
        _unit_tech_tree,
        _data,
        _tech_tree,
        _unit_types_scope,
        multi_threaded,
    )
}
