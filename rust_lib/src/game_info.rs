use crate::combat_unit::{CombatUnit, IS_MELEE};
use crate::generated_enums::{AbilityId, UnitTypeId, UpgradeId};
use crate::num_traits::{FromPrimitive, ToPrimitive};
use pyo3::derive_utils::IntoPyResult;
use pyo3::types::PyAny;
use pyo3::{
    FromPy, FromPyObject, IntoPy, ObjectProtocol, PyErr, PyObject, PyResult, Python, ToPyObject,
};
use rayon::prelude::*;
use sc2_techtree::UnitType;
use sc2_techtree::{Attribute as A, TechData};
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::f32::EPSILON;
use std::fmt;
use std::hash::Hash;
use rustc_hash::{FxHashSet, FxHashMap};

//cached_key! {
//    GET_TECH_DATA: SizedCache<UnitTypeId, UnitType> = SizedCache::with_size(50);
//    Key = {unit};
pub fn get_tech_data(unit: UnitTypeId, tech_tree: &TechData) -> UnitType {
    tech_tree.unittype(unit.to_tt()).unwrap()
}
//    }
//}
//cached_key! {
//    GET_UNIT_DATA: SizedCache<UnitTypeId, UnitTypeData> = SizedCache::with_size(50);
//    Key = {unit};
pub fn get_unit_data(unit: UnitTypeId, data: &GameInfo) -> UnitTypeData {
    data.get_unit_data(unit).unwrap().clone()
}
//}

#[allow(missing_docs)]
#[derive(Primitive, Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub enum Attribute {
    NULL = 0,
    LIGHT = 1,
    ARMORED = 2,
    BIOLOGICAL = 3,
    MECHANICAL = 4,
    ROBOTIC = 5,
    PSIONIC = 6,
    MASSIVE = 7,
    STRUCTURE = 8,
    HOVER = 9,
    HEROIC = 10,
    SUMMONED = 11,
}

impl Default for Attribute {
    fn default() -> Self {
        Attribute::NULL
    }
}

impl ToPyObject for Attribute {
    fn to_object(&self, py: Python) -> PyObject {
        self.to_i32().unwrap().to_object(py)
    }
}

impl FromPy<Attribute> for PyObject {
    fn from_py(other: Attribute, py: Python) -> Self {
        let _other: i32 = other.to_i32().unwrap();
        _other.into_py(py)
    }
}

impl<'source> FromPyObject<'source> for Attribute {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        //        println!("{:?}", "Extracting Attribute");
        let ob1: i32 = ob.extract::<i32>().unwrap();
        let x: Attribute = Attribute::from_i32(ob1).unwrap_or_default();
        Ok(x).into_py_result()
    }
}

impl Attribute {
    pub fn to_tt(self) -> A {
        match self {
            Attribute::LIGHT => A::Light,
            Attribute::ARMORED => A::Armored,
            Attribute::STRUCTURE => A::Structure,
            Attribute::MASSIVE => A::Massive,
            Attribute::BIOLOGICAL => A::Biological,
            Attribute::MECHANICAL => A::Mechanical,
            Attribute::PSIONIC => A::Psionic,
            Attribute::HEROIC => A::Heroic,
            Attribute::SUMMONED => A::Summoned,
            Attribute::ROBOTIC => A::Robotic,
            Attribute::HOVER => A::Hover,
            _ => A::Light,
        }
        //		A::new(self.to_u32().unwrap())
    }
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub struct DamageBonus {
    attribute: Attribute,
    bonus: f32,
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

impl ToPyObject for WeaponTargetType {
    fn to_object(&self, py: Python) -> PyObject {
        self.to_i32().unwrap().to_object(py)
    }
}

impl FromPy<WeaponTargetType> for PyObject {
    fn from_py(other: WeaponTargetType, py: Python) -> Self {
        let _other: i32 = other.to_i32().unwrap();
        _other.into_py(py)
    }
}

impl<'source> FromPyObject<'source> for WeaponTargetType {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        //        println!("{:?}", "Extracting WeaponTargetType");
        let ob1: i32 = ob.extract::<i32>().unwrap();
        let x: WeaponTargetType = WeaponTargetType::from_i32(ob1).unwrap_or_default();
        Ok(x).into_py_result()
    }
}
#[allow(missing_docs)]
#[derive(Primitive, Debug, Copy, Clone, PartialEq, Deserialize, Serialize)]
pub enum Race {
    NORACE = 0,
    TERRAN = 1,
    ZERG = 2,
    PROTOSS = 3,
    RANDOM = 4,
}

impl Default for Race {
    fn default() -> Self {
        Race::NORACE
    }
}

impl ToPyObject for Race {
    fn to_object(&self, py: Python) -> PyObject {
        self.to_i32().unwrap().to_object(py)
    }
}

impl FromPy<Race> for PyObject {
    fn from_py(other: Race, py: Python) -> Self {
        let _other: i32 = other.to_i32().unwrap();
        _other.into_py(py)
    }
}

impl<'source> FromPyObject<'source> for Race {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        //        println!("{:?}", "Extracting Race");
        let ob1: i32 = ob.extract::<i32>().unwrap();
        let x: Race = Race::from_i32(ob1).unwrap_or_default();
        Ok(x).into_py_result()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitTypeData {
    unit_type: UnitTypeId,
    name: String,
    available: bool,
    //    cargo_size: u32,
    mineral_cost: u32,
    vespene_cost: u32,
    attributes: Vec<Attribute>,
    movement_speed: f32,
    armor: f32,
    weapons: Vec<Weapon>,
    //    food_required: f32,
    //    food_provided: f32,
    //    ability: AbilityId,
    race: Option<Race>,
    //    build_time: f32,
    //    has_minerals: bool,
    //    has_vespene: bool,
    //    tech_alias: Vec<UnitTypeId>,
    //    unit_alias: UnitTypeId,
    //    tech_requirement: UnitTypeId,
    //    require_attached: bool,
}

impl UnitTypeData {
    /// Stable unit ID.
    pub fn get_id(&self) -> UnitTypeId {
        self.unit_type
    }
    /// Unit type name (corresponds to the game's catalog).
    pub fn get_name(&self) -> &str {
        &self.name
    }
    /// Whether this unit is available to the current mods/map.
    pub fn is_available(&self) -> bool {
        self.available
    }
    //    /// Number of cargo slots this unit occupies in a transport.
    //    pub fn get_cargo_size(&self) -> u32 {
    //        self.cargo_size
    //    }
    /// Cost in minerals to build this unit.
    pub fn get_mineral_cost(&self) -> u32 {
        self.mineral_cost
    }
    /// Cost in vespene to build this unit.
    pub fn get_vespene_cost(&self) -> u32 {
        self.vespene_cost
    }

