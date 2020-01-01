from simulator import *
from combat_upgrades import *
from data_caching import DataCache
from tech_tree import TechTree

from sc2.ids.unit_typeid import UnitTypeId
from sc2.ids.upgrade_id import UpgradeId
from sc2.data import Race
from sc2.constants import TARGET_AIR, TARGET_BOTH, TARGET_GROUND
from sc2.dicts.unit_abilities import UNIT_ABILITIES

class WeaponInfo:
    def __init__(self, base_dps=0, available=False, splash=0, weapon=None, type=None, upgrades=None, target_upgrades=None, unit_types=None):
        self._dps_cache = []
        if not type:
            self._base_dps = base_dps
            self.available = available
            self.splash = splash
            self.weapon = weapon
        else:
            # assert(upgrades is not None and target_upgrades is not None)
            self.available = True
            self.splash = 0
            self.weapon = weapon
            # TODO: Range upgrades!

            self._base_dps = (weapon.damage + get_damage_bonus(type, upgrades)) * weapon.attacks / weapon.speed

            if not unit_types:
                dc = DataCache()
                unit_types = dc.unit_data
            tech_tree = TechTree()
            for i in range(len(unit_types)):
                self._dps_cache.append(calculate_dps(type, i, weapon, upgrades, target_upgrades,tech_tree))         

    
    def get_dps(self, target=None, modifier=0):
        if not target:
            return self._base_dps
        else:
            if not available:
                return 0
    
            if int(target) >= len(self._dps_cache):
                raise BaseException
            

            # TODO: Modifier ignores speed upgrades
            return max(0.0, self._dps_cache[int(target)] + modifier*weapon.attacks/weapon.speed)
    
    def range(self):
         return self.weapon.range if self.weapon else 0

class UnitCombatInfo:
    def __init__(self, type, upgrades, target_upgrades, data_cache=None):
        self.ground_weapon: WeaponInfo = WeaponInfo()
        self.air_weapon: WeaponInfo = WeaponInfo()
        if not data_cache:
            dc = DataCache()
            data = dc.get_unit_data(type)
        else:
            data = data_cache.get_unit_data(type)
        if data and hasattr(data, "weapons"):
            for weapon in data.weapons:
                if weapon.type in TARGET_AIR:
                    if self.air_weapon.available:
                        print("Weapon already used")
                        assert(False)
                    self.air_weapon = WeaponInfo(weapon=weapon,type= type,upgrades= upgrades, target_upgrades=target_upgrades)
                
                if weapon.type in TARGET_GROUND:
                    if self.ground_weapon.available:
                        print("Weapon already used")
                        assert(False)
                    self.ground_weapon = WeaponInfo(weapon=weapon,type= type, upgrades= upgrades, target_upgrades=target_upgrades)
    
    def attack_interval(self):
        v = math.inf
        if (self.air_weapon.weapon):
            v = self.air_weapon.weapon.speed
        if (self.ground_weapon.weapon):
            v = min(v, self.ground_weapon.weapon.speed)
        return v

