import math
import random
import time as Time

from tech_tree import TechTree
from data_caching import DataCache
from sc2.ids.unit_typeid import UnitTypeId
from sc2.unit import Unit
from sc2.data import Attribute
import sc2_helper as sh
import time as Time

class SurroundInfo:
    def __init__(self, max_attackers_per_defender, max_melee_attackers):
        self.max_attackers_per_defender = max_attackers_per_defender
        self.max_melee_attackers = max_melee_attackers

class CombatResult:
    def __init__(self):
        self.time = 0
        self.average_health_time = []

class CombatSettings:
    def __init__(self):
        self.bad_micro = False
        self.debug = False
        self.enable_splash = True
        self.enable_timing_adjustment = False
        self.enable_surround_limits = True
        self.enable_melee_blocking = True
        self.workers_do_no_damage = False
        self.assume_reasonable_positioning = True
        self.max_time = math.inf
        self.start_time = 0

class CombatPredictor:
    def __init__(self, units1=None, units2=None, bot_object=None):
        self.units1 = None
        self.units2 = None
        if units1 and units2:
            self.units1 = units1
            self.units2 = units2
        if bot_object:
            self.data_cache = DataCache(bot_object=bot_object)
        else:
            self.data_cache =DataCache()
        self.tech_tree = TechTree()
        
    def init(self):
        pass
    
    def predict_engage(self, settings=None, recording=None, defender_player=1):
        start_time = Time.time()
        if not settings:
            settings = CombatSettings()
        # dc = DataCache()
        # tech_tree = TechTree()
        debug = settings.debug
        zealot_radius = self.data_cache.get_unit_data(UnitTypeId.ZEALOT).unit_radius

        temporary_units = []
        HEALING_PER_NORMAL_SPEED_SECOND  = 12.6 / 1.4


        random.shuffle(self.units1)
        random.shuffle(self.units2)

        average_health_by_time = [0,0]
        average_health_by_time_weight  = [0,0]

        max_range_defender = 0
        fastest_attacker_speed = 0
        combined_units = self.units1 + self.units2
        

        if defender_player == 1 or defender_player ==2: 
            for u in self.units1 if defender_player ==1 else self.units2:
                # unit_data = self.data_cache.get_unit_data(u.type)
                max_range_defender = max(max_range_defender, u.data.attack_range)
            
            for u in self.units2 if defender_player ==1 else self.units1:
                # unit_data = self.data_cache.get_unit_data(u.type)
                fastest_attacker_speed = max(fastest_attacker_speed, u.data.movement_speed)
        
        else:
            for u in combined_units:
                # unit_data = self.data_cache.get_unit_data(u.type)
                max_range_defender = max(max_range_defender, u.data.attack_range)
            
            for u in combined_units:
                # unit_data = self.data_cache.get_unit_data(u.type)
                fastest_attacker_speed = max(fastest_attacker_speed, u.data.movement_speed)
        
        time = settings.start_time
        changed = True
        MAX_ITERATIONS = 100
        recording_start_tick = 0
        
        if settings.start_time == 0:
            for i, u in enumerate(combined_units):
                combined_units[i].buff_timer = 0
        
        for it in range(MAX_ITERATIONS):       
            if changed:
                has_air1 = 0
                has_air2 = 0
                has_ground1 = 0
                has_ground2 = 0
                ground_area1 = 0.0
                ground_area2 = 0.0

                for u in self.units1 + self.units2:
                    # unit_data = self.data_cache.get_unit_data(u.type)

                    if u.health > 0:      
                        r = u.data.unit_radius       
                        if u.owner ==1:           
                            has_air1 += u.data.can_be_attacked_by_air_weapons
                            has_ground1 += not u.is_flying
                            ground_area1 += r * r
                        else:
                            has_air2 += u.data.can_be_attacked_by_air_weapons
                            has_ground2 += not u.is_flying
                            ground_area2 += r * r

                        average_health_by_time[u.owner-1] += time * (u.health + u.shield)
                        average_health_by_time_weight[u.owner-1] += u.health + u.shield
                
                x,y = max_surround(ground_area2*math.pi, has_ground2, zealot_radius)
                surround_info1 = SurroundInfo(x,y)
                x,y = max_surround(ground_area1*math.pi, has_ground1, zealot_radius)
                surround_info2 = SurroundInfo(x,y)


                dt = min(5, 1 + (it/10))
                if debug:
                    print("Iteration ", it, " Time: ", time)
                changed = False

                guardian_shield_units = 4.5*4.5*math.pi * 0.4

                guardian_shield_unit_fraction = [0,0]
                guardian_shield_covers_all_units = [False, False]

                for group in range(2):
                    guardian_shielded_area = 0
                    # g = self.units1 if group == 0 else self.units2

                    # for i, u in enumerate(g):
                    #     if u.type == UnitTypeId.SENTRY and u.buff_timer > 0:
                    #         g[i].buff_timer -= dt
                    #         guardian_shielded_area += guardian_shield_units
                    # total_area = 0

                    # for i in g:
                    #     # unit_data = self.data_cache.get_unit_data(i.type)
                    #     r = i.data.unit_radius
                    #     total_area += r*r*math.pi
                    
                    guardian_shield_covers_all_units[group] = guardian_shielded_area > total_area
                    guardian_shield_unit_fraction[group] = min(0.8, guardian_shielded_area / (0.001+ total_area))

                for group in range(2):
                    g1 = self.units1 if group == 0 else self.units2 
                    g2 = self.units2 if group == 0 else self.units1
                    list_len1 = len(g1)
                    list_len2 =  len(g2)
                    surround = surround_info1 if group ==0 else surround_info2
                    max_extra_melee_distance = math.sqrt(ground_area1/math.pi) * math.pi + math.sqrt(ground_area2/math.pi)*math.pi

                    num_melee_units_used = 0
                    did_activate_guardian_shield = False

                    opponent_fraction_melee_units = 0

                    for u in g2:
                        # unit_data = self.data_cache.get_unit_data(u.type)
                        if u.data.is_melee and u.health > 0:
                            opponent_fraction_melee_units +=1
                    
                    if len(g2) > 0:
                        opponent_fraction_melee_units /= list_len2
                    
                    has_been_healed = [False for _ in range(list_len1)]
                    melee_unit_attack_count = [0 for _ in range(list_len2)]

                    if debug:
                        print("Max melee attackers: " , surround.max_melee_attackers , " " , surround.max_attackers_per_defender , " num units: " , len(g1))

                    for x, unit in enumerate(g1):
                        # list_len = len(g1)
                        # unit = g1[x]
                        
                        if unit.health ==0:
                            continue
                        
                        # unit_type_data = self.data_cache.get_unit_data(unit.type)
                        air_dps = unit.data.air_weapons.get_dps
                        
                        ground_dps = unit.data.ground_weapons.get_dps

                        if debug:
                            print("Processing ", unit.type," " , unit.health ,"+" ,unit.shield ," ", "e=",unit.energy)
                        
                        if unit.type == UnitTypeId.MEDIVAC:
                            if unit.energy > 0:
                                offset = int(random.randint(0,list_len1))
                                for j in range(list_len1):
                                    index = int((j + offset) % list_len1)
                                    other = g1[index]
                                    # other_data = self.data_cache.get_unit_data(other.type)
                                    if index != x and other.health > 0 and other.health < other.health_max and Attribute.Biological in other.data._attributes: ## TODO: Check if this actually works
                                        other.modify_health(HEALING_PER_NORMAL_SPEED_SECOND*dt)
                                        has_been_healed[index] == True
                                        changed = True
                                        break
                            continue
                        
                        if unit.type == UnitTypeId.SHIELDBATTERY:
                            if unit.energy > 0:
                                offset = int(random.randint(0,list_len1))
                                SHIELDS_PER_NORMAL_SPEED_SECOND = 50.4 / 1.4
                                ENERGY_USE_PER_SHIELD = 1.0 / 3.0
                                for j in range(list_len1):
                                    index = int((j + offset) % list_len1)
                                    other = g1[index]
                                    if index != x and not has_been_healed[index] and other.health > 0 and other.shield < other.shield_max:
                                        delta = min(min(other.shield_max - other.shield, SHIELDS_PER_NORMAL_SPEED_SECOND*dt), unit.energy/ENERGY_USE_PER_SHIELD)
                                        assert(delta >=0)
                                        other.shield += delta
                                        assert(other.shield >= 0 and other.shield <= other.shield_max + 0.001)
                                        unit.energy -= delta * ENERGY_USE_PER_SHIELD
                                        has_been_healed[index] = True
                                        changed = True
                                        break
                            continue
                        
                        if unit.type == UnitTypeId.INFESTOR:
                            if unit.energy > 25:
                                unit.energy -= 25
                                u = make_unit(unit.owner, UnitTypeId.INFESTEDTERRAN, self.tech_tree)
                                u.energy - 21 * 1.4
                                temporary_units.append(u)
                                g1.append(u[-1])
                                changed = True
                            continue
                        
                        if unit.type == UnitTypeId.INFESTEDTERRAN:
                            unit.energy -= dt
                            if unit.energy <= 0:
                                unit.modify_health(-100000)
                                changed = True
                                continue
                        
                        if unit.type == UnitTypeId.SENTRY and unit.energy >= 75 and not did_activate_guardian_shield:
                            if not guardian_shield_covers_all_units[group]:
                                unit.energy -= 75;
                                unit.buff_timer = 11.0;
                               
                                did_activate_guardian_shield = True;
                        
                        if air_dps ==0 and ground_dps ==0:
                            continue

                        if settings.workers_do_no_damage and unit.data.is_basic_harvester: ## TODO: Figure out
                            continue

                        is_unit_melee = unit.data.is_melee

                        if is_unit_melee and num_melee_units_used >= surround.max_melee_attackers and settings.enable_surround_limits:
                            continue

                        if settings.enable_timing_adjustment:
                            if group +1 != defender_player:
                                distance_to_enemy = max_range_defender
                                if is_unit_melee:
                                    distance_to_enemy += max_extra_melee_distance * (x / float(list_len1))
                                
                                time_to_reach_enemy = time_to_be_able_to_attack(unit.data, distance_to_enemy)

                                if time < time_to_reach_enemy:
                                    changed=True
                                    continue
                            
                            else:
                                time_to_reach_enemy = ((max_range_defender - unit.data.attack_range)/fastest_attacker_speed) if fastest_attacker_speed > 0 else 100000
                                if time < time_to_reach_enemy:
                                    changed=True
                                    continue
                        
                        best_target = None
                        best_target_index = -1
                        best_score = 0
                        best_weapon = None

                        for j, other in enumerate(g2):
                            # other = g2[j]
                            if other.health ==0:
                                continue
                            # other_data = self.data_cache.get_unit_data(other.type)

                            if (other.data.can_be_attacked_by_air_weapons and air_dps > 0) or (not other.data.is_flying and ground_dps >0):
                                air_dps2 = other.data.air_weapons.get_dps
                                ground_dps2 = other.data.ground_weapons.get_dps

                                dps = max(ground_dps2, air_dps2)
                                hg = has_ground1 if group ==0 else has_ground2
                                ha = has_air1 if group ==0 else has_air2
                                score = dps * self.target_score(other, hg, ha)* 0.001

                                if group ==1 and settings.bad_micro:
                                    score = -score
                                
                                if is_unit_melee:
                                    if settings.enable_surround_limits and melee_unit_attack_count[j] >= surround.max_attackers_per_defender:
                                        continue

                                    if not settings.bad_micro and settings.assume_reasonable_positioning:
                                        score = -score
                                    
                                    if settings.enable_melee_blocking and other.data.is_melee:
                                        score += 1000
                                    
                                    elif settings.enable_melee_blocking and unit.data.movement_speed < 1.05 * other.data.movement_speed:
                                        score -= 500
                                else:
                                    if not unit.is_flying:
                                        range_diff = other.data.attack_range - unit.data.attack_range
                                        if opponent_fraction_melee_units > 0.5 and range_diff > 0.5:
                                            score -= 1000
                                        elif opponent_fraction_melee_units > 0.3 and range_diff > 1.0:
                                            score -= 1000
                                
                                if best_target is None or score > best_score or (score == best_score and unit.health + unit.shield < best_target.health + best_target.shield):
                                    best_score = score
                                    best_target = g2[j]
                                    best_target_index = j
                                    best_weapon = other.data.ground_weapons if ground_dps2 > air_dps2 else other.data.air_weapons

                        if best_target is not None:
                            if debug:
                                print('Best target =', best_target.type, best_target.health)
                            if is_unit_melee:
                                num_melee_units_used +=1
                            
                                melee_unit_attack_count[best_target_index] += 1

                            remaining_splash = max(1.0, best_weapon.splash)

                            other = best_target
                            # other_data = self.data_cache.get_unit_data(best_target.type)
                            changed = True

                            shielded = not is_unit_melee and random.random() < guardian_shield_unit_fraction[1-group]
                            dps = best_weapon.get_dps
                            damage_multiplier = 1
                            if unit.type == UnitTypeId.CARRIER:
                                damage_multiplier = (unit.health + unit.shield)/(unit.health_max + unit.shield_max)

                                damage_multiplier *= min(1.0, time/4.0)
                            if debug:
                                print('Target health before damage', other.health)
                            g2[best_target_index] = other.modify_health(-dps* damage_multiplier * dt)
                            other = g2[best_target_index]
                            if debug:
                                print('Target health after damage', other.health)
                            other = g2[best_target_index]
                            if other.health ==0:
                                g2[best_target_index] = g2[-1]
                                melee_unit_attack_count[best_target_index] = melee_unit_attack_count[-1]
                                g2.pop()
                                melee_unit_attack_count.pop()
                                best_target = None
                            
                            remaining_splash -=1

                            if settings.enable_splash and remaining_splash > 0.001 and (not is_unit_melee or best_target.data.is_melee) and len(g2) >0:
                                splash_index = (j + offset) % len(g2)
                                splash_other = g2[splash_index]

                                if splash_other != best_target and splash_other.health > 0 and (not is_unit_melee or other.data.is_melee):
                                    shielded_other = not is_unit_melee and random.random() < guardian_shield_unit_fraction[1-group]
                                    dps = best_weapon.get_dps(splash_other.type, ((-2) if shielded_other else 0)*min(1,0, remaining_splash))

                                    if dps > 0:
                                        splash_other.modify_health(-dps * damage_multiplier * dt)
                                        remaining_splash -= 1.0

                                        if splash_other.health ==0:
                                            g2[splash_index] = g2[-1]
                                            melee_unit_attack_count[splash_index] = melee_unit_attack_count[-1]
                                            g2.pop()
                                            melee_unit_attack_count.pop()
                                            j -=1
                                            if len(g2) ==0:
                                                break

                    if debug:
                        print("Melee attackers used: " , num_melee_units_used , "did change during last iteration: " ,changed )

                time += dt;
                if (time >= settings.max_time):
                    break
        if debug:
            print("Player1 units left=", len(self.units1))
            print("Player2 units left=", len(self.units2))
        result = CombatResult()
        result.time = time

        average_health_by_time[0] /= max(0.01, average_health_by_time_weight[0])
        average_health_by_time[1] /= max(0.01, average_health_by_time_weight[1])

        result.average_health_time = average_health_by_time;
        end = Time.time()
        print('Total time: ',end-start_time)
        return result

    def owner_with_best_outcome(self):
        player1 = 0
        player2 = 0
        for u in self.units1:
            player1 += u.health + u.shield
        for u in self.units2:
            player2 += u.health + u.shield

        winner = 1 if player1 > player2 else 2

        return winner
    
    def target_score(self, unit, has_ground:bool, has_air:bool):
        VESPENE_MULTIPLIER = 1.5
        
        unit_data = unit.data
        cost = unit_data._mineral_cost + VESPENE_MULTIPLIER * unit_data._vespene_cost
        # unit._type_data.cost.minerals + VESPENE_MULTIPLIER * unit._type_data.cost.vespene

        score = 0

        air_dps = unit_data.air_weapons.get_dps
        ground_dps = unit_data.ground_weapons.get_dps

        score += 0.01 * cost
        score += 1000 * max(ground_dps, air_dps)

        if not has_ground and air_dps ==0:
            score *= 0.01
        elif not has_air and ground_dps == 0:
            score *= 0.01
        elif air_dps ==0 and ground_dps ==0:
            score *= 0.01
        
        return score

    # def mineral_score(self, combat_result: CombatResult,
    #                   player: int,
    #                   time_to_produce_units: list,
    #                   upgrades):
    #     assert(len(time_to_produce_units) == 3)

    #     # The combat result may contain more units due to temporary units spawning (e.g. infested terran, etc.)
    #     # however never fewer.
    #     # The first N units correspond to all the N units in the initial state.
    #     assert(len(combat_result.state.units) >= len(initial_state.units))

    #     total_score = 0
    #     our_score = 0
    #     enemy_score = 0
    #     loss_score = 0
    #     our_supply = 0
        
    #     for i in range(len(initial_state.units)):
    #         unit1 = initial_state.units[i]
    #         unit2 = combat_result.state.units[i]

    #         assert(unit1.type == unit2.type)

    #         health_diff = (unit2.health - unit1.health) + (unit2.shield - unit1.shield)
    #         damage_taken_fraction = -health_diff / (unit1.health_max + unit1.shield_max)

    #         unit_type_data = self.data_cache.get_unit_data(unit1)

    #         cost = unit_type_data.minerals + 1.2 * unit_type_data.vespene

    #         if unit1.owner == player:
    #             our_score += cost *-(1 + damage_taken_fraction)
            
    #         else:
    #             if self.default_combat_environment.calculate_dps(unit2, False) > 0:
    #                 loss_score += cost * (-10 *(1-damage_taken_fraction))
    #                 our_supply += unit_type_data.food_required
                
    #             else:
    #                 loss_score += cost * (-1 *(1- damage_taken_fraction))
                
    #             enemy_score += cost *(1 + damage_taken_fraction)
            
    #     for u in upgrades:
    #         data = u._upgrade_data
    #         our_score -= data.cost.minerals + 1.2 * data.cost.vespene
        
    #     for i in range(initial_state.units, combat_result.state.units):
    #         unit2 = combat_result.state.units[i]

    #         health_diff = unit2.health + unit2.shield
    #         damage_taken_fraction = -health_diff / (unit2.health_max + unit2.shield_max)

    #         cost = 5

    #         if unit2.owner != player:
    #             loss_score += cost * (-100 * (1 - damage_taken_fraction))
    #             enemy_score += cost *(1+ damage_taken_fraction)
        
    #     our_score -= time_to_produce_units[1] + 1.2 * time_to_produce_units[2]
        
    #     # Add pylon/overlord/supply depot cost
    #     our_supply -= (our_supply/8.0)*100

    #     time_mult = min(1,2*30.(30+time_to_produce_units[0]))
    #     total_score = our_score + enemy_score*time_mult + loss_score

    #     if combat_result.time > 20:
    #         has_any_ground_units = False
    #         for i in initial_state.units:
    #             if i.owner == player and i.health > 0 and not i.is_flying:
    #                 has_any_ground_units = True
            
    #         if not has_any_ground_units:
    #             total_score -= 1000 * (combat_result.time - 20)/20.0

    #     return total_score

