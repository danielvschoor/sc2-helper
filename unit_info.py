from sc2.ids.unit_typeid import UnitTypeId
from sc2.ids.upgrade_id import UpgradeId
from sc2.game_data import UnitTypeData
from sc2.constants import TARGET_AIR, TARGET_BOTH, TARGET_GROUND
from sc2.data import TargetType, Attribute

from tech_tree import TechTree
from functools import lru_cache as cache
from sc2.cache import property_cache_forever
IS_MELEE = {
    UnitTypeId.PROBE,
    UnitTypeId.ZEALOT,
    UnitTypeId.DARKTEMPLAR,
    UnitTypeId.SCV,
    UnitTypeId.HELLIONTANK,
    UnitTypeId.DRONE,
    UnitTypeId.ZERGLING,
    UnitTypeId.ZERGLINGBURROWED,
    UnitTypeId.BANELING,
    UnitTypeId.BANELINGBURROWED,
    UnitTypeId.ULTRALISK,
    UnitTypeId.BROODLING
    }

IS_BASIC_HARVESTER ={
    UnitTypeId.SCV,
    UnitTypeId.DRONE,
    UnitTypeId.PROBE,
}

IS_UPGRADE_WITH_LEVELS={
   UpgradeId.TERRANINFANTRYWEAPONSLEVEL1,
   UpgradeId.TERRANINFANTRYARMORSLEVEL1,
   UpgradeId.TERRANVEHICLEWEAPONSLEVEL1,
   UpgradeId.TERRANSHIPWEAPONSLEVEL1,
   UpgradeId.PROTOSSGROUNDWEAPONSLEVEL1,
   UpgradeId.PROTOSSGROUNDARMORSLEVEL1,
   UpgradeId.PROTOSSSHIELDSLEVEL1,
   UpgradeId.ZERGMELEEWEAPONSLEVEL1,
   UpgradeId.ZERGGROUNDARMORSLEVEL1,
   UpgradeId.ZERGMISSILEWEAPONSLEVEL1,
   UpgradeId.ZERGFLYERWEAPONSLEVEL1,
   UpgradeId.ZERGFLYERARMORSLEVEL1,
   UpgradeId.PROTOSSAIRWEAPONSLEVEL1,
   UpgradeId.PROTOSSAIRARMORSLEVEL1,
   UpgradeId.TERRANVEHICLEANDSHIPARMORSLEVEL1,
}

class WeaponInfo:
    def __init__(self, weapon=None):
        if weapon:
            self._weapon = weapon
            self._weapon_type = weapon.type
            self._attacks = weapon.attacks
            self._damage = weapon.damage
            self._damage_bonus = weapon.damage_bonus
            self._range = weapon.range
            self._speed = weapon.speed
            self._base_dps = weapon.damage * weapon.attacks / weapon.speed
            self._dps_cache = []
            self._splash = 0
        else:
            self._weapon = None
            self._weapon_type = None
            self._attacks = 0
            self._damage = 0
            self._damage_bonus = []
            self._range = 0
            self._speed = 0
            self._base_dps = 0
            self._dps_cache = []
            self._splash = 0
    
    def init_battle_cruiser_air(self):
        self._weapon_type = TargetType.Air.value
        self._damage_ = 5
        self._damage_bonus = []
        self._attacks = 1
        self._range = 6
        self._speed = 0.16 * 1.4
    
    def init_battle_cruiser_ground(self):
        self._weapon_type = TargetType.Ground.value
        self._damage_ = 8
        self._damage_bonus = []
        self._attacks = 1
        self._range = 6
        self._speed = 0.16 * 1.4
    

    
    @property
    def get_dps(self, target=None, modifier=0):
        return self._base_dps
    
    @property
    def range(self):
         return self._range 
    
    @property
    def splash(self):
        return self._splash