    /// Unit attributes (may change based on upgrades).
    pub fn get_attributes(&self) -> &[Attribute] {
        &self.attributes
    }
    /// Movement speed of this unit.
    pub fn get_movement_speed(&self) -> f32 {
        self.movement_speed
    }
    /// Armor of this unit.
    pub fn get_armor(&self) -> f32 {
        self.armor
    }
    /// Weapons on this unit.
    pub fn get_weapons(&self) -> &[Weapon] {
        &self.weapons
    }
    //    / How much food this unit requires.
    //    pub fn get_food_required(&self) -> f32 {
    //        self.food_required
    //    }
    //    /// How much food this unit provides.
    //    pub fn get_food_provided(&self) -> f32 {
    //        self.food_provided
    //    }
    //    /// Which ability id creates this unit.
    //    pub fn get_ability(&self) -> AbilityId {
    //        self.ability
    //    }
    /// The race this unit belongs to.
    pub fn get_race(&self) -> Option<Race> {
        self.race
    }
    //    /// How long a unit takes to build.
    //    pub fn get_build_time(&self) -> f32 {
    //        self.build_time
    //    }
    //    /// Whether this unit can have minerals (mineral patches).
    //    pub fn has_minerals(&self) -> bool {
    //        self.has_minerals
    //    }
    //    /// Whether this unit can have vespene (vespene geysers).
    //    pub fn has_vespene(&self) -> bool {
    //        self.has_vespene
    //    }
    //
    //    /// Units this is equivalent to in terms of satisfying tech
    //    /// requirements.
    //    pub fn get_tech_alias(&self) -> &[UnitTypeId] {
    //        &self.tech_alias
    //    }
    //    /// Units that are morphed variants of the same unit.
    //    pub fn get_unit_alias(&self) -> UnitTypeId {
    //        self.unit_alias
    //    }
    //    /// Structure required to build this unit (or any with same tech alias).
    //    pub fn get_tech_requirement(&self) -> UnitTypeId {
    //        self.tech_requirement
    //    }
    //    /// Whether tech requirement is an addon.
    //    pub fn get_require_attached(&self) -> bool {
    //        self.require_attached
    //    }
}

impl<'source> FromPyObject<'source> for UnitTypeData {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        //        println!("{:?}", "Extracting UnitTypeData");
        unsafe {
            let py = Python::assume_gil_acquired();
            let obj = ob.to_object(py);
            //            let proto: UnitTypeId = obj.getattr(py, "p_unit_id")?.extract(py)?;
            //            let proto: Vec<Weapon> =obj.getattr(py, "p_weapons")?.extract(py)?;
            //            println!("{:?}", "weapon extracted");

            Ok(Self {
                unit_type: obj.getattr(py, "p_unit_id")?.extract(py)?,
                name: obj.getattr(py, "p_name")?.extract(py)?,
                available: obj.getattr(py, "p_available")?.extract(py)?,
                //                cargo_size: obj.getattr(py, "p_cargo_size")?.extract(py)?,
                mineral_cost: obj.getattr(py, "p_mineral_cost")?.extract(py)?,
                vespene_cost: obj.getattr(py, "p_vespene_cost")?.extract(py)?,
                attributes: obj.getattr(py, "p_attributes")?.extract(py)?,
                movement_speed: obj.getattr(py, "p_movement_speed")?.extract(py)?,
                armor: obj.getattr(py, "p_armor")?.extract(py)?,
                weapons: obj.getattr(py, "p_weapons")?.extract(py)?,
                //                food_required: obj.getattr(py, "p_food_required")?.extract(py)?,
                //                food_provided: obj.getattr(py, "p_food_provided")?.extract(py)?,
                //                ability: obj.getattr(py, "p_ability_id")?.extract(py)?,
                race: obj.getattr(py, "p_race")?.extract(py)?,
                //                build_time: obj.getattr(py, "p_build_time")?.extract(py)?,
                //                has_minerals: obj.getattr(py, "p_has_minerals")?.extract(py)?,
                //                has_vespene: obj.getattr(py, "p_has_vespene")?.extract(py)?,
                //                tech_alias: obj.getattr(py, "p_tech_alias")?.extract(py)?,
                //                unit_alias: obj.getattr(py, "p_unit_alias")?.extract(py)?,
                //                tech_requirement: obj.getattr(py, "p_tech_requirement")?.extract(py)?,
                //                require_attached: obj.getattr(py, "p_require_attached")?.extract(py)?
            })
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Weapon {
    target_type: WeaponTargetType,
    damage: f32,
    damage_bonus: Vec<DamageBonus>,
    attacks: u32,
    range: f32,
    speed: f32,
}

impl PartialEq for Weapon {
    fn eq(&self, other: &Self) -> bool {
        self.target_type == other.target_type
            && (self.damage - other.damage).abs() < EPSILON
            && self.damage_bonus == other.damage_bonus
            && self.attacks == other.attacks
            && (self.range - other.range) < EPSILON
            && (self.speed - other.speed) < EPSILON
    }
}
impl Eq for Weapon {}

impl Weapon {
    /// Weapon's target type.
    pub fn get_target_type(&self) -> WeaponTargetType {
        self.target_type
    }
    /// Weapon damage.
    pub fn get_damage(&self) -> f32 {
        self.damage
    }
    /// Any damage bonuses that apply to the weapon.
    pub fn get_damage_bonus(&self) -> &[DamageBonus] {
        &self.damage_bonus
    }
    /// Number of hits per attack (eg. Colossus has 2 beams).
    pub fn get_attacks(&self) -> u32 {
        self.attacks
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
}

impl<'source> FromPyObject<'source> for Weapon {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        //        println!("{:?}","Extracting weapon");
        unsafe {
            let py = Python::assume_gil_acquired();
            let obj = ob.to_object(py);
            let mut _target_type: WeaponTargetType;
            let mut _damage: f32;
            let mut _damage_bonus: Vec<DamageBonus>;
            let mut _attacks: u32;
            let mut _range: f32;
            let mut _speed: f32;
            let do_steps = || -> Result<PyObject, PyErr> { obj.getattr(py, "_weapon_type") };

            if let Err(_err) = do_steps() {
                _target_type = obj.getattr(py, "type")?.extract(py)?;
                _damage = obj.getattr(py, "damage")?.extract(py)?;
                _damage_bonus = obj.getattr(py, "damage_bonus")?.extract(py)?;
                _attacks = obj.getattr(py, "attacks")?.extract(py)?;
                _range = obj.getattr(py, "range")?.extract(py)?;
                _speed = obj.getattr(py, "speed")?.extract(py)?;
            } else {
                _target_type = obj.getattr(py, "_weapon_type")?.extract(py)?;
                _damage = obj.getattr(py, "_damage")?.extract(py)?;
                _damage_bonus = obj.getattr(py, "_damage_bonus")?.extract(py)?;
                _attacks = obj.getattr(py, "_attacks")?.extract(py)?;
                _range = obj.getattr(py, "_range")?.extract(py)?;
                _speed = obj.getattr(py, "_speed")?.extract(py)?;
            }
            Ok(Self {
                target_type: _target_type,
                damage: _damage,
                damage_bonus: _damage_bonus,
                attacks: _attacks,
                range: _range,
                speed: _speed,
            })
        }
    }
}

#[derive(Clone)]
pub struct WeaponInfo {
    weapon: Weapon,
    pub splash: f32,
    pub base_dps: f32,
    dps_cache: Vec<(usize, f32)>,
}

impl WeaponInfo {
    pub fn new(
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
    ) -> Self {
        let _splash: f32 = 0.0;
        let bonus_damage: f32 =
            get_damage_bonus(_type, _upgrades, _unit_data, _unit_tech_tree) as f32;

        let _base_dps: f32 = _weapon.damage + bonus_damage;
        //        let max : &usize = _unit_types_scope.iter().max().unwrap();
        //        let mut dps_cache: HashMap<usize, f32, RandomState> = HashMap::with_capacity(100);
        //            let mut dps_cache:Vec<(usize,f32)> = Vec::with_capacity(20);
        //                dps_cache.par_extend(_unit_types_scope.par_iter().map(|x|(*x, calculate_dps(_type,
        //                                UnitTypeId::from_usize(*x).unwrap(),
        //                                _weapon,
        //                                _unit_data,
        //                                _unit_tech_tree,
        //                                _upgrades,
        //                                _target_upgrades,
        //                                _data,
        //                                _tech_tree,))));
        let mut dps_cache: Vec<(usize, f32)> = Vec::with_capacity(50);
        if multi_threaded {
            dps_cache = _unit_types_scope
                .par_iter()
                .map(|x| {
                    (
                        *x,
                        calculate_dps(
                            _type,
                            UnitTypeId::from_usize(*x).unwrap(),
                            _weapon,
                            _unit_data,
                            _unit_tech_tree,
                            _upgrades,
                            _target_upgrades,
                            _data,
                            _tech_tree,
                        ),
                    )
                })
                .collect();
        //            dps_cache.par_extend(_unit_types_scope.par_iter().map(|x| {
        //                (
        //                    *x,
        //                    calculate_dps(
        //                        _type,
        //                        UnitTypeId::from_usize(*x).unwrap(),
        //                        _weapon,
        //                        _unit_data,
        //                        _unit_tech_tree,
        //                        _upgrades,
        //                        _target_upgrades,
        //                        _data,
        //                        _tech_tree,
        //                    ),
        //                )
        //            }));
        } else {
            //            let mut dps_cache:Vec<(usize,f32)> = Vec::with_capacity(50);
            for x in _unit_types_scope.iter() {
                dps_cache.push((
                    *x,
                    calculate_dps(
                        _type,
                        UnitTypeId::from_usize(*x).unwrap(),
                        _weapon,
                        _unit_data,
                        _unit_tech_tree,
                        _upgrades,
                        _target_upgrades,
                        _data,
                        _tech_tree,
                    ),
                ));
            }
        }

        //        for (x,y) in &dps_cache{
        //            println!("{:?}: {:?}", x,y)
        //        };
        WeaponInfo {
            weapon: _weapon.clone(),
            splash: _splash,
            base_dps: _base_dps,
            dps_cache,
        }
    }

    pub fn get_dps(&self, target: UnitTypeId) -> f32 {
        let target = target as usize;
        for (x, y) in &self.dps_cache {
            if *x == target {
                *y
            } else {
                continue;
            };
        }
        0.0
        //        *self.dps_cache.get(&(target as usize)).unwrap()
    }

    pub fn set_weapon(
        &mut self,
        _target_type: WeaponTargetType,
        _damage: f32,
        _damage_bonus: Vec<DamageBonus>,
        _attacks: u32,
        _range: f32,
        _speed: f32,
    ) {
        self.weapon = Weapon {
            target_type: _target_type,
            damage: _damage,
            damage_bonus: _damage_bonus,
            attacks: _attacks,
            range: _range,
            speed: _speed,
        }
    }
    pub fn get_target_type(&self) -> WeaponTargetType {
        self.weapon.target_type
    }
    /// Weapon damage.
    pub fn get_damage(&self) -> f32 {
        self.weapon.damage
    }
    /// Any damage bonuses that apply to the weapon.
    pub fn get_damage_bonus(&self) -> &[DamageBonus] {
        &self.weapon.damage_bonus
    }
    /// Number of hits per attack (eg. Colossus has 2 beams).
    pub fn get_attacks(&self) -> u32 {
        self.weapon.attacks
    }
    /// Attack range.
    pub fn get_range(&self) -> f32 {
        self.weapon.range
    }
    /// Time between attacks.
    pub fn get_speed(&self) -> f32 {
        self.weapon.speed
    }
}

impl<'source> FromPyObject<'source> for WeaponInfo {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        //        println!("{:?}", "Extracting WeaponInfo");
        unsafe {
            let py = Python::assume_gil_acquired();
            let obj = ob.to_object(py);
            Ok(Self {
                weapon: Weapon {
                    target_type: obj.getattr(py, "_weapon_type")?.extract(py)?,
                    damage: obj.getattr(py, "_damage")?.extract(py)?,
                    damage_bonus: obj.getattr(py, "_damage_bonus")?.extract(py)?,
                    attacks: obj.getattr(py, "_attacks")?.extract(py)?,
                    range: obj.getattr(py, "_range")?.extract(py)?,
                    speed: obj.getattr(py, "_speed")?.extract(py)?,
                },
                splash: obj.getattr(py, "splash")?.extract(py)?,
                base_dps: obj.getattr(py, "_base_dps")?.extract(py)?,
                dps_cache: Vec::new(),
            })
        }
    }
}
impl std::fmt::Debug for WeaponInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Hi: {}", 1)
    }
}

#[derive(Debug, Clone)]
pub struct UnitInfo {
    ability_id: AbilityId,
    air_weapons: WeaponInfo,
    armor: f32,
    attack_range: f32,
    can_be_attacked_by_air_weapons: bool,
    ground_weapons: WeaponInfo,
    is_basic_harvester: bool,
    is_flying: bool,
    is_melee: bool,
    is_structure: bool,
    max_health: f32,
    max_shield: f32,
    movement_speed: f32,
    race: i32,
    type_data: UnitTypeData,
    unit_radius: f32,
}

//impl ToPyObject for UnitInfo{
//	fn to_object(&self, py: Python) -> PyObject {
//		self.to_object(py)
//	}
//}

impl FromPy<UnitInfo> for PyObject {
    fn from_py(other: UnitInfo, py: Python) -> Self {
        let _other: UnitInfo = other;
        _other.into_py(py)
    }
}

impl<'source> FromPyObject<'source> for UnitInfo {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        //        println!("{:?}", "Extracting UnitInfo");
        unsafe {
            let py = Python::assume_gil_acquired();
            let obj = ob.to_object(py);
            Ok(Self {
                ability_id: obj.getattr(py, "_ability_id")?.extract(py)?,
                air_weapons: obj.getattr(py, "air_weapons")?.extract(py)?,
                armor: obj.getattr(py, "armor")?.extract(py)?,
                attack_range: obj.getattr(py, "attack_range")?.extract(py)?,
                can_be_attacked_by_air_weapons: obj
                    .getattr(py, "can_be_attacked_by_air_weapons")?
                    .extract(py)?,
                ground_weapons: obj.getattr(py, "ground_weapons")?.extract(py)?,
                is_basic_harvester: obj.getattr(py, "is_basic_harvester")?.extract(py)?,
                is_flying: obj.getattr(py, "is_flying")?.extract(py)?,
                is_melee: obj.getattr(py, "is_melee")?.extract(py)?,
                is_structure: obj.getattr(py, "is_structure")?.extract(py)?,
                max_health: obj.getattr(py, "max_health")?.extract(py)?,
                max_shield: obj.getattr(py, "max_shield")?.extract(py)?,
                movement_speed: obj.getattr(py, "movement_speed")?.extract(py)?,
                race: obj.getattr(py, "race")?.extract(py)?,
                type_data: obj.getattr(py, "type_data")?.extract(py)?,
                unit_radius: obj.getattr(py, "unit_radius")?.extract(py)?,
            })
        }
    }
}