def time_to_be_able_to_attack(unit_type_data, distance_to_enemy: float):
    return (max(0, distance_to_enemy - unit_type_data.attack_range)/unit_type_data.movement_speed) if unit_type_data.movement_speed > 0 else 100000


def max_surround(enemy_ground_unit_area: float, enemy_ground_units: int, unit_radius):
    if enemy_ground_units >1:
        enemy_ground_unit_area /= 0.6
    radius = math.sqrt(enemy_ground_unit_area/ math.pi)

    representative_melee_unit_radius = unit_radius
    
    circumference_defenders = radius * (2 * math.pi)
    circumference_attackers = (radius +representative_melee_unit_radius) * (2* math.pi)

    approximate_defenders_in_melee_range = min(enemy_ground_units, circumference_defenders/ (2* representative_melee_unit_radius))
    approximate_attackers_in_melee_range = circumference_attackers / (2* representative_melee_unit_radius)

    max_attackers_per_defender = math.ceil(approximate_attackers_in_melee_range/approximate_defenders_in_melee_range) if approximate_defenders_in_melee_range > 0 else 1
    max_melee_attackers = int(math.ceil(approximate_attackers_in_melee_range))
    
    return max_attackers_per_defender, max_melee_attackers

class CombatUnit(Unit):
    def __init__(self, unit=None,owner=None,type=None, health=None, flying=None, data_cache=None):
        # self.cache = {}
        if not data_cache:
            data_cache = DataCache()
        self.data_cache = data_cache
        
        # self._data = None
        # self._data_dict = None
        if unit:
            self._proto = unit._proto
            self.type_data = unit.type_data
            self._bot_object = unit._bot_object
            super().__init__(self._proto, self._bot_object)
            self._owner = self.owner_id
            self._type = self.type_id
            self._buff_timer = 0
            self._health_max = self._proto.health_max
            self._health = self._proto.health
            self._shield_max = self._proto.shield_max 
            self._shield = self._proto.shield
            self._energy = self._proto.energy
            self._is_flying = self._proto.is_flying
            
        elif owner is not None and type and health and flying is not None:
            
            self._owner = owner
            self._health_max = health
            self._health = health
            self._shield_max = 0
            self._shield = 0
            self._energy = 50
            self._is_flying = flying
            self._type = type
            self._buff_timer = 0
            self.type_data = self.data_cache.get_raw_unit_data(self._type)
            print(self.type_data)

        else:
            self._owner = 0
            self._health_max = 0
            self._health = 0
            self._shield_max = 0
            self._shield = 0
            self._energy = 0
            self._is_flying = 0
            self._type = 0
            self._buff_timer = 0
        
        # self._data = self.data_cache.get_unit_data(self._type)
    
    # @property
    # def data(self):
    #     if not self._data:
    #         self._data = self.data_cache.get_unit_data(self._type) 
    #     return self._data
    
    # @property
    # def data_dict(self):
    #     if not self._data_dict:
    #         self._data_dict = self.data_cache.get_data_as_dict(self._type)
    #     return self._data_dict

    @property
    def owner(self):
        return self._owner
    
    @owner.setter
    def owner(self, value):
        self._owner = value
    
    @property
    def health_max(self):
        return self._health_max
    
    @health_max.setter
    def health_max(self, value):
        self._health_max = value
    
    @property
    def health(self):
        return self._health
    
    @health.setter
    def health(self, value):
        self._health = value
    
    @property
    def shield_max(self):
        return self._shield_max
    
    @shield_max.setter
    def shield_max(self, value):
        self._shield_max = value
    
    @property
    def shield(self):
        return self._shield
    
    @shield.setter
    def shield(self, value):
        self._shield = value
    
    @property
    def energy(self):
        return self._energy
    
    @energy.setter
    def energy(self, value):
        self._energy = value
    
    @property
    def is_flying(self):
        return self._is_flying
    
    @is_flying.setter
    def is_flying(self, value):
        self._is_flying = value
    
    @property
    def type(self):
        return self._type
    
    @type.setter
    def type(self, value):
        self._type = value
    
    @property
    def buff_timer(self):
        return self._buff_timer
    
    @buff_timer.setter
    def buff_timer(self, value):
        self._buff_timer = value
    
    def to_rust(self):
        return sh.CombatUnit(_owner=self.owner, 
                    _unit_type=self.type.value, 
                    _health=self.health, 
                    _health_max=self.health_max, 
                    _shield=self.shield, 
                    _shield_max=self.shield_max,
                    _energy=self.energy, 
                    _flying=self.is_flying, 
                    _buff_timer=self.buff_timer,
                    _type_data = self.type_data)