class UnitInfo:
    def __init__(self, unit_type_data: UnitTypeData):
        self.type_data = unit_type_data
        self.init_attributes()
        self._tech_tree = TechTree()
    
    def init_attributes(self):
        self._ability_id = self.type_data._proto.ability_id
        self._armor = self.type_data._proto.armor
        self._attributes = [Attribute(x) for x in self.type_data._proto.attributes]
        self._available = self.type_data._proto.available
        self._build_time = self.type_data._proto.build_time
        self._cargo_size = self.type_data._proto.cargo_size
        self._food_provided = self.type_data._proto.food_provided
        self._food_required = self.type_data._proto.food_required
        self._has_minerals = self.type_data._proto.has_minerals
        self._has_vespene = self.type_data._proto.has_vespene
        self._mineral_cost = self.type_data._proto.mineral_cost
        self._movement_speed = self.type_data._proto.movement_speed
        self._name = self.type_data._proto.name
        self._race = self.type_data._proto.race
        self._require_attached = self.type_data._proto.require_attached
        self._sight_range = self.type_data._proto.sight_range
        self._tech_alias = self.type_data._proto.tech_alias
        self._tech_requirement = self.type_data._proto.tech_requirement
        self._unit_alias = self.type_data._proto.unit_alias
        self._unit_id = self.type_data._proto.unit_id
        self._vespene_cost = self.type_data._proto.vespene_cost
        self._weapons = self.type_data._proto.weapons
        self._id = self.type_data.id
        self._is_structure = None
        self._unit_radius = None
        self._is_flying = None
        self._is_melee = None
        self._is_basic_harvester = None
        self._can_be_attacked_by_air_weapons = None
        self._max_health = None
        self._max_shield = None
        self._air_weapons = None
        self._ground_weapons = None
        self._attack_range = None

    def get_unit_info_as_dict(self):
        attribs = {k:v for k,v in self.__dict__.items() if not k.startswith("_")}
        for name in dir(self.__class__):
            obj = getattr(self.__class__, name)
            if isinstance(obj, property):
                obj.__get__(self, self.__class__)
                val = obj.__get__(self, self.__class__)
                attribs[name]=val

        return attribs

    @property
    def unit_radius(self):
        if not self._unit_radius:
            for x in self._tech_tree.units:
                if x['id'] == self._unit_id:
                    self._unit_radius = x['radius']
                    break
        
        return self._unit_radius
    
    @property
    def movement_speed(self):
        return self._movement_speed
    
    
    @property
    def is_structure(self):
        if not self._is_structure:
            for x in self._tech_tree.units:
                if x['id'] == self._unit_id:
                    self._is_structure = x['is_structure']
                    break
        
        return self._is_structure
    
    
    @property
    def is_flying(self):
        if not self._is_flying: 
            for x in self._tech_tree.units:
                if x['id'] == self._unit_id:
                    self._is_flying = x['is_flying']
                    break
        return self._is_flying
        
    
    @property
    def is_melee(self):
        if not self._is_melee:
            self._is_melee = self._id in IS_MELEE
        return self._is_melee
        
    
    @property
    def is_basic_harvester(self):
        if not self._is_basic_harvester:
            self._is_basic_harvester = self._id in IS_BASIC_HARVESTER
        return self._is_basic_harvester
        
    @property
    def can_be_attacked_by_air_weapons(self):
        if not self._can_be_attacked_by_air_weapons:
            self._can_be_attacked_by_air_weapons = self.is_flying or self._id == UnitTypeId.COLOSSUS
        return self._can_be_attacked_by_air_weapons
    
    
    @property
    def max_health(self):
        if not self._max_health:                
            for x in self._tech_tree.units:
                if x['id'] == self._unit_id:
                    self._max_health= x['max_health']
                    break
        return self._max_health
    
    
    @property
    def max_shield(self):
        if not self._max_shield:
            for x in self._tech_tree.units:
                if x['id'] == self._unit_id:
                    self._max_shield = x.get('max_shield',0)
                    break
        return self._max_shield

    # def is_upgrade_with_levels(self, upgrade):
    #     return upgrade in IS_UPGRADE_WITH_LEVELS

    @property
    def race(self):
       return self._race        
    
    @property
    def armor(self):
        return self._armor
    
    
    @property
    def air_weapons(self):
        if not self._air_weapons:
            if self._id == UnitTypeId.BATTLECRUISER:
                weapon = WeaponInfo()
                weapon.init_battle_cruiser_air() 
                return weapon
            air=[WeaponInfo(x) for x in self._weapons if x.type in TARGET_AIR]
            if len(air) >0:
                self._air_weapons =  air[0]
            else:
                self._air_weapons = WeaponInfo()
        
        return self._air_weapons
    
    
    @property
    def ground_weapons(self):
        if not self._ground_weapons:
            if self._id == UnitTypeId.BATTLECRUISER:
                weapon = WeaponInfo()
                weapon.init_battle_cruiser_ground() 
                return weapon
            
            ground=[WeaponInfo(x) for x in self._weapons if x.type in TARGET_GROUND]
            if len(ground) >0:
                self._ground_weapons = ground[0]
            else:
                self._ground_weapons = WeaponInfo()
        return self._ground_weapons
    
    
    @property
    def attack_range(self):
        if not self._attack_range:
            self._attack_range = max(0, self.ground_weapons.range, self.air_weapons.range)
        return self._attack_range
        # _range = max(self.air_weapon.range(), info.ground_weapon.range())

        #     if unit.type == UnitTypeId.COLOSSUS and upgrades[unit.owner-1].has_upgrade(UpgradeId.EXTENDEDTHERMALLANCE):
        #         _range += 2

        #     return _range
        # else:
        #     info = self.combat_info[owner - 1][type]
        #     return max(info.air_weapon.range(), info.ground_weapon.range())

    @property
    def ability_id(self):
        return self._ability_id
    