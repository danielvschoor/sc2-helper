import json
from sc2.ids.unit_typeid import UnitTypeId
from sc2.ids.upgrade_id import UpgradeId
from sc2.constants import TARGET_AIR, TARGET_BOTH, TARGET_GROUND

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
class Weapon:
    def __init__(self):
        pass


class TechTree:
    def __init__(self):
        self.tech_tree = self.load_tech_tree()
        self.units = self.load_units()
        self.abilities = self.load_abilities()
        self.upgrades = self.load_upgrades()

    def load_tech_tree(self, file='sc2-techtree\data\data.json'):
        with open(file, 'r') as f:
            tech_tree = json.load(f)
        return tech_tree
    
    def load_units(self):
        return self.tech_tree['Unit']
    
    def load_abilities(self):
        return self.tech_tree['Ability']
    
    def load_upgrades(self):
        return self.tech_tree['Upgrade']
    
    def unit_radius(self, unit):
        if isinstance(unit, UnitTypeId):
            unit = unit.value
        for x in self.units:
            if x['id'] == unit:
                return x['radius']
    
    def movement_speed(self, unit):
        if isinstance(unit, UnitTypeId):
            unit = unit.value
        for x in self.units:
            if x['id'] == unit:
                return x['speed']

    def is_structure(self, unit):
        if isinstance(unit, UnitTypeId):
            unit = unit.value
        for x in self.units:
            if x['id'] == unit:
                return x['is_structure']
    
    def is_flying(self, unit):
        if isinstance(unit, UnitTypeId):
            unit = unit.value
        for x in self.units:
            if x['id'] == unit:
                return x['is_flying']
    
    def is_melee(self, unit):
        return unit in IS_MELEE
        
    
    def is_basic_harvester(self, unit):
        return unit in IS_BASIC_HARVESTER
         
    
    def can_be_attacked_by_air_weapons(self, unit):
        return self.is_flying(unit) or unit == UnitTypeId.COLOSSUS
    
    def max_health(self, unit):
        if isinstance(unit, UnitTypeId):
            unit = unit.value
        for x in self.units:
            if x['id'] == unit:
                return x['max_health']
    
    def max_shield(self, unit):
        if isinstance(unit, UnitTypeId):
            unit = unit.value
        for x in self.units:
            if x['id'] == unit:
                return x.get('max_shield',0)

    def is_upgrade_with_levels(self, upgrade):
        return upgrade in IS_UPGRADE_WITH_LEVELS

    def race(self, unit):
        if isinstance(unit, UnitTypeId):
            unit = unit.value
        for x in self.units:
            if x['id'] == unit:
                return x['race']         
    
    def armor(self, unit):
        if isinstance(unit, UnitTypeId):
            unit = unit.value
        for x in self.units:
            if x['id'] == unit:
                return x.get('armor',0)  
    
    def air_weapons(self, unit):
        if isinstance(unit, UnitTypeId):
            unit = unit.value
        for x in self.units:
            if x['id'] == unit:
                return [weapon for weapon in x['weapons'] if weapon['target_type'] in TARGET_AIR]

    def ground_weapons(self, unit):
        pass
    
    def attack_range(self, unit_type: UnitTypeId=None):
        if isinstance(unit_type, UnitTypeId):
            unit_type = unit_type.value
        for x in self.units:
            if x['id'] == unit_type:
                for x in x['weapons']:
                    pass

        # _range = max(info.air_weapon.range(), info.ground_weapon.range())

        # return _range
          
def main():
    t = TechTree()
    t.tech_tree
    x = t.can_be_attacked_by_air_weapons(UnitTypeId.MARINE)
    x = t.air_weapons(UnitTypeId.MARINE)
    x
    
if __name__ == "__main__":
    main()