#[derive(Primitive, Debug, Copy, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub enum AbilityTarget {
    NONE = 1,
    /// Ability targets a location.
    POINT = 2,
    /// Ability targets another unit.
    UNIT = 3,
    /// Ability can target either a location or a unit.
    POINTORUNIT = 4,
    /// Ability can target either a location or nothing.
    POINTORNONE = 5,
}

impl Default for AbilityTarget {
    fn default() -> Self {
        AbilityTarget::NONE
    }
}

impl ToPyObject for AbilityTarget {
    fn to_object(&self, py: Python) -> PyObject {
        self.to_i32().unwrap().to_object(py)
    }
}

impl FromPy<AbilityTarget> for PyObject {
    fn from_py(other: AbilityTarget, py: Python) -> Self {
        let _other: i32 = other.to_i32().unwrap();
        _other.into_py(py)
    }
}

impl<'source> FromPyObject<'source> for AbilityTarget {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        //        println!("{:?}", "Extracting Race");
        let ob1: i32 = ob.extract::<i32>().unwrap();
        let x: AbilityTarget = AbilityTarget::from_i32(ob1).unwrap_or_default();
        Ok(x).into_py_result()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AbilityData {
    available: bool,
    ability: AbilityId,
    link_name: String,
    link_index: u32,
    button_name: String,
    friendly_name: String,
    hotkey: String,
    remaps_to_ability: Option<AbilityId>,
    //    remaps_from_ability: Vec<AbilityId>,
    target: Option<AbilityTarget>,
    allow_minimap: bool,
    allow_autocast: bool,
    is_building: bool,
    footprint_radius: Option<f32>,
    is_instant_placement: bool,
    cast_range: f32,
}

impl AbilityData {
    /// Get the most generalized id of the ability.
    pub fn get_generalized_ability(&self) -> AbilityId {
        match self.remaps_to_ability {
            Some(remap) => remap,
            None => self.ability,
        }
    }

