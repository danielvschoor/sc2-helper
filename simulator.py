from sc2.unit import Unit
from sc2.ids.unit_typeid import UnitTypeId
from sc2.ids.upgrade_id import UpgradeId


from scipy.stats import bernoulli
import numpy as np
import math, random, time

from combat_upgrades import *
from combat_environment import *
from tech_tree import TechTree
from data_caching import DataCache

seen_combats = dict()
counter = 0
CACHE_COMBAT = True
UPGRADE_ID_OFFSET = 1000000

class CombatUnit(Unit):
    def __init__(self, unit=None):
        self.cache = {}
        if unit:
            self._proto = unit._proto
            self._bot_object = unit._bot_object
            super().__init__(self._proto, self._bot_object)
            self._owner = self.owner_id
            self._type = self.type_id
            self._buff_timer = 0
            self._health_max = self.health_max
            self._health = self.health
            self._shield_max = self.shield_max 
            self._shield = self.shield
            self._energy = self.energy
            self._is_flying = self.is_flying
            
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
    

    def modify_health(self, delta):
        if delta < 0:
            delta = -delta
            self.shield = -delta
            if self.shield < 0:
                delta = -self.shield
                self.shield = 0
                self.health = max(0, self.health-delta)
        else:
            self.health += delta
            self.health = min(self.health, self.health_max)

class CombatState:
    def __init__(self, units=None):
        self.units = units
        self.combat_environment = None
    
    def owner_with_best_outcome(self):
        max_index = 0
        for u in self.units:
            max_index = max(max_index, u.owner)
        total_health =[0.0 for _ in range(max_index+1)]

        for u in self.units:
            total_health[u.owner] += u.health + u.shield
        
        winner = 0

        for i in range(max_index):
            if total_health[i] > total_health[winner]:
                winner = i
        return winner
    
class CombatResult:
    def __init__(self):
        self.time = 0
        self.average_health_time = []
        self.state = CombatState()

class CombatRecordingFrame:
    def __init__(self):
        self.tick = None
        self.healths = []
    
    def add(self, type: UnitTypeId, owner: int, health: float, shield: float):
        for y,x in enumerate(self.healths):
            if x[0] == type and x[1] == owner:
                self.healths[y] = [type, owner, x[2]+health+shield]
                return
        self.healths.append([type, owner, health+shield])


class CombatRecording:
    def __init__(self):
        self.frames = None
    
    def write(self, file_name):
        pass


class CombatSettings:
    def __init__(self):
        self.bad_micro = False
        self.debug = True
        self.enable_splash = True
        self.enable_timing_adjustment = True
        self.enable_surround_limits = True
        self.enable_melee_blocking = True
        self.workers_do_no_damage = False
        self.assume_reasonable_positioning = True
        self.max_time = math.inf
        self.start_time = 0