def make_unit(owner:int, type:UnitTypeId, tech_tree=None):
    if not tech_tree:
        tech_tree = TechTree()
    unit = CombatUnit()
    
    max_health = tech_tree.max_health(type)
    unit.health_max = max_health
    unit.health = max_health
   
    max_shield = tech_tree.max_shield(type)
    unit.shield_max = max_shield
    unit.shield = max_shield

    # Immortals absorb up to 100 damage over the first two seconds that it is attacked (barrier ability).
    # Approximate this by adding 50 extra shields.
    # Note: shield > max_shield prevents the shield battery from healing it during this time.
    if type == UnitTypeId.IMMORTAL:
        unit.shield += 50
    
    unit.energy = 100
    unit.owner = owner
    unit.is_flying = tech_tree.is_flying(type)
    unit.type = type
    return unit

def test():
    import time
    tech_tree = TechTree()
    
    units1=[CombatUnit(owner=1, type=UnitTypeId.MARAUDER, health=50, flying=False) for _ in range(20)]+[make_unit(1, UnitTypeId.MEDIVAC,tech_tree) for _ in range(5)]
    units2= [CombatUnit(owner=2, type=UnitTypeId.ZERGLING, health=35, flying=False) for _ in range(50)]
    print(units1[0].data_dict)
    # cp = CombatPredictor(units1, units2)
    # start = time.time()
    # r = cp.predict_engage()
    # end = time.time()
    # print(end-start)
    # winner = cp.owner_with_best_outcome()
    # print(winner)
    # assert(winner == 1)
   
    # units1=[
    #         make_unit(1, UnitTypeId.PYLON,tech_tree),
    #         make_unit(1, UnitTypeId.PHOTONCANNON,tech_tree),
    #         make_unit(1, UnitTypeId.SHIELDBATTERY,tech_tree),
    #         make_unit(1, UnitTypeId.SHIELDBATTERY,tech_tree),
    #         make_unit(1, UnitTypeId.SHIELDBATTERY,tech_tree),
    #         ]
    # units2= [
    #         make_unit(2, UnitTypeId.MEDIVAC,tech_tree),
    #         make_unit(2, UnitTypeId.MARINE,tech_tree)
    #         ]
    # cp = CombatPredictor(units1, units2)
    
    # start = time.time()
    # r = cp.predict_engage()
    # end = time.time()
    # print(end-start)
    # winner = cp.owner_with_best_outcome()
    # print(winner)
    # assert(winner == 1)
    
    # units1=[
    #         make_unit(1, UnitTypeId.MARINE,tech_tree),
	# 	    make_unit(1, UnitTypeId.MEDIVAC,tech_tree)
    #         ]
    # units2= [
    #         make_unit(2, UnitTypeId.MARINE, tech_tree),
	# 	    make_unit(2, UnitTypeId.MARINE, tech_tree),
    #         ]
    # cp = CombatPredictor(units1, units2)
    # r = cp.predict_engage()
    # winner = cp.owner_with_best_outcome()
    # assert(winner == 1) 
    
    
    

    # units1=[
    #         CombatUnit(owner=2, type=UnitTypeId.MARINE, health=50, flying=False),
    #         ]
    # units2= [
	# 	    CombatUnit(owner=1, type=UnitTypeId.ZERGLING, health=35, flying=False),
    #         ]
    # cp = CombatPredictor(units1, units2)
    # r = cp.predict_engage()
    # winner = cp.owner_with_best_outcome()
    # assert(winner == 2)

    
    # units1=[
    #         CombatUnit(owner=1, type=UnitTypeId.MARINE, health=50, flying=False),
    #         CombatUnit(owner=1, type=UnitTypeId.MARINE, health=50, flying=False),
    #         CombatUnit(owner=1, type=UnitTypeId.MARINE, health=50, flying=False),
    #         CombatUnit(owner=1, type=UnitTypeId.MARINE, health=50, flying=False),
    #         ]
    # units2= [
	# 	    CombatUnit(owner=2, type=UnitTypeId.ZERGLING, health=35, flying=False),
    #         CombatUnit(owner=2, type=UnitTypeId.ZERGLING, health=35, flying=False),
    #         CombatUnit(owner=2, type=UnitTypeId.ZERGLING, health=35, flying=False),
    #         CombatUnit(owner=2, type=UnitTypeId.ZERGLING, health=35, flying=False),
    #         ]
    # cp = CombatPredictor(units1, units2)
    # r = cp.predict_engage()
    # winner = cp.owner_with_best_outcome()
    # assert(winner == 1)


    # assert(combat_winner(predictor, CombatState([
	# 	CombatUnit(1, UnitTypeId.MARINE, 50, False),
	# 	CombatUnit(2, UnitTypeId.ZERGLING, 35, False),
	# 	CombatUnit(2, UnitTypeId.ZERGLING, 35, False),
	# 	CombatUnit(2, UnitTypeId.ZERGLING, 35, False),
	# ])) == 2)

    # assert(combat_winner(predictor, CombatState([
	# 	make_unit(1, UnitTypeId.SPORECRAWLER,tech_tree),
	# 	make_unit(1, UnitTypeId.SPORECRAWLER,tech_tree),
    #     make_unit(1, UnitTypeId.SPORECRAWLER,tech_tree),
    #     make_unit(2, UnitTypeId.REAPER,tech_tree),
    #     make_unit(2, UnitTypeId.REAPER,tech_tree),
    #     make_unit(2, UnitTypeId.REAPER,tech_tree),
	# ])) == 2)

    # assert(combat_winner(predictor, CombatState([
	# 	CombatUnit(1, UnitTypeId.LIBERATOR, 180, True),
	# 	CombatUnit(2, UnitTypeId.BROODLORD, 225, True),
	# 	CombatUnit(2, UnitTypeId.BROODLORD, 225, True),
	# 	CombatUnit(2, UnitTypeId.BROODLORD, 225, True),
	# 	CombatUnit(2, UnitTypeId.BROODLORD, 225, True),
	# 	CombatUnit(2, UnitTypeId.BROODLORD, 225, True),
	#     ])) == 1)

    # assert(combat_winner(predictor, CombatState([
	# 	CombatUnit(1, UnitTypeId.CYCLONE, 180, True),
	# 	CombatUnit(2, UnitTypeId.IMMORTAL, 200, True),
	# ])) == 2)

    # units1 = [make_unit(1, UnitTypeId.BATTLECRUISER, tech_tree)]
    # units2 = [make_unit(2, UnitTypeId.THORAP, tech_tree)]

    # cp = CombatPredictor(units1, units2)
    # r = cp.predict_engage()
    # winner = cp.owner_with_best_outcome()
    # # assert(winner == 1)
    # print(winner
    # )
    # assert(combat_winner(predictor, CombatState([
	# 	make_unit(1, UnitTypeId.INFESTOR,tech_tree),
	# 	make_unit(2, UnitTypeId.BANSHEE,tech_tree),
	# ])) == 1)

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
    # assert(combat_winner(predictor, CombatState([
	# 	CombatUnit(1, UnitTypeId.LIBERATOR, 180, True),
	# 	CombatUnit(2, UnitTypeId.COLOSSUS, 200, False),
	# ])) == 1)

	# Do not assume all enemies will just target the most beefy unit and leave the banshee alone
	# while it takes out the hydras
    # assert(combat_winner(predictor, CombatState([
	# 	make_unit(1, UnitTypeId.ROACH,tech_tree),
	# 	make_unit(1, UnitTypeId.ROACH,tech_tree),
	# 	make_unit(1, UnitTypeId.ROACH,tech_tree),
	# 	make_unit(1, UnitTypeId.ROACH,tech_tree),
	# 	make_unit(1, UnitTypeId.ROACH,tech_tree),
	# 	make_unit(1, UnitTypeId.ROACH,tech_tree),
	# 	make_unit(1, UnitTypeId.HYDRALISK,tech_tree),
	# 	make_unit(1, UnitTypeId.HYDRALISK,tech_tree),
	# 	make_unit(1, UnitTypeId.HYDRALISK,tech_tree),
	# 	make_unit(1, UnitTypeId.HYDRALISK,tech_tree),
	# 	make_unit(1, UnitTypeId.ZERGLING,tech_tree),
	# 	make_unit(1, UnitTypeId.ZERGLING,tech_tree),
	# 	make_unit(1, UnitTypeId.ZERGLING,tech_tree),
	# 	make_unit(1, UnitTypeId.ZERGLING,tech_tree),
	# 	make_unit(1, UnitTypeId.ZERGLING,tech_tree),
	# 	make_unit(1, UnitTypeId.ZERGLING,tech_tree),
	# 	make_unit(1, UnitTypeId.ZERGLING,tech_tree),
	# 	make_unit(2, UnitTypeId.BANSHEE,tech_tree),
	# 	make_unit(2, UnitTypeId.THOR,tech_tree),
	# ])) == 1)

def main():
    test()
    


if __name__ == "__main__":
    main()