# import sc2
from sc2.ids.unit_typeid import UnitTypeId
# from sc2.bot_ai import *
# from sc2.player import Bot, Computer
# from simulator import CombatUnit
# from OverReactBot import OverReactBot
# from sc2 import Difficulty
from data_caching import DataCache
# from simulator_python import CombatPredictor, CombatUnit
import simulator_python as sp
import time

# from sc2_helper import CombatUnit, CombatPredictor, CombatUnits
import sc2_helper as sh
def test_data():
    # start_time = time.time()
    # cu = CombatUnit(_owner=1, _unit_type=64, _health=100.0, _shield=1200.0, _flying=False)
    dc = DataCache()
    # or_start_time = time.time()
    # start_time1 = time.time()
    start_time2=time.time()
    cp = sh.CombatPredictor(_game_info=dc, path="C:\\Users\\danie\\Desktop\\Combat Simulator\\sc2-techtree\\data\\data.json")
    cp.init()
    end_time1 = time.time()
    print(end_time1-start_time2)
    

    cu = sp.CombatUnit(unit=None,owner=2, type=UnitTypeId.ROACH,health=145.0,flying=False).to_rust()
    cu2 = sp.CombatUnit(unit=None, owner=1, type=UnitTypeId.MARINE, health=45.0, flying=False).to_rust()
    cus = [cu for _ in range(100)]
    cus2 = [cu2 for _ in range(100)]
    cs = sh.CombatSettings()
    cs.debug = False
    
    for x in range(1):
        input("Press key")
        start_time2=time.time()
        # cp.units1 = cus
        # cp.units2 = cus
        w, health_left = cp.predict_engage(cus, cus2, 2, cs)

        end_time1 = time.time()
        print(end_time1-start_time2)
        print("Winner = ", w, " Health left=", health_left)

    print("Done")
    input()

# cu2 = sh.CombatUnit(_owner=cu.owner, 
#                     _unit_type=cu.type.value, 
#                     _health=cu.health, 
#                     _health_max=cu.health_max, 
#                     _shield=cu.shield, 
#                     _shield_max=cu.shield_max,
#                     _energy=cu.energy, 
#                     _flying=cu.is_flying, 
#                     _buff_timer=cu.buff_timer, 
#                     _data=cu.data)
# end_time2 = time.time()
# or_end_time = time.time()
# print(or_end_time-or_start_time)
# print(end_time1-start_time1)
# print(end_time2-start_time2)
# # start_time = time.time()
# cus = [cu for _ in range(50)]
# cus2 = [cu for _ in range(50)]
# end_time = time.time()
# print(end_time-start_time)
# start_time = time.time()
# cp = CombatPredictor(CombatUnits(cus), CombatUnits(cus2))
# winner = cp.predict_engage()
# end_time = time.time()
# print(end_time-start_time)
# print("winner=", winner)
# print(cu.len())
# cp=CombatPredictor(cu1, cu2)
# for x in cp.units:
#     print(x.health)


# cp = CombatPredictor([])

# start_time = time.time()
# unit = CombatUnit(1,2,100.0,False)
# print(unit.health)
# unit.health = 1000
# print(unit.health)
# end_time = time.time()
# print(end_time-start_time)

# from simulator_python import CombatUnit
# start_time = time.time()
# unit = CombatUnit(owner=1,type=2,health=100.0,flying=False)
# print(unit.health)
# unit.health = 1000
# print(unit.health)
# end_time = time.time()
# print(end_time-start_time)
import pickle
import sc2
from sc2.ids.unit_typeid import UnitTypeId
from sc2.bot_ai import *
from sc2.player import Bot, Computer
# from simulator import CombatUnit
# from OverReactBot import OverReactBot
from sc2 import Difficulty
from data_caching import DataCache
# from simulator_python import CombatPredictor, CombatUnit
import simulator_python as sp
import time

def save_data():   

    class TestBot(sc2.BotAI):
        def __init__(self):
            self.combat_predictor = None
        async def on_step(self, iteration):
            if iteration ==0:
                with open("cached_data/game_data.bin", "wb") as f:
                    data = self._game_data
                    pickle.dump(data,f)
                print("done")
                input()
                



    def main():
        sc2.run_game(
            sc2.maps.get("AutomatonLE"),
            [Bot(Race.Terran, TestBot()),  Computer(Race.Random, Difficulty.CheatInsane)],
            realtime=False,
        )
    main()


# import pickle
# with open("cached_data/game_data.bin", "rb") as f:
#     gd = pickle.load(f)

# print('Hello')

# save_data()
# dc = DataCache()
# save_data()
def test_predictor():

    class TestBot(sc2.BotAI):
        def __init__(self):
            self.combat_predictor = None
            self.dc = None
            self.combat_predictor = None
            self.combat_settings = sp.CombatSettings()

        async def micro(self):
            # enemy_combat_units = [sp.CombatUnit(unit=x).to_rust() for x in self.enemy_units]
            # my_units = [sp.CombatUnit(unit=x).to_rust() for x in self.units]
            enemy_units = []
            my_units = []
            attack_units = []
            for unit in self.units:
                close_enemies = self.enemy_units.closer_than(10, unit)
                if close_enemies:
                    my_units.append(sp.CombatUnit(unit=unit).to_rust())
                    attack_units.append(unit)
                    close_allies = self.units.closer_than(10, unit)
                    
                    for enemy_unit in close_enemies:
                        enemy_units.append(sp.CombatUnit(unit=enemy_unit).to_rust())
                    
                    for close_ally in close_allies:
                        my_units.append(sp.CombatUnit(unit=close_ally).to_rust())
                        attack_units.append(close_ally)
            
            if my_units and enemy_units:
                if self.combat_predictor.predict_engage(my_units, enemy_units, 1, self.combat_settings) == 1:
                    for unit in attack_units:
                        p = self.enemy_units.closest_to(unit)
                        if p:
                            self.do(unit.attack(p))
                else:
                    for unit in attack_units:
                        self.do(unit.move(self.start_location))


        async def on_step(self, iteration):
            
            if iteration ==0:
                self.dc = DataCache(self)
                self.combat_predictor = sh.CombatPredictor(_game_info=self.dc, path="C:\\Users\\danie\\Desktop\\Combat Simulator\\sc2-techtree\\data\\data_readable.json")
                self.combat_settings.debug = False
                for x in self.workers:
                    self.do(x.attack(self.enemy_start_locations[0]))

            if self.enemy_units:
                await self.micro()

                # enemy_combat_units = [sp.CombatUnit(unit=x).to_rust() for x in self.enemy_units]
                # my_units = [sp.CombatUnit(unit=x).to_rust() for x in self.units]
                # winner = self.combat_predictor.predict_engage(my_units, enemy_combat_units,1, self.combat_settings)
                # await self.chat_send(str(winner))
            


                



    def main():
        sc2.run_game(
            sc2.maps.get("AutomatonLE"),
            [Bot(Race.Terran, TestBot()),  Computer(Race.Random, Difficulty.VeryEasy)],
            realtime=False,
        )
    main()

print("press any key to continue")
input()
test_data()
# test_predictor()