class CombatEnvironment:
    def __init__(self, upgrades=None, target_upgrades=None, unit_types=None):
        dc = DataCache()
        self.combat_info = [[],[]]
        self.upgrades = []
        if upgrades and target_upgrades:
            for upgrade in upgrades | target_upgrades: # TODO: Are they really combined?
                self.upgrades.append([upgrade, 2])
        if not unit_types:
            unit_types = dc.unit_data

        for i in range(len(unit_types)):
            self.combat_info[0].append(UnitCombatInfo(i, upgrades, target_upgrades, dc))
        
        for i in range(len(unit_types)):
            self.combat_info[1].append(UnitCombatInfo(i, target_upgrades, upgrades, dc))
        
        for owner in range(2):
            self.combat_info[owner][UnitTypeId.LIBERATOR.value].air_weapon.splash = 3.0
            self.combat_info[owner][UnitTypeId.MISSILETURRET.value].air_weapon.splash = 3.0;
            self.combat_info[owner][UnitTypeId.SIEGETANKSIEGED.value].ground_weapon.splash = 4.0;
            self.combat_info[owner][UnitTypeId.HELLION.value].ground_weapon.splash = 2;
            self.combat_info[owner][UnitTypeId.HELLIONTANK.value].ground_weapon.splash = 3;
            self.combat_info[owner][UnitTypeId.MUTALISK.value].ground_weapon.splash = 1.44;
            self.combat_info[owner][UnitTypeId.MUTALISK.value].air_weapon.splash = 1.44;
            self.combat_info[owner][UnitTypeId.THOR.value].air_weapon.splash = 3;
            self.combat_info[owner][UnitTypeId.ARCHON.value].ground_weapon.splash = 3;
            self.combat_info[owner][UnitTypeId.ARCHON.value].air_weapon.splash = 3;
            self.combat_info[owner][UnitTypeId.COLOSSUS.value].ground_weapon.splash = 3;
    
    def attack_range(self, owner: int=None, type: UnitTypeId=None, unit=None):
        if not owner or not type:
            assert(unit is not None)
            info = self.combat_info[unit.owner - 1][unit.type.value]
            _range = max(info.air_weapon.range(), info.ground_weapon.range())

            if unit.type == UnitTypeId.COLOSSUS and upgrades[unit.owner-1].has_upgrade(UpgradeId.EXTENDEDTHERMALLANCE):
                _range += 2

            return _range
        else:
            info = self.combat_info[owner - 1][type];
            return max(info.air_weapon.range(), info.ground_weapon.range())

    def get_combat_info(self, unit):
        return self.combat_info[unit.owner - 1][unit.type]
    
    def calculate_dps(self, owner: int=None, type: UnitTypeId=None, air: bool=None, units: list=None, unit=None, unit1=None, unit2=None):
        if owner and type and air:
            return self.calculate_dps1(owner, type, air)
        
        elif units and air:
            return self.calculate_dps2(units, air)
        
        elif unit and air:
            return self.calculate_dps3(unit, air)
        
        elif unit1 and unit2:
            return self.calculate_dps4(unit1, unit2)

    
    def calculate_dps1(self, owner: int, type: UnitTypeId, air: bool):
        info = self.combat_info[owner - 1][type.value]
        return info.air_weapon.get_dps() if air else info.ground_weapon.get_dps()
    
    def calculate_dps2(self, units: list, air: bool):
        dps = 0
        for u in units:
            dps += self.calculate_dps1(u.owner, u.type, air)
        return dps

    def calculate_dps3(self, unit, air: bool):
        info = self.combat_info[unit.owner - 1][unit.type.value]
        return info.air_weapon.get_dps() if air else info.ground_weapon.get_dps()
    
    def calculate_dps4(self, unit1=None, unit2=None):
        info = self.combat_info[unit1.owner - 1][unit1.type.value]
        return max(info.ground_weapon.get_dps(unit2.type), info.air_weapon.get_dps(unit2.type))

