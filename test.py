# import sc2
# from sc2.ids.buff_id import BuffId
# from sc2.bot_ai import *
# from sc2.player import Bot, Computer
# # from simulator import CombatUnit
# # from OverReactBot import OverReactBot
# from sc2 import Difficulty
# from data_caching import DataCache
# from simulator_python import CombatPredictor, CombatUnit
import time

from sc2_helper import CombatUnits, CombatPredictor, CombatUnit

# cu = CombatUnits()
# cu.add_multiple([(1,2,100.0,False)])
start_time = time.time()
cus = [CombatUnit(1,2,100.0,False) for _ in range(1000)]
cus2 = [CombatUnit(1,2,100.0,False) for _ in range(1000)]
cp = CombatPredictor(CombatUnits(cus), CombatUnits(cus2))
winner = cp.predict_engage()
end_time = time.time()
print(end_time-start_time)
print("winner=", winner)
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
# class TestBot(sc2.BotAI):
#     def __init__(self):
#         self.combat_predictor = None
#     async def on_step(self, iteration):
#         if not self.combat_predictor:
#             self.combat_predictor = CombatPredictor(bot_object=self)
#         if iteration ==0:
#             for u in self.units:
#                 self.do(u.attack(self.enemy_start_locations[0]))
#         if self.enemy_units:
#             my_units = self.units
#             enemy_units = self.enemy_units
#             self.combat_predictor.units1 = [CombatUnit(unit=x) for x in my_units]
#             self.combat_predictor.units2 = [CombatUnit(unit=x) for x in self.enemy_units]
#             start = time.time()
#             r = self.combat_predictor.predict_engage()
#             end = time.time()
#             print(end-start)
#             winner = self.combat_predictor.owner_with_best_outcome()
#             print(winner)
            



# def main():
#     sc2.run_game(
#         sc2.maps.get("AutomatonLE"),
#         [Bot(Race.Terran, TestBot()),  Computer(Race.Random, Difficulty.CheatInsane)],
#         realtime=False,
#     )


# if __name__ == "__main__":
#     main()