use pyo3::prelude::*;
//use dict_derive::{FromPyObject, IntoPyObject};
use crate::num_traits::FromPrimitive;
use crate::generated_enums::{UnitTypeId, UpgradeId};
use crate::game_info::{UnitTypeData, GameInfo, WeaponTargetType, WeaponInfo, CombatUpgrades};
use sc2_techtree::{TechData, UnitType};
//use std::any::Any;
//use std::collections::HashSet;
use std::borrow::Borrow;
//use std::cmp::max;
//use crate::generated_enums::UnitTypeId::WEAPON;
//use crate::generated_enums::UnitTypeId::WEAPON;

//use pyo3::types::PyAny;
//use crate::game_info::{UnitInfo};
//use pyo3::types::PyAny;
pub const IS_MELEE: [UnitTypeId; 12] =[
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
];

pub const IS_BASIC_HARVESTER: [UnitTypeId; 3] =[
    UnitTypeId::SCV,
    UnitTypeId::PROBE,
    UnitTypeId::DRONE
];

pub const IS_UPGRADE_WITH_LEVELS: [UpgradeId; 15] =[
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
    UpgradeId::TERRANVEHICLEANDSHIPARMORSLEVEL1];

#[pyclass]
#[derive(Clone, Debug)]
pub struct CombatUnit {
    #[pyo3(get,set)]
    pub owner: i32,
    #[pyo3(get,set)]
    pub unit_type: UnitTypeId,
    #[pyo3(get,set)]
    pub health: f32,
    #[pyo3(get,set)]
    pub health_max: f32,
    #[pyo3(get,set)]
    pub shield: f32,
    #[pyo3(get,set)]
    pub shield_max: f32,
    #[pyo3(get,set)]
    pub energy: f32,
    #[pyo3(get,set)]
    pub is_flying: bool,
    #[pyo3(get,set)]
    pub buff_timer: f32,
    pub type_data: Option<UnitTypeData>,
    pub tech_data: Option<UnitType>,
    pub air_weapons: Option<WeaponInfo>,
    pub ground_weapons: Option<WeaponInfo>


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
            buff_timer: self.buff_timer,
            type_data: self.type_data.clone(),
            tech_data: self.tech_data.clone(),
            air_weapons: self.air_weapons.clone(),
            ground_weapons: self.ground_weapons.clone()
        }
    }
//    #[args(_owner, _unit_type, _health, _health_max=0.0, _shield, _shield_max=0.0, _energy=0.0, _flying, _buff_timer=0.0)]
    #[new]
     fn new(obj: &PyRawObject,
            _owner: i32,
            _unit_type: i32,
            _health: f32,
            mut _health_max: Option<f32>,
            _shield:f32,
            mut _shield_max:Option<f32>,
            mut _energy:Option<f32>,
            _flying:bool,
            mut _buff_timer:Option<f32>,
        ){
        obj.init(CombatUnit{
             owner: _owner,
             unit_type: UnitTypeId::from_i32(_unit_type).unwrap_or_default(),
             health: _health,
             is_flying: _flying,
             buff_timer:_buff_timer.get_or_insert(0.0).to_owned(),
             energy:_energy.get_or_insert(0.0).to_owned(),
             health_max: _health_max.get_or_insert(_health).to_owned(),
             shield_max: _shield_max.get_or_insert(_shield).to_owned(),
             shield:_shield,
             type_data: None,
             tech_data: None,
             air_weapons: None,
             ground_weapons:None
        })
     }
    fn show_unit_type(&self)-> PyResult<String> {
        Ok(self.unit_type.to_string())
    }


}
impl CombatUnit{
    pub fn can_be_attacked_by_air_weapons(&self)-> bool{
        self.is_flying || self.unit_type == UnitTypeId::COLOSSUS
    }

    pub fn get_movement_speed(&self)->f32{
//        let movement_speed =;
        self.type_data.as_ref().unwrap().get_movement_speed()
    }
    pub fn get_radius(&self)->f32{
        self.tech_data.as_ref().unwrap().radius.into()
    }

    pub fn get_attack_range(&self)->f32{
        let mut range = 0.0;
        for w in self.type_data.as_ref().unwrap().get_weapons(){
            if w.get_range() > range{
                range = w.get_range();
            }
        }
        range
    }

    pub fn get_name(&self)-> &str{
        self.type_data.as_ref().unwrap().get_name()
    }

    pub fn is_melee(&self)-> bool{
        IS_MELEE.contains(self.unit_type.borrow())
    }

    pub fn is_basic_harvester(&self)-> bool{
        IS_BASIC_HARVESTER.contains(self.unit_type.borrow())
    }

