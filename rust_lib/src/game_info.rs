use crate::generated_enums::{AbilityId, UnitTypeId, UpgradeId};
use pyo3::{FromPyObject, PyResult, PyObject, FromPy, Python, IntoPy, ToPyObject, ObjectProtocol, PyErr};
use std::collections::HashMap;

use pyo3::types::{PyAny};
use pyo3::derive_utils::IntoPyResult;
use crate::num_traits::{FromPrimitive, ToPrimitive};
use std::borrow::Borrow;
//use std::{hash, cmp};

#[allow(missing_docs)]
#[derive(Primitive, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Attribute {
    NULL=0,
    LIGHT=1,
    ARMORED=2,
    BIOLOGICAL=3,
    MECHANICAL=4,
    ROBOTIC=5,
    PSIONIC=6,
    MASSIVE=7,
    STRUCTURE=8,
    HOVER=9,
    HEROIC=10,
    SUMMONED=11,
}

impl Default for Attribute {
	fn default() -> Self {
		Attribute::NULL
	}
}

impl ToPyObject for Attribute{
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

impl<'source> FromPyObject<'source> for Attribute{

	fn extract(ob: &'source PyAny)-> PyResult<Self>{
//        println!("{:?}", "Extracting Attribute");
		let ob1: i32 = ob.extract::<i32>().unwrap();
		let x : Attribute = Attribute::from_i32(ob1).unwrap_or_default();
		Ok(x).into_py_result()
	}
}

#[derive(Debug, Copy, Clone)]
pub struct DamageBonus {
    attribute: Attribute,
    bonus: f32,
}

impl DamageBonus {
    /// Affected attribute.
    pub fn get_attribute(&self) -> Attribute {
        self.attribute
    }

    /// Damage bonus.
    pub fn get_bonus(&self) -> f32 {
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
                bonus: obj.getattr(py, "bonus")?.extract(py)?
            })
        }
    }
}

#[allow(missing_docs)]
#[derive(Primitive, Debug, Copy, Clone, Eq, PartialEq)]
pub enum WeaponTargetType {
    NULL=0,
    GROUND=1,
    AIR=2,
    ANY=3,
}

impl Default for WeaponTargetType {
	fn default() -> Self {
		WeaponTargetType::NULL
	}
}

impl ToPyObject for WeaponTargetType{
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

impl<'source> FromPyObject<'source> for WeaponTargetType{

	fn extract(ob: &'source PyAny)-> PyResult<Self>{
//        println!("{:?}", "Extracting WeaponTargetType");
		let ob1: i32 = ob.extract::<i32>().unwrap();
		let x : WeaponTargetType = WeaponTargetType::from_i32(ob1).unwrap_or_default();
		Ok(x).into_py_result()
	}
}
#[allow(missing_docs)]
#[derive(Primitive, Debug, Copy, Clone)]
pub enum Race {
    NORACE=0,
    TERRAN=1,
    ZERG=2,
    PROTOSS=3,
    RANDOM=4,

}

impl Default for Race {
	fn default() -> Self {
		Race::NORACE
	}
}

impl ToPyObject for Race{
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

impl<'source> FromPyObject<'source> for Race{