class CombatPredictor:
    def __init__(self):
        self._combat_environment = dict()
        self.default_combat_environment = CombatEnvironment()
        
    def init(self):
        pass
    
    def predict_engage(self, input_state: CombatState, settings=None, recording=None, defender_player=1):
        if not settings:
            settings = CombatSettings()
        tech_tree = TechTree()
        dc = DataCache()
        env =  input_state.combat_environment if input_state.combat_environment else self.default_combat_environment
        debug = settings.debug
        result = CombatResult()
        result.state = input_state
        state = result.state

        temporary_units = []

        units1 = filter_by_owner(state.units,1)
        units2 = filter_by_owner(state.units,2)

        random.shuffle(units1)
        random.shuffle(units2)

        average_health_by_time = [0,0]
        average_health_by_time_weight  = [0,0]

        max_range_defender = 0
        fastest_attacker_speed = 0

        if defender_player == 1 or defender_player ==2:
            
            
            for u in units1 if defender_player ==1 else units2:
                max_range_defender = max(max_range_defender, env.attack_range(unit=u))
            
            for u in units2 if defender_player ==1 else units1:
                fastest_attacker_speed = max(fastest_attacker_speed, tech_tree.movement_speed(u.type))
        
        else:
            for u in state.units:
                 max_range_defender = max(max_range_defender, env.attack_range(unit=u))
            
            for u in state.units:
                fastest_attacker_speed = max(fastest_attacker_speed, tech_tree.movement_speed(u.type))
        
        time = settings.start_time
        changed = True
        MAX_ITERATIONS = 100
        recording_start_tick = 0

        if recording and recording.frames:
            recording_start_tick = ticks_to_seconds(recording.frames[-1].tick)+1 - time
        
        if settings.start_time == 0:
            for u in state.units:
                u.buff_timer = 0

        for it in range(MAX_ITERATIONS):
            if changed:
                has_air1 = 0
                has_air2 = 0
                has_ground1 = 0
                has_ground2 = 0
                ground_area1 = 0.0
                ground_area2 = 0.0
                for u in units1:
                    if u.health > 0:
                        has_air1 += u.is_flying or u.type == UnitTypeId.COLOSSUS
                        has_ground1 += not u.is_flying
                        r = tech_tree.unit_radius(u.type) # TODO: Figure out radius
                        ground_area1 += r * r
                        average_health_by_time[0] += time * (u.health + u.shield)
                        average_health_by_time_weight[0] += u.health + u.shield
                
                for u in units2:
                    if u.health > 0:
                        has_air2 += u.is_flying or u.type == UnitTypeId.COLOSSUS
                        has_ground2 += not u.is_flying
                        r = tech_tree.unit_radius(u.type) # TODO: Figure out radius
                        ground_area2 += r * r
                        average_health_by_time[1] += time * (u.health + u.shield)
                        average_health_by_time_weight[1] += u.health + u.shield
                
                if recording:
                    frame = CombatRecordingFrame()
                    frame.tick = int(round(recording_start_tick + time)*22.4)
                    for u in units1:
                        frame.add(u.type, u.owner, u.health, u.shield)
                    for u in units2:
                        frame.add(u.type, u.owner, u.health, u.shield)
                    recording.frames.append(frame)
                
                surround_info1 = max_surround(ground_area2*math.pi, has_ground2, tech_tree)
                surround_info2 = max_surround(ground_area1*math.pi, has_ground1, tech_tree)

                dt = min(5, 1 + (it/10))
                if debug:
                    print("Iteration ", it, " Time: ", time)
                changed = False

                guardian_shield_units = 4.5*4.5*math.pi * 0.4

                guardian_shield_unit_fraction = [0,0]
                guardian_shield_covers_all_units = [False, False]

                for group in range(1):
                    guardian_shielded_area = 0
                    g = units1 if group == 0 else units2

                    for u in g:
                        if u.type == UnitTypeId.SENTRY and u.buff_timer > 0:
                            u.buff_timer -= dt
                            guardian_shielded_area += guardian_shield_units
                    total_area = 0

                    for i in g:
                        r = tech_tree.unit_radius(i.type)
                        total_area += r*r*math.pi
                    
                    guardian_shield_covers_all_units[group] = guardian_shielded_area > total_area
                    guardian_shield_unit_fraction[group] = min(0.8, guardian_shielded_area / (0.001+ total_area))

                for group in range(1):
                    g1 = units1 if group == 0 else units2 
                    g2 = units2 if group == 0 else units1
                    surround = surround_info1 if group ==0 else surround_info2
                    max_extra_melee_distance = math.sqrt(ground_area1/math.pi) * math.pi + math.sqrt(ground_area2/math.pi)*math.pi

                    num_melee_units_used = 0
                    did_activate_guardian_shield = False

                    opponent_fraction_melee_units = 0

                    for u in g2:
                        if tech_tree.is_melee(u.type) and u.health > 0:
                            opponent_fraction_melee_units +=1
                    
                    if len(g2) > 0:
                        opponent_fraction_melee_units /= len(g2)
                    
                    has_been_healed = [False for _ in range(len(g1))]
                    melee_unit_attack_count = [0 for _ in range(len(g2))]

                    if debug:
                        print("Max melee attackers: " , surround.max_melee_attackers , " " , surround.max_attackers_per_defender , " num units: " , len(g1))

                    for x in range(len(g1)):
                        unit = g1[x]
                        
                        if unit.health ==0:
                            continue

                        unit_type_data = dc.get_unit_data(unit.type)
                        air_dps = env.calculate_dps(unit=unit, air=True)
                        ground_dps = env.calculate_dps(unit=unit, air=False)

                        if debug:
                            print("Processing ", unit.type," " ,unit.health ,"+" ,unit.shield ," ", "e=",unit.energy)
                        
                        if unit.type == UnitTypeId.MEDIVAC:
                            if unit.energy > 0:
                                offset = int(random.randint(0,len(g1)))
                                HEALING_PER_NORMAL_SPEED_SECOND  = 12.6 / 1.4
                                for j in range(len(g1)):
                                    index = int((j + offset) % len(g1))
                                    other = g1[index]
                                    if index != x and other.health > 0 and other.health < other.health_max and 'Biological' in other.attributes: ## TODO: Check if this actually works
                                        other.modify_health(HEALING_PER_NORMAL_SPEED_SECOND*dt)
                                        has_been_healed[index] == True
                                        changed = True
                                        break
                            continue
                        
                        if unit.type == UnitTypeId.SHIELDBATTERY:
                            if unit.energy > 0:
                                offset = int(random.randint(0,len(g1)))
                                SHIELDS_PER_NORMAL_SPEED_SECOND = 50.4 / 1.4
                                ENERGY_USE_PER_SHIELD = 1.0 / 3.0
                                for j in range(len(g1)):
                                    index = int((j + offset) % len(g1))
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
                                u = make_unit(unit.owner, UnitTypeId.INFESTEDTERRAN, tech_tree)
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

                        if settings.workers_do_no_damage and tech_tree.is_basic_harvester(unit.type): ## TODO: FIgure out
                            continue

                        is_unit_melee = tech_tree.is_melee(unit.type)

                        if is_unit_melee and num_melee_units_used >= surround.max_melee_attackers and settings.enable_surround_limits:
                            continue

                        if settings.enable_timing_adjustment:
                            if group +1 != defender_player:
                                distance_to_enemy = max_range_defender
                                if is_unit_melee:
                                    distance_to_enemy += max_extra_melee_distance * (x/ float(len(g1)))
                                
                                time_to_reach_enemy = time_to_be_able_to_attack(env, unit, distance_to_enemy)

                                if time < time_to_reach_enemy:
                                    changed=True
                                    continue
                            
                            else:
                                time_to_reach_enemy = ((max_range_defender - env.attack_range(unit=unit))/fastest_attacker_speed) if fastest_attacker_speed > 0 else 100000
                                if time < time_to_reach_enemy:
                                    changed=True
                                    continue
                        
                        best_target = None
                        best_target_index = -1
                        best_score = 0
                        best_weapon = None

                        for j in range(len(g2)):
                            other = g2[j]
                            if other.health ==0:
                                continue

                            if (tech_tree.can_be_attacked_by_air_weapons(other.type) and air_dps > 0) or (not other.is_flying and ground_dps >0):
                                info = env.get_combat_info(unit)
                                other_data = dc.get_unit_data(unit)

                                air_dps2 = info.air_weapon.get_dps(other.type)
                                ground_dps2 = info.ground_weapon.get_dps(other.type)

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
                                    
                                    if settings.enable_melee_blocking and tech_tree.is_melee(other.type):
                                        score += 1000
                                    
                                    elif settings.enable_melee_blocking and unit_type_data.movement_speed < 1.05 * other_data.movement_speed:
                                        score -= 500
                                else:
                                    if not unit.is_flying:
                                        range_diff = env.attack_range(unit=other) - env.attack_range(unit=unit)
                                        if opponent_fraction_melee_units > 0.5 and range_diff > 0.5:
                                            score -= 1000
                                        elif opponent_fraction_melee_units > 0.3 and range_diff > 1.0:
                                            score -= 1000
                                
                                if best_target is None or score > best_score or (score == best_score and unit.health + unit.shield < best_target.health + best_target.shield):
                                    best_score = score
                                    best_target = g2[j]
                                    best_target_index = j
                                    best_weapon = info.ground_weapon if ground_dps2 > air_dps2 else info.air_weapon

                        if best_target is not None:
                            if is_unit_melee:
                                num_melee_units_used +=1
                            
                            melee_unit_attack_count[best_target_index] += 1

                            remaining_splash = max(1.0, best_weapon.splash)

                            other = best_target
                            changed = True

                            shielded = not is_unit_melee and random.random() < guardian_shield_unit_fraction[1-group]
                            dps = best_weapon.get_dps(other.type, -2 if shielded else 0)* min(1, remaining_splash)
                            damage_multiplier = 1
                            if unit.type == UnitTypeId.CARRIER:
                                damage_multiplier = (unit.health + unit.shield)/(unit.health_max + unit.shield_max)

                                damage_multiplier *= min(1.0, time/4.0)
                            
                            other.modify_health(-dps* damage_multiplier * dt)

                            if other.health ==0:
                                g2[best_target_index] = g2[-1]
                                melee_unit_attack_count[best_target_index] = melee_unit_attack_count[-1]
                                g2.pop()
                                melee_unit_attack_count.pop()
                                best_target = None
                            
                            remaining_splash -=1

                            if settings.enable_splash and remaining_splash > 0.001 and (not is_unit_melee or tech_tree.is_melee(other.type)) and len(g2) >0:
                                splash_index = (j + offset) % len(g2)
                                splash_other = g2[splash_index]

                                if splash_other != best_target and splash_other.health > 0 and (not is_unit_melee or tech_tree.is_melee(other.type)):
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
                        print("Meleee attackers used: " , num_melee_units_used , "did change during last iteration: " ,changed )
                
                time += dt;
                if (time >= settings.max_time):
                    break
            

                result.time = time;

                average_health_by_time[0] /= max(0.01, average_health_by_time_weight[0]);
                average_health_by_time[1] /= max(0.01, average_health_by_time_weight[1]);

                result.average_health_time = average_health_by_time;

                return result;

    def get_combat_environment(self, upgrades, target_upgrades):
        _hash = (hash(upgrades) * 5123143) ^ hash(target_upgrades)
        it = self._combat_environment.find(hash);
        
        if it != self._combat_environment[-1]:
            return it[1] 

        self._combat_environment.append([hash, CombatEnvironment(upgrades, target_upgrades)])
        return self._combat_environment[0][1]

    def combine_combat_environment(self, env:CombatEnvironment,  upgrades, upgrades_owner:int):
        assert(upgrades_owner ==1 or upgrades_owner ==2)

        new_upgrades = env.upgrades if env else [[[0,0],[0,0]]]
        new_upgrades[upgrades_owner - 1].combine(upgrades)

        return self.get_combat_environment(new_upgrades[0], new_upgrades[1])
    
    def target_score(self, unit:CombatUnit, has_ground:bool, has_air:bool):
        dc = DataCache()
        VESPENE_MULTIPLIER = 1.5
        cost = dc.get_unit_data(unit).minerals +VESPENE_MULTIPLIER * dc.get_unit_data(unit).vespene
        # unit._type_data.cost.minerals + VESPENE_MULTIPLIER * unit._type_data.cost.vespene

        score = 0

        air_dps = self.default_combat_environment.calculate_dps(1, unit.type, True)
        ground_dps = self.default_combat_environment.calculate_dps(1, unit.type, False)

        score += 0.01 * cost
        score += 1000 * max(ground_dps, air_dps)

        if not has_ground and air_dps ==0:
            score *= 0.01
        elif not has_air and ground_dps == 0:
            score *= 0.01
        elif air_dps ==0 and ground_dps ==0:
            score *= 0.01
        
        return score

    def mineral_score(self, initial_state:CombatState,
                      combat_result: CombatResult,
                      player: int,
                      time_to_produce_units: list,
                      upgrades):
        assert(len(time_to_produce_units) == 3)
        dc= DataCache()

        # The combat result may contain more units due to temporary units spawning (e.g. infested terran, etc.)
        # however never fewer.
        # The first N units correspond to all the N units in the initial state.
        assert(len(combat_result.state.units) >= len(initial_state.units))

        total_score = 0
        our_score = 0
        enemy_score = 0
        loss_score = 0
        our_supply = 0
        
        for i in range(len(initial_state.units)):
            unit1 = initial_state.units[i]
            unit2 = combat_result.state.units[i]

            assert(unit1.type == unit2.type)

            health_diff = (unit2.health - unit1.health) + (unit2.shield - unit1.shield)
            damage_taken_fraction = -health_diff / (unit1.health_max + unit1.shield_max)

            unit_type_data = dc.get_unit_data(unit1)

            cost = unit_type_data.minerals + 1.2 * unit_type_data.vespene

            if unit1.owner == player:
                our_score += cost *-(1 + damage_taken_fraction)
            
            else:
                if self.default_combat_environment.calculate_dps(unit2, False) > 0:
                    loss_score += cost * (-10 *(1-damage_taken_fraction))
                    our_supply += unit_type_data.food_required
                
                else:
                    loss_score += cost * (-1 *(1- damage_taken_fraction))
                
                enemy_score += cost *(1 + damage_taken_fraction)
            
        for u in upgrades:
            data = u._upgrade_data
            our_score -= data.cost.minerals + 1.2 * data.cost.vespene
        
        for i in range(initial_state.units, combat_result.state.units):
            unit2 = combat_result.state.units[i]

            health_diff = unit2.health + unit2.shield
            damage_taken_fraction = -health_diff / (unit2.health_max + unit2.shield_max)

            cost = 5

            if unit2.owner != player:
                loss_score += cost * (-100 * (1 - damage_taken_fraction))
                enemy_score += cost *(1+ damage_taken_fraction)
        
        our_score -= time_to_produce_units[1] + 1.2 * time_to_produce_units[2]
        
        # Add pylon/overlord/supply depot cost
        our_supply -= (our_supply/8.0)*100

        time_mult = min(1,2*30.(30+time_to_produce_units[0]))
        total_score = our_score + enemy_score*time_mult + loss_score

        if combat_result.time > 20:
            has_any_ground_units = False
            for i in initial_state.units:
                if i.owner == player and i.health > 0 and not i.is_flying:
                    has_any_ground_units = True
            
            if not has_any_ground_units:
                total_score -= 1000 * (combat_result.time - 20)/20.0

        return total_score

    def mineral_score_fixed_time(self, initial_state:CombatState, combat_result:CombatResult, player:int, time_to_produce_units:list, upgrades):
        if combat_result.state.owner_with_best_outcome() !=  player:
            return -10000 + mineral_score(initial_state, combat_result, player, time_to_produce_units, upgrades)
        
        assert(len(time_to_produce_units) ==3)
        # The combat result may contain more units due to temporary units spawning (e.g. infested terran, etc.) however never fewer.
        # The first N units correspond to all the N units in the initial state.
        assert(len(combat_result.state.units) > len(initial_state.units))

        total_score = 0
        our_score = 0
        enemy_score = 0
        loss_score = 0
        total_cost = 0
        our_damage_taken_cost = 0

        for i in range(len(initial_state.units)):
            unit1 = initial_state.units[i]
            unit2 = combat_result.state.units[i]
            
            assert(unit1.type == unit2.type)
            health_diff = (unit2.health - unit1.health) + (unit2.shield - unit1.shield)
            damage_taken_fraction = -health_diff / (unit1.health_max + unit1.shield_max)

            unit_type_data = unit1._type_data

            cost = unit_type_data.cost.minerals + 1.2 * unit_type_data.cost.vespene

            if unit1.owner == player:
                total_cost += cost
                our_score += cost * -(1+damage_taken_fraction)
                our_damage_taken_cost += cost *damage_taken_fraction

            else:
                if self.default_combat_environment.calculate_dps(unit2, False)> 0:
                    loss_score += cost * (-10 * (1- damage_taken_fraction))
                else:
                    loss_score += cost * (-1 * (1- damage_taken_fraction))
                enemy_score += cost * (1 +damage_taken_fraction)

        for u in upgrades:
            data = u._upgrade_data
            our_score -= data.cost.minerals + 1.2 * data.cost.vespene
        
        time_to_defeat_enemy = combat_result.average_health_time[0]

        if combat_result.state.owner_with_best_outcome() != player:
            time_to_defeat_enemy = 20
        
        time_to_defeat_enemy = min(20, time_to_defeat_enemy)

        our_score -= time_to_produce_units[1]+ 1.2 * time_to_produce_units[2]
        
        enemy_score -= our_damage_taken_cost
        time_mult = 1.0
        total_score = enemy_score * time_mult

        total_score += our_score * 0.001

        if combat_result.time > 20:
            has_any_ground_units = False
            for i in initial_state.units:
                if i.owner == player and i.health > 0 and not i.is_flying:
                    has_any_ground_units = True
            
            if not has_any_ground_units:
                total_score -= 1000 * (combat_result.time - 20)/20.0
        print("Estimated time to produce units: " , time_to_produce_units[0] ," total cost " , total_cost)
        
        return total_score