    /// Indicates whether the ability is available to the current mods/map.
    pub fn is_available(&self) -> bool {
        self.available
    }

    /// Stable ID for the ability.
    pub fn get_id(&self) -> AbilityId {
        self.ability
    }

    /// Catalog (game data xml) name of the ability.
    pub fn get_link_name(&self) -> &str {
        &self.link_name
    }

    /// Catalog (game data xml) index of the ability.
    pub fn get_link_index(&self) -> u32 {
        self.link_index
    }

    /// Name of the button for the command card.
    pub fn get_button_name(&self) -> &str {
        &self.button_name
    }
    /// In case the button name is not descriptive.
    pub fn get_friendly_name(&self) -> &str {
        &self.friendly_name
    }
    /// UI hotkey.
    pub fn get_hotkey(&self) -> &str {
        &self.hotkey
    }

    //    /// Other abilities that can remap to this generic ability.
    //    pub fn get_remap_abilities(&self) -> &[AbilityId] {
    //        &self.remaps_from_ability
    //    }

    /// Type of target that this ability uses.
    pub fn get_target(&self) -> Option<AbilityTarget> {
        self.target
    }
    /// Can be cast in the minimap (unimplemented).
    pub fn casts_in_minimap(&self) -> bool {
        self.allow_minimap
    }
    /// Autocast can be set.
    pub fn can_autocast(&self) -> bool {
        self.allow_autocast
    }
    /// Requires placement to construct a building.
    pub fn is_building(&self) -> bool {
        self.is_building
    }
    /// If the ability is placing a building, give the radius of the footprint.
    pub fn get_footprint_radius(&self) -> Option<f32> {
        self.footprint_radius
    }
    /// Placement next to an existing structure (an addon like a Tech Lab).
    pub fn is_instant_placement(&self) -> bool {
        self.is_instant_placement
    }
    /// Range unit can cast ability without needing to approach target.
    pub fn get_cast_range(&self) -> f32 {
        self.cast_range
    }
}

//impl ToPyObject for AbilityData{
//	fn to_object(&self, py: Python) -> PyObject {
//		self.to_object(py)
//	}
//}

impl FromPy<AbilityData> for PyObject {
    fn from_py(other: AbilityData, py: Python) -> Self {
        let _other: AbilityData = other;
        _other.into_py(py)
    }
}

impl<'source> FromPyObject<'source> for AbilityData {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        //        println!("{:?}", "Extracting UnitInfo");
        unsafe {
            let py = Python::assume_gil_acquired();
            let obj = ob.to_object(py);
            Ok(Self {
                available: obj.getattr(py, "p_available")?.extract(py)?,
                ability: obj.getattr(py, "p_ability")?.extract(py)?,
                link_name: obj.getattr(py, "p_link_name")?.extract(py)?,
                link_index: obj.getattr(py, "p_link_index")?.extract(py)?,
                button_name: obj.getattr(py, "p_button_name")?.extract(py)?,
                friendly_name: obj.getattr(py, "p_friendly_name")?.extract(py)?,
                hotkey: obj.getattr(py, "p_hotkey")?.extract(py)?,
                remaps_to_ability: obj.getattr(py, "p_remaps_to_ability")?.extract(py)?,
                //                remaps_from_ability: obj.getattr(py, "p_remaps_from_ability")?.extract(py)?,
                target: obj.getattr(py, "p_target")?.extract(py)?,
                allow_minimap: obj.getattr(py, "p_allow_minimap")?.extract(py)?,
                allow_autocast: obj.getattr(py, "p_allow_autocast")?.extract(py)?,
                is_building: obj.getattr(py, "p_is_building")?.extract(py)?,
                footprint_radius: obj.getattr(py, "p_footprint_radius")?.extract(py)?,
                is_instant_placement: obj.getattr(py, "p_is_instant_placement")?.extract(py)?,
                cast_range: obj.getattr(py, "p_cast_range")?.extract(py)?,
            })
        }
    }
}