	fn extract(ob: &'source PyAny)-> PyResult<Self>{
//        println!("{:?}", "Extracting Race");
		let ob1: i32 = ob.extract::<i32>().unwrap();
		let x : Race = Race::from_i32(ob1).unwrap_or_default();
		Ok(x).into_py_result()
	}
}
//
//#[derive(Debug, Clone)]
//pub struct UnitTypeDataProto{
//    ability_id: i32,
//    armor: f62,
//    attributes: Vec<i32>,
//    available: bool,
//    build_time: f62,
//    cargo_size: i32,
//    food_provided: f62,
//    food_required: f62,
//    has_minerals: bool,
//    has_vespene: bool,
//    mineral_cost: i32,
//    movement_speed: f62,
//    name: String,
//    race: i32,
//    require_attached: bool,
//    sight_range: f62,
//    tech_alias: Vec<i32>,
//    tech_requirement: i32,
//    unit_alias: i32,
//    unit_id: i32,
//    vespene_cost: i32,
//    weapons: Vec<Weapon>
//
//}
//
//impl<'source> FromPyObject<'source> for UnitTypeData {
//    fn extract(ob: &'source PyAny) -> PyResult<Self> {
//        println!("{:?}", "Extracting UnitTypeData");
//        unsafe {
//            let py = Python::assume_gil_acquired();
//            let obj = ob.to_object(py);
//            let proto: UnitTypeDataProto = obj.getattr(py, "_proto")?.extract(py)?;
//            println!("{:?}", "Id extracted");
//
//            Ok(Self {
//                unit_type: obj.getattr(py, "id")?.extract(py)?,
//                name: obj.getattr(py, "name")?.extract(py)?,
//                available: obj.getattr(py, "_proto.available")?.extract(py)?,
//                cargo_size: obj.getattr(py, "cargo_size")?.extract(py)?,
//                mineral_cost: obj.getattr(py, "_proto.mineral_cost")?.extract(py)?,
//                vespene_cost: obj.getattr(py, "_proto.vespene_cost")?.extract(py)?,
//                attributes: obj.getattr(py, "_proto.attributes")?.extract(py)?,
//                movement_speed: obj.getattr(py, "_proto.movement_speed")?.extract(py)?,
//                armor: obj.getattr(py, "_proto.armor")?.extract(py)?,
//                weapons: obj.getattr(py, "_proto.weapons")?.extract(py)?,
//                food_required: obj.getattr(py, "_proto.food_required")?.extract(py)?,
//                food_provided: obj.getattr(py, "_proto.food_provided")?.extract(py)?,
//                ability: obj.getattr(py, "_proto.ability_id")?.extract(py)?,
//                race: obj.getattr(py, "_proto.race")?.extract(py)?,
//                build_time: obj.getattr(py, "_proto.build_time")?.extract(py)?,
//                has_minerals: obj.getattr(py, "_proto.has_minerals")?.extract(py)?,
//                has_vespene: obj.getattr(py, "_proto.has_vespene")?.extract(py)?,
//                tech_alias: obj.getattr(py, "_proto.tech_alias")?.extract(py)?,
//                unit_alias: obj.getattr(py, "_proto.unit_alias")?.extract(py)?,
//                tech_requirement: obj.getattr(py, "_proto.tech_requirement")?.extract(py)?,
//                require_attached: obj.getattr(py, "_proto.require_attached")?.extract(py)?
//            })
//        }
//    }
//}
#[derive(Debug, Clone)]
pub struct UnitTypeData {
    unit_type: UnitTypeId,
    name: String,
    available: bool,
    cargo_size: u32,
    mineral_cost: u32,
    vespene_cost: u32,
    attributes: Vec<Attribute>,
    movement_speed: f32,
    armor: f32,
    weapons: Vec<Weapon>,
    food_required: f32,
    food_provided: f32,
    ability: AbilityId,
    race: Option<Race>,
    build_time: f32,
    has_minerals: bool,
    has_vespene: bool,
    tech_alias: Vec<UnitTypeId>,
    unit_alias: UnitTypeId,
    tech_requirement: UnitTypeId,
    require_attached: bool,
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
    /// Number of cargo slots this unit occupies in a transport.
    pub fn get_cargo_size(&self) -> u32 {
        self.cargo_size
    }
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
    /// How much food this unit requires.
    pub fn get_food_required(&self) -> f32 {
        self.food_required
    }
    /// How much food this unit provides.
    pub fn get_food_provided(&self) -> f32 {
        self.food_provided
    }
    /// Which ability id creates this unit.
    pub fn get_ability(&self) -> AbilityId {
        self.ability
    }
    /// The race this unit belongs to.
    pub fn get_race(&self) -> Option<Race> {
        self.race
    }
    /// How long a unit takes to build.
    pub fn get_build_time(&self) -> f32 {
        self.build_time
    }
    /// Whether this unit can have minerals (mineral patches).
    pub fn has_minerals(&self) -> bool {
        self.has_minerals
    }
    /// Whether this unit can have vespene (vespene geysers).
    pub fn has_vespene(&self) -> bool {
        self.has_vespene
    }