def get_damage_bonus(unit: UnitTypeId, upgrades):
    tech_tree = TechTree()
    dc = DataCache()
    
    if tech_tree.is_structure(unit):
        return 0
    
    bonus = 0
    race = tech_tree.race(unit)

    if race == Race.Protoss:
        if tech_tree.is_flying(unit):
            if upgrades.has_upgrade(UpgradeId.PROTOSSAIRWEAPONSLEVEL1):
                bonus += 1
            if upgrades.has_upgrade(UpgradeId.PROTOSSAIRWEAPONSLEVEL2):
                bonus += 1
            if upgrades.has_upgrade(UpgradeId.PROTOSSAIRWEAPONSLEVEL3):
                bonus += 1
        else:
            if upgrades.has_upgrade(UpgradeId.PROTOSSGROUNDWEAPONSLEVEL1):
                    bonus += 1
            if upgrades.has_upgrade(UpgradeId.PROTOSSGROUNDWEAPONSLEVEL2):
                    bonus += 1
            if upgrades.has_upgrade(UpgradeId.PROTOSSGROUNDWEAPONSLEVEL3):
                    bonus += 1
        
    elif race == Race.Zerg:
        if tech_tree.is_flying(unit):
            if upgrades.has_upgrade(UpgradeId.ZERGFLYERWEAPONSLEVEL1):
                bonus += 1
            if upgrades.has_upgrade(UpgradeId.ZERGFLYERWEAPONSLEVEL2):
                bonus += 1
            if upgrades.has_upgrade(UpgradeId.ZERGFLYERWEAPONSLEVEL3):
                bonus += 1
        elif tech_tree.is_melee(unit):
            if upgrades.has_upgrade(UpgradeId.ZERGMELEEWEAPONSLEVEL1):
                bonus += 1
            if upgrades.has_upgrade(UpgradeId.ZERGMELEEWEAPONSLEVEL2):
                bonus += 1
            if upgrades.has_upgrade(UpgradeId.ZERGMELEEWEAPONSLEVEL3):
                bonus += 1
        else:
            if upgrades.has_upgrade(UpgradeId.ZERGMISSILEWEAPONSLEVEL1):
                bonus += 1
            if upgrades.has_upgrade(UpgradeId.ZERGMISSILEWEAPONSLEVEL12):
                bonus += 1
            if upgrades.has_upgrade(UpgradeId.ZERGMISSILEWEAPONSLEVEL3):
                bonus += 1
    
    elif race == Race.Terran:
        canonical = canonicalize(unit, dc)
        casters = ability_to_caster_unit(dc.get_unit_data(canonical).ability_id)
        if len(casters())>0 and casters[0] == UnitTypeId.BARRACKS:
            if upgrades.has_upgrade(UpgradeId.TERRANINFANTRYWEAPONSLEVEL1):
                bonus += 1
            if upgrades.has_upgrade(UpgradeId.TERRANINFANTRYWEAPONSLEVEL2):
                bonus += 1
            if upgrades.has_upgrade(UpgradeId.TERRANINFANTRYWEAPONSLEVEL3):
                bonus += 1
        elif len(casters())>0 and casters[0] == UnitTypeId.FACTORY:
            if upgrades.has_upgrade(UpgradeId.TERRANVEHICLEWEAPONSLEVEL1):
                bonus += 1
            if upgrades.has_upgrade(UpgradeId.TERRANVEHICLEWEAPONSLEVEL2):
                bonus += 1
            if upgrades.has_upgrade(UpgradeId.TERRANVEHICLEWEAPONSLEVEL3):
                bonus += 1
        elif len(casters())>0 and casters[0] == UnitTypeId.STARPORT:
            if upgrades.has_upgrade(UpgradeId.TERRANSHIPWEAPONSLEVEL1):
                bonus += 1
            if upgrades.has_upgrade(UpgradeId.TERRANSHIPWEAPONSLEVEL2):
                bonus += 1
            if upgrades.has_upgrade(UpgradeId.TERRANSHIPWEAPONSLEVEL3):
                bonus += 1
    
    return bonus

def canonicalize(unit, dc=None):
    if not dc:
        dc = DataCache()
    unit_data = dc.get_unit_data(unit)
    if unit_data.unit_alias:
        return unit_data.unit_alias
    else:
        return unit

def ability_to_caster_unit(ability):
    for key, value in UNIT_ABILITIES.items():
        if ability in value:
            return key