/// Upgrade data.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpgradeData {
    upgrade: UpgradeId,
    name: String,
    mineral_cost: u32,
    vespene_cost: u32,
    ability: AbilityId,
    research_time: f32,
}

impl UpgradeData {
    /// Stable upgrade ID.
    pub fn get_id(&self) -> UpgradeId {
        self.upgrade
    }
    /// Upgrade name (corresponds to the game's catalog).
    pub fn get_name(&self) -> &str {
        &self.name
    }
    /// Mineral cost of researching this upgrade.
    pub fn get_mineral_cost(&self) -> u32 {
        self.mineral_cost
    }
    /// Vespene cost of researching this upgrade.
    pub fn get_vespene_cost(&self) -> u32 {
        self.vespene_cost
    }
    /// Ability that researches this upgrade.
    pub fn get_ability(&self) -> AbilityId {
        self.ability
    }
    /// Time in game steps to research this upgrade.
    pub fn get_research_time(&self) -> f32 {
        self.research_time
    }
}

//impl ToPyObject for UpgradeData{
//	fn to_object(&self, py: Python) -> PyObject {
//		self.to_object(py)
//	}
//}

impl FromPy<UpgradeData> for PyObject {
    fn from_py(other: UpgradeData, py: Python) -> Self {
        let _other: UpgradeData = other;
        _other.into_py(py)
    }
}