    /// Units this is equivalent to in terms of satisfying tech
    /// requirements.
    pub fn get_tech_alias(&self) -> &[UnitTypeId] {
        &self.tech_alias
    }
    /// Units that are morphed variants of the same unit.
    pub fn get_unit_alias(&self) -> UnitTypeId {
        self.unit_alias
    }
    /// Structure required to build this unit (or any with same tech alias).
    pub fn get_tech_requirement(&self) -> UnitTypeId {
        self.tech_requirement
    }
    /// Whether tech requirement is an addon.
    pub fn get_require_attached(&self) -> bool {
        self.require_attached
    }
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
                cargo_size: obj.getattr(py, "p_cargo_size")?.extract(py)?,
                mineral_cost: obj.getattr(py, "p_mineral_cost")?.extract(py)?,
                vespene_cost: obj.getattr(py, "p_vespene_cost")?.extract(py)?,
                attributes: obj.getattr(py, "p_attributes")?.extract(py)?,
                movement_speed: obj.getattr(py, "p_movement_speed")?.extract(py)?,
                armor: obj.getattr(py, "p_armor")?.extract(py)?,
                weapons: obj.getattr(py, "p_weapons")?.extract(py)?,
                food_required: obj.getattr(py, "p_food_required")?.extract(py)?,
                food_provided: obj.getattr(py, "p_food_provided")?.extract(py)?,
                ability: obj.getattr(py, "p_ability_id")?.extract(py)?,
                race: obj.getattr(py, "p_race")?.extract(py)?,
                build_time: obj.getattr(py, "p_build_time")?.extract(py)?,
                has_minerals: obj.getattr(py, "p_has_minerals")?.extract(py)?,
                has_vespene: obj.getattr(py, "p_has_vespene")?.extract(py)?,
                tech_alias: obj.getattr(py, "p_tech_alias")?.extract(py)?,
                unit_alias: obj.getattr(py, "p_unit_alias")?.extract(py)?,
                tech_requirement: obj.getattr(py, "p_tech_requirement")?.extract(py)?,
                require_attached: obj.getattr(py, "p_require_attached")?.extract(py)?
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct Weapon {
    target_type: WeaponTargetType,
    damage: f32,
    damage_bonus: Vec<DamageBonus>,
    attacks: u32,
    range: f32,
    speed: f32,
}

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
            let do_steps = || -> Result<PyObject, PyErr> {
                    obj.getattr(py, "_weapon_type")
                };


            if let Err(_err) = do_steps() {
                _target_type = obj.getattr(py, "type")?.extract(py)?;
                _damage = obj.getattr(py, "damage")?.extract(py)?;
                _damage_bonus = obj.getattr(py, "damage_bonus")?.extract(py)?;
                _attacks = obj.getattr(py, "attacks")?.extract(py)?;
                _range = obj.getattr(py, "range")?.extract(py)?;
                _speed = obj.getattr(py, "speed")?.extract(py)?;
            }
            else {
                _target_type =  obj.getattr(py, "_weapon_type")?.extract(py)?;
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
                speed: _speed
            })
        }
    }
}

#[derive(Debug, Clone)]
struct WeaponInfo{
    weapon: Weapon,
    get_dps: f32,
    splash: f32,
    base_dps: f32
}

