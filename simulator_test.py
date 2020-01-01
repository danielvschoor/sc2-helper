from simulator import *
from data_caching import DataCache
from tech_tree import TechTree

import math

def combat_winner(predictor: CombatPredictor, state: CombatState):
    return predictor.predict_engage(state).state.owner_with_best_outcome()

def unit_test_surround(tech_tree=None):
    if not tech_tree:
        tech_tree = TechTree()
    # One unit can be surrounded by 6 melee units and attacked by all 6 at the same time
    assert(max_surround(pow(tech_tree.unit_radius(UnitTypeId.MARINE),2) * math.pi * 1, 1,tech_tree).max_attackers_per_defender == 6)
    assert(max_surround(pow(tech_tree.unit_radius(UnitTypeId.MARINE), 2) * math.pi * 1, 1,tech_tree).max_melee_attackers == 6)

    # Two units can be surrounded by 8 melee units, but each one can only be attacked by at most 4
    assert(max_surround(pow(tech_tree.unit_radius(UnitTypeId.MARINE), 2) * math.pi * 2, 2,tech_tree).max_attackers_per_defender == 4)
    assert(max_surround(pow(tech_tree.unit_radius(UnitTypeId.MARINE), 2) * math.pi * 2, 2,tech_tree).max_melee_attackers == 8)

    # Two units can be surrounded by 9 melee units, but each one can only be attacked by at most 3
    assert(max_surround(pow(tech_tree.unit_radius(UnitTypeId.MARINE), 2) * math.pi * 3, 3,tech_tree).max_attackers_per_defender == 3)
    assert(max_surround(pow(tech_tree.unit_radius(UnitTypeId.MARINE), 2) * math.pi * 3, 3,tech_tree).max_melee_attackers == 9)

    assert(max_surround(pow(tech_tree.unit_radius(UnitTypeId.MARINE), 2) * math.pi * 4, 4,tech_tree).max_attackers_per_defender == 3)
    assert(max_surround(pow(tech_tree.unit_radius(UnitTypeId.MARINE), 2) * math.pi * 4, 4,tech_tree).max_melee_attackers == 10)

    # One thor can be attacked by 10 melee units at a time.
    # This seems to be slightly incorrect, the real number is only 9, but it's approximately correct at least
    assert(max_surround(pow(tech_tree.unit_radius(UnitTypeId.THOR), 2) * math.pi * 1, 1,tech_tree).max_attackers_per_defender == 10)
    assert(max_surround(pow(tech_tree.unit_radius(UnitTypeId.THOR), 2) * math.pi * 1, 1,tech_tree).max_melee_attackers == 10)