class ArmyComposition:
    def __init__(self):
        self.unit_counts = None
        self.upgrades = None
    
    def combine(self, other):
        for u in other.unit_counts:
            found = False
            for u2 in self.unit_counts:
                if u2[0] == u[0]:
                    u2 = [u2[0], u2[1]+ u[1]]
                    found = True
                    break
        
            if not found:
                self.unit_counts.append(u)
        self.upgrades.combine(other.upgrades)
        
class CompositionSearchSettings:
    def __init__(self, combat_predictor, available_unit_types, build_time_predictor):
        self.combat_predictor = combat_predictor
        self.available_unit_types = available_unit_types
        self.build_time_predictor = build_time_predictor
        self.available_time = 4 * 60

class CombatRecorder:
    def __init__(self):
        self._frames =None
    
    def tick(self, observation):
        units = []
        for u in observation.units:
            units.append(u)
        self._frames.append(observation.game_loop, units)
    
    def finalize(self, filaname='recording.csv'):
        pass

class SurroundInfo:
    def __init__(self, max_attackers_per_defender, max_melee_attackers):
        self.max_attackers_per_defender = max_attackers_per_defender
        self.max_melee_attackers = max_melee_attackers

class CompositionGene:
    def __init__(self, available_unit_types=None,mean_total_count=None, seed=None, units=None):
        if available_unit_types and mean_total_count and seed:
            self.init1(available_unit_types,mean_total_count,seed)
        elif available_unit_types and units:
            self.init2(available_unit_types, units)
        elif units:
            if isinstance(int, units):
                self.unit_counts = [0 for _ in range(units)]
                # self.unit_counts.append(units)
            elif isinstance(list, units):
                for x in units:
                    self.unit_counts.append(x)
        else:
            self.unit_counts = [0] 
    
    def get_units(self, available_unit_types):
        assert(len(self.unit_counts) == len(available_unit_types))
        result = []

        for i in range(self.unit_counts):
            if self.unit_counts[i] >0:
                type = available_unit_types.get_unit_type()
                if type != UnitTypeId.NOTAUNIT:
                    result.append([type, self.unit_counts[i]])
        
        return result

    def get_upgrades(self, available_unit_types, tech_tree=None):
        if not tech_tree:
            tech_tree = TechTree()
        result = CombatUpgrades()

        for i in range(len(self.unit_counts)):
            if self.unit_counts[i] > 0:
                item = available_unit_types.get_build_order_item(i)
                if not item.is_unit_type:
                    upgrade = item.upgrade_id

                    # If this is an enumerated upgrade then add LEVEL1, LEVEL2 up to LEVEL3 depending on the upgrade count.
                    # If it is a normal upgrade, then just add it regardless of the unit count
                    if tech_tree.is_upgrade_with_levels(upgrade):
                        final_upgrade = int(upgrade+min(self.unit_counts[i]-1,2))
                        for upgrade_index in range(upgrade, final_upgrade):
                            result.add(UpgradeId(upgrade_index))
                        break
                    else:
                        result.add(upgrade)
        
        return result

    def get_units_untyped(self, available_unit_types):
        assert(len(self.unit_counts) == len(available_unit_types))
        result = []

        for i in range(len(self.unit_counts)):
            if self.unit_counts[i]>0:
                type = available_unit_types.get_unit_type(i)
                
                if type != UnitTypeId.NOTAUNIT:
                    result.append([type, self.unit_counts[i]])
        
        return result
    
    def add_to_state(self, combat_predictor: CombatPredictor, state: CombatState, available_unit_types, owner: int):
        assert(len(self.unit_counts) == len(available_unit_types))
        for i in range(len(self.unit_counts)):
            
            if self.unit_counts[i]>0:
                type = available_unit_types.get_unit_type(i)
                if type != UnitTypeId.NOTAUNIT:
                    state.units.append(make_unit(owner, available_unit_types.get_unit_type(i)));
        
        state.environment = combat_predictor.combine_combat_environment(state.environment, get_upgrades(available_unit_types), owner);

    def mutate(self, amount: float, seed, available_unit_types):
        should_mutate_from_none = bernoulli(amount)
        should_mutate = bernoulli(amount*2)#TODO: This will not work. Needs to create a random true/false
        should_swap_mutate = bernoulli(0.2)#TODO: This will not work. Needs to create a random true/false

        for i in range(len(self.unit_counts)):
            condition = should_mutate if self.unit_counts[i] > 0 else should_mutate_from_none
            if condition:
                mx = available_unit_types.army_composition_maximum(i)
                if mx ==1:
                    dist = bernoulli(0.2)
                    self.unit_counts[i] = int(dist)
                else:
                    if self.unit_counts[i] > 0 and should_swap_mutate:
                        swap_index = random.randint(0, len(self.unit_counts)-1)
                        swap(self.unit_counts, i, swap_index)
                    else:
                        dist = np.random.geometric(1.0/(2+self.unit_counts[i]))
                        self.unit_counts[i] = min(dist, mx)

    def scale(self, scale: float, available_unit_types):
        offset = 0
        for i in range(len(self.unit_counts)):
            mx = available_unit_types.army_composition_maximum(i)
            next_point = offset + self.unit_counts[i] * scale
            self.unit_counts[i] = int(round(next_point)) - int(round(offset))
            self.unit_counts[i] = min(self.unit_counts[i], mx)
            offset = next_point
    
    def crossover(self, parent1, parent2, seed):
        assert(len(parent1.unit_counts) == len(parent2.unit_counts))
        dist = bernoulli(0.5) #TODO: This will not work. Needs to create a random true/false
        gene = CompositionGene(len(parent1.unit_counts))

        for i in range(len(gene.unit_counts)):
            gene.unit_counts[i] = parent1.unit_counts[i] if dist else parent2.unit_counts[i]
        
        return gene

    def init1(self, available_unit_types, mean_total_count,seed):
        self.unit_counts = [0 for _ in range(len(available_unit_types))]
        total_units = int(round(np.random.exponential(1.0/mean_total_count)))

        split_points = [0 for _ in range(len(self.unit_counts)+1)]


        for i in range(len(self.unit_counts)):
            split_points[i] = random.randint(0, total_units)
        
        split_points[i] = 0
        split_points[len(split_points)-1] = total_units
        split_points.sort()

        for i in range(len(self.unit_counts)):
            self.unit_counts[i] = split_points[i+1] - split_points[i]
            self.unit_counts[i] = min(available_unit_types.army_composition_maximum(i), self.units_counts[i])
        
    def init2(self,available_unit_types, units ):
        self.unit_counts = [0 for _ in range(len(available_unit_types))]
        for u in units:
            self.unit_counts[available_unit_types.index(u[0])] += u[1]