impl WeaponInfo{
    pub fn set_weapon(&mut self,
                      _target_type: WeaponTargetType,
                      _damage: f32,
                      _damage_bonus: Vec<DamageBonus>,
                      _attacks: u32,
                      _range: f32,
                      _speed: f32){
        self.weapon = Weapon{
            target_type: _target_type,
            damage: _damage,
            damage_bonus: _damage_bonus,
            attacks: _attacks,
            range: _range,
            speed: _speed
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
                weapon: Weapon{
                    target_type: obj.getattr(py, "_weapon_type")?.extract(py)?,
                    damage: obj.getattr(py, "_damage")?.extract(py)?,
                    damage_bonus: obj.getattr(py, "_damage_bonus")?.extract(py)?,
                    attacks: obj.getattr(py, "_attacks")?.extract(py)?,
                    range: obj.getattr(py, "_range")?.extract(py)?,
                    speed: obj.getattr(py, "_speed")?.extract(py)?
                },
                get_dps: obj.getattr(py, "get_dps")?.extract(py)?,
                splash: obj.getattr(py, "splash")?.extract(py)?,
                base_dps: obj.getattr(py, "_base_dps")?.extract(py)?
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct UnitInfo{
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
    unit_radius: f32
}

impl ToPyObject for UnitInfo{
	fn to_object(&self, py: Python) -> PyObject {
		self.to_object(py)
	}
}

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
                ability_id: obj.getattr(py,"_ability_id")?.extract(py)?,
                air_weapons: obj.getattr(py,"air_weapons")?.extract(py)?,
                armor: obj.getattr(py,"armor")?.extract(py)?,
                attack_range: obj.getattr(py,"attack_range")?.extract(py)?,
                can_be_attacked_by_air_weapons: obj.getattr(py,"can_be_attacked_by_air_weapons")?.extract(py)?,
                ground_weapons: obj.getattr(py,"ground_weapons")?.extract(py)?,
                is_basic_harvester: obj.getattr(py,"is_basic_harvester")?.extract(py)?,
                is_flying: obj.getattr(py,"is_flying")?.extract(py)?,
                is_melee: obj.getattr(py,"is_melee")?.extract(py)?,
                is_structure: obj.getattr(py,"is_structure")?.extract(py)?,
                max_health: obj.getattr(py,"max_health")?.extract(py)?,
                max_shield: obj.getattr(py,"max_shield")?.extract(py)?,
                movement_speed: obj.getattr(py,"movement_speed")?.extract(py)?,
                race: obj.getattr(py,"race")?.extract(py)?,
                type_data: obj.getattr(py,"type_data")?.extract(py)?,
                unit_radius: obj.getattr(py,"unit_radius")?.extract(py)?
            })
        }
    }
}

#[derive(Primitive, Debug, Copy, Clone, Eq, PartialEq)]
pub enum AbilityTarget {
    NONE =1,
    /// Ability targets a location.
    POINT=2,
    /// Ability targets another unit.
    UNIT=3,
    /// Ability can target either a location or a unit.
    POINTORUNIT=4,
    /// Ability can target either a location or nothing.
    POINTORNONE=5,
}

impl Default for AbilityTarget {
	fn default() -> Self {
		AbilityTarget::NONE
	}
}

impl ToPyObject for AbilityTarget{
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

impl<'source> FromPyObject<'source> for AbilityTarget{

	fn extract(ob: &'source PyAny)-> PyResult<Self>{
//        println!("{:?}", "Extracting Race");
		let ob1: i32 = ob.extract::<i32>().unwrap();
		let x : AbilityTarget = AbilityTarget::from_i32(ob1).unwrap_or_default();
		Ok(x).into_py_result()
	}
}
#[derive(Debug, Clone)]
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

impl ToPyObject for AbilityData{
	fn to_object(&self, py: Python) -> PyObject {
		self.to_object(py)
	}
}

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
                cast_range: obj.getattr(py, "p_cast_range")?.extract(py)?
            })
        }
    }
}

/// Upgrade data.
#[derive(Debug, Clone)]
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

impl ToPyObject for UpgradeData{
	fn to_object(&self, py: Python) -> PyObject {
		self.to_object(py)
	}
}

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
                research_time: obj.getattr(py, "p_research_time")?.extract(py)?
            })
        }
    }
}

pub struct GameInfo{
    ability_data: HashMap<u32, AbilityData>,
    unit_data: HashMap<u32, UnitTypeData>,
    upgrade_data: HashMap<u32, UpgradeData>
}

impl ToPyObject for GameInfo{
	fn to_object(&self, py: Python) -> PyObject {
		self.to_object(py)
	}
}

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
                ability_data: obj.getattr(py, "ability_data")?.extract(py)?,
                unit_data:  obj.getattr(py, "unit_data")?.extract(py)?,
                upgrade_data: obj.getattr(py, "upgrade_data")?.extract(py)?,
            })
        }
    }
}

impl GameInfo{
    pub(crate) fn get_unit_data(&self, unit_id: UnitTypeId) -> Option<&UnitTypeData>{
        let id: u32 =  unit_id.to_u32().unwrap();
        self.unit_data.get(id.borrow())
    }
}