from sc2.ids.unit_typeid import UnitTypeId
from unit_info import UnitInfo
import pickle

# Constants
UNIT_DATA_CACHE_PATH = "cached_data/units.data"
UPGRADE_DATA_CACHE_PATH = "cached_data/upgrades.bin"
ABILITY_DATA_CACHE_PATH = "cached_data/abilities.bin"

class DataCache:
    def __init__(self, bot_object=None, game_data_file= 'cached_data/game_data.dat'):
        if bot_object:
            self.unit_data = bot_object.game_data.units
            self.ability_data = bot_object.game_data.abilities
            self.upgrade_data = bot_object.game_data.upgrades
        else:
            self.game_data = self.load_data(game_data_file)
            self.unit_data = self.load_unit_data()
            self.ability_data = self.load_ability_data()
            self.upgrade_data = self.load_upgrade_data()

    def get_unit_data(self, unit_type):
        if isinstance(unit_type, UnitTypeId):
            unit_type = unit_type.value
        return UnitInfo(self.unit_data.get(unit_type, None))
    
    def save_unit_data(self, unit_types, path=UNIT_DATA_CACHE_PATH):
        pass
    
    def load_unit_data(self):
        return self.game_data.units

    def load_data(self, file):
        with open(file,'rb') as f:
            game_data = pickle.load(f)
        return game_data

    def save_ability_data(self, abilities):
        pass

    def save_upgrade_data(self, upgrades):
        pass

    def load_ability_data(self):
        return self.game_data.abilities

    def load_upgrade_data(self):
        return self.game_data.upgrades

def main():
    x = DataCache()
    p = x.get_unit_data(UnitTypeId.SIEGETANKSIEGED)
    air= p.ground_weapons
    x= p.attack_range
    air
    

if __name__ == "__main__":
    main()