def scale_until_winning(predictor: CombatPredictor, opponent: CombatState, available_unit_types, gene):
    for its in range(5):
        state: CombatState = opponent
        gene.add_to_state(predictor, state, available_unit_types, 2)
        if predictor.predict_engage(state, False, False).state.owner_with_best_outcome() == 2:
            break

        gene.scale(1.5, available_unit_types)

def calculate_fitness(predictor: CombatPredictor, opponent: CombatState, available_unit_types, gene, time_to_produce_units: list):
    state: CombatState = opponent
    upgrades = gene.get_upgrades(available_unit_types)

    if state.environment:
        upgrades.remove(state.environment.upgrades[1])
    gene.add_to_state(predictor, state, available_unit_types, 2)
    
    return predictor.mineral_score(state, predictor.predict_engage(state, False, False), 2, time_to_produce_units, upgrades)

def calculate_fitness_fixed_time(predictor: CombatPredictor, opponent: CombatState, available_unit_types, gene, time_to_produce_units: list):
    state: CombatState = opponent
    upgrades = gene.get_upgrades(available_unit_types)

    if state.environment:
        upgrades.remove(state.environment.upgrades[1])
    gene.add_to_state(predictor, state, available_unit_types, 2)
    return predictor.mineral_score(state, predictor.predict_engage(state, False, False), 2, time_to_produce_units, gene.get_upgrades(available_unit_types))