def main():
    data_cache = DataCache()
    tech_tree = TechTree()

    predictor = CombatPredictor()
    predictor.init()
    unit_test_surround(tech_tree)
    x = combat_winner(predictor, CombatState([
		make_unit(1, UnitTypeId.VIKINGFIGHTER,tech_tree),
        make_unit(2, UnitTypeId.COLOSSUS,tech_tree),]))
    assert(x) == 1

    assert(combat_winner(predictor, 
    CombatState(
        [
            make_unit(1, UnitTypeId.PYLON,tech_tree),
            make_unit(1, UnitTypeId.PHOTONCANNON,tech_tree),
            make_unit(1, UnitTypeId.SHIELDBATTERY,tech_tree),
            make_unit(1, UnitTypeId.SHIELDBATTERY,tech_tree),
            make_unit(1, UnitTypeId.SHIELDBATTERY,tech_tree),
            make_unit(2, UnitTypeId.MEDIVAC,tech_tree),
            make_unit(2, UnitTypeId.MARINE,tech_tree)
            ]
            )) == 1)

	# Medivacs are pretty good (this is a very narrow win though)
    assert(combat_winner(predictor, CombatState([
		make_unit(1, UnitTypeId.MARINE,tech_tree),
		make_unit(1, UnitTypeId.MEDIVAC,tech_tree),
		make_unit(2, UnitTypeId.MARINE,tech_tree),
		make_unit(2, UnitTypeId.MARINE,tech_tree),
	])) == 1)

	# 1 marine wins against 1 zergling
    assert(combat_winner(predictor, CombatState([
		CombatUnit(1, UnitTypeId.MARINE, 50, False),
		CombatUnit(2, UnitTypeId.ZERGLING, 35, False),
	])) == 1)

	# Symmetric
    assert(combat_winner(predictor, CombatState([
		CombatUnit(2, UnitTypeId.MARINE, 50, False),
		CombatUnit(1, UnitTypeId.ZERGLING, 35, False),
	])) == 2)

	# 4 marines win against 4 zerglings
    assert(combat_winner(predictor, CombatState([
		CombatUnit(1, UnitTypeId.MARINE, 50, False),
		CombatUnit(1, UnitTypeId.MARINE, 50, False),
		CombatUnit(1, UnitTypeId.MARINE, 50, False),
		CombatUnit(1, UnitTypeId.MARINE, 50, False),
		CombatUnit(2, UnitTypeId.ZERGLING, 35, False),
		CombatUnit(2, UnitTypeId.ZERGLING, 35, False),
		CombatUnit(2, UnitTypeId.ZERGLING, 35, False),
		CombatUnit(2, UnitTypeId.ZERGLING, 35, False),
	])) == 1)

    assert(combat_winner(predictor, CombatState([
		CombatUnit(1, UnitTypeId.MARINE, 50, False),
		CombatUnit(2, UnitTypeId.ZERGLING, 35, False),
		CombatUnit(2, UnitTypeId.ZERGLING, 35, False),
		CombatUnit(2, UnitTypeId.ZERGLING, 35, False),
	])) == 2)

    assert(combat_winner(predictor, CombatState([
		make_unit(1, UnitTypeId.SPORECRAWLER,tech_tree),
		make_unit(1, UnitTypeId.SPORECRAWLER,tech_tree),
        make_unit(1, UnitTypeId.SPORECRAWLER,tech_tree),
        make_unit(2, UnitTypeId.REAPER,tech_tree),
        make_unit(2, UnitTypeId.REAPER,tech_tree),
        make_unit(2, UnitTypeId.REAPER,tech_tree),
	])) == 2)

    assert(combat_winner(predictor, CombatState([
		CombatUnit(1, UnitTypeId.LIBERATOR, 180, True),
		CombatUnit(2, UnitTypeId.BROODLORD, 225, True),
		CombatUnit(2, UnitTypeId.BROODLORD, 225, True),
		CombatUnit(2, UnitTypeId.BROODLORD, 225, True),
		CombatUnit(2, UnitTypeId.BROODLORD, 225, True),
		CombatUnit(2, UnitTypeId.BROODLORD, 225, True),
	    ])) == 1)

    assert(combat_winner(predictor, CombatState([
		CombatUnit(1, UnitTypeId.CYCLONE, 180, True),
		CombatUnit(2, UnitTypeId.IMMORTAL, 200, True),
	])) == 2)

    assert(combat_winner(predictor, CombatState([
		make_unit(1, UnitTypeId.BATTLECRUISER,tech_tree),
		make_unit(2, UnitTypeId.THOR,tech_tree),
	])) == 1)

    assert(combat_winner(predictor, CombatState([
		make_unit(1, UnitTypeId.INFESTOR,tech_tree),
		make_unit(2, UnitTypeId.BANSHEE,tech_tree),
	])) == 1)

	# Wins due to splash damage
	# Really depends on microing though...
	# assert(combat_winner(predictor, CombatState([
	# 	CombatUnit(1, UnitTypeId.LIBERATOR, 180, True),
	# 	CombatUnit(1, UnitTypeId.LIBERATOR, 180, True),
	# 	CombatUnit(1, UnitTypeId.LIBERATOR, 180, True),
	# 	CombatUnit(1, UnitTypeId.LIBERATOR, 180, True),
	# 	CombatUnit(1, UnitTypeId.LIBERATOR, 180, True),
	# 	CombatUnit(2, UnitTypeId.MUTALISK, 120, True),
	# 	CombatUnit(2, UnitTypeId.MUTALISK, 120, True),
	# 	CombatUnit(2, UnitTypeId.MUTALISK, 120, True),
	# 	CombatUnit(2, UnitTypeId.MUTALISK, 120, True),
	# 	CombatUnit(2, UnitTypeId.MUTALISK, 120, True),
	# 	CombatUnit(2, UnitTypeId.MUTALISK, 120, True),
	# 	CombatUnit(2, UnitTypeId.MUTALISK, 120, True),
	# 	CombatUnit(2, UnitTypeId.MUTALISK, 120, True),
	# 	CombatUnit(2, UnitTypeId.MUTALISK, 120, True),
	# ])) == 1)

	# Colossus can be attacked by air weapons
    assert(combat_winner(predictor, CombatState([
		CombatUnit(1, UnitTypeId.LIBERATOR, 180, True),
		CombatUnit(2, UnitTypeId.COLOSSUS, 200, False),
	])) == 1)

	# Do not assume all enemies will just target the most beefy unit and leave the banshee alone
	# while it takes out the hydras
    assert(combat_winner(predictor, CombatState([
		make_unit(1, UnitTypeId.ROACH,tech_tree),
		make_unit(1, UnitTypeId.ROACH,tech_tree),
		make_unit(1, UnitTypeId.ROACH,tech_tree),
		make_unit(1, UnitTypeId.ROACH,tech_tree),
		make_unit(1, UnitTypeId.ROACH,tech_tree),
		make_unit(1, UnitTypeId.ROACH,tech_tree),
		make_unit(1, UnitTypeId.HYDRALISK,tech_tree),
		make_unit(1, UnitTypeId.HYDRALISK,tech_tree),
		make_unit(1, UnitTypeId.HYDRALISK,tech_tree),
		make_unit(1, UnitTypeId.HYDRALISK,tech_tree),
		make_unit(1, UnitTypeId.ZERGLING,tech_tree),
		make_unit(1, UnitTypeId.ZERGLING,tech_tree),
		make_unit(1, UnitTypeId.ZERGLING,tech_tree),
		make_unit(1, UnitTypeId.ZERGLING,tech_tree),
		make_unit(1, UnitTypeId.ZERGLING,tech_tree),
		make_unit(1, UnitTypeId.ZERGLING,tech_tree),
		make_unit(1, UnitTypeId.ZERGLING,tech_tree),
		make_unit(2, UnitTypeId.BANSHEE,tech_tree),
		make_unit(2, UnitTypeId.THOR,tech_tree),
	])) == 1)

    green = "\x1b[38202550m"
    reset = "\033[0m"
    print(green, "Ok", reset)
    return 0

if __name__ == "__main__":
    main()