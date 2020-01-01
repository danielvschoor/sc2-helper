import sc2
from sc2.ids.buff_id import BuffId
from sc2.bot_ai import *
from sc2.player import Bot, Computer
# from simulator import CombatUnit
# from OverReactBot import OverReactBot
from sc2 import Difficulty
from data_caching import DataCache

import pickle

class TestBot(sc2.BotAI):
    async def on_step(self, iteration):
        with open("cached_data/game_data.bin",'wb') as f:
            pickle.dump(self.game_data, f)
        input()
        



def main():
    sc2.run_game(
        sc2.maps.get("AutomatonLE"),
        [Bot(Race.Terran, TestBot()),  Computer(Race.Protoss, Difficulty.CheatVision)],
        realtime=False,
    )


if __name__ == "__main__":
    main()