def micros():
    return time.time() ## TODO: Dunno what this is supposed to return

def swap(l, p1, p2):
    l[p1], l[p2] = l[p2], l[p1]
    return l

def log_recordings(state:CombatState, predictor:CombatPredictor, spawn_offset: float, msg:str='Recording'):
    pass

def time_to_be_able_to_attack(env:CombatEnvironment, unit:CombatUnit, distance_to_enemy: float):
    dc = DataCache()
    unit_type_data = dc.get_unit_data(unit)
    return (max(0, distance_to_enemy - env.attack_range(unit=unit))/unit_type_data.movement_speed) if unit_type_data.movement_speed > 0 else 100000

def max_surround(enemy_ground_unit_area: float, enemy_ground_units: int, tech_tree=None):
    if not tech_tree:
        tech_tree = TechTree()
    if enemy_ground_units >1:
        enemy_ground_unit_area /= 0.6
    radius = math.sqrt(enemy_ground_unit_area/ math.pi)
    representative_melee_unit_radius = tech_tree.unit_radius(UnitTypeId.ZEALOT) # TODO: Find radius
    
    circumference_defenders = radius * (2 * math.pi)
    circumference_attackers = (radius +representative_melee_unit_radius) * (2* math.pi)

    approximate_defenders_in_melee_range = min(enemy_ground_units, circumference_defenders/ (2* representative_melee_unit_radius))
    approximate_attackers_in_melee_range = circumference_attackers / (2* representative_melee_unit_radius)

    max_attackers_per_defender = math.ceil(approximate_attackers_in_melee_range/approximate_defenders_in_melee_range) if approximate_defenders_in_melee_range > 0 else 1
    max_melee_attackers = int(math.ceil(approximate_attackers_in_melee_range))
    
    return SurroundInfo(max_attackers_per_defender, max_melee_attackers)

