use pyo3::prelude::*;
//use dict_derive::{FromPyObject, IntoPyObject};
use crate::num_traits::FromPrimitive;
use crate::generated_enums::{UnitTypeId, UpgradeId};
use crate::game_info::{UnitTypeData, GameInfo, WeaponTargetType, WeaponInfo, CombatUpgrades, get_tech_data, Weapon};
use sc2_techtree::{TechData, UnitType};
//use std::any::Any;
//use std::collections::HashSet;
use std::borrow::Borrow;
use cache_macro::cache;
use lru_cache::LruCache;
use std::collections::HashSet;
use stopwatch::Stopwatch;
use std::hash::{Hash, Hasher};
use std::f32::EPSILON;
//use std::cmp::max;
//use crate::generated_enums::UnitTypeId::WEAPON;
//use crate::generated_enums::UnitTypeId::WEAPON;

//use pyo3::types::PyAny;
//use crate::game_info::{UnitInfo};
//use pyo3::types::PyAny;
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
    ].iter().cloned().collect();


    pub static ref IS_BASIC_HARVESTER: HashSet<UnitTypeId> =[
        UnitTypeId::SCV,
        UnitTypeId::PROBE,
        UnitTypeId::DRONE
    ].iter().cloned().collect();

    pub static ref IS_UPGRADE_WITH_LEVELS: HashSet<UpgradeId> =[
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
        UpgradeId::TERRANVEHICLEANDSHIPARMORSLEVEL1].iter().cloned().collect();
    }

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
    pub air_weapons: Option<WeaponInfo>,
    pub tech_data: Option<UnitType>,
    pub type_data: Option<UnitTypeData>,
    pub ground_weapons: Option<WeaponInfo>,
    movement_speed: Option<f32>,
    unit_radius: Option<f32>



}
impl Hash for CombatUnit {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.unit_type.hash(state);
        self.owner.hash(state);
    }
}
impl PartialEq for CombatUnit {
    fn eq(&self, other: &Self) -> bool {
        self.owner == other.owner &&
            self.unit_type == other.unit_type &&
            (self.health - other.health).abs() < EPSILON
    }
}
impl Eq for CombatUnit {}

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
            air_weapons: self.air_weapons.clone(),
            tech_data: None,
            type_data: None,
            ground_weapons: self.ground_weapons.clone(),
            movement_speed: None,
            unit_radius: None
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
             air_weapons: None,
            tech_data: None,
            type_data: None,
            ground_weapons:None,
            movement_speed: None,
            unit_radius: None
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
        self.type_data.as_ref().unwrap().get_movement_speed()

    }
    pub fn get_radius(&self)->f32{
        self.tech_data.as_ref().unwrap().radius.into()
    }

    pub fn get_attack_range(&self)->f32{
        let range = 0.0;
        let ground_range:f32 = match &self.ground_weapons{
            Some(t) => t.get_range(),
            None => 0.0
        };
        let air_range: f32 = match &self.air_weapons{
            Some(t) => t.get_range(),
            None => 0.0,
        };
        ground_range.max(air_range)
    }

    pub fn get_name(&self)-> &str{
        self.type_data.as_ref().unwrap().get_name()
    }
    #[inline]
    pub fn is_melee(&self)-> bool{
        IS_MELEE.contains(self.unit_type.borrow())
    }

    #[inline]
    pub fn is_basic_harvester(&self)-> bool{
        IS_BASIC_HARVESTER.contains(self.unit_type.borrow())
    }

    pub fn load_data(&mut self, data: &GameInfo, tech_tree: &TechData, _upgrades: Option<&CombatUpgrades>, _target_upgrades: Option<&CombatUpgrades>, unit_types_scope: &HashSet<usize>){
//        let sw1 = Stopwatch::start_new();
        let upgrades: CombatUpgrades = match _upgrades{
                None => CombatUpgrades::new(vec![]),
                Some(t)=> t.clone()
            };

        let target_upgrades: CombatUpgrades = match _target_upgrades{
            None => CombatUpgrades::new(vec![]),
            Some(t) => t.clone()

        };

        self.tech_data = Some(get_tech_data(self.unit_type, tech_tree));


        self.type_data = match data.get_unit_data(self.unit_type){

            Some(t) => Some(t.clone()),
            None => {
                println!("Unit data for {:?} is none", self.unit_type);
                None
            },
        };
//        let before_init_weapons = sw1.elapsed();
//        let sw2 = Stopwatch::start_new();
        self.init_weapons(data,tech_tree, &upgrades, &target_upgrades, unit_types_scope);
//        let after_init_weapons = sw2.elapsed();
//        let total_time = sw1.elapsed();
//        println!("Total time before init_weapons {:?}", before_init_weapons);
//        println!("Total time in init_weapons {:?}", after_init_weapons);
//        println!("Total time in load_data {:?}", total_time);

    }
    pub fn init_weapons(&mut self, _data: &GameInfo, _tech_tree: &TechData, _upgrades: &CombatUpgrades,_target_upgrades: &CombatUpgrades, unit_types_scope: &HashSet<usize>){
//        println!("Loading weapons for {:?}, type_data: {:?}, tech_data: {:?}", self.unit_type, self.type_data.is_some(), self.tech_data.is_some());
        let unwrapped_type_data = self.type_data.as_ref().unwrap();

        let unwrapped_tech_data = self.tech_data.as_ref().unwrap();



        for weapon in unwrapped_type_data.get_weapons(){

            let target_type: WeaponTargetType = weapon.get_target_type();
            if target_type == WeaponTargetType::AIR || target_type == WeaponTargetType::ANY{
                self.air_weapons = Some(get_new_weapons(weapon,
                                                        self.unit_type,
                                                        _upgrades,
                                                        _target_upgrades,
                                                        unwrapped_type_data,
                                                        unwrapped_tech_data,
                                                        _data,
                                                        _tech_tree,
                                                        unit_types_scope, true));
            }

            if target_type == WeaponTargetType::GROUND || target_type == WeaponTargetType::ANY{
                self.ground_weapons = Some(get_new_weapons(weapon,
                                                        self.unit_type,
                                                        _upgrades,
                                                        _target_upgrades,
                                                        unwrapped_type_data,
                                                        unwrapped_tech_data,
                                                        _data,
                                                        _tech_tree,
                                                        unit_types_scope, false));
            }
        }
//        println!("Loading weapons complete");
    }
    pub fn get_dps(&self, air:bool)-> f32 {
        if air {
            let air_weapon_dps: f32 = match &self.air_weapons {
                Some(t) => t.base_dps,
                None => 0.0,
            };
             air_weapon_dps
        }
        else {
            let ground_weapon_dps: f32 = match &self.ground_weapons {
                Some(t) => t.base_dps,
                None => 0.0,
            };

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


pub fn clone_vec(vec: Vec<&CombatUnit>) -> Vec<CombatUnit> {
    vec.into_iter().map(|f| f.clone()).collect()
    }

#[cache(LruCache : LruCache::new(100))]
#[cache_cfg(ignore_args = _weapon, _tech_tree, _data, _unit_data, _unit_tech_tree, _unit_types_scope)]
#[cache_cfg(thread_local)]
pub fn get_new_weapons(_weapon: &Weapon, _type: UnitTypeId, _upgrades: &CombatUpgrades,
               _target_upgrades: &CombatUpgrades, _unit_data: &UnitTypeData,
               _unit_tech_tree: &UnitType, _data: &GameInfo,_tech_tree: &TechData, _unit_types_scope: &HashSet<usize>, air: bool) -> WeaponInfo{

    WeaponInfo::new(_weapon, _type, _upgrades,
               _target_upgrades, _unit_data,
               _unit_tech_tree, _data,_tech_tree, _unit_types_scope)
}