def get_armor_bonus(unit: UnitTypeId, upgrades):
    tech_tree = TechTree()
    dc = DataCache()
    race = tech_tree.armor(unit)
    if tech_tree.is_structure(unit):
        if race == Race.Terran and upgrades.has_upgrade(UpgradeId.TERRANBUILDINGARMOR):
            return 2
        
        return 0
    
    bonus = 0

    if race == Race.Protoss:
        if tech_tree.is_flying(unit):
            if upgrades.has_upgrade(UpgradeId.PROTOSSAIRARMORSLEVEL1):
                bonus += 1
            if (upgrades.has_upgrade(UpgradeId.PROTOSSAIRARMORSLEVEL2)):
                bonus += 1
            if upgrades.has_upgrade(UpgradeId.PROTOSSAIRARMORSLEVEL3):
                bonus += 1
        else: 
            if upgrades.has_upgrade(UpgradeId.PROTOSSGROUNDARMORSLEVEL1):
                 bonus += 1
            if upgrades.has_upgrade(UpgradeId.PROTOSSGROUNDARMORSLEVEL2):
                bonus += 1
            if upgrades.has_upgrade(UpgradeId.PROTOSSGROUNDARMORSLEVEL3):
                bonus += 1
        

        if upgrades.has_upgrade(UpgradeId.PROTOSSSHIELDSLEVEL1):
            bonus += 1
        if upgrades.has_upgrade(UpgradeId.PROTOSSSHIELDSLEVEL2):
            bonus += 1
        if upgrades.has_upgrade(UpgradeId.PROTOSSSHIELDSLEVEL3):
            bonus += 1
            
    elif race == Race.Zerg:
        if tech_tree.is_flying(unit):
            if upgrades.has_upgrade(UpgradeId.ZERGFLYERARMORSLEVEL1):
                bonus += 1
            if upgrades.has_upgrade(UpgradeId.ZERGFLYERARMORSLEVEL2):
                bonus += 1
            if upgrades.has_upgrade(UpgradeId.ZERGFLYERARMORSLEVEL3):
                bonus += 1
        else: 
            if upgrades.has_upgrade(UpgradeId.ZERGGROUNDARMORSLEVEL1):
                bonus += 1
            if upgrades.has_upgrade(UpgradeId.ZERGGROUNDARMORSLEVEL2):
                bonus += 1
            if upgrades.has_upgrade(UpgradeId.ZERGGROUNDARMORSLEVEL3):
                bonus += 1
    
    elif race == Race.Terran: 
        canonical = canonicalize(unit, dc)
        casters = ability_to_caster_unit(dc.get_unit_data(canonical).ability_id)
        if (len(casters) > 0 and casters[0] == UnitTypeId.BARRACKS):
            if upgrades.has_upgrade(UpgradeId.TERRANINFANTRYARMORSLEVEL1):
                bonus += 1
            if upgrades.has_upgrade(UpgradeId.TERRANINFANTRYARMORSLEVEL2):
                bonus += 1
            if upgrades.has_upgrade(UpgradeId.TERRANINFANTRYARMORSLEVEL3):
                bonus += 1
        elif len(casters) > 0 and (casters[0] == UnitTypeId.FACTORY or casters[0] == UnitTypeId.STARPORT):
            if upgrades.has_upgrade(UpgradeId.TERRANVEHICLEANDSHIPARMORSLEVEL1):
                bonus += 1
            if upgrades.has_upgrade(UpgradeId.TERRANVEHICLEANDSHIPARMORSLEVEL2):
                bonus += 1
            if upgrades.has_upgrade(UpgradeId.TERRANVEHICLEANDSHIPARMORSLEVEL3):
                bonus += 1
            

    return bonus

def calculate_dps(attacker: UnitTypeId, target: UnitTypeId, weapon, attacker_upgrades, target_upgrades, tech_tree=None):
    # canBeAttackedByAirWeapons is primarily for coloussus.
    can_attack_air = weapon.type in TARGET_AIR
    if not tech_tree:
        tech_tree = TechTree()
    can_be_attacked_by_air_weapons = tech_tree.can_be_attacked_by_air_weapons(target)
    if can_be_attacked_by_air_weapons and can_attack_air:
        dmg = weapon.damage
        for b in weapon.damage_bonus:
            if b.attribute in target._type_data.attributes:
                dmg += b.bonus
            
        

        dmg += get_damage_bonus(attacker, attacker_upgrades)
        
        armor = tech_tree.armor(target) + get_armor_bonus(target, target_upgrades)

        # Note: cannot distinguish between damage to shields and damage to health yet, so average out so that the armor is about 0.5 over the whole shield+health of the unit
        # Important only for protoss
        armor *= tech_tree.max_health(target) / (tech_tree.max_health(target) + tech_tree.max_shield(target))

        time_between_attacks = weapon.speed

        if attacker == UnitTypeId.ADEPT and attacker_upgrades.has_upgrade(UpgradeId.ADEPTPIERCINGATTACK):
             time_between_attacks /= 1.45

        return max(0.0, dmg - armor) * weapon.attacks / time_between_attacks;
    

    return 0;