def filter_by_owner(units, owner):
    result = []
    for u in units:
        if u.owner == owner:
            result.append(u)
    
    return result

def combat_hash(state:CombatState, bad_micro:bool, defender_player:int, max_time:float):
    h = 0
    h = h^defender_player
    h = h*31 ^ int(bad_micro)
    h = h*31 ^ int(round(max_time))
    for u in state.units:
        h = (h * 31) ^ int(u.energy)
        h = (h * 31) ^ int(u.health)
        h = (h * 31) ^ int(u.shield)
        h = (h * 31) ^ int(u.type)
        h = (h * 31) ^ int(u.owner)
    
    return h

def find_best_composition_genetic(    
    predictor: CombatPredictor, 
    opponent: CombatState, 
    starting_build_state,
    seed_composition: list,
    settings: CompositionSearchSettings=None, 
    available_unit_types=None,     
    build_time_predictor=None,      
    ):
    if not settings:
        assert (available_unit_types is not None and build_time_predictor is not None)
        settings: CompositionSearchSettings = CompositionSearchSettings(predictor, available_unit_types, build_time_predictor)
    predictor = settings.combat_predictor
    build_time_predictor = settings.build_time_predictor
    available_unit_types = settings.available_unit_types

    watch: StopWatch =None # TODO: Implement stopwatch
    
    POOL_SIZE: int = 20
    mutation_rate: float = 0.2
    generation = [CompositionGene() for _ in range(POOL_SIZE)]
    random.seed(micros())
    rnd = random.random()


    for i, gene in enumerate(generation):
        generation[i] = CompositionGene(available_unit_types, 10, rnd)
    
    starting_unitsNN = []

    if starting_build_state and build_time_predictor:
        for u in starting_build_state.units:
            starting_unitsNN.append([int(u.type), u.units])
        for u in starting_build_state.upgrades:
            starting_unitsNN.append([int(u+UPGRADE_ID_OFFSET), 1])
    
    for i in range(50):
        assert(len(generation) == POOL_SIZE)
        if (i == 20 and seed_composition):
            generation[len(generation)-1] = CompositionGene(available_unit_types, seed_composition)
        
        fitness = [0.0 for _ in range(len(generation))]
        indices = [0 for _ in range(len(generation))]

        if False:
            pass ## TODO: There is some code here, but it will always return False, so I don't think it's needed
        else:
            for i in range(4):
                target_unitsNN = [[[0,0]] for _ in range(len(generation))]
                for j in range(len(generation)):
                    assert(len(generation[j].unit_counts) == len(available_unit_types))
                    target_unitsNN[j] = generation[j].get_units_untyped(available_unit_types)
                    upgrades = generation[j].get_upgrades(available_unit_types)
                    upgrades.remove(starting_build_state.upgrades)
                    for u in upgrades:
                        target_unitsNN[j].append([u+UPGRADE_ID_OFFSET, 1])
                time_to_produce_units = build_time_predictor.predict_time_to_build(starting_unitsNN, starting_build_state.resources, target_unitsNN) if starting_build_state and build_time_predictor else [[0.0] for _ in range(len(generation))] #TODO: Might not work
                
                for j in range(len(generation)):
                    factor = settings.available_time / max(0.001, time_to_produce_units[j][0])
                    if abs(factor-1.0) > 0.01:
                        generation[j].scale(factor, available_unit_types)
            
            target_unitsNN = [[[0,0]] for _ in range(len(generation))]
            for j in range(len(generation)):
                target_unitsNN[j] = generation[j].get_units_untyped(available_unit_types)
                upgrades = generation[j].get_upgrades(available_unit_types)
                upgrades.remove(starting_build_state.upgrades)
                for u in upgrades:
                        target_unitsNN[j].append([u+UPGRADE_ID_OFFSET, 1])
            time_to_produce_units = build_time_predictor.predict_time_to_build(starting_unitsNN, starting_build_state.resources, target_unitsNN) 
            for j in range(len(generation)):
                indices[j] = j
                fitness[j] = calculate_fitness_fixed_time(predictor, opponent, available_unit_types, generation[j], time_to_produce_units[j])
            reverse(fitness) #TODO: double-check this
            
            next_generation =[]
            for j in range(5):
                next_generation.append(generation[indices[j]])
            
            next_generation.append(generation[random.randint(0,len(indices)-1)])
            
            while len(next_generation) < POOL_SIZE:
                next_generation.append(CompositionGene.crossover(generation[random.randint(0,len(indices)-1)],generation[random.randint(0,len(indices)-1)], random.random()))
            
            generation, next_generation = next_generation, generation

        result = ArmyComposition()
        result.unit_counts = generation[0].get_units(available_unit_types)
        result.upgrades = generation[0].get_upgrades(available_unit_types)
        return result
                    

def ticks_to_seconds(ticks):
    return ticks / 22.4

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
    unit.type = type;
    return unit;