impl<'source> FromPyObject<'source> for UpgradeData {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        //        println!("{:?}", "Extracting UnitInfo");
        unsafe {
            let py = Python::assume_gil_acquired();
            let obj = ob.to_object(py);
            Ok(Self {
                upgrade: obj.getattr(py, "p_upgrade_id")?.extract(py)?,
                name: obj.getattr(py, "p_name")?.extract(py)?,
                mineral_cost: obj.getattr(py, "p_mineral_cost")?.extract(py)?,
                vespene_cost: obj.getattr(py, "p_vespene_cost")?.extract(py)?,
                ability: obj.getattr(py, "p_ability_id")?.extract(py)?,
                research_time: obj.getattr(py, "p_research_time")?.extract(py)?,
            })
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GameInfo {
    //    ability_data: HashMap<u32, AbilityData>,
    unit_data: FxHashMap<u32, UnitTypeData>,
    //    upgrade_data: HashMap<u32, UpgradeData>,
    available_units: FxHashSet<usize>,
}

//impl ToPyObject for GameInfo{
//	fn to_object(&self, py: Python) -> PyObject {
//		self.to_object(py)
//	}
//}

//impl FromPy<GameInfo> for GameInfo {
//    fn from_py(other: GameInfo, py: Python) -> Self {
//        let _other: GameInfo = other;
//        _other.into_py(py)
//    }
//}

impl<'source> FromPyObject<'source> for GameInfo {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        //        println!("{:?}", "Extracting UnitInfo");
        unsafe {
            let py = Python::assume_gil_acquired();
            let obj = ob.to_object(py);
            Ok(Self {
                //                ability_data: obj.getattr(py, "ability_data")?.extract(py)?,
                unit_data: obj.getattr(py, "unit_data")?.extract(py)?,
                //                upgrade_data: obj.getattr(py, "upgrade_data")?.extract(py)?,
                available_units: FxHashSet::with_capacity_and_hasher(10, Default::default()),
            })
        }
    }
}

impl GameInfo {
    pub(crate) fn get_unit_data(&self, unit_id: UnitTypeId) -> Option<&UnitTypeData> {
        let id: u32 = unit_id as u32;
        self.unit_data.get(id.borrow())
    }

    pub fn load_available_units(&mut self) {
        for (_k, v) in self.unit_data.iter().filter(|&(_k, v)| v.available) {
            self.available_units.insert(v.unit_type.to_usize().unwrap());
        }
    }