    pub fn load_data(&mut self, data: &GameInfo, tech_tree: &TechData){
        self.tech_data = match tech_tree.unittype(self.unit_type.to_tt()){
            None => None,
            Some(t) => Some(t)
        };
        self.type_data = match data.get_unit_data(self.unit_type){
            None => None,
            Some(t) => Some(t.clone())
        };
        self.init_weapons()

    }
    pub fn init_weapons(&mut self){
        for weapon in self.type_data.as_ref().unwrap().get_weapons(){
            let target_type: WeaponTargetType = weapon.get_target_type();
            if target_type == WeaponTargetType::AIR || target_type == WeaponTargetType::ANY{
                self.air_weapons = Some(WeaponInfo::new(weapon, self.unit_type, None, None, self.type_data.as_ref().unwrap().borrow(), self.tech_data.as_ref().unwrap().borrow()));
            }
            if target_type == WeaponTargetType::GROUND || target_type == WeaponTargetType::ANY{
                self.ground_weapons = Some(WeaponInfo::new(weapon, self.unit_type, None, None, self.type_data.as_ref().unwrap().borrow(), self.tech_data.as_ref().unwrap().borrow()));
            }
        }
    }
    pub fn get_dps(&self, air:bool)-> f32{
        let air_weapon_dps:f32 = match &self.air_weapons{
            None => 0.0,
            Some(t) => t.get_dps()
        };
        let ground_weapon_dps:f32 = match &self.ground_weapons {
            None => 0.0,
            Some(t) => t.get_dps()
        };
        if air{
            air_weapon_dps
        }
        else{
            ground_weapon_dps
        }

//        if ground_weapon_dps > air_weapon_dps {ground_weapon_dps} else {air_weapon_dps}
    }

    pub fn modify_health(&mut self, mut delta: f32){
        if delta < 0.0 {
            delta = -delta;
            self.shield -= delta;
            if self.shield < 0.0{
                delta = -self.shield;
                self.shield = 0.0;
                self.health += -delta;
                if self.health < 0.0{
                    self.health = 0.0;
                }
            }
        }
        else{
            self.health += delta;
            if self.health > self.health_max{
                self.health = self.health_max;
            }

        }
    }
}
//
//impl ToPyObject for CombatUnit{
//	fn to_object(&self, py: Python) -> PyObject {
//		self.to_object(py)
//	}
//}
//
//impl FromPy<CombatUnit> for PyObject {
//    fn from_py(other: CombatUnit, py: Python) -> Self {
//        let _other: CombatUnit = other;
//        _other.into_py(py)
//    }
//}
//#[derive(Clone)]
//pub struct CombatUnits(Vec<CombatUnit>);
//impl<'source> FromPyObject<'source> for CombatUnits {
//    fn extract(ob: &'source PyAny) -> PyResult<Self> {
//    }
//}
//impl IntoIterator for CombatUnits {
//    type Item = CombatUnit;
//    type IntoIter = std::vec::IntoIter<Self::Item>;
//
//    fn into_iter(self) -> Self::IntoIter {
//        self.0.into_iter()
//    }
//}

pub fn clone_vec(vec: Vec<&CombatUnit>) -> Vec<CombatUnit> {
    vec.into_iter().map(|f| f.dup()).collect()
    }

struct CombatUnitInfo{
    ground_weapon: Option<WeaponInfo>,
    air_weapon: Option<WeaponInfo>
}
impl CombatUnitInfo{
    pub fn new(_type: UnitTypeId, _upgrades: &CombatUpgrades, _target_upgrades: &CombatUpgrades, _data: &GameInfo, _tech_tree: &TechData)-> Self{
        let data: &UnitTypeData = _data.get_unit_data(_type).unwrap();
        let tech_tree = _tech_tree.unittype(_type.to_tt()).unwrap();
        let mut _air_weapon: Option<WeaponInfo> = None;
        let mut _ground_weapon: Option<WeaponInfo>=None;
        for weapon in data.get_weapons(){
            if weapon.get_target_type() == WeaponTargetType::ANY || weapon.get_target_type() == WeaponTargetType::AIR{
                _air_weapon = Some(WeaponInfo::new(weapon, _type, Some(_upgrades), Some(_target_upgrades), data, &tech_tree));
            }
            if weapon.get_target_type() == WeaponTargetType::ANY || weapon.get_target_type() == WeaponTargetType::GROUND{
                _ground_weapon = Some(WeaponInfo::new(weapon, _type, Some(_upgrades), Some(_target_upgrades), data, &tech_tree));
            }
        }
        CombatUnitInfo{
            ground_weapon: _air_weapon,
            air_weapon: _ground_weapon
        }
    }

    pub fn attack_interval(&self) -> f32{
        let a: f32 = match &self.air_weapon {
            None => 0.0,
            Some(t) => t.get_speed()
        };
        let g: f32 = match &self.ground_weapon{
            None => 0.0,
            Some(t) => t.get_speed()
        };
        if a > g{
            a
        }
        else{
            g
        }
    }
}
//#[pyclass]
//#[derive(Clone)]
//// #[derive(FromPyObject, IntoPyObject)]
//pub struct CombatUnits{
//    pub units: Vec<CombatUnit>
//}
//
//#[pymethods]
//impl CombatUnits{
//    #[new]
//    fn new(obj: &PyRawObject, _units1:  Vec<&CombatUnit>){
//        let new_vec: Vec<CombatUnit> = clone_vec(_units1);
//        obj.init(CombatUnits{units: new_vec})
//
//    }
//    fn len(&self)-> PyResult<usize>{
//       Ok(self.units.len())
//    }
//
//    fn clear(&mut self){
//        self.units = Vec::<CombatUnit>::new()
//    }
//    #[getter]
//    fn get_units(&mut self)->PyResult<Vec<CombatUnit>>{
//        Ok(self.units.clone())
//    }
//}