    pub fn get_available_units(&self) -> &FxHashSet<usize> {
        &self.available_units
    }
}
//#[cache(LruCache : LruCache::new(30))]
pub fn is_melee(unit: UnitTypeId) -> bool {
    IS_MELEE.contains(&unit)
}

//#[cache(LruCache : LruCache::new(30))]
//#[cache_cfg(ignore_args = _data, _tech_tree)]
//#[cache_cfg(thread_local)]
pub fn get_damage_bonus(
    unit: UnitTypeId,
    upgrades: &CombatUpgrades,
    _data: &UnitTypeData,
    _tech_tree: &UnitType,
) -> i32 {
    if _tech_tree.is_structure {
        return 0;
    }

    let mut bonus: i32 = 0;
    match _data.race.unwrap() {
        Race::PROTOSS => {
            if _tech_tree.is_flying {
                if upgrades.has_upgrade(UpgradeId::PROTOSSAIRWEAPONSLEVEL1) {
                    bonus += 1;
                }
                if upgrades.has_upgrade(UpgradeId::PROTOSSAIRWEAPONSLEVEL2) {
                    bonus += 1;
                }
                if upgrades.has_upgrade(UpgradeId::PROTOSSAIRWEAPONSLEVEL3) {
                    bonus += 1;
                }
            } else {
                if upgrades.has_upgrade(UpgradeId::PROTOSSGROUNDWEAPONSLEVEL1) {
                    bonus += 1;
                }
                if upgrades.has_upgrade(UpgradeId::PROTOSSGROUNDWEAPONSLEVEL2) {
                    bonus += 1;
                }
                if upgrades.has_upgrade(UpgradeId::PROTOSSGROUNDWEAPONSLEVEL3) {
                    bonus += 1;
                }
            }
        }
        Race::ZERG => {
            if _tech_tree.is_flying {
                if upgrades.has_upgrade(UpgradeId::ZERGFLYERWEAPONSLEVEL1) {
                    bonus += 1;
                }
                if upgrades.has_upgrade(UpgradeId::ZERGFLYERWEAPONSLEVEL2) {
                    bonus += 1;
                }
                if upgrades.has_upgrade(UpgradeId::ZERGFLYERWEAPONSLEVEL3) {
                    bonus += 1;
                }
            } else if is_melee(unit) {
                if upgrades.has_upgrade(UpgradeId::ZERGMELEEWEAPONSLEVEL1) {
                    bonus += 1;
                }
                if upgrades.has_upgrade(UpgradeId::ZERGMELEEWEAPONSLEVEL2) {
                    bonus += 1;
                }
                if upgrades.has_upgrade(UpgradeId::ZERGMELEEWEAPONSLEVEL3) {
                    bonus += 1;
                }
            } else {
                if upgrades.has_upgrade(UpgradeId::ZERGMISSILEWEAPONSLEVEL1) {
                    bonus += 1;
                }
                if upgrades.has_upgrade(UpgradeId::ZERGMISSILEWEAPONSLEVEL2) {
                    bonus += 1;
                }
                if upgrades.has_upgrade(UpgradeId::ZERGMISSILEWEAPONSLEVEL3) {
                    bonus += 1;
                }
            }
        }
        Race::TERRAN => {
            //TODO: Figure out mech and bio
            let mechanical: A = Attribute::MECHANICAL.to_tt();
            if _tech_tree.is_flying {
                if upgrades.has_upgrade(UpgradeId::TERRANSHIPWEAPONSLEVEL1) {
                    bonus += 1;
                }
                if upgrades.has_upgrade(UpgradeId::TERRANSHIPWEAPONSLEVEL2) {
                    bonus += 1;
                }
                if upgrades.has_upgrade(UpgradeId::TERRANSHIPWEAPONSLEVEL3) {
                    bonus += 1;
                }
            } else if _tech_tree.attributes.contains(&mechanical) {
                if upgrades.has_upgrade(UpgradeId::TERRANVEHICLEWEAPONSLEVEL1) {
                    bonus += 1;
                }
                if upgrades.has_upgrade(UpgradeId::TERRANVEHICLEWEAPONSLEVEL2) {
                    bonus += 1;
                }
                if upgrades.has_upgrade(UpgradeId::TERRANVEHICLEWEAPONSLEVEL3) {
                    bonus += 1;
                }
            } else {
                if upgrades.has_upgrade(UpgradeId::TERRANINFANTRYWEAPONSLEVEL1) {
                    bonus += 1;
                }
                if upgrades.has_upgrade(UpgradeId::TERRANINFANTRYWEAPONSLEVEL2) {
                    bonus += 1;
                }
                if upgrades.has_upgrade(UpgradeId::TERRANINFANTRYWEAPONSLEVEL3) {
                    bonus += 1;
                }
            }
        }

        _ => println!("Unknown Race for {:?}", unit),
    }
    bonus
}

//#[cache(LruCache : LruCache::new(30))]
//#[cache_cfg(ignore_args = _data,_tech_tree)]
//#[cache_cfg(thread_local)]
pub fn get_armor_bonus(
    unit: UnitTypeId,
    upgrades: &CombatUpgrades,
    _data: &UnitTypeData,
    _tech_tree: &UnitType,
) -> f32 {
    if _tech_tree.is_structure {
        if _data.race.unwrap() == Race::TERRAN
            && upgrades.has_upgrade(UpgradeId::TERRANBUILDINGARMOR)
        {
            return 2.0;
        }
        return 0.0;
    }
    let mut bonus: f32 = 0.0;

    match _data.race.unwrap() {
        Race::PROTOSS => {
            if _tech_tree.is_flying {
                if upgrades.has_upgrade(UpgradeId::PROTOSSAIRARMORSLEVEL1) {
                    bonus += 1.0;
                }
                if upgrades.has_upgrade(UpgradeId::PROTOSSAIRARMORSLEVEL2) {
                    bonus += 1.0;
                }
                if upgrades.has_upgrade(UpgradeId::PROTOSSAIRARMORSLEVEL3) {
                    bonus += 1.0;
                }
            } else {
                if upgrades.has_upgrade(UpgradeId::PROTOSSGROUNDARMORSLEVEL1) {
                    bonus += 1.0;
                }
                if upgrades.has_upgrade(UpgradeId::PROTOSSGROUNDARMORSLEVEL2) {
                    bonus += 1.0;
                }
                if upgrades.has_upgrade(UpgradeId::PROTOSSGROUNDARMORSLEVEL3) {
                    bonus += 1.0;
                }
            }
        }
        Race::ZERG => {
            if _tech_tree.is_flying {
                if upgrades.has_upgrade(UpgradeId::ZERGFLYERARMORSLEVEL1) {
                    bonus += 1.0;
                }
                if upgrades.has_upgrade(UpgradeId::ZERGFLYERARMORSLEVEL2) {
                    bonus += 1.0;
                }
                if upgrades.has_upgrade(UpgradeId::ZERGFLYERARMORSLEVEL3) {
                    bonus += 1.0;
                }
            } else {
                if upgrades.has_upgrade(UpgradeId::ZERGGROUNDARMORSLEVEL1) {
                    bonus += 1.0;
                }
                if upgrades.has_upgrade(UpgradeId::ZERGGROUNDARMORSLEVEL2) {
                    bonus += 1.0;
                }
                if upgrades.has_upgrade(UpgradeId::ZERGGROUNDARMORSLEVEL3) {
                    bonus += 1.0;
                }
            }
        }
        Race::TERRAN => {
            let mechanical: A = Attribute::MECHANICAL.to_tt();
            if _tech_tree.is_flying || _tech_tree.attributes.contains(&mechanical) {
                if upgrades.has_upgrade(UpgradeId::TERRANVEHICLEANDSHIPARMORSLEVEL1) {
                    bonus += 1.0;
                }
                if upgrades.has_upgrade(UpgradeId::TERRANVEHICLEANDSHIPARMORSLEVEL2) {
                    bonus += 1.0;
                }
                if upgrades.has_upgrade(UpgradeId::TERRANVEHICLEANDSHIPARMORSLEVEL3) {
                    bonus += 1.0;
                }
            } else {
                if upgrades.has_upgrade(UpgradeId::TERRANINFANTRYARMORSLEVEL1) {
                    bonus += 1.0;
                }
                if upgrades.has_upgrade(UpgradeId::TERRANINFANTRYARMORSLEVEL2) {
                    bonus += 1.0;
                }
                if upgrades.has_upgrade(UpgradeId::TERRANINFANTRYARMORSLEVEL2) {
                    bonus += 1.0;
                }
            }
        }

        _ => println!("Unknown Race for {:?}", unit),
    }
    bonus
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct CombatUpgrades(Vec<UpgradeId>);

impl CombatUpgrades {
    pub fn new(upgrades: Vec<UpgradeId>) -> Self {
        CombatUpgrades(upgrades)
    }
    pub fn has_upgrade(&self, upgrade: UpgradeId) -> bool {
        self.0.contains(&upgrade)
    }
}
//macro_rules! unwrap_or_return {
//    ( $e:expr ) => {
//        match $e {
//            Some(x) => x,
//            None => return 0.0,
//        }
//    };
//}

//#[cache(LruCache : LruCache::new(30))]
//#[cache_cfg(ignore_args = attacker_game_data, attacker_tech_data,_data,_tech_tree, weapon)]
//#[cache_cfg(thread_local)]

pub fn can_be_attacked_by_air_weapons(unit: &CombatUnit) -> bool {
    unit.unit_type == UnitTypeId::COLOSSUS || unit.is_flying
}

//#[cache(LruCache : LruCache::new(100))]
//#[cache_cfg(ignore_args = weapon, attacker_game_data, attacker_tech_data, attacker_upgrades, target_upgrades, _data, _tech_tree)]
//#[cache_cfg(thread_local)]

fn calculate_dps(
    attacker: UnitTypeId,
    target: UnitTypeId,
    weapon: &Weapon,
    attacker_game_data: &UnitTypeData,
    attacker_tech_data: &UnitType,
    attacker_upgrades: &CombatUpgrades,
    target_upgrades: &CombatUpgrades,
    _data: &GameInfo,
    _tech_tree: &TechData,
) -> f32 {
    // canBeAttackedByAirWeapons is primarily for colossus.
    let target_tech_data: UnitType = get_tech_data(target, _tech_tree); //unwrap_or_return!(_tech_tree.unittype(target.to_tt()));

    let target_game_data: &UnitTypeData = _data.get_unit_data(target).unwrap();

    //    let attacker_tech_data: UnitType = unwrap_or_return!(_tech_tree.unittype(attacker.to_tt()));
    //    let attacker_game_data: &UnitTypeData = unwrap_or_return!(_data.get_unit_data(attacker));
    if weapon.target_type == WeaponTargetType::ANY
        || if target == UnitTypeId::COLOSSUS || target_tech_data.is_flying {
            weapon.target_type == WeaponTargetType::AIR
        } else {
            !target_tech_data.is_flying
        }
    {
        let mut dmg: f32 = weapon.damage;
        //        println!("For attacker: {:?}, weapon damage is {:?}", attacker, dmg);
        for b in &weapon.damage_bonus {
            //            println!("Bonus attribute {:?}, dmg {:?}", b.attribute, b.bonus);
            if target_tech_data.attributes.contains(&b.attribute.to_tt()) {
                dmg += b.bonus;
            }
        }

        dmg += get_damage_bonus(
            attacker,
            attacker_upgrades,
            attacker_game_data,
            &attacker_tech_data,
        ) as f32;
        //        println!("DMG after getting damage bonus= {:?}", dmg);

        let mut armor: f32 = target_game_data.armor
            + get_armor_bonus(target, target_upgrades, target_game_data, &target_tech_data);
        //        println!("Target {:?}, Armor {:?}",target, armor);

        // Note: cannot distinguish between damage to shields and damage to health yet, so average out so that the armor is about 0.5 over the whole shield+health of the unit
        // Important only for protoss
        let max_health: f32 = target_tech_data.max_health.into();

        let max_shield: f32 = match target_tech_data.max_shield {
            None => 0.0,
            Some(t) => t.into(),
        };
        //        println!("Past max_shield");
        armor = armor * max_health / (max_shield + max_health);
        //        println!("Target {:?}, Armor-calculated {:?}",target, armor);

        let mut time_between_attacks: f32 = weapon.speed;
        //        println!("Weapon speed={:?}", weapon.speed);

        if attacker == UnitTypeId::ADEPT
            && attacker_upgrades.has_upgrade(UpgradeId::ADEPTPIERCINGATTACK)
        {
            time_between_attacks /= 1.45;
        }
        //        println!("DPS of {:?} against {:?} is {:?}", attacker, target, ((dmg - armor) * weapon.attacks as f32) / time_between_attacks);
        return if dmg - armor > 0.0 {
            ((dmg - armor) * weapon.attacks as f32) / time_between_attacks
        } else {
            0.0
        };
    }

    0